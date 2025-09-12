//! DevOps服务接口定义

use crate::domain_errors::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DevOps平台类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevOpsPlatform {
    /// GitHub 平台
    GitHub,
    /// GitLab 平台
    GitLab,
    /// Azure DevOps 平台
    AzureDevOps,
    /// Jenkins 平台
    Jenkins,
    /// 自定义平台（自定义标识）
    Custom(String),
}

/// DevOps配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    /// 目标 DevOps 平台
    pub platform: DevOpsPlatform,
    /// API 基础 URL
    pub api_url: String,
    /// 访问令牌
    pub token: String,
    /// 项目或空间标识
    pub project_id: Option<String>,
    /// 仓库名称（组织/仓库）
    pub repository: Option<String>,
}

/// Issue信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    /// Issue 唯一标识
    pub id: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 当前状态
    pub status: IssueStatus,
    /// 负责人
    pub assignee: Option<String>,
    /// 标签
    pub labels: Vec<String>,
    /// 创建时间（Unix 时间戳）
    pub created_at: i64,
    /// 更新时间（Unix 时间戳）
    pub updated_at: i64,
}

/// Issue状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueStatus {
    /// 待处理
    Open,
    /// 进行中
    InProgress,
    /// 已关闭
    Closed,
    /// 已解决
    Resolved,
}

/// Pull Request信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    /// PR 唯一标识
    pub id: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 当前状态
    pub status: PRStatus,
    /// 作者
    pub author: String,
    /// 评审者列表
    pub reviewers: Vec<String>,
    /// 源分支
    pub source_branch: String,
    /// 目标分支
    pub target_branch: String,
    /// 创建时间（Unix 时间戳）
    pub created_at: i64,
    /// 更新时间（Unix 时间戳）
    pub updated_at: i64,
}

/// PR状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PRStatus {
    /// 开启
    Open,
    /// 已合并
    Merged,
    /// 已关闭
    Closed,
    /// 草稿
    Draft,
}

/// 构建信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// 构建 ID
    pub id: String,
    /// 构建状态
    pub status: BuildStatus,
    /// 构建分支
    pub branch: String,
    /// 构建提交哈希
    pub commit: String,
    /// 构建耗时（秒）
    pub duration: Option<u64>,
    /// 开始时间（Unix 时间戳）
    pub started_at: i64,
    /// 结束时间（Unix 时间戳）
    pub finished_at: Option<i64>,
}

/// 构建状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuildStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 已取消
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
    async fn get_issue(
        &self,
        issue_id: &str,
    ) -> std::result::Result<IssueInfo, crate::domain_errors::DomainError>;

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
    async fn get_pull_request(
        &self,
        pr_id: &str,
    ) -> std::result::Result<PullRequestInfo, crate::domain_errors::DomainError>;

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
    async fn get_build_status(
        &self,
        build_id: &str,
    ) -> std::result::Result<BuildInfo, crate::domain_errors::DomainError>;
}

/// DevOps服务提供者
#[async_trait]
pub trait DevOpsProvider: Send + Sync {
    /// 创建DevOps服务
    fn create_service(&self, config: DevOpsConfig) -> Result<Box<dyn DevOpsService>, DomainError>;

    /// 支持的DevOps平台
    fn supported_platforms(&self) -> Vec<DevOpsPlatform>;
}
