/// MCP 服务注册表模块
///
/// 提供动态服务注册和发现功能，支持服务版本管理、依赖检查和运行时服务管理
use crate::mcp::{GitAiMcpService, McpError, McpResult};
use log::{debug, error, info};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 服务状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// 服务运行正常
    Up,
    /// 服务停止
    Down,
    /// 服务性能降级
    Degraded,
    /// 服务启动中
    Starting,
    /// 服务停止中
    Stopping,
}

/// 服务元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// 服务唯一标识符
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: Version,
    /// 服务描述
    pub description: String,
    /// 服务提供的工具列表
    pub tools: Vec<String>,
    /// 服务依赖关系
    pub dependencies: Vec<ServiceDependency>,
    /// 服务状态
    pub status: ServiceStatus,
    /// 服务注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 最后健康检查时间
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    /// 服务配置
    pub config: serde_json::Value,
}

/// 服务依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// 依赖的服务名称
    pub service_name: String,
    /// 版本要求
    pub version_req: VersionReq,
    /// 是否为可选依赖
    pub optional: bool,
}

/// 服务注册表
pub struct ServiceRegistry {
    /// 已注册的服务元数据
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    /// 服务实例
    service_instances: Arc<RwLock<HashMap<String, Arc<dyn GitAiMcpService + Send + Sync>>>>,
    /// 服务事件监听器
    event_listeners: Arc<RwLock<Vec<Arc<dyn ServiceEventListener + Send + Sync>>>>,
}

/// 服务事件类型
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    /// 服务注册事件
    Registered {
        service_id: String,
        metadata: ServiceMetadata,
    },
    /// 服务注销事件
    Unregistered { service_id: String, reason: String },
    /// 服务状态变更事件
    StatusChanged {
        service_id: String,
        old_status: ServiceStatus,
        new_status: ServiceStatus,
    },
    /// 服务健康检查事件
    HealthCheckCompleted {
        service_id: String,
        healthy: bool,
        response_time: std::time::Duration,
    },
}

