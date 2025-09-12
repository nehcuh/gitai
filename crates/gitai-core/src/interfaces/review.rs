//! 代码审查服务接口定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 审查类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewType {
    /// 代码质量审查
    CodeQuality,
    /// 安全审查
    Security,
    /// 性能审查
    Performance,
    /// 架构审查
    Architecture,
    /// 自定义类型
    Custom(String),
}

/// 审查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    /// 审查类型
    pub review_type: ReviewType,
    /// 指定语言（可选）
    pub language: Option<String>,
    /// 包含的路径模式
    pub include_patterns: Vec<String>,
    /// 排除的路径模式
    pub exclude_patterns: Vec<String>,
    /// 最低报告严重级别（可选）
    pub severity_threshold: Option<ReviewSeverity>,
}

/// 审查严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReviewSeverity {
    /// 提示
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 审查请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// 待审查的文件列表
    pub files: Vec<ReviewFile>,
    /// 额外上下文（可选）
    pub context: Option<HashMap<String, String>>,
    /// 审查配置
    pub config: ReviewConfig,
}

/// 审查文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFile {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: String,
    /// 语言（可选）
    pub language: Option<String>,
    /// 文件大小（字节）
    pub size: usize,
}

/// 审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 审查任务ID
    pub review_id: String,
    /// 审查类型
    pub review_type: ReviewType,
    /// 审查的文件数量
    pub files_reviewed: usize,
    /// 开始时间（unix秒）
    pub start_time: i64,
    /// 结束时间（unix秒）
    pub end_time: i64,
    /// 发现的问题
    pub issues: Vec<ReviewIssue>,
    /// 审查摘要
    pub summary: ReviewSummary,
}

/// 审查问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line_number: u32,
    /// 严重级别
    pub severity: ReviewSeverity,
    /// 问题类别
    pub category: String,
    /// 简要信息
    pub message: String,
    /// 详细描述
    pub description: String,
    /// 修复建议（可选）
    pub suggestion: Option<String>,
}

/// 审查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    /// 问题总数
    pub total_issues: usize,
    /// 按严重级别统计
    pub issues_by_severity: HashMap<ReviewSeverity, usize>,
    /// 按类别统计
    pub issues_by_category: HashMap<String, usize>,
    /// 综合评分（0-100）
    pub overall_score: f64,
}

/// 审查服务接口
#[async_trait]
pub trait ReviewService: Send + Sync {
    /// 执行代码审查
    async fn review(
        &self,
        request: ReviewRequest,
    ) -> std::result::Result<ReviewResult, crate::domain_errors::DomainError>;

    /// 批量审查
    async fn batch_review(
        &self,
        requests: Vec<ReviewRequest>,
    ) -> std::result::Result<Vec<ReviewResult>, crate::domain_errors::DomainError>;

    /// 获取审查历史
    async fn review_history(
        &self,
        limit: Option<usize>,
    ) -> std::result::Result<Vec<ReviewResult>, crate::domain_errors::DomainError>;

    /// 获取审查规则
    async fn get_rules(
        &self,
        review_type: ReviewType,
    ) -> std::result::Result<Vec<ReviewRule>, crate::domain_errors::DomainError>;
}

/// 审查规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRule {
    /// 规则唯一标识
    pub rule_id: String,
    /// 规则类别（例如：安全、风格、复杂度）
    pub category: String,
    /// 严重级别
    pub severity: ReviewSeverity,
    /// 规则描述
    pub description: String,
    /// 是否启用
    pub enabled: bool,
}

/// 审查服务提供者
#[async_trait]
pub trait ReviewProvider: Send + Sync {
    /// 创建审查服务
    fn create_service(
        &self,
        config: ReviewConfig,
    ) -> std::result::Result<Box<dyn ReviewService>, crate::domain_errors::DomainError>;

    /// 支持的审查类型
    fn supported_review_types(&self) -> Vec<ReviewType>;
}
