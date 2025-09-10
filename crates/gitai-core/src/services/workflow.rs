//! 工作流服务

use gitai_types::common::Workflow;
use crate::domain_errors::DomainError;
use async_trait::async_trait;

/// 工作流服务接口
#[async_trait]
pub trait WorkflowService: Send + Sync {
    /// 执行工作流
    async fn execute_workflow(&self, workflow: Workflow) -> std::result::Result<String, crate::domain_errors::DomainError>;

    /// 验证工作流
    async fn validate_workflow(&self, workflow: &Workflow) -> Result<Vec<String>, DomainError>;

    /// 停止工作流
    async fn stop_workflow(&self, workflow_id: &str) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 获取工作流状态
    async fn get_workflow_status(&self, workflow_id: &str) -> std::result::Result<Workflow, crate::domain_errors::DomainError>;
}
