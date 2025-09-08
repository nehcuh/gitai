//! 应用服务

use async_trait::async_trait;
use crate::domain::errors::DomainError;
use crate::domain::entities::common::{Project, AnalysisResult};

/// 应用服务接口
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// 初始化项目
    async fn initialize_project(&self, project: Project) -> Result<(), DomainError>;
    
    /// 分析项目
    async fn analyze_project(&self, project_id: &str) -> Result<AnalysisResult, DomainError>;
    
    /// 生成项目报告
    async fn generate_report(&self, project_id: &str) -> Result<String, DomainError>;
}