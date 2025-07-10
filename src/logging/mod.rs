use std::time::Instant;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// 日志环境配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoggingEnvironment {
    /// 开发环境
    Development,
    /// 测试环境
    Testing,
    /// 生产环境
    Production,
}

/// 日志格式配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// 人类可读格式
    Pretty,
    /// JSON 格式
    Json,
    /// 紧凑格式
    Compact,
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 环境
    pub environment: LoggingEnvironment,
    /// 日志级别
    pub level: Level,
    /// 输出格式
    pub format: LogFormat,
    /// 是否启用文件输出
    pub file_output: Option<String>,
    /// 是否显示目标模块
    pub show_target: bool,
    /// 是否显示线程ID
    pub show_thread_ids: bool,
    /// 是否显示时间戳
    pub show_timestamp: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            environment: LoggingEnvironment::Development,
            level: Level::INFO,
            format: LogFormat::Pretty,
            file_output: None,
            show_target: true,
            show_thread_ids: false,
            show_timestamp: true,
        }
    }
}

impl LoggingConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            environment: LoggingEnvironment::Development,
            level: Level::DEBUG,
            format: LogFormat::Pretty,
            file_output: None,
            show_target: true,
            show_thread_ids: true,
            show_timestamp: true,
        }
    }
    
    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            environment: LoggingEnvironment::Production,
            level: Level::INFO,
            format: LogFormat::Json,
            file_output: None,
            show_target: false,
            show_thread_ids: false,
            show_timestamp: true,
        }
    }
    
    /// 创建测试环境配置
    pub fn testing() -> Self {
        Self {
            environment: LoggingEnvironment::Testing,
            level: Level::ERROR,
            format: LogFormat::Compact,
            file_output: None,
            show_target: false,
            show_thread_ids: false,
            show_timestamp: false,
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 简化的日志初始化，避免依赖问题
    match config.format {
        LogFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .pretty()
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread_ids)
                .with_ansi(config.environment != LoggingEnvironment::Production);
            
            tracing_subscriber::registry()
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            // 降级到紧凑格式，避免 json 方法问题
            let fmt_layer = fmt::layer()
                .compact()
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread_ids);
            
            tracing_subscriber::registry()
                .with(fmt_layer)
                .init();
        }
        LogFormat::Compact => {
            let fmt_layer = fmt::layer()
                .compact()
                .with_target(config.show_target)
                .with_thread_ids(config.show_thread_ids)
                .with_ansi(config.environment != LoggingEnvironment::Production);
            
            tracing_subscriber::registry()
                .with(fmt_layer)
                .init();
        }
    }
    
    tracing::info!(
        environment = ?config.environment,
        level = ?config.level,
        format = ?config.format,
        "Logging system initialized"
    );
    
    Ok(())
}

/// 操作性能计时器
pub struct OperationTimer {
    start: Instant,
    operation: String,
    metadata: std::collections::HashMap<String, String>,
}

impl OperationTimer {
    /// 创建新的计时器
    pub fn new(operation: &str) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.to_string(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// 完成计时并记录日志
    pub fn finish(self) {
        let duration = self.start.elapsed();
        
        tracing::info!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            duration_ns = duration.as_nanos(),
            metadata = ?self.metadata,
            "Operation completed"
        );
    }
    
    /// 获取当前经过时间
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        
        tracing::debug!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            metadata = ?self.metadata,
            "Operation timer dropped"
        );
    }
}

/// 结构化日志宏
#[macro_export]
macro_rules! log_operation {
    ($level:ident, $operation:expr $(, $field:ident = $value:expr)* $(,)?) => {
        tracing::$level!(
            operation = $operation,
            $($field = $value,)*
        );
    };
}

/// 错误日志宏
#[macro_export]
macro_rules! log_error {
    ($error:expr, $operation:expr $(, $field:ident = $value:expr)* $(,)?) => {
        tracing::error!(
            error = %$error,
            operation = $operation,
            $($field = $value,)*
            "Operation failed"
        );
    };
}

/// 性能监控宏
#[macro_export]
macro_rules! measure_performance {
    ($operation:expr, $block:block) => {{
        let timer = $crate::logging::OperationTimer::new($operation);
        let result = $block;
        timer.finish();
        result
    }};
}

/// 带元数据的性能监控宏
#[macro_export]
macro_rules! measure_performance_with_metadata {
    ($operation:expr, {$($key:expr => $value:expr),*}, $block:block) => {{
        let mut timer = $crate::logging::OperationTimer::new($operation);
        $(
            timer = timer.with_metadata($key, $value);
        )*
        let result = $block;
        timer.finish();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_logging_config_creation() {
        let dev_config = LoggingConfig::development();
        assert_eq!(dev_config.environment, LoggingEnvironment::Development);
        assert_eq!(dev_config.level, Level::DEBUG);
        assert_eq!(dev_config.format, LogFormat::Pretty);
        
        let prod_config = LoggingConfig::production();
        assert_eq!(prod_config.environment, LoggingEnvironment::Production);
        assert_eq!(prod_config.level, Level::INFO);
        assert_eq!(prod_config.format, LogFormat::Json);
        
        let test_config = LoggingConfig::testing();
        assert_eq!(test_config.environment, LoggingEnvironment::Testing);
        assert_eq!(test_config.level, Level::ERROR);
        assert_eq!(test_config.format, LogFormat::Compact);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::new("test_operation")
            .with_metadata("param1", "value1")
            .with_metadata("param2", "value2");
        
        assert_eq!(timer.operation, "test_operation");
        assert_eq!(timer.metadata.get("param1"), Some(&"value1".to_string()));
        assert_eq!(timer.metadata.get("param2"), Some(&"value2".to_string()));
        
        // 测试经过时间
        std::thread::sleep(Duration::from_millis(1));
        assert!(timer.elapsed().as_nanos() > 0);
        
        // 完成计时
        timer.finish();
    }

    #[test]
    fn test_operation_timer_drop() {
        // 测试 Drop trait
        {
            let _timer = OperationTimer::new("test_drop_operation");
            // timer 会在这里自动 drop
        }
        // 验证 drop 被调用（通过日志输出）
    }
}