/// 服务事件监听器 trait
#[async_trait::async_trait]
pub trait ServiceEventListener: Send + Sync {
    /// 处理服务事件
    async fn on_service_event(&self, event: ServiceEvent);
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    pub fn new() -> Self {
        info!("🔧 初始化服务注册表");
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            service_instances: Arc::new(RwLock::new(HashMap::new())),
            event_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 注册服务
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<()> {
        let service_name = service.name().to_string();
        let service_id = generate_service_id(&service_name);

        debug!("📝 注册服务: {} (ID: {})", service_name, service_id);

        // 解析服务版本
        let version = self.parse_service_version(service.as_ref())?;

        // 创建服务元数据
        let metadata = ServiceMetadata {
            id: service_id.clone(),
            name: service_name.clone(),
            version,
            description: service.description().to_string(),
            tools: service
                .tools()
                .into_iter()
                .map(|tool| tool.name.to_string())
                .collect(),
            dependencies: self.extract_service_dependencies(service.as_ref()).await?,
            status: ServiceStatus::Starting,
            registered_at: chrono::Utc::now(),
            last_health_check: None,
            config,
        };

        // 检查依赖关系
        self.validate_dependencies(&metadata).await?;

        // 注册服务
        {
            let mut services = self.services.write().await;
            let mut instances = self.service_instances.write().await;

            // 检查是否已经有同名服务注册
            if services.values().any(|m| m.name == service_name) {
                return Err(McpError::ConfigurationError(format!(
                    "Service '{}' is already registered",
                    service_name
                )));
            }

            if services.contains_key(&service_id) {
                return Err(McpError::ConfigurationError(format!(
                    "Service {} is already registered",
                    service_name
                )));
            }

            services.insert(service_id.clone(), metadata.clone());
            instances.insert(service_id.clone(), service.clone());
        }

        // 检测循环依赖
        if let Err(cycles) = self.detect_circular_dependencies().await {
            // 回滚注册
            {
                let mut services = self.services.write().await;
                let mut instances = self.service_instances.write().await;
                services.remove(&service_id);
                instances.remove(&service_id);
            }

            error!("❌ 检测到循环依赖，服务注册失败");
            for cycle in &cycles {
                error!("循环路径: {}", cycle.join(" -> "));
            }

            return Err(McpError::ConfigurationError(format!(
                "Circular dependency detected when registering service {}. Cycles: {}",
                service_name,
                cycles
                    .into_iter()
                    .map(|c| c.join(" -> "))
                    .collect::<Vec<_>>()
                    .join("; ")
            )));
        }

        // 更新服务状态为运行中
        self.update_service_status(&service_id, ServiceStatus::Up)
            .await?;

        // 触发注册事件
        self.emit_event(ServiceEvent::Registered {
            service_id: service_id.clone(),
            metadata: metadata.clone(),
        })
        .await;

        info!("✅ 服务 '{}' 注册成功 (ID: {})", service_name, service_id);
        Ok(())
    }

    /// 注销服务
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()> {
        debug!("📝 注销服务: {} (原因: {})", service_id, reason);

        // 检查服务是否存在
        let service_name = {
            let services = self.services.read().await;
            match services.get(service_id) {
                Some(metadata) => metadata.name.clone(),
                None => {
                    return Err(McpError::InvalidParameters(format!(
                        "Service {} not found",
                        service_id
                    )))
                }
            }
        };

        // 检查是否有其他服务依赖此服务
        self.check_service_dependencies_before_removal(service_id)
            .await?;

        // 更新服务状态为停止中
        self.update_service_status(service_id, ServiceStatus::Stopping)
            .await?;

        // 移除服务
        {
            let mut services = self.services.write().await;
            let mut instances = self.service_instances.write().await;

            services.remove(service_id);
            instances.remove(service_id);
        }

        // 触发注销事件
        self.emit_event(ServiceEvent::Unregistered {
            service_id: service_id.to_string(),
            reason: reason.clone(),
        })
        .await;

        info!(
            "✅ 服务 '{}' 注销成功 (ID: {}, 原因: {})",
            service_name, service_id, reason
        );
        Ok(())
    }

    /// 获取服务列表
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// 获取服务实例
    pub async fn get_service(
        &self,
        service_id: &str,
    ) -> Option<Arc<dyn GitAiMcpService + Send + Sync>> {
        let instances = self.service_instances.read().await;
        instances.get(service_id).cloned()
    }

    /// 根据工具名称查找服务
    pub async fn find_service_by_tool(
        &self,
        tool_name: &str,
    ) -> Option<Arc<dyn GitAiMcpService + Send + Sync>> {
        let services = self.services.read().await;
        let instances = self.service_instances.read().await;

        for (service_id, metadata) in services.iter() {
            if metadata.tools.contains(&tool_name.to_string()) {
                if let Some(service) = instances.get(service_id) {
                    return Some(service.clone());
                }
            }
        }
        None
    }

    /// 更新服务状态
    pub async fn update_service_status(
        &self,
        service_id: &str,
        new_status: ServiceStatus,
    ) -> McpResult<()> {
        let old_status = {
            let mut services = self.services.write().await;
            match services.get_mut(service_id) {
                Some(metadata) => {
                    let old = metadata.status.clone();
                    metadata.status = new_status.clone();
                    metadata.last_health_check = Some(chrono::Utc::now());
                    old
                }
                None => {
                    return Err(McpError::InvalidParameters(format!(
                        "Service {} not found",
                        service_id
                    )))
                }
            }
        };

        // 只在状态真正改变时才触发事件
        if old_status != new_status {
            debug!(
                "🔄 服务 {} 状态变更: {:?} -> {:?}",
                service_id, old_status, new_status
            );

            self.emit_event(ServiceEvent::StatusChanged {
                service_id: service_id.to_string(),
                old_status,
                new_status,
            })
            .await;
        }

        Ok(())
    }

    /// 获取健康的服务列表
    pub async fn get_healthy_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read().await;
        services
            .values()
            .filter(|metadata| metadata.status == ServiceStatus::Up)
            .cloned()
            .collect()
    }

    /// 添加事件监听器
    pub async fn add_event_listener(&self, listener: Arc<dyn ServiceEventListener + Send + Sync>) {
        let mut listeners = self.event_listeners.write().await;
        listeners.push(listener);
        debug!("📡 添加服务事件监听器，当前监听器数量: {}", listeners.len());
    }

