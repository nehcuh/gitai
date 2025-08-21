use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// 扫描工具类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanTool {
    Semgrep,
    CodeQL,
    Both,
}

impl std::str::FromStr for ScanTool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "semgrep" => Ok(ScanTool::Semgrep),
            "codeql" => Ok(ScanTool::CodeQL),
            "both" => Ok(ScanTool::Both),
            _ => Err(format!("Invalid scan tool: {}", s)),
        }
    }
}

impl std::fmt::Display for ScanTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanTool::Semgrep => write!(f, "semgrep"),
            ScanTool::CodeQL => write!(f, "codeql"),
            ScanTool::Both => write!(f, "both"),
        }
    }
}

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 扫描工具
    pub tool: ScanTool,
    /// 扫描路径
    pub path: PathBuf,
    /// 是否全量扫描
    pub full_scan: bool,
    /// 是否使用远程服务
    pub remote: bool,
    /// 是否强制更新规则
    pub update_rules: bool,
    /// 输出格式
    pub output_format: OutputFormat,
    /// 启用AI翻译
    pub enable_ai_translation: bool,
    /// Semgrep专用配置
    pub semgrep_config: SemgrepConfig,
    /// CodeQL专用配置
    pub codeql_config: CodeQLConfig,
}

/// 输出格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Text,
    Markdown,
    Sarif,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "text" => Ok(OutputFormat::Text),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "sarif" => Ok(OutputFormat::Sarif),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// Semgrep配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemgrepConfig {
    /// 规则路径
    pub rules_path: PathBuf,
    /// 扫描深度
    pub depth: String,
    /// 并发数
    pub concurrency: usize,
    /// 排除模式
    pub exclude_patterns: Vec<String>,
    /// 超时时间（秒）
    pub timeout: u64,
}

/// CodeQL配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQLConfig {
    /// 标准库路径
    pub standard_library_path: PathBuf,
    /// 数据库创建超时（分钟）
    pub database_timeout: u64,
    /// 查询超时（分钟）
    pub query_timeout: u64,
    /// 仅安全查询
    pub security_only: bool,
    /// 内存限制（MB）
    pub memory_limit: usize,
}

/// 扫描请求
#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub config: ScanConfig,
    pub language_filter: Option<String>,
    pub focus_areas: Option<Vec<String>>,
}

/// 扫描进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub stage: ScanStage,
    pub progress: f32,
    pub message: String,
    pub tool: Option<String>, // 工具名称字符串
}

/// 扫描阶段
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanStage {
    Initializing,
    DownloadingRules,
    BuildingDatabase,
    RunningScan,
    AnalyzingResults,
    GeneratingReport,
    Completed,
    Failed(String),
}

/// 扫描统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files: usize,
    pub scanned_files: usize,
    pub findings_count: usize,
    pub high_severity: usize,
    pub medium_severity: usize,
    pub low_severity: usize,
    pub scan_duration_seconds: f64,
    pub tools_used: Vec<ScanTool>,
}