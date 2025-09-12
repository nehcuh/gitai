//! 配置服务

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use gitai_types::common::Configuration;

/// 配置服务接口
#[async_trait]
pub trait ConfigurationService: Send + Sync {
    /// 加载配置
    async fn load_config(
        &self,
        path: &str,
    ) -> std::result::Result<Configuration, crate::domain_errors::DomainError>;

    /// 保存配置
    async fn save_config(
        &self,
        config: &Configuration,
        path: &str,
    ) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 验证配置
    async fn validate_config(&self, config: &Configuration) -> Result<Vec<String>, DomainError>;

    /// 更新配置
    async fn update_config(
        &self,
        updates: &[(String, String)],
    ) -> std::result::Result<Configuration, crate::domain_errors::DomainError>;
}
