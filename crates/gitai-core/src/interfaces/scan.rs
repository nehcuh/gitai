//! 安全扫描服务接口定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 扫描类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanType {
    /// 安全漏洞扫描
    Security,
    /// 依赖漏洞/版本扫描
    Dependency,
    /// 许可证合规扫描
    License,
    /// 自定义扫描类型
    Custom(String),
}

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 扫描类型
    pub scan_type: ScanType,
    /// 目标路径（文件或目录）
    pub target_path: String,
    /// 规则文件或目录（可选）
    pub rules_path: Option<String>,
    /// 包含的路径模式
    pub include_patterns: Vec<String>,
    /// 排除的路径模式
    pub exclude_patterns: Vec<String>,
    /// 最低报告严重级别（可选）
    pub severity_threshold: Option<Severity>,
}

/// 严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 扫描任务ID
    pub scan_id: String,
    /// 扫描类型
    pub scan_type: ScanType,
    /// 扫描目标路径
    pub target_path: String,
    /// 开始时间（unix秒）
    pub start_time: i64,
    /// 结束时间（unix秒）
    pub end_time: i64,
    /// 发现的问题列表
    pub issues: Vec<ScanIssue>,
    /// 扫描摘要
    pub summary: ScanSummary,
}

/// 扫描问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanIssue {
    /// 触发的规则ID
    pub rule_id: String,
    /// 严重级别
    pub severity: Severity,
    /// 文件路径
    pub file_path: String,
    /// 行号
    pub line_number: u32,
    /// 列号
    pub column_number: u32,
    /// 简要信息
    pub message: String,
    /// 详细描述
    pub description: String,
    /// 修复建议（可选）
    pub remediation: Option<String>,
}

/// 扫描摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    /// 扫描的文件总数
    pub total_files: usize,
    /// 存在问题的文件数
    pub files_with_issues: usize,
    /// 发现的问题总数
    pub total_issues: usize,
    /// 按严重级别统计的问题数
    pub issues_by_severity: HashMap<Severity, usize>,
}

/// 扫描服务接口
#[async_trait]
pub trait ScanService: Send + Sync {
    /// 执行扫描
    async fn scan(
        &self,
        config: ScanConfig,
    ) -> std::result::Result<ScanResult, crate::domain_errors::DomainError>;

    /// 批量扫描
    async fn batch_scan(
        &self,
        configs: Vec<ScanConfig>,
    ) -> std::result::Result<Vec<ScanResult>, crate::domain_errors::DomainError>;

    /// 获取扫描历史
    async fn scan_history(
        &self,
        limit: Option<usize>,
    ) -> std::result::Result<Vec<ScanResult>, crate::domain_errors::DomainError>;

    /// 检查扫描规则
    async fn validate_rules(
        &self,
        rules_path: &str,
    ) -> std::result::Result<Vec<RuleValidation>, crate::domain_errors::DomainError>;
}

/// 规则验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleValidation {
    /// 规则ID
    pub rule_id: String,
    /// 是否有效
    pub is_valid: bool,
    /// 错误信息（当无效时）
    pub error_message: Option<String>,
}

/// 扫描服务提供者
#[async_trait]
pub trait ScanProvider: Send + Sync {
    /// 创建扫描服务
    fn create_service(
        &self,
        config: ScanConfig,
    ) -> std::result::Result<Box<dyn ScanService>, crate::domain_errors::DomainError>;

    /// 支持的扫描类型
    fn supported_scan_types(&self) -> Vec<ScanType>;
}
