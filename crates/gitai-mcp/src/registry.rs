//! MCP 服务注册表 - 简化版本
#![allow(clippy::await_holding_lock)]

use async_trait::async_trait;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

use crate::error::{McpError, McpResult};
use crate::services::McpService;

/// 服务元数据
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    /// 服务唯一标识符
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务描述
    pub description: String,
    /// 服务状态
    pub status: ServiceStatus,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 最后健康检查时间
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    /// 健康检查响应时间
    pub health_check_response_time: Option<Duration>,
    /// 配置信息
    pub config: serde_json::Value,
}

/// 服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    /// 健康的
    Healthy,
    /// 不健康的
    Unhealthy,
    /// 未知状态
    Unknown,
    /// 启动中
    Starting,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
}

/// 服务事件
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    /// 服务已注册
    Registered {
        /// 服务ID
        service_id: String,
        /// 元数据
        metadata: ServiceMetadata,
    },
    /// 服务已注销
    Unregistered {
        /// 服务ID
        service_id: String,
        /// 注销原因
        reason: String,
    },
    /// 服务状态变更
    StatusChanged {
        /// 服务ID
        service_id: String,
        /// 旧状态
        old_status: ServiceStatus,
        /// 新状态
        new_status: ServiceStatus,
    },
    /// 健康检查完成
    HealthCheckCompleted {
        /// 服务ID
        service_id: String,
        /// 是否健康
        healthy: bool,
        /// 响应时间
        response_time: Duration,
    },
}

/// 服务事件监听器
#[async_trait]
pub trait ServiceEventListener: Send + Sync {
    /// 处理服务事件
    async fn on_service_event(
        &self,
        event: ServiceEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 服务健康检查器
pub struct HealthChecker {
    /// 检查间隔
    check_interval: Duration,
    /// 超时时间
    timeout: Duration,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
        }
    }
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置检查间隔
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 检查服务健康状态
    pub async fn check_health(&self, service: &dyn McpService) -> bool {
        // 简单的健康检查：检查服务是否可用
        service.is_available().await
    }
}

/// MCP 服务注册表
pub struct ServiceRegistry {
    /// 服务映射
    services: parking_lot::RwLock<HashMap<String, ServiceInstance>>,
    /// 事件监听器
    event_listeners: parking_lot::RwLock<Vec<Arc<dyn ServiceEventListener>>>,
    /// 健康检查器
    health_checker: HealthChecker,
    /// 运行状态
    running: std::sync::atomic::AtomicBool,
}

