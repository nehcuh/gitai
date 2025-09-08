//! 容器适配器 - 桥接新容器和现有代码
//! 
//! 提供向后兼容性，允许渐进式迁移

use super::v2::{ServiceContainer as NewContainer, ContainerError as NewContainerError};
use crate::domain::interfaces::{
    ConfigProvider, GitService, AiService, CacheService,
    config::{AiConfig, ScanConfig, DevOpsConfig, McpConfig, CacheConfig, LoggingConfig, FeatureFlags},
};
use std::sync::Arc;
use std::any::Any;

/// 容器适配器 - 提供统一的接口
pub struct ContainerAdapter {
    /// 新容器实例
    new_container: Arc<NewContainer>,
    /// 迁移特性开关
    migration_flags: Arc<std::sync::RwLock<MigrationFlags>>,
}

/// 迁移特性标志
#[derive(Debug, Clone)]
pub struct MigrationFlags {
    /// 是否启用新容器
    pub use_new_container: bool,
    /// 是否启用模块化配置
    pub use_modular_config: bool,
    /// 是否启用结构化错误
    pub use_structured_errors: bool,
}

impl Default for MigrationFlags {
    fn default() -> Self {
        Self {
            use_new_container: false,  // 默认关闭，确保向后兼容
            use_modular_config: false,
            use_structured_errors: false,
        }
    }
}

impl ContainerAdapter {
    /// 创建新的适配器
    pub fn new() -> Self {
        Self {
            new_container: Arc::new(NewContainer::new()),
            migration_flags: Arc::new(std::sync::RwLock::new(MigrationFlags::default())),
        }
    }
    
