//! 代码审查服务接口定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::domain::errors::DomainError;

/// 审查类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewType {
    CodeQuality,
    Security,
    Performance,
    Architecture,
    Custom(String),
}

/// 审查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    pub review_type: ReviewType,
    pub language: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub severity_threshold: Option<ReviewSeverity>,
}

/// 审查严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReviewSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 审查请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub files: Vec<ReviewFile>,
    pub context: Option<HashMap<String, String>>,
    pub config: ReviewConfig,
}

/// 审查文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFile {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub size: usize,
}

/// 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub review_id: String,
    pub review_type: ReviewType,
    pub files_reviewed: usize,
    pub start_time: i64,
    pub end_time: i64,
    pub issues: Vec<ReviewIssue>,
    pub summary: ReviewSummary,
}

/// 审查问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub file_path: String,
    pub line_number: u32,
    pub severity: ReviewSeverity,
    pub category: String,
    pub message: String,
    pub description: String,
    pub suggestion: Option<String>,
}

/// 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_issues: usize,
    pub issues_by_severity: HashMap<ReviewSeverity, usize>,
    pub issues_by_category: HashMap<String, usize>,
    pub overall_score: f64,
}

/// 审查服务接口
#[async_trait]
pub trait ReviewService: Send + Sync {
    /// 执行代码审查
    async fn review(&self, request: ReviewRequest) -> Result<ReviewResult, DomainError>;
    
    /// 批量审查
    async fn batch_review(&self, requests: Vec<ReviewRequest>) -> Result<Vec<ReviewResult>, DomainError>;
    
    /// 获取审查历史
    async fn review_history(&self, limit: Option<usize>) -> Result<Vec<ReviewResult>, DomainError>;
    
    /// 获取审查规则
    async fn get_rules(&self, review_type: ReviewType) -> Result<Vec<ReviewRule>, DomainError>;
}

/// 审查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRule {
    pub rule_id: String,
    pub category: String,
    pub severity: ReviewSeverity,
    pub description: String,
    pub enabled: bool,
}

/// 审查服务提供者
#[async_trait]
pub trait ReviewProvider: Send + Sync {
    /// 创建审查服务
    fn create_service(&self, config: ReviewConfig) -> Result<Box<dyn ReviewService>, DomainError>;
    
    /// 支持的审查类型
    fn supported_review_types(&self) -> Vec<ReviewType>;
}