/// 服务实例
struct ServiceInstance {
    /// 服务对象
    service: Arc<dyn McpService + Send + Sync>,
    /// 元数据
    metadata: ServiceMetadata,
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    pub fn new() -> Self {
        Self {
            services: parking_lot::RwLock::new(HashMap::new()),
            event_listeners: parking_lot::RwLock::new(Vec::new()),
            health_checker: HealthChecker::new(),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 注册服务
    pub async fn register_service(
        &self,
        service: Arc<dyn McpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<String> {
        let service_id = Uuid::new_v4().to_string();

        // 创建服务元数据
        let metadata = ServiceMetadata {
            id: service_id.clone(),
            name: service.name().to_string(),
            description: service.description().to_string(),
            status: ServiceStatus::Healthy,
            registered_at: chrono::Utc::now(),
            last_health_check: None,
            health_check_response_time: None,
            config,
        };

        // 创建服务实例
        let instance = ServiceInstance { service, metadata };

        // 注册服务
        {
            let mut services = self.services.write();
            services.insert(service_id.clone(), instance);
        }

        // 发送注册事件
        let service_for_event = {
            let services = self.services.read();
            services.get(&service_id).unwrap().metadata.clone()
        };

        self.emit_event(ServiceEvent::Registered {
            service_id: service_id.clone(),
            metadata: service_for_event,
        })
        .await;

        info!("✅ 服务注册成功: {service_id}");
        Ok(service_id)
    }

    /// 注销服务
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()> {
        // 获取服务实例
        let _instance = {
            let mut services = self.services.write();
            services.remove(service_id).ok_or_else(|| {
                McpError::InvalidParameters(format!("Service not found: {service_id}"))
            })?
        };

        // 发送注销事件
        self.emit_event(ServiceEvent::Unregistered {
            service_id: service_id.to_string(),
            reason,
        })
        .await;

        info!("📤 服务注销成功: {service_id}");
        Ok(())
    }

    /// 获取服务
    pub async fn get_service(&self, service_id: &str) -> Option<Arc<dyn McpService + Send + Sync>> {
        let services = self.services.read();
        services
            .get(service_id)
            .map(|instance| instance.service.clone())
    }

    /// 列出所有服务
    pub async fn list_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read();
        services
            .values()
            .map(|instance| instance.metadata.clone())
            .collect()
    }

    /// 获取健康的服务
    pub async fn get_healthy_services(&self) -> Vec<ServiceMetadata> {
        let services = self.services.read();
        services
            .values()
            .filter(|instance| instance.metadata.status == ServiceStatus::Healthy)
            .map(|instance| instance.metadata.clone())
            .collect()
    }

    /// 添加事件监听器
    pub async fn add_event_listener(&self, listener: Arc<dyn ServiceEventListener>) {
        let mut listeners = self.event_listeners.write();
        listeners.push(listener);
    }

    /// 移除事件监听器
    pub async fn remove_event_listener(&self, listener: &Arc<dyn ServiceEventListener>) {
        let mut listeners = self.event_listeners.write();
        listeners.retain(|l| !Arc::ptr_eq(l, listener));
    }

    /// 发送事件
    async fn emit_event(&self, event: ServiceEvent) {
        let listeners = self.event_listeners.read();
        for listener in listeners.iter() {
            if let Err(e) = listener.on_service_event(event.clone()).await {
                error!("❌ 事件监听器错误: {e}");
            }
        }
    }

    /// 执行健康检查
    pub async fn perform_health_check(&self) {
        let services = self.services.read();
        let service_ids: Vec<String> = services.keys().cloned().collect();
        drop(services);

        for service_id in service_ids {
            if let Some(instance) = self.services.read().get(&service_id) {
                let start = Instant::now();
                let healthy = self.health_checker.check_health(&*instance.service).await;
                let response_time = start.elapsed();

                let old_status = instance.metadata.status.clone();
                let new_status = if healthy {
                    ServiceStatus::Healthy
                } else {
                    ServiceStatus::Unhealthy
                };

                // 更新服务状态
                {
                    let mut services = self.services.write();
                    if let Some(instance) = services.get_mut(&service_id) {
                        instance.metadata.status = new_status.clone();
                        instance.metadata.last_health_check = Some(chrono::Utc::now());
                        instance.metadata.health_check_response_time = Some(response_time);
                    }
                }

                // 发送状态变更事件
                if old_status != new_status {
                    self.emit_event(ServiceEvent::StatusChanged {
                        service_id: service_id.clone(),
                        old_status,
                        new_status: new_status.clone(),
                    })
                    .await;
                }

                // 发送健康检查完成事件
                self.emit_event(ServiceEvent::HealthCheckCompleted {
                    service_id: service_id.clone(),
                    healthy,
                    response_time,
                })
                .await;
            }
        }
    }

    /// 启动健康检查循环
    pub async fn start_health_check_loop(&self) {
        if self
            .running
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            warn!("⚠️ 健康检查循环已经在运行");
            return;
        }

        info!("🔄 启动健康检查循环");
        let check_interval = self.health_checker.check_interval;

        while self.running.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::time::sleep(check_interval).await;
            self.perform_health_check().await;
        }
    }

    /// 停止健康检查循环
    pub fn stop_health_check_loop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
        info!("⏹️ 健康检查循环已停止");
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ReviewService;

