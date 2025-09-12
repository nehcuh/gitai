//! GitAI MCP Error types and handling - 简化版本

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// GitAI MCP 错误类型
#[derive(Debug, Clone)]
pub enum McpError {
    /// 参数验证错误
    InvalidParameters(String),
    /// 服务执行错误
    ExecutionFailed(String),
    /// 配置错误
    ConfigurationError(String),
    /// 文件操作错误
    FileOperationError(String),
    /// 网络错误
    NetworkError(String),
    /// 外部工具错误
    ExternalToolError(String),
    /// 权限错误
    PermissionError(String),
    /// 超时错误
    TimeoutError(String),
    /// 未知错误
    Unknown(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::InvalidParameters(msg) => write!(f, "Invalid parameters: {msg}"),
            McpError::ExecutionFailed(msg) => write!(f, "Execution failed: {msg}"),
            McpError::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            McpError::FileOperationError(msg) => write!(f, "File operation error: {msg}"),
            McpError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            McpError::ExternalToolError(msg) => write!(f, "External tool error: {msg}"),
            McpError::PermissionError(msg) => write!(f, "Permission error: {msg}"),
            McpError::TimeoutError(msg) => write!(f, "Timeout error: {msg}"),
            McpError::Unknown(msg) => write!(f, "Unknown error: {msg}"),
        }
    }
}

impl std::error::Error for McpError {}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::InvalidParameters(format!("JSON parsing error: {err}"))
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                McpError::FileOperationError(format!("File not found: {err}"))
            }
            std::io::ErrorKind::PermissionDenied => {
                McpError::PermissionError(format!("Permission denied: {err}"))
            }
            std::io::ErrorKind::TimedOut => McpError::TimeoutError(format!("Timeout: {err}")),
            _ => McpError::FileOperationError(format!("IO error: {err}")),
        }
    }
}

impl From<tokio::time::error::Elapsed> for McpError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        McpError::TimeoutError(format!("Operation timeout: {err}"))
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for McpError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        McpError::ExternalToolError(format!("External service error: {err}"))
    }
}

/// 类型别名
pub type McpResult<T> = Result<T, McpError>;

// 错误创建辅助函数
/// 构造 参数无效 错误
pub fn invalid_parameters_error<T: Into<String>>(msg: T) -> McpError {
    McpError::InvalidParameters(msg.into())
}

/// 构造 执行失败 错误
pub fn execution_failed_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ExecutionFailed(msg.into())
}

/// 构造 配置错误 错误
pub fn configuration_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ConfigurationError(msg.into())
}

/// 构造 文件操作错误 错误
pub fn file_operation_error<T: Into<String>>(msg: T) -> McpError {
    McpError::FileOperationError(msg.into())
}

/// 构造 网络错误 错误
pub fn network_error<T: Into<String>>(msg: T) -> McpError {
    McpError::NetworkError(msg.into())
}

/// 构造 外部工具错误 错误
pub fn external_tool_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ExternalToolError(msg.into())
}

/// 构造 权限错误 错误
pub fn permission_error<T: Into<String>>(msg: T) -> McpError {
    McpError::PermissionError(msg.into())
}

/// 构造 超时 错误
pub fn timeout_error<T: Into<String>>(msg: T) -> McpError {
    McpError::TimeoutError(msg.into())
}

/// 构造 未知错误 错误
pub fn unknown_error<T: Into<String>>(msg: T) -> McpError {
    McpError::Unknown(msg.into())
}

/// 性能统计结构
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// 工具调用次数
    pub tool_calls: u64,
    /// 成功的工具调用次数
    pub successful_calls: u64,
    /// 失败的工具调用次数
    pub failed_calls: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    /// 各工具的调用统计
    pub tool_stats: HashMap<String, ToolStats>,
}

/// 单个工具的统计信息
#[derive(Debug, Clone)]
pub struct ToolStats {
    /// 调用次数
    pub calls: u64,
    /// 成功次数
    pub successful_calls: u64,
    /// 失败次数
    pub failed_calls: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 最短执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最长执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
}

