//! 代码审查相关实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代码审查请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub files: Vec<ReviewFile>,
    pub created_at: DateTime<Utc>,
    pub status: ReviewStatus,
    pub reviewer: Option<String>,
}

/// 审查文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFile {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub size: usize,
    pub additions: usize,
    pub deletions: usize,
}

/// 审查状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Rejected,
}

/// 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub id: String,
    pub request_id: String,
    pub reviewer: String,
    pub completed_at: DateTime<Utc>,
    pub overall_score: f64,
    pub status: ReviewStatus,
    pub issues: Vec<ReviewIssue>,
    pub summary: ReviewSummary,
}

/// 审查问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub id: String,
    pub file_path: String,
    pub line_number: u32,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub description: String,
    pub suggestion: Option<String>,
    pub rule_id: Option<String>,
}

/// 问题严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 问题类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    CodeQuality,
    Security,
    Performance,
    Maintainability,
    Style,
    Documentation,
    Custom(String),
}

/// 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_files: usize,
    pub files_with_issues: usize,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<IssueSeverity, usize>,
    pub issues_by_category: HashMap<IssueCategory, usize>,
    pub recommendations: Vec<String>,
}

/// 审查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub enabled: bool,
    pub configuration: HashMap<String, serde_json::Value>,
}

/// 审查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfiguration {
    pub rules: Vec<ReviewRule>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub severity_threshold: IssueSeverity,
    pub auto_review_enabled: bool,
}
