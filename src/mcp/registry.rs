use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::mcp::rmcp_compat::ServiceError;

/// 简化的 MCP 服务注册表（适配 rmcp 库）
#[derive(Debug)]
pub struct McpServiceRegistry {
    /// 已注册的服务
    registered_services: Arc<RwLock<HashMap<String, ServiceRegistration>>>,
}

/// 服务注册信息
#[derive(Debug, Clone)]
pub struct ServiceRegistration {
    /// 服务名称
    pub name: String,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 是否活跃
    pub active: bool,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
}

impl McpServiceRegistry {
    /// 创建新的服务注册表
    pub fn new() -> Self {
        Self {
            registered_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册服务
    pub async fn register(&self, service_name: String) -> Result<(), ServiceError> {
        let registration = ServiceRegistration {
            name: service_name.clone(),
            registered_at: chrono::Utc::now(),
            active: true,
            metadata: HashMap::new(),
        };

        let mut services = self.registered_services.write().await;
        services.insert(service_name.clone(), registration);
        
        tracing::info!("服务 '{}' 已注册", service_name);
        Ok(())
    }

    /// 注销服务
    pub async fn unregister(&self, service_name: &str) -> Result<(), ServiceError> {
        let mut services = self.registered_services.write().await;
        if services.remove(service_name).is_some() {
            tracing::info!("服务 '{}' 已注销", service_name);
            Ok(())
        } else {
            Err(ServiceError::invalid_params(format!("服务 '{}' 未找到", service_name)))
        }
    }

    /// 检查服务是否已注册
    pub async fn is_registered(&self, service_name: &str) -> bool {
        let services = self.registered_services.read().await;
        services.contains_key(service_name)
    }

    /// 列出所有注册的服务
    pub async fn list_services(&self) -> Vec<ServiceRegistration> {
        let services = self.registered_services.read().await;
        services.values().cloned().collect()
    }

    /// 获取服务注册信息
    pub async fn get_service(&self, service_name: &str) -> Option<ServiceRegistration> {
        let services = self.registered_services.read().await;
        services.get(service_name).cloned()
    }

    /// 更新服务状态
    pub async fn update_service_status(&self, service_name: &str, active: bool) -> Result<(), ServiceError> {
        let mut services = self.registered_services.write().await;
        if let Some(registration) = services.get_mut(service_name) {
            registration.active = active;
            tracing::debug!("服务 '{}' 状态更新为: {}", service_name, active);
            Ok(())
        } else {
            Err(ServiceError::invalid_params(format!("服务 '{}' 未找到", service_name)))
        }
    }

    /// 获取活跃服务列表
    pub async fn get_active_services(&self) -> Vec<String> {
        let services = self.registered_services.read().await;
        services.values()
            .filter(|s| s.active)
            .map(|s| s.name.clone())
            .collect()
    }
}

impl Default for McpServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let registry = McpServiceRegistry::new();
        
        // 注册服务
        let result = registry.register("test-service".to_string()).await;
        assert!(result.is_ok());
        
        // 检查服务是否已注册
        assert!(registry.is_registered("test-service").await);
        
        // 列出服务
        let services = registry.list_services().await;
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        
        // 注销服务
        let result = registry.unregister("test-service").await;
        assert!(result.is_ok());
        
        // 检查服务是否已注销
        assert!(!registry.is_registered("test-service").await);
    }

    #[tokio::test]
    async fn test_service_status_update() {
        let registry = McpServiceRegistry::new();
        
        // 注册服务
        registry.register("test-service".to_string()).await.unwrap();
        
        // 初始状态应该是活跃的
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 1);
        
        // 更新状态为非活跃
        registry.update_service_status("test-service", false).await.unwrap();
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 0);
        
        // 更新状态为活跃
        registry.update_service_status("test-service", true).await.unwrap();
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_service() {
        let registry = McpServiceRegistry::new();
        
        // 尝试注销不存在的服务
        let result = registry.unregister("nonexistent-service").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_services() {
        let registry = McpServiceRegistry::new();
        
        // 注册多个服务
        registry.register("service-1".to_string()).await.unwrap();
        registry.register("service-2".to_string()).await.unwrap();
        registry.register("service-3".to_string()).await.unwrap();
        
        // 检查服务数量
        let services = registry.list_services().await;
        assert_eq!(services.len(), 3);
        
        // 检查活跃服务
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 3);
        
        // 停用一个服务
        registry.update_service_status("service-2", false).await.unwrap();
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 2);
        
        // 注销一个服务
        registry.unregister("service-3").await.unwrap();
        let services = registry.list_services().await;
        assert_eq!(services.len(), 2);
        let active_services = registry.get_active_services().await;
        assert_eq!(active_services.len(), 1);
    }
}