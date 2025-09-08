//! 服务提供者实现
//! 
//! 为各种服务提供具体的实例化逻辑

use super::container::{ServiceContainer, ContainerError};
use crate::config::Config;
use std::sync::Arc;

/// 配置提供者
pub struct ConfigProvider {
    config: Arc<Config>,
}

impl ConfigProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

/// Git服务提供者
pub struct GitServiceProvider {
    config: Arc<Config>,
}

impl GitServiceProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

/// AI服务提供者  
pub struct AiServiceProvider {
    config: Arc<Config>,
}

impl AiServiceProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

/// 缓存服务提供者
pub struct CacheServiceProvider {
    config: Arc<Config>,
}

impl CacheServiceProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

/// 服务提供者工厂
pub struct ProviderFactory {
    config: Arc<Config>,
}

impl ProviderFactory {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
    
    /// 注册所有默认服务提供者
    pub async fn register_default_providers(&self, container: &ServiceContainer
    ) -> Result<(), ContainerError> {
        // 注册配置提供者
        // v2: use simple registration closures
        container.register_singleton_simple(|| Ok(ConfigProvider::new(self.config.clone()))).await;
        
        // 注册Git服务提供者
        container.register_singleton_simple(|| Ok(GitServiceProvider::new(self.config.clone()))).await;
        
        // 注册AI服务提供者
        container.register_singleton_simple(|| Ok(AiServiceProvider::new(self.config.clone()))).await;
        
        // 注册缓存服务提供者
        container.register_singleton_simple(|| Ok(CacheServiceProvider::new(self.config.clone()))).await;
        
        Ok(())
    }
    
    /// 注册特定环境的服务提供者
    pub async fn register_environment_providers(&self, 
        container: &ServiceContainer,
        environment: &str
    ) -> Result<(), ContainerError> {
        match environment {
            "test" => {
                // 注册测试用的mock服务
                self.register_test_providers(container).await?;
            }
            "development" => {
                // 注册开发环境的调试服务
                self.register_development_providers(container).await?;
            }
            "production" => {
                // 注册生产环境的高性能服务
                self.register_production_providers(container).await?;
            }
            _ => {
                return Err(ContainerError::CreationFailed(
                    format!("Unknown environment: {}", environment)
                ));
            }
        }
        
        Ok(())
    }
    
    async fn register_test_providers(&self, container: &ServiceContainer
    ) -> Result<(), ContainerError> {
        // TODO: 注册测试专用的mock服务
        Ok(())
    }
    
    async fn register_development_providers(&self, container: &ServiceContainer
    ) -> Result<(), ContainerError> {
        // TODO: 注册开发环境的调试服务
        Ok(())
    }
    
    async fn register_production_providers(&self, container: &ServiceContainer
    ) -> Result<(), ContainerError> {
        // TODO: 注册生产环境的高性能服务
        Ok(())
    }
}