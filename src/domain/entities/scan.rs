//! 安全扫描相关实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub id: String,
    pub name: String,
    pub scan_type: ScanType,
    pub target_path: String,
    pub rules_path: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub severity_threshold: Severity,
    pub enabled: bool,
}

/// 扫描类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanType {
    Security,
    Dependency,
    License,
    CodeQuality,
    Custom(String),
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
    pub id: String,
    pub config_id: String,
    pub scan_type: ScanType,
    pub target_path: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: ScanStatus,
    pub issues: Vec<ScanIssue>,
    pub summary: ScanSummary,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 扫描状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 扫描问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanIssue {
    pub id: String,
    pub rule_id: String,
    pub severity: Severity,
    pub file_path: String,
    pub line_number: u32,
    pub column_number: u32,
    pub message: String,
    pub description: String,
    pub remediation: Option<String>,
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
}

/// 扫描摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_files: usize,
    pub files_with_issues: usize,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<Severity, usize>,
    pub issues_by_rule: HashMap<String, usize>,
    pub scan_duration: u64,
}

/// 扫描规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub category: String,
    pub enabled: bool,
    pub language: Option<String>,
    pub pattern: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 扫描规则集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRuleSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<ScanRule>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 扫描统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatistics {
    pub total_scans: usize,
    pub successful_scans: usize,
    pub failed_scans: usize,
    pub total_issues_found: usize,
    pub average_scan_duration: u64,
    pub issues_by_severity: HashMap<Severity, usize>,
}
