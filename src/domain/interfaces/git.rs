//! Git服务接口定义

use super::{ConfigurableInterface, HealthCheckInterface, VersionedInterface};
use crate::domain::entities::common::FilePath;
use crate::domain::errors::GitError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Git服务接口
#[async_trait]
pub trait GitService:
    VersionedInterface + ConfigurableInterface + HealthCheckInterface + Send + Sync
{
    /// 执行Git命令
    async fn execute_command(&self, args: &[String]) -> Result<GitCommandOutput, GitError>;

    /// 获取Git状态
    async fn get_status(&self) -> Result<GitStatus, GitError>;

    /// 获取暂存区的diff
    async fn get_staged_diff(&self) -> Result<String, GitError>;

    /// 获取工作区的diff
    async fn get_working_diff(&self) -> Result<String, GitError>;

    /// 获取所有diff（包括暂存区、工作区和未跟踪文件）
    async fn get_all_diff(&self) -> Result<String, GitError>;

    /// 获取最后一次提交的diff
    async fn get_last_commit_diff(&self) -> Result<String, GitError>;

    /// 获取两个提交之间的diff
    async fn get_commit_diff(&self, from: &str, to: &str) -> Result<String, GitError>;

    /// 获取未跟踪的文件列表
    async fn get_untracked_files(&self) -> Result<Vec<FilePath>, GitError>;

    /// 获取修改的文件列表
    async fn get_modified_files(&self) -> Result<Vec<FilePath>, GitError>;

    /// 获取提交历史
    async fn get_commit_history(
        &self,
        options: CommitHistoryOptions,
    ) -> Result<Vec<CommitInfo>, GitError>;

    /// 获取当前分支信息
    async fn get_current_branch(&self) -> Result<BranchInfo, GitError>;

    /// 获取分支列表
    async fn get_branches(&self) -> Result<Vec<BranchInfo>, GitError>;

    /// 获取标签列表
    async fn get_tags(&self) -> Result<Vec<TagInfo>, GitError>;

    /// 获取远程仓库信息
    async fn get_remotes(&self) -> Result<Vec<RemoteInfo>, GitError>;

    /// 检查是否是Git仓库
    async fn is_git_repository(&self) -> Result<bool, GitError>;

    /// 获取仓库根目录
    async fn get_repository_root(&self) -> Result<PathBuf, GitError>;

    /// 获取当前HEAD的commit hash
    async fn get_head_commit(&self) -> Result<String, GitError>;

    /// 获取文件在指定提交中的内容
    async fn get_file_content(
        &self,
        file_path: &FilePath,
        commit: &str,
    ) -> Result<String, GitError>;

    /// 获取文件的blame信息
    async fn get_file_blame(&self, file_path: &FilePath) -> Result<Vec<BlameLine>, GitError>;

    /// 获取仓库统计信息
    async fn get_repository_stats(&self) -> Result<RepositoryStats, GitError>;

    /// 检查工作区是否干净
    async fn is_working_directory_clean(&self) -> Result<bool, GitError>;

    /// 暂存文件
    async fn stage_file(&self, file_path: &FilePath) -> Result<(), GitError>;

    /// 取消暂存文件
    async fn unstage_file(&self, file_path: &FilePath) -> Result<(), GitError>;

    /// 创建提交
    async fn create_commit(
        &self,
        message: &str,
        author: Option<&GitAuthor>,
    ) -> Result<CommitInfo, GitError>;

    /// 获取配置信息
    async fn get_config(&self, key: &str) -> Result<Option<String>, GitError>;

    /// 设置配置信息
    async fn set_config(&self, key: &str, value: &str) -> Result<(), GitError>;
}

/// Git命令输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommandOutput {
    /// 命令执行结果
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
}

impl GitCommandOutput {
    /// 创建成功的命令输出
    pub fn success(stdout: String, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            stdout,
            stderr: String::new(),
            exit_code: Some(0),
            execution_time_ms,
        }
    }

    /// 创建失败的命令输出
    pub fn failure(stderr: String, exit_code: Option<i32>, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr,
            exit_code,
            execution_time_ms,
        }
    }
}