    /// 移除事件监听器
    pub async fn remove_event_listener(&self, listener_ptr: *const dyn ServiceEventListener) {
        let mut listeners = self.event_listeners.write().await;
        listeners.retain(|l| {
            !std::ptr::addr_eq(
                Arc::as_ptr(l) as *const dyn ServiceEventListener,
                listener_ptr,
            )
        });
        debug!("📡 移除服务事件监听器，当前监听器数量: {}", listeners.len());
    }

    /// 触发服务事件
    async fn emit_event(&self, event: ServiceEvent) {
        let listeners = self.event_listeners.read().await;
        for listener in listeners.iter() {
            listener.on_service_event(event.clone()).await;
        }
    }

    /// 解析服务版本
    fn parse_service_version(&self, service: &dyn GitAiMcpService) -> McpResult<Version> {
        Ok(service.version())
    }

    /// 提取服务依赖关系
    async fn extract_service_dependencies(
        &self,
        service: &dyn GitAiMcpService,
    ) -> McpResult<Vec<ServiceDependency>> {
        Ok(service.dependencies())
    }

    /// 验证服务依赖关系
    async fn validate_dependencies(&self, metadata: &ServiceMetadata) -> McpResult<()> {
        let services = self.services.read().await;

        for dependency in &metadata.dependencies {
            if let Some(dep_service) = services
                .values()
                .find(|s| s.name == dependency.service_name)
            {
                // 检查版本兼容性
                if !dependency.version_req.matches(&dep_service.version) {
                    return Err(McpError::ConfigurationError(format!(
                        "Service {} depends on {} version {}, but version {} is registered",
                        metadata.name,
                        dependency.service_name,
                        dependency.version_req,
                        dep_service.version
                    )));
                }

                // 检查依赖服务状态
                if dep_service.status != ServiceStatus::Up && !dependency.optional {
                    return Err(McpError::ConfigurationError(format!(
                        "Service {} depends on {} which is not available (status: {:?})",
                        metadata.name, dependency.service_name, dep_service.status
                    )));
                }
            } else if !dependency.optional {
                return Err(McpError::ConfigurationError(format!(
                    "Service {} depends on {} which is not registered",
                    metadata.name, dependency.service_name
                )));
            }
        }

        Ok(())
    }

    /// 检查移除服务前的依赖关系
    async fn check_service_dependencies_before_removal(&self, service_id: &str) -> McpResult<()> {
        let services = self.services.read().await;
        let service_to_remove = services.get(service_id).ok_or_else(|| {
            McpError::InvalidParameters(format!("Service {} not found", service_id))
        })?;

        // 检查是否有其他服务依赖于将要移除的服务
        let dependent_services: Vec<&ServiceMetadata> = services
            .values()
            .filter(|metadata| {
                metadata.id != service_id
                    && metadata
                        .dependencies
                        .iter()
                        .any(|dep| dep.service_name == service_to_remove.name && !dep.optional)
            })
            .collect();

        if !dependent_services.is_empty() {
            let dependent_names: Vec<&str> =
                dependent_services.iter().map(|s| s.name.as_str()).collect();
            return Err(McpError::ConfigurationError(format!(
                "Cannot remove service {} because it is required by: {}",
                service_to_remove.name,
                dependent_names.join(", ")
            )));
        }

        Ok(())
    }