/// 性能统计收集器
#[derive(Debug)]
pub struct PerformanceCollector {
    /// 总调用次数
    total_calls: AtomicU64,
    /// 成功调用次数
    successful_calls: AtomicU64,
    /// 失败调用次数
    failed_calls: AtomicU64,
    /// 总执行时间（毫秒）
    total_execution_time_ms: AtomicU64,
    /// 各工具的统计信息
    tool_stats: Arc<parking_lot::RwLock<HashMap<String, ToolStats>>>,
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            successful_calls: AtomicU64::new(0),
            failed_calls: AtomicU64::new(0),
            total_execution_time_ms: AtomicU64::new(0),
            tool_stats: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
}

impl PerformanceCollector {
    /// 创建新的性能统计收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录工具调用开始
    pub fn record_call_start(&self, _tool_name: &str) -> Instant {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        Instant::now()
    }

    /// 记录工具调用成功
    pub fn record_call_success(&self, tool_name: &str, execution_time_ms: u64) {
        self.successful_calls.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms
            .fetch_add(execution_time_ms, Ordering::Relaxed);

        let mut tool_stats = self.tool_stats.write();
        let stats = tool_stats
            .entry(tool_name.to_string())
            .or_insert(ToolStats {
                calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
                average_execution_time_ms: 0.0,
            });

        stats.calls += 1;
        stats.successful_calls += 1;
        stats.total_execution_time_ms += execution_time_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(execution_time_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(execution_time_ms);
        stats.average_execution_time_ms = stats.total_execution_time_ms as f64 / stats.calls as f64;
    }

    /// 记录工具调用失败
    pub fn record_call_failure(&self, tool_name: &str, execution_time_ms: u64) {
        self.failed_calls.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms
            .fetch_add(execution_time_ms, Ordering::Relaxed);

        let mut tool_stats = self.tool_stats.write();
        let stats = tool_stats
            .entry(tool_name.to_string())
            .or_insert(ToolStats {
                calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
                average_execution_time_ms: 0.0,
            });

        stats.calls += 1;
        stats.failed_calls += 1;
        stats.total_execution_time_ms += execution_time_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(execution_time_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(execution_time_ms);
        stats.average_execution_time_ms = stats.total_execution_time_ms as f64 / stats.calls as f64;
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> PerformanceStats {
        let total_calls = self.total_calls.load(Ordering::Relaxed);
        let successful_calls = self.successful_calls.load(Ordering::Relaxed);
        let failed_calls = self.failed_calls.load(Ordering::Relaxed);
        let total_execution_time_ms = self.total_execution_time_ms.load(Ordering::Relaxed);

        let average_execution_time_ms = if total_calls > 0 {
            total_execution_time_ms as f64 / total_calls as f64
        } else {
            0.0
        };

        let tool_stats = self.tool_stats.read().clone();

        PerformanceStats {
            tool_calls: total_calls,
            successful_calls,
            failed_calls,
            total_execution_time_ms,
            average_execution_time_ms,
            tool_stats,
        }
    }

    /// 重置统计信息
    pub fn reset(&self) {
        self.total_calls.store(0, Ordering::Relaxed);
        self.successful_calls.store(0, Ordering::Relaxed);
        self.failed_calls.store(0, Ordering::Relaxed);
        self.total_execution_time_ms.store(0, Ordering::Relaxed);
        self.tool_stats.write().clear();
    }
}

// =============================================================================
// MCP Error Conversion Helpers - Eliminate repetition following Linus's taste
// =============================================================================

/// Convert parameter parsing error to MCP error
/// Linus principle: eliminate the pattern "Failed to parse XXX parameters: {}"
pub fn parse_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    invalid_parameters_error(format!("Failed to parse {service_name} parameters: {e}"))
}

/// Convert execution error to MCP error
/// Linus principle: eliminate the pattern "XXX execution failed: {}"
pub fn execution_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    execution_failed_error(format!("{service_name} execution failed: {e}"))
}

/// Convert serialization error to MCP error
/// Linus principle: eliminate the pattern "Failed to serialize XXX result: {}"
pub fn serialize_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    execution_failed_error(format!("Failed to serialize {service_name} result: {e}"))
}