/// Git状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    /// 当前分支
    pub current_branch: Option<String>,
    /// 当前HEAD的commit hash
    pub head_commit: Option<String>,
    /// 是否有未提交的更改
    pub has_changes: bool,
    /// 暂存的文件数
    pub staged_files_count: u32,
    /// 未暂存的修改文件数
    pub modified_files_count: u32,
    /// 未跟踪的文件数
    pub untracked_files_count: u32,
    /// 冲突的文件数
    pub conflicted_files_count: u32,
    /// 工作区状态
    pub working_tree_status: WorkingTreeStatus,
    /// 远程分支跟踪信息
    pub remote_tracking: Option<RemoteTrackingInfo>,
}

/// 工作区状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkingTreeStatus {
    /// 干净（无更改）
    Clean,
    /// 有未暂存的修改
    Modified,
    /// 有暂存的修改
    Staged,
    /// 有冲突
    Conflicted,
    /// 有未跟踪的文件
    Untracked,
    /// 混合状态
    Mixed,
}

/// 远程分支跟踪信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTrackingInfo {
    /// 远程分支名称
    pub remote_branch: String,
    /// 领先远程分支的提交数
    pub ahead_count: u32,
    /// 落后远程分支的提交数
    pub behind_count: u32,
}

/// 提交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// 提交的hash
    pub hash: String,
    /// 简短的hash（前7位）
    pub short_hash: String,
    /// 作者信息
    pub author: GitAuthor,
    /// 提交者信息
    pub committer: GitAuthor,
    /// 提交时间
    pub timestamp: DateTime<Utc>,
    /// 提交消息
    pub message: String,
    /// 提交标题（第一行）
    pub title: String,
    /// 父提交的hash列表
    pub parent_hashes: Vec<String>,
    /// 修改的文件数
    pub files_changed: u32,
    /// 插入的行数
    pub insertions: u32,
    /// 删除的行数
    pub deletions: u32,
}

impl CommitInfo {
    /// 创建简化的提交信息
    pub fn simplified(&self) -> SimplifiedCommitInfo {
        SimplifiedCommitInfo {
            hash: self.hash.clone(),
            short_hash: self.short_hash.clone(),
            title: self.title.clone(),
            author_name: self.author.name.clone(),
            timestamp: self.timestamp,
        }
    }
}

/// 简化的提交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub title: String,
    pub author_name: String,
    pub timestamp: DateTime<Utc>,
}

/// Git作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAuthor {
    /// 作者名称
    pub name: String,
    /// 作者邮箱
    pub email: String,
    /// 作者时间
    pub timestamp: Option<DateTime<Utc>>,
}

impl GitAuthor {
    /// 创建新的作者
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            timestamp: None,
        }
    }

    /// 格式化为标准Git格式
    pub fn to_git_format(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}

/// 分支信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// 分支名称
    pub name: String,
    /// 是否是当前分支
    pub is_current: bool,
    /// 是否是远程分支
    pub is_remote: bool,
    /// 跟踪的远程分支
    pub upstream: Option<String>,
    /// 分支的commit hash
    pub commit_hash: String,
    /// 分支的简短消息
    pub commit_message: String,
}

/// 标签信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// 标签名称
    pub name: String,
    /// 标签指向的commit hash
    pub commit_hash: String,
    /// 标签消息
    pub message: Option<String>,
    /// 标签创建者
    pub tagger: Option<GitAuthor>,
    /// 标签创建时间
    pub timestamp: Option<DateTime<Utc>>,
    /// 是否是附注标签
    pub is_annotated: bool,
}

/// 远程仓库信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    /// 远程仓库名称
    pub name: String,
    /// 远程仓库URL
    pub url: String,
    /// 推送URL（如果与拉取URL不同）
    pub push_url: Option<String>,
    /// 跟踪的分支
    pub tracked_branches: Vec<String>,
}

/// 代码归属信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameLine {
    /// 行号
    pub line_number: u32,
    /// 行的内容
    pub content: String,
    /// 提交的hash
    pub commit_hash: String,
    /// 作者名称
    pub author_name: String,
    /// 作者邮箱
    pub author_email: String,
    /// 提交时间
    pub commit_timestamp: DateTime<Utc>,
    /// 原始行号（在父提交中）
    pub original_line_number: u32,
}

