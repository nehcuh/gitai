//! 配置服务

use crate::domain::entities::common::Configuration;
use crate::domain::errors::DomainError;
use async_trait::async_trait;

/// 配置服务接口
#[async_trait]
pub trait ConfigurationService: Send + Sync {
    /// 加载配置
    async fn load_config(&self, path: &str) -> Result<Configuration, DomainError>;

    /// 保存配置
    async fn save_config(&self, config: &Configuration, path: &str) -> Result<(), DomainError>;

    /// 验证配置
    async fn validate_config(&self, config: &Configuration) -> Result<Vec<String>, DomainError>;

    /// 更新配置
    async fn update_config(
        &self,
        updates: &[(String, String)],
    ) -> Result<Configuration, DomainError>;
}