    #[test]
    fn test_service_status_values() {
        // 测试服务状态的各种值
        assert_eq!(ServiceStatus::Healthy, ServiceStatus::Healthy);
        assert_ne!(ServiceStatus::Healthy, ServiceStatus::Unhealthy);
        assert_ne!(ServiceStatus::Starting, ServiceStatus::Stopped);
    }

    #[test]
    fn test_service_metadata_creation() {
        let metadata = ServiceMetadata {
            id: "test-id".to_string(),
            name: "test-service".to_string(),
            description: "test description".to_string(),
            status: ServiceStatus::Healthy,
            registered_at: chrono::Utc::now(),
            last_health_check: None,
            health_check_response_time: None,
            config: serde_json::json!({"key": "value"}),
        };

        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.name, "test-service");
        assert_eq!(metadata.description, "test description");
        assert_eq!(metadata.status, ServiceStatus::Healthy);
        assert!(metadata.last_health_check.is_none());
        assert!(metadata.health_check_response_time.is_none());
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ServiceRegistry::new();

        let services = registry.list_services().await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_default() {
        let registry = ServiceRegistry::default();

        let services = registry.list_services().await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new();

        // 验证健康检查器能够创建并检查服务
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = ReviewService::new(config);

        let is_healthy = checker.check_health(&service).await;
        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_health_checker_with_custom_values() {
        let checker = HealthChecker::new()
            .with_check_interval(Duration::from_secs(60))
            .with_timeout(Duration::from_secs(20));

        // 验证自定义配置的健康检查器仍然工作
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = ReviewService::new(config);

        let is_healthy = checker.check_health(&service).await;
        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_registry_service_registration() {
        let registry = ServiceRegistry::new();
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = std::sync::Arc::new(ReviewService::new(config));

        // 注册服务
        let config_value = serde_json::json!({"test": true});
        let result = registry.register_service(service.clone(), config_value).await;
        assert!(result.is_ok());

        // 验证服务已注册
        let services = registry.list_services().await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "代码评审");
        assert_eq!(services[0].status, ServiceStatus::Healthy);
    }

    #[tokio::test]
    async fn test_registry_service_retrieval() {
        let registry = ServiceRegistry::new();
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = std::sync::Arc::new(ReviewService::new(config));

        // 注册服务
        let config_value = serde_json::json!({"test": true});
        let service_id = registry.register_service(service.clone(), config_value).await.unwrap();

        // 获取服务
        let retrieved_service = registry.get_service(&service_id).await;
        assert!(retrieved_service.is_some());
        assert_eq!(retrieved_service.unwrap().name(), "代码评审");

        // 获取不存在的服务
        let non_existent_service = registry.get_service("non-existent-id").await;
        assert!(non_existent_service.is_none());
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = ServiceRegistry::new();
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = std::sync::Arc::new(ReviewService::new(config));

        // 注册服务
        let config_value = serde_json::json!({"test": true});
        let service_id = registry.register_service(service.clone(), config_value).await.unwrap();

        // 注销服务
        let result = registry.unregister_service(&service_id, "test cleanup".to_string()).await;
        assert!(result.is_ok());

        // 验证服务已移除
        let services = registry.list_services().await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_healthy_services() {
        let registry = ServiceRegistry::new();
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = std::sync::Arc::new(ReviewService::new(config));

        // 注册服务
        let config_value = serde_json::json!({"test": true});
        let _service_id = registry.register_service(service.clone(), config_value).await.unwrap();

        // 获取健康服务
        let healthy_services = registry.get_healthy_services().await;
        assert_eq!(healthy_services.len(), 1);
        assert_eq!(healthy_services[0].name, "代码评审");
        assert_eq!(healthy_services[0].status, ServiceStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checker_check_service() {
        let checker = HealthChecker::new();
        let config = std::sync::Arc::new(gitai_core::config::Config::default());
        let service = ReviewService::new(config);

        // 检查服务健康状态
        let is_healthy = checker.check_health(&service).await;
        assert!(is_healthy);
    }
}
