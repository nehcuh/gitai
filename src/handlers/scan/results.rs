use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::types::scan::types::*;

/// 扫描发现的问题
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// 唯一标识符
    pub id: String,
    /// 问题标题
    pub title: String,
    /// 问题描述
    pub description: String,
    /// 严重程度
    pub severity: Severity,
    /// 问题类型
    pub rule_type: RuleType,
    /// 规则ID
    pub rule_id: String,
    /// 源工具
    pub source_tool: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 位置信息
    pub location: Location,
    /// 代码片段
    pub code_snippet: Option<CodeSnippet>,
    /// 修复建议
    pub fix_suggestions: Vec<FixSuggestion>,
    /// 相关标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 严重程度
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" | "high" => Ok(Severity::Error),
            "warning" | "medium" => Ok(Severity::Warning),
            "info" | "low" => Ok(Severity::Info),
            "style" => Ok(Severity::Style),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Info => write!(f, "INFO"),
            Severity::Style => write!(f, "STYLE"),
        }
    }
}

/// 规则类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    Security,
    Performance,
    Correctness,
    Maintainability,
    Complexity,
    Style,
    BestPractice,
    Custom(String),
}

/// 代码位置
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// 起始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 起始列号
    pub start_column: Option<usize>,
    /// 结束列号
    pub end_column: Option<usize>,
}

/// 代码片段
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// 代码内容
    pub content: String,
    /// 高亮范围
    pub highlight_range: Option<Location>,
    /// 上下文行数
    pub context_lines: usize,
}

/// 修复建议
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixSuggestion {
    /// 建议描述
    pub description: String,
    /// 修复代码
    pub fix_code: Option<String>,
    /// 置信度
    pub confidence: Confidence,
    /// 是否自动修复
    pub auto_fixable: bool,
}

/// 置信度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 扫描ID
    pub scan_id: String,
    /// 扫描时间
    pub scan_time: chrono::DateTime<chrono::Utc>,
    /// 扫描配置
    pub config_hash: String,
    /// 发现的问题
    pub findings: Vec<Finding>,
    /// 统计信息
    pub stats: ScanStats,
    /// 工具结果
    pub tool_results: HashMap<String, ToolResult>,
    /// 扫描状态
    pub status: ScanStatus,
}

/// 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 工具名称
    pub tool_name: String,
    /// 工具版本
    pub tool_version: String,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 原始输出
    pub raw_output: String,
    /// 解析状态
    pub parse_status: ParseStatus,
    /// 错误信息
    pub error: Option<String>,
}

/// 解析状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseStatus {
    Success,
    PartialSuccess,
    Failed,
}

/// 扫描状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanStatus {
    Completed,
    PartiallyCompleted,
    Failed(String),
    Cancelled,
}

/// 汇总报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    /// 基本信息
    pub summary: ScanSummary,
    /// 按文件分组的结果
    pub file_results: HashMap<PathBuf, FileScanResult>,
    /// 按严重程度分组
    pub severity_breakdown: HashMap<Severity, Vec<Finding>>,
    /// 按类型分组
    pub type_breakdown: HashMap<RuleType, Vec<Finding>>,
    /// 建议
    pub recommendations: Vec<Recommendation>,
}

/// 扫描摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    /// 扫描ID
    pub scan_id: String,
    /// 扫描时间
    pub scan_time: chrono::DateTime<chrono::Utc>,
    /// 项目名称
    pub project_name: String,
    /// 扫描路径
    pub scan_path: PathBuf,
    /// 使用的工具
    pub tools_used: Vec<String>,
    /// 总文件数
    pub total_files: usize,
    /// 发现问题数
    pub total_findings: usize,
    /// 严重程度分布
    pub severity_counts: HashMap<Severity, usize>,
    /// 扫描状态
    pub status: ScanStatus,
}

/// 文件扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScanResult {
    /// 文件路径
    pub file_path: PathBuf,
    /// 文件类型
    pub file_type: String,
    /// 该文件的问题
    pub findings: Vec<Finding>,
    /// 文件统计
    pub stats: FileStats,
}

/// 文件统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    /// 代码行数
    pub lines_of_code: usize,
    /// 问题密度（每千行代码的问题数）
    pub finding_density: f64,
    /// 最高严重程度
    pub max_severity: Option<Severity>,
}

/// 建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// 建议优先级
    pub priority: Priority,
    /// 建议标题
    pub title: String,
    /// 建议描述
    pub description: String,
    /// 相关问题
    pub related_findings: Vec<String>,
    /// 预估工作量（人天）
    pub estimated_effort: Option<f64>,
}

/// 优先级
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}