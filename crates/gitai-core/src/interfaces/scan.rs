//! 安全扫描服务接口定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 扫描类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanType {
    Security,
    Dependency,
    License,
    Custom(String),
}

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub scan_type: ScanType,
    pub target_path: String,
    pub rules_path: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub severity_threshold: Option<Severity>,
}

/// 严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub scan_type: ScanType,
    pub target_path: String,
    pub start_time: i64,
    pub end_time: i64,
    pub issues: Vec<ScanIssue>,
    pub summary: ScanSummary,
}

/// 扫描问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanIssue {
    pub rule_id: String,
    pub severity: Severity,
    pub file_path: String,
    pub line_number: u32,
    pub column_number: u32,
    pub message: String,
    pub description: String,
    pub remediation: Option<String>,
}

/// 扫描摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_files: usize,
    pub files_with_issues: usize,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<Severity, usize>,
}

/// 扫描服务接口
#[async_trait]
pub trait ScanService: Send + Sync {
    /// 执行扫描
    async fn scan(&self, config: ScanConfig) -> std::result::Result<ScanResult, crate::domain_errors::DomainError>;

    /// 批量扫描
    async fn batch_scan(&self, configs: Vec<ScanConfig>) -> std::result::Result<Vec<ScanResult>, crate::domain_errors::DomainError>;

    /// 获取扫描历史
    async fn scan_history(&self, limit: Option<usize>) -> std::result::Result<Vec<ScanResult>, crate::domain_errors::DomainError>;

    /// 检查扫描规则
    async fn validate_rules(&self, rules_path: &str) -> std::result::Result<Vec<RuleValidation>, crate::domain_errors::DomainError>;
}

/// 规则验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleValidation {
    pub rule_id: String,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

/// 扫描服务提供者
#[async_trait]
pub trait ScanProvider: Send + Sync {
    /// 创建扫描服务
    fn create_service(&self, config: ScanConfig) -> std::result::Result<Box<dyn ScanService>, crate::domain_errors::DomainError>;

    /// 支持的扫描类型
    fn supported_scan_types(&self) -> Vec<ScanType>;
}