    /// 检测循环依赖
    pub async fn detect_circular_dependencies(&self) -> Result<(), Vec<Vec<String>>> {
        let services = self.services.read().await;
        let mut errors = Vec::new();

        for (service_id, _metadata) in services.iter() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();

            if let Err(cycle) =
                Self::dfs_detect_cycle(&services, service_id, &mut visited, &mut path)
            {
                errors.push(cycle);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 深度优先搜索检测循环
    fn dfs_detect_cycle(
        services: &HashMap<String, ServiceMetadata>,
        current_id: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), Vec<String>> {
        if path.contains(&current_id.to_string()) {
            // 发现循环
            let cycle_start = path.iter().position(|s| s == current_id).unwrap();
            let mut cycle = path[cycle_start..].to_vec();
            cycle.push(current_id.to_string());
            return Err(cycle);
        }

        if visited.contains(current_id) {
            return Ok(());
        }

        visited.insert(current_id.to_string());
        path.push(current_id.to_string());

        if let Some(metadata) = services.get(current_id) {
            for dep in &metadata.dependencies {
                // 查找依赖服务
                if let Some(dep_metadata) = services.values().find(|m| m.name == dep.service_name) {
                    Self::dfs_detect_cycle(services, &dep_metadata.id, visited, path)?;
                }
            }
        }

        path.pop();
        Ok(())
    }

    /// 获取服务启动顺序（拓扑排序）
    pub async fn get_startup_order(&self) -> McpResult<Vec<String>> {
        let services = self.services.read().await;
        let mut in_degree = HashMap::new();
        let mut graph = HashMap::new();

        // 构建依赖图
        for (id, _metadata) in services.iter() {
            in_degree.insert(id.clone(), 0);
            graph.insert(id.clone(), Vec::new());
        }

        for (id, metadata) in services.iter() {
            for dep in &metadata.dependencies {
                if let Some(dep_metadata) = services.values().find(|m| m.name == dep.service_name) {
                    graph.get_mut(&dep_metadata.id).unwrap().push(id.clone());
                    *in_degree.get_mut(id).unwrap() += 1;
                }
            }
        }

        // 拓扑排序
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        for (id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(id.clone());
            }
        }

        while let Some(id) = queue.pop_front() {
            result.push(id.clone());

            if let Some(neighbors) = graph.get(&id) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if result.len() != services.len() {
            return Err(McpError::ConfigurationError(
                "Circular dependency detected in service graph".to_string(),
            ));
        }

        Ok(result)
    }

    /// 获取服务依赖图
    pub async fn get_dependency_graph(&self) -> HashMap<String, Vec<String>> {
        let services = self.services.read().await;
        let mut graph = HashMap::new();

        for (id, metadata) in services.iter() {
            let deps: Vec<String> = metadata
                .dependencies
                .iter()
                .filter_map(|dep| {
                    services
                        .values()
                        .find(|m| m.name == dep.service_name)
                        .map(|m| m.id.clone())
                })
                .collect();

            graph.insert(id.clone(), deps);
        }

        graph
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成服务 ID
fn generate_service_id(service_name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let timestamp = chrono::Utc::now().timestamp_millis();
    let mut hasher = DefaultHasher::new();
    service_name.hash(&mut hasher);
    timestamp.hash(&mut hasher);

    format!("{}_{:x}", service_name, hasher.finish())
}

/// 服务注册表构建器
pub struct ServiceRegistryBuilder {
    registry: ServiceRegistry,
}

impl ServiceRegistryBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            registry: ServiceRegistry::new(),
        }
    }

    /// 添加事件监听器
    pub async fn with_event_listener(
        self,
        listener: Arc<dyn ServiceEventListener + Send + Sync>,
    ) -> Self {
        self.registry.add_event_listener(listener).await;
        self
    }

    /// 构建服务注册表
    pub fn build(self) -> ServiceRegistry {
        self.registry
    }
}

impl Default for ServiceRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::Tool;
    use serde_json::Map;
    use std::sync::Arc;
    use tokio;

    /// 测试用的模拟服务
    struct MockService {
        name: String,
        description: String,
    }

    impl MockService {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl GitAiMcpService for MockService {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn tools(&self) -> Vec<Tool> {
            vec![Tool {
                name: format!("test_{}", self.name).into(),
                description: format!("Test tool for {}", self.name).into(),
                input_schema: Arc::new(Map::new()),
            }]
        }

        async fn handle_tool_call(
            &self,
            _name: &str,
            _arguments: serde_json::Value,
        ) -> McpResult<serde_json::Value> {
            Ok(serde_json::json!({"result": "mock"}))
        }
    }

    /// 测试用的事件监听器
    #[derive(Default)]
    struct TestEventListener {
        events: Arc<RwLock<Vec<ServiceEvent>>>,
    }

    impl TestEventListener {
        fn new() -> Self {
            Self {
                events: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn get_events(&self) -> Vec<ServiceEvent> {
            self.events.read().await.clone()
        }
    }

    #[async_trait::async_trait]
    impl ServiceEventListener for TestEventListener {
        async fn on_service_event(&self, event: ServiceEvent) {
            self.events.write().await.push(event);
        }
    }

    #[tokio::test]
    async fn test_service_registration() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("test_service", "Test service"));
        let config = serde_json::json!({"key": "value"});

        let result = registry
            .register_service(service.clone(), config.clone())
            .await;
        assert!(result.is_ok());

        let services = registry.list_services().await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test_service");
        assert_eq!(services[0].status, ServiceStatus::Up);
    }

