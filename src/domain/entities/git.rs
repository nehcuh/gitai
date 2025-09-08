//! Git相关实体

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Git提交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub parents: Vec<String>,
    pub files_changed: Vec<String>,
}

/// Git分支信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit_hash: String,
    pub is_remote: bool,
    pub is_current: bool,
    pub upstream: Option<String>,
}

/// Git标签信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit_hash: String,
    pub message: Option<String>,
    pub tagger: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Git仓库状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub branch: String,
    pub is_clean: bool,
    pub ahead: usize,
    pub behind: usize,
    pub staged_files: Vec<FileStatus>,
    pub unstaged_files: Vec<FileStatus>,
    pub untracked_files: Vec<String>,
}

/// 文件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub path: String,
    pub status: FileChangeType,
    pub additions: Option<usize>,
    pub deletions: Option<usize>,
}

/// 文件变更类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// Git差异信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    pub old_file: Option<String>,
    pub new_file: Option<String>,
    pub change_type: FileChangeType,
    pub hunks: Vec<DiffHunk>,
}

/// 差异块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

/// 差异行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

/// 差异行类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

/// Git配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub remote_url: Option<String>,
    pub branch: Option<String>,
    pub settings: HashMap<String, String>,
}