/// 提交历史查询选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitHistoryOptions {
    /// 最大返回数量
    pub max_count: Option<u32>,
    /// 跳过的提交数
    pub skip: u32,
    /// 分支名称（默认为当前分支）
    pub branch: Option<String>,
    /// 作者过滤
    pub author: Option<String>,
    /// 提交者过滤
    pub committer: Option<String>,
    /// 时间范围过滤
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    /// 是否包含统计信息
    pub include_stats: bool,
    /// 搜索关键词
    pub grep: Option<String>,
}

impl Default for CommitHistoryOptions {
    fn default() -> Self {
        Self {
            max_count: Some(10),
            skip: 0,
            branch: None,
            author: None,
            committer: None,
            since: None,
            until: None,
            include_stats: false,
            grep: None,
        }
    }
}

/// 仓库统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    /// 总提交数
    pub total_commits: u64,
    /// 总文件数
    pub total_files: u64,
    /// 总行数
    pub total_lines: u64,
    /// 贡献者数量
    pub contributor_count: u64,
    /// 分支数量
    pub branch_count: u32,
    /// 标签数量
    pub tag_count: u32,
    /// 最早提交时间
    pub first_commit_date: Option<DateTime<Utc>>,
    /// 最新提交时间
    pub last_commit_date: Option<DateTime<Utc>>,
    /// 按语言的文件统计
    pub language_stats: HashMap<String, LanguageStats>,
}

/// 语言统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    /// 文件数
    pub file_count: u32,
    /// 代码行数
    pub lines_of_code: u64,
    /// 字节数
    pub bytes: u64,
    /// 占总代码的百分比
    pub percentage: f64,
}

/// Git diff解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffInfo {
    /// 变更的文件列表
    pub files: Vec<DiffFile>,
    /// 总插入行数
    pub total_insertions: u32,
    /// 总删除行数
    pub total_deletions: u32,
    /// 变更的文件数
    pub files_changed: u32,
}

/// diff文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    /// 文件路径
    pub file_path: FilePath,
    /// 旧文件路径（重命名时）
    pub old_file_path: Option<FilePath>,
    /// 变更类型
    pub change_type: FileChangeType,
    /// 插入行数
    pub insertions: u32,
    /// 删除行数
    pub deletions: u32,
    /// 变更块列表
    pub hunks: Vec<DiffHunk>,
    /// 是否二进制文件
    pub is_binary: bool,
    /// 相似度（重命名时）
    pub similarity: Option<u8>,
}

/// 文件变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeType {
    /// 新增
    Added,
    /// 删除
    Deleted,
    /// 修改
    Modified,
    /// 重命名
    Renamed,
    /// 类型变更（模式变更）
    TypeChanged,
}

/// diff变更块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// 旧文件的起始行
    pub old_start_line: u32,
    /// 旧文件的行数
    pub old_line_count: u32,
    /// 新文件的起始行
    pub new_start_line: u32,
    /// 新文件的行数
    pub new_line_count: u32,
    /// 变更内容
    pub lines: Vec<DiffLine>,
    /// 上下文行数
    pub context_lines: u32,
}

/// diff行信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// 行内容
    pub content: String,
    /// 变更类型
    pub change_type: crate::domain::entities::common::ChangeType,
    /// 旧文件行号
    pub old_line_number: Option<u32>,
    /// 新文件行号
    pub new_line_number: Option<u32>,
}

/// Git操作选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationOptions {
    /// 是否禁用分页器
    pub no_pager: bool,
    /// 工作目录
    pub working_directory: Option<PathBuf>,
    /// 环境变量
    pub environment_variables: HashMap<String, String>,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u64>,
    /// 是否捕获输出
    pub capture_output: bool,
}

impl Default for GitOperationOptions {
    fn default() -> Self {
        Self {
            no_pager: true,
            working_directory: None,
            environment_variables: HashMap::new(),
            timeout_seconds: Some(60),
            capture_output: true,
        }
    }
}

/// Git配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfigOptions {
    /// 全局配置
    pub global: bool,
    /// 本地配置
    pub local: bool,
    /// 系统配置
    pub system: bool,
    /// 工作树配置
    pub worktree: bool,
    /// 文件路径（指定配置文件）
    pub file: Option<PathBuf>,
}

impl Default for GitConfigOptions {
    fn default() -> Self {
        Self {
            global: false,
            local: true,
            system: false,
            worktree: false,
            file: None,
        }
    }
}