    #[tokio::test]
    async fn test_service_unregistration() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("test_service", "Test service"));
        let config = serde_json::json!({});

        // 注册服务
        registry
            .register_service(service.clone(), config)
            .await
            .unwrap();
        let services = registry.list_services().await;
        assert_eq!(services.len(), 1);

        // 注销服务
        let service_id = &services[0].id;
        registry
            .unregister_service(service_id, "Test removal".to_string())
            .await
            .unwrap();

        let services = registry.list_services().await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_find_service_by_tool() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("test_service", "Test service"));
        let config = serde_json::json!({});

        registry
            .register_service(service.clone(), config)
            .await
            .unwrap();

        let found_service = registry.find_service_by_tool("test_test_service").await;
        assert!(found_service.is_some());

        let not_found = registry.find_service_by_tool("nonexistent_tool").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_service_status_update() {
        let registry = ServiceRegistry::new();
        let service = Arc::new(MockService::new("test_service", "Test service"));
        let config = serde_json::json!({});

        registry
            .register_service(service.clone(), config)
            .await
            .unwrap();
        let services = registry.list_services().await;
        let service_id = &services[0].id;

        // 更新状态为降级
        registry
            .update_service_status(service_id, ServiceStatus::Degraded)
            .await
            .unwrap();

        let services = registry.list_services().await;
        assert_eq!(services[0].status, ServiceStatus::Degraded);

        // 获取健康服务列表应该为空
        let healthy_services = registry.get_healthy_services().await;
        assert_eq!(healthy_services.len(), 0);
    }

    #[tokio::test]
    async fn test_event_listener() {
        let registry = ServiceRegistry::new();
        let listener = Arc::new(TestEventListener::new());

        registry.add_event_listener(listener.clone()).await;

        let service = Arc::new(MockService::new("test_service", "Test service"));
        let config = serde_json::json!({});

        registry
            .register_service(service.clone(), config)
            .await
            .unwrap();

        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let events = listener.get_events().await;
        assert!(!events.is_empty()); // 至少应该有注册事件

        // 检查是否有注册事件
        let has_registration_event = events
            .iter()
            .any(|event| matches!(event, ServiceEvent::Registered { .. }));
        assert!(has_registration_event);
    }

    #[tokio::test]
    async fn test_duplicate_service_registration() {
        let registry = ServiceRegistry::new();
        let service1 = Arc::new(MockService::new("test_service", "Test service 1"));
        let service2 = Arc::new(MockService::new("test_service", "Test service 2"));
        let config = serde_json::json!({});

        // 注册第一个服务应该成功
        let result1 = registry.register_service(service1, config.clone()).await;
        assert!(result1.is_ok());

        // 注册同名服务应该失败
        let result2 = registry.register_service(service2, config).await;
        assert!(result2.is_err());
    }

    /// 测试用的服务，支持依赖关系
    struct MockServiceWithDependencies {
        name: String,
        description: String,
        dependencies: Vec<ServiceDependency>,
        version: Version,
    }

    impl MockServiceWithDependencies {
        fn new(name: &str, description: &str, version: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                dependencies: Vec::new(),
                version: Version::parse(version).unwrap(),
            }
        }

        fn with_dependency(
            mut self,
            service_name: &str,
            version_req: &str,
            optional: bool,
        ) -> Self {
            self.dependencies.push(ServiceDependency {
                service_name: service_name.to_string(),
                version_req: VersionReq::parse(version_req).unwrap(),
                optional,
            });
            self
        }
    }

    #[async_trait::async_trait]
    impl GitAiMcpService for MockServiceWithDependencies {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn version(&self) -> Version {
            self.version.clone()
        }

        fn dependencies(&self) -> Vec<ServiceDependency> {
            self.dependencies.clone()
        }

        fn tools(&self) -> Vec<Tool> {
            vec![Tool {
                name: format!("test_{}", self.name).into(),
                description: format!("Test tool for {}", self.name).into(),
                input_schema: Arc::new(Map::new()),
            }]
        }

        async fn handle_tool_call(
            &self,
            _name: &str,
            _arguments: serde_json::Value,
        ) -> McpResult<serde_json::Value> {
            Ok(serde_json::json!({"result": "mock"}))
        }
    }

    #[tokio::test]
    async fn test_service_dependencies() {
        let registry = ServiceRegistry::new();

        // 创建基础服务
        let base_service = Arc::new(MockServiceWithDependencies::new(
            "base_service",
            "Base service",
            "1.0.0",
        ));

        // 创建依赖基础服务的服务
        let dependent_service = Arc::new(
            MockServiceWithDependencies::new("dependent_service", "Dependent service", "1.0.0")
                .with_dependency("base_service", ">=1.0.0", false),
        );

        let config = serde_json::json!({});

        // 先注册依赖服务应该失败
        let result = registry
            .register_service(dependent_service.clone(), config.clone())
            .await;
        assert!(result.is_err());

        // 先注册基础服务
        let result = registry
            .register_service(base_service, config.clone())
            .await;
        assert!(result.is_ok());

        // 现在注册依赖服务应该成功
        let result = registry.register_service(dependent_service, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let registry = ServiceRegistry::new();

        // 创建两个相互依赖的服务，形成简单循环: A -> B -> A
        let service_a = Arc::new(MockServiceWithDependencies::new(
            "service_a",
            "Service A",
            "1.0.0",
        ));

        // 先注册服务 A（没有依赖）
        let config = serde_json::json!({});
        registry
            .register_service(service_a, config.clone())
            .await
            .unwrap();

        // 创建依赖服务 A 的服务 B
        let service_b_with_a_dep =
            Arc::new(
                MockServiceWithDependencies::new("service_b", "Service B", "1.0.0")
                    .with_dependency("service_a", ">=1.0.0", false),
            );

        // 注册服务 B
        registry
            .register_service(service_b_with_a_dep, config.clone())
            .await
            .unwrap();

        // 现在创建一个更新的服务 A，依赖服务 B（形成循环）
        let _service_a_with_b_dep = Arc::new(
            MockServiceWithDependencies::new("service_a_updated", "Service A Updated", "1.0.1")
                .with_dependency("service_b", ">=1.0.0", false),
        );

        // 创建一个反向依赖，形成循环：更新 A -> B，而 B -> A
        let service_c =
            Arc::new(
                MockServiceWithDependencies::new("service_c", "Service C", "1.0.0")
                    .with_dependency("service_b", ">=1.0.0", false),
            );

        // 注册服务 C
        registry
            .register_service(service_c, config.clone())
            .await
            .unwrap();

        // 创建一个服务 D，依赖 C
        let service_d =
            Arc::new(
                MockServiceWithDependencies::new("service_d", "Service D", "1.0.0")
                    .with_dependency("service_c", ">=1.0.0", false),
            );

        // 注册服务 D
        registry
            .register_service(service_d, config.clone())
            .await
            .unwrap();

        // 现在尝试修改服务 A 使其依赖 D，形成循环：A -> D -> C -> B -> A
        // 需要先注销原来的 A
        let services = registry.list_services().await;
        let a_id = services
            .iter()
            .find(|s| s.name == "service_a")
            .unwrap()
            .id
            .clone();

        // 由于 B 依赖 A，所以需要先删除 B
        let b_id = services
            .iter()
            .find(|s| s.name == "service_b")
            .unwrap()
            .id
            .clone();
        let c_id = services
            .iter()
            .find(|s| s.name == "service_c")
            .unwrap()
            .id
            .clone();
        let d_id = services
            .iter()
            .find(|s| s.name == "service_d")
            .unwrap()
            .id
            .clone();

        // 删除 D、C、B，然后删除 A
        registry
            .unregister_service(&d_id, "test".to_string())
            .await
            .unwrap();
        registry
            .unregister_service(&c_id, "test".to_string())
            .await
            .unwrap();
        registry
            .unregister_service(&b_id, "test".to_string())
            .await
            .unwrap();
        registry
            .unregister_service(&a_id, "test".to_string())
            .await
            .unwrap();

        // 现在重新创建循环依赖：A -> B -> A
        let service_a_new =
            Arc::new(
                MockServiceWithDependencies::new("service_a", "Service A", "1.0.0")
                    .with_dependency("service_b", ">=1.0.0", false),
            );

        let service_b_new =
            Arc::new(
                MockServiceWithDependencies::new("service_b", "Service B", "1.0.0")
                    .with_dependency("service_a", ">=1.0.0", false),
            );

        // 由于相互依赖，两个都不能单独注册
        let result_a = registry
            .register_service(service_a_new, config.clone())
            .await;
        assert!(result_a.is_err()); // A 依赖 B，但 B 不存在

        let result_b = registry.register_service(service_b_new, config).await;
        assert!(result_b.is_err()); // B 依赖 A，但 A 不存在
    }

    #[tokio::test]
    async fn test_version_compatibility() {
        let registry = ServiceRegistry::new();

        // 创建一个 1.0.0 版本的服务
        let base_v1 = Arc::new(MockServiceWithDependencies::new(
            "base_service",
            "Base service v1",
            "1.0.0",
        ));

        // 创建需要 >=2.0.0 版本的依赖服务
        let dependent = Arc::new(
            MockServiceWithDependencies::new("dependent_service", "Dependent service", "1.0.0")
                .with_dependency("base_service", ">=2.0.0", false),
        );

        let config = serde_json::json!({});

        // 注册基础服务 v1
        registry
            .register_service(base_v1, config.clone())
            .await
            .unwrap();

        // 注册依赖服务应该失败，因为版本不匹配
        let result = registry.register_service(dependent, config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[tokio::test]
    async fn test_cascade_unregistration_prevention() {
        let registry = ServiceRegistry::new();

        // 创建依赖关系: dependent -> base
        let base_service = Arc::new(MockServiceWithDependencies::new(
            "base_service",
            "Base service",
            "1.0.0",
        ));

        let dependent_service = Arc::new(
            MockServiceWithDependencies::new("dependent_service", "Dependent service", "1.0.0")
                .with_dependency("base_service", ">=1.0.0", false),
        );

        let config = serde_json::json!({});

        // 注册两个服务
        registry
            .register_service(base_service, config.clone())
            .await
            .unwrap();
        registry
            .register_service(dependent_service, config)
            .await
            .unwrap();

        let services = registry.list_services().await;
        let base_id = services
            .iter()
            .find(|s| s.name == "base_service")
            .unwrap()
            .id
            .clone();

        // 尝试移除基础服务应该失败，因为有依赖
        let result = registry
            .unregister_service(&base_id, "test removal".to_string())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required by"));
    }

    #[tokio::test]
    async fn test_optional_dependency() {
        let registry = ServiceRegistry::new();

        // 创建带可选依赖的服务
        let service_with_optional = Arc::new(
            MockServiceWithDependencies::new(
                "service_with_optional",
                "Service with optional dependency",
                "1.0.0",
            )
            .with_dependency("optional_service", ">=1.0.0", true),
        );

        let config = serde_json::json!({});

        // 即使可选依赖不存在，服务应该能够注册
        let result = registry
            .register_service(service_with_optional, config)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_startup_order() {
        let registry = ServiceRegistry::new();

        // 创建依赖链: C -> B -> A
        let service_a = Arc::new(MockServiceWithDependencies::new(
            "service_a",
            "Service A",
            "1.0.0",
        ));

        let service_b =
            Arc::new(
                MockServiceWithDependencies::new("service_b", "Service B", "1.0.0")
                    .with_dependency("service_a", ">=1.0.0", false),
            );

        let service_c =
            Arc::new(
                MockServiceWithDependencies::new("service_c", "Service C", "1.0.0")
                    .with_dependency("service_b", ">=1.0.0", false),
            );

        let config = serde_json::json!({});

        // 必须按正确顺序注册：先 A，然后 B，最后 C
        registry
            .register_service(service_a, config.clone())
            .await
            .unwrap();
        registry
            .register_service(service_b, config.clone())
            .await
            .unwrap();
        registry.register_service(service_c, config).await.unwrap();

        // 获取启动顺序
        let startup_order = registry.get_startup_order().await.unwrap();
        let services = registry.list_services().await;

        // 将 ID 映射回服务名
        let startup_names: Vec<String> = startup_order
            .iter()
            .map(|id| services.iter().find(|s| &s.id == id).unwrap().name.clone())
            .collect();

        // 验证启动顺序：A 应该在 B 之前，B 应该在 C 之前
        let a_pos = startup_names.iter().position(|n| n == "service_a").unwrap();
        let b_pos = startup_names.iter().position(|n| n == "service_b").unwrap();
        let c_pos = startup_names.iter().position(|n| n == "service_c").unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }
}
