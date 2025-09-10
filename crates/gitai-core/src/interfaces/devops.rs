//! DevOps服务接口定义

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DevOps平台类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevOpsPlatform {
    GitHub,
    GitLab,
    AzureDevOps,
    Jenkins,
    Custom(String),
}

/// DevOps配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    pub platform: DevOpsPlatform,
    pub api_url: String,
    pub token: String,
    pub project_id: Option<String>,
    pub repository: Option<String>,
}

/// Issue信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: IssueStatus,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Issue状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueStatus {
    Open,
    InProgress,
    Closed,
    Resolved,
}

/// Pull Request信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: PRStatus,
    pub author: String,
    pub reviewers: Vec<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// PR状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PRStatus {
    Open,
    Merged,
    Closed,
    Draft,
}

/// 构建信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub id: String,
    pub status: BuildStatus,
    pub branch: String,
    pub commit: String,
    pub duration: Option<u64>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

/// 构建状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

/// DevOps服务接口
#[async_trait]
pub trait DevOpsService: Send + Sync {
    /// 获取Issue列表
    async fn get_issues(
        &self,
        status: Option<IssueStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<IssueInfo>, DomainError>;

    /// 获取Issue详情
    async fn get_issue(&self, issue_id: &str) -> std::result::Result<IssueInfo, crate::domain_errors::DomainError>;

    /// 更新Issue状态
    async fn update_issue_status(
        &self,
        issue_id: &str,
        status: IssueStatus,
    ) -> std::result::Result<(), crate::domain_errors::DomainError>;

    /// 获取Pull Request列表
    async fn get_pull_requests(
        &self,
        status: Option<PRStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>, DomainError>;

    /// 获取Pull Request详情
    async fn get_pull_request(&self, pr_id: &str) -> std::result::Result<PullRequestInfo, crate::domain_errors::DomainError>;

    /// 获取构建历史
    async fn get_builds(
        &self,
        branch: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<BuildInfo>, DomainError>;

    /// 触发构建
    async fn trigger_build(
        &self,
        branch: &str,
        parameters: Option<HashMap<String, String>>,
    ) -> std::result::Result<String, crate::domain_errors::DomainError>;

    /// 获取构建状态
    async fn get_build_status(&self, build_id: &str) -> std::result::Result<BuildInfo, crate::domain_errors::DomainError>;
}

/// DevOps服务提供者
#[async_trait]
pub trait DevOpsProvider: Send + Sync {
    /// 创建DevOps服务
    fn create_service(&self, config: DevOpsConfig) -> Result<Box<dyn DevOpsService>, DomainError>;

    /// 支持的DevOps平台
    fn supported_platforms(&self) -> Vec<DevOpsPlatform>;
}