    /// 注册默认服务
    pub async fn register_default_services(&self, 
        config: Arc<crate::config::Config>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync> {
        let flags = self.migration_flags.read().unwrap();
        
        if flags.use_new_container {
            self.register_services_new(config).await
        } else {
            self.register_services_legacy(config).await
        }
    }
    
    /// 使用新容器注册服务
    async fn register_services_new(&self, 
        config: Arc<crate::config::Config>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync> {
        let container = self.new_container.clone();
        
        // 注册配置提供者
        if flags.use_modular_config {
            container.register(move |_| {
                let config_provider = crate::infrastructure::config::ModularConfigProvider::new(config.clone());
                Ok(config_provider) as Result<_, Box<dyn std::error::Error + Send + Sync>>
            });
        }
        
        // 注册Git服务
        container.register(move |_| {
            let git_service = crate::infrastructure::git::GitServiceImpl::new(config.clone());
            Ok(git_service) as Result<_, Box<dyn std::error::Error + Send + Sync>>
        });
        
        // 注册AI服务
        container.register(move |_| {
            let ai_service = crate::infrastructure::ai::AiServiceImpl::new(config.clone());
            Ok(ai_service) as Result<_, Box<dyn std::error::Error + Send + Sync>>
        });
        
        // 注册缓存服务
        container.register(move |_| {
            let cache_service = crate::infrastructure::cache::CacheServiceImpl::new(config.clone());
            Ok(cache_service) as Result<_, Box<dyn std::error::Error + Send + Sync>>
        });
        
        Ok(())
    }
    
    /// 使用旧方式注册服务（保持兼容）
    async fn register_services_legacy(&self, 
        _config: Arc<crate::config::Config>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync> {
        // 旧的服务注册逻辑 - 暂时留空，保持现有代码不变
        log::info!("Using legacy service registration");
        Ok(())
    }
    
    /// 获取服务 - 统一的接口
    pub async fn resolve<T: Send + Sync + 'static>(&self
    ) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync> {
        let flags = self.migration_flags.read().unwrap();
        
        if flags.use_new_container {
            self.new_container.resolve::<T>().await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        } else {
            // 使用旧的解析逻辑
            Err("Legacy service resolution not implemented".into())
        }
    }
    
    /// 更新迁移标志
    pub fn update_migration_flags<F>(&self, updater: F
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync> 
    where 
        F: FnOnce(&mut MigrationFlags) -> Result<(), Box<dyn std::error::Error + Send + Sync>
    {
        let mut flags = self.migration_flags.write().unwrap();
        updater(&mut flags)
    }
    
    /// 获取当前迁移标志
    pub fn get_migration_flags(&self) -> MigrationFlags {
        self.migration_flags.read().unwrap().clone()
    }
    
    /// 渐进式启用新特性
    pub fn enable_next_feature(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync> {
        let mut flags = self.migration_flags.write().unwrap();
        
        if !flags.use_new_container {
            flags.use_new_container = true;
            log::info!("Enabled new container");
            return Ok(true);
        }
        
        if !flags.use_modular_config {
            flags.use_modular_config = true;
            log::info!("Enabled modular config");
            return Ok(true);
        }
        
        if !flags.use_structured_errors {
            flags.use_structured_errors = true;
            log::info!("Enabled structured errors");
            return Ok(true);
        }
        
        Ok(false) // 所有特性都已启用
    }
    
    /// 获取容器统计信息
    pub fn get_container_stats(&self
    ) -> Result<super::v2::ContainerStats, Box<dyn std::error::Error + Send + Sync> {
        Ok(self.new_container.get_stats())
    }
    
    /// 检查服务是否已注册
    pub fn is_service_registered<T: 'static>(&self
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync> {
        Ok(self.new_container.is_registered::<T>())
    }
}

impl Default for ContainerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷的全局容器实例
lazy_static::lazy_static! {
    static ref GLOBAL_CONTAINER: Arc<ContainerAdapter> = Arc::new(ContainerAdapter::new());
}

/// 获取全局容器
pub fn global_container() -> Arc<ContainerAdapter> {
    GLOBAL_CONTAINER.clone()
}

/// 在全局容器中注册服务
pub async fn register_global_service<T, F>(factory: F
) -> Result<(), Box<dyn std::error::Error + Send + Sync>
where
    F: Fn(&ServiceContainer) -> T + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    let container = global_container();
    container.new_container.register(factory);
    Ok(())
}

/// 从全局容器解析服务
pub async fn resolve_global_service<T: Send + Sync + 'static>(
) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync> {
    let container = global_container();
    container.resolve::<T>().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    struct TestService {
        value: i32,
    }
    
    #[tokio::test]
    async fn test_adapter_basic_functionality() {
        let adapter = ContainerAdapter::new();
        
        // 注册服务
        adapter.new_container.register(|_| Ok(TestService { value: 42 }));
        
        // 解析服务
        let service = adapter.resolve::<TestService>().await.unwrap();
        assert_eq!(service.value, 42);
    }
    
    #[tokio::test]
    async fn test_migration_flags() {
        let adapter = ContainerAdapter::new();
        
        // 初始状态
        let flags = adapter.get_migration_flags();
        assert!(!flags.use_new_container);
        assert!(!flags.use_modular_config);
        assert!(!flags.use_structured_errors);
        
        // 启用第一个特性
        assert!(adapter.enable_next_feature().unwrap());
        let flags = adapter.get_migration_flags();
        assert!(flags.use_new_container);
        assert!(!flags.use_modular_config);
        
        // 启用第二个特性
        assert!(adapter.enable_next_feature().unwrap());
        let flags = adapter.get_migration_flags();
        assert!(flags.use_new_container);
        assert!(flags.use_modular_config);
        
        // 启用第三个特性
        assert!(adapter.enable_next_feature().unwrap());
        let flags = adapter.get_migration_flags();
        assert!(flags.use_structured_errors);
        
        // 没有更多特性可以启用
        assert!(!adapter.enable_next_feature().unwrap());
    }
    
    #[tokio::test]
    async fn test_global_container() {
        // 注册全局服务
        register_global_service(|_| Ok(TestService { value: 100 })).await.unwrap();
        
        // 从全局容器解析
        let service = resolve_global_service::<TestService>().await.unwrap();
        assert_eq!(service.value, 100);
    }
}