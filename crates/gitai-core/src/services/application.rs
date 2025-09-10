//! 应用服务

use gitai_types::common::{AnalysisResult, Project};
use async_trait::async_trait;

/// 应用服务接口
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// 初始化项目
    async fn initialize_project(&self, project: Project) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 分析项目
    async fn analyze_project(&self, project_id: &str) -> std::result::Result<AnalysisResult, crate::domain_errors::DomainError>;

    /// 生成项目报告
    async fn generate_report(&self, project_id: &str) -> std::result::Result<String, crate::domain_errors::DomainError>;
}
