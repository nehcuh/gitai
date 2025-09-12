//! DevOps adapter traits and models

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Supported DevOps platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevOpsPlatform {
    /// Coding.net
    Coding,
    /// GitHub
    GitHub,
    /// GitLab
    GitLab,
    /// Azure DevOps
    Azure,
    /// Custom string id
    Custom(String),
}

/// Basic issue info returned by adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    /// Issue ID
    pub id: String,
    /// Issue 标题
    pub title: String,
    /// Issue 描述
    pub description: String,
    /// Issue 状态
    pub status: String,
    /// 指派人
    pub assignee: Option<String>,
    /// 标签列表
    pub labels: Vec<String>,
    /// Issue URL
    pub url: Option<String>,
}

/// Basic PR info returned by adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    /// PR ID
    pub id: String,
    /// PR 标题
    pub title: String,
    /// PR 状态
    pub status: String,
    /// 作者
    pub author: String,
    /// 审查者列表
    pub reviewers: Vec<String>,
    /// 源分支
    pub source_branch: String,
    /// 目标分支
    pub target_branch: String,
    /// PR URL
    pub url: Option<String>,
}

/// DevOps provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    /// DevOps 平台
    pub platform: DevOpsPlatform,
    /// 基础 URL
    pub base_url: String,
    /// 访问令牌
    pub token: String,
    /// 项目名称
    pub project: Option<String>,
    /// 仓库名称
    pub repository: Option<String>,
}

/// Adapter trait for DevOps providers
#[async_trait]
pub trait DevOpsAdapter: Send + Sync {
    /// provider name (e.g., coding, github)
    fn name(&self) -> &'static str;

    /// get single issue by id (e.g., #123)
    async fn get_issue(&self, id: &str) -> anyhow::Result<IssueInfo>;

    /// search issues by query string
    async fn search_issues(&self, query: &str, limit: usize) -> anyhow::Result<Vec<IssueInfo>>;

    /// get pull request by id
    async fn get_pull_request(&self, id: &str) -> anyhow::Result<PullRequestInfo>;
}
