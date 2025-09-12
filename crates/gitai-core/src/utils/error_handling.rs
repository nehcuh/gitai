//! 统一错误处理工具模块
//!
//! 提供安全的错误处理模式，替代 .unwrap() 调用

use gitai_types::error::*;

// Define Result type aliases for domain-specific errors
type ConfigResult<T> = std::result::Result<T, ConfigError>;
type GitResult<T> = std::result::Result<T, GitError>;
type AiResult<T> = std::result::Result<T, AiError>;
type ScanResult<T> = std::result::Result<T, ScanError>;

// Import GitAIError and Result from gitai-types instead of crate::error
use gitai_types::error::{GitAIError, Result};

/// 安全的 Result 处理工具
pub struct SafeResult;

impl SafeResult {
    /// 安全地处理 Result，提供默认值和错误日志
    pub fn unwrap_or_log<T, E>(result: std::result::Result<T, E>, default: T, context: &str) -> T
    where
        E: std::fmt::Display,
    {
        match result {
            Ok(value) => value,
            Err(e) => {
                log::warn!("{} failed: {}", context, e);
                default
            }
        }
    }

    /// 安全地处理 Result，提供错误回调
    pub fn unwrap_or_else_log<T, E, F>(
        result: std::result::Result<T, E>,
        fallback: F,
        context: &str,
    ) -> T
    where
        E: std::fmt::Display,
        F: FnOnce() -> T,
    {
        match result {
            Ok(value) => value,
            Err(e) => {
                log::warn!("{} failed: {}", context, e);
                fallback()
            }
        }
    }

    /// 安全地处理 Option，提供默认值和日志
    pub fn some_or_log<T>(option: Option<T>, default: T, context: &str) -> T {
        match option {
            Some(value) => value,
            None => {
                log::warn!("{}: Option is None", context);
                default
            }
        }
    }

    /// 安全地处理 Option，提供错误回调
    pub fn some_or_else_log<T, F>(option: Option<T>, fallback: F, context: &str) -> T
    where
        F: FnOnce() -> T,
    {
        match option {
            Some(value) => value,
            None => {
                log::warn!("{}: Option is None", context);
                fallback()
            }
        }
    }

    /// 将 Result 转换为 GitAIError::Other
    pub fn convert_unknown<T, E>(result: std::result::Result<T, E>, context: &str) -> Result<T>
    where
        E: std::fmt::Display,
    {
        result.map_err(|e| {
            let error_msg = format!("{}: {}", context, e);
            log::error!("{}", error_msg);
            GitAIError::Other(error_msg)
        })
    }

    /// 将 Result 转换为 GitAIError::Other，提供上下文
    pub fn convert_unknown_with<T, E>(
        result: std::result::Result<T, E>,
        context: &str,
        operation: &str,
    ) -> Result<T>
    where
        E: std::fmt::Display,
    {
        result.map_err(|e| {
            let error_msg = format!("{} in {}: {}", context, operation, e);
            log::error!("{}", error_msg);
            GitAIError::Other(error_msg)
        })
    }
}

/// 领域特定的错误处理工具
pub struct DomainErrorHandler;

impl DomainErrorHandler {
    /// 处理配置错误
    pub fn handle_config_error<T>(
        result: std::result::Result<T, impl std::fmt::Display>,
        operation: &str,
    ) -> ConfigResult<T> {
        result.map_err(|e| {
            log::error!("Config operation '{}' failed: {}", operation, e);
            ConfigError::LoadFailed(format!("{}: {}", operation, e))
        })
    }

    /// 处理 Git 错误
    pub fn handle_git_error<T>(
        result: std::result::Result<T, impl std::fmt::Display>,
        operation: &str,
    ) -> GitResult<T> {
        result.map_err(|e| {
            log::error!("Git operation '{}' failed: {}", operation, e);
            GitError::CommandFailed(format!("{}: {}", operation, e))
        })
    }

    /// 处理 AI 错误
    pub fn handle_ai_error<T>(
        result: std::result::Result<T, impl std::fmt::Display>,
        operation: &str,
    ) -> AiResult<T> {
        result.map_err(|e| {
            log::error!("AI operation '{}' failed: {}", operation, e);
            AiError::ApiCallFailed(format!("{}: {}", operation, e))
        })
    }

    /// 处理扫描错误
    pub fn handle_scan_error<T>(
        result: std::result::Result<T, impl std::fmt::Display>,
        operation: &str,
    ) -> ScanResult<T> {
        result.map_err(|e| {
            log::error!("Scan operation '{}' failed: {}", operation, e);
            ScanError::ScanExecutionFailed(format!("{}: {}", operation, e))
        })
    }
}

/// 针对常见场景的便捷宏
#[macro_export]
macro_rules! safe_unwrap {
    ($result:expr, $default:expr, $context:expr) => {
        $crate::utils::error_handling::SafeResult::unwrap_or_log($result, $default, $context)
    };
}

#[macro_export]
macro_rules! safe_unwrap_or_else {
    ($result:expr, $fallback:expr, $context:expr) => {
        $crate::utils::error_handling::SafeResult::unwrap_or_else_log($result, $fallback, $context)
    };
}

#[macro_export]
macro_rules! safe_some {
    ($option:expr, $default:expr, $context:expr) => {
        $crate::utils::error_handling::SafeResult::some_or_log($option, $default, $context)
    };
}

#[macro_export]
macro_rules! safe_some_or_else {
    ($option:expr, $fallback:expr, $context:expr) => {
        $crate::utils::error_handling::SafeResult::some_or_else_log($option, $fallback, $context)
    };
}

#[macro_export]
macro_rules! convert_error {
    ($result:expr, $context:expr) => {
        $crate::utils::error_handling::SafeResult::convert_unknown($result, $context)
    };
}

/// 针对特定操作类型的错误处理宏
#[macro_export]
macro_rules! handle_config_error {
    ($result:expr, $operation:expr) => {
        $crate::utils::error_handling::DomainErrorHandler::handle_config_error($result, $operation)
    };
}

#[macro_export]
macro_rules! handle_git_error {
    ($result:expr, $operation:expr) => {
        $crate::utils::error_handling::DomainErrorHandler::handle_git_error($result, $operation)
    };
}

#[macro_export]
macro_rules! handle_ai_error {
    ($result:expr, $operation:expr) => {
        $crate::utils::error_handling::DomainErrorHandler::handle_ai_error($result, $operation)
    };
}

#[macro_export]
macro_rules! handle_scan_error {
    ($result:expr, $operation:expr) => {
        $crate::utils::error_handling::DomainErrorHandler::handle_scan_error($result, $operation)
    };
}

/// 特定场景的便捷函数
pub mod convenience {

    /// 安全解析数字字符串
    pub fn safe_parse_number<T>(s: &str, default: T, context: &str) -> T
    where
        T: std::str::FromStr + std::fmt::Display,
    {
        s.trim().parse().unwrap_or_else(|_| {
            log::warn!(
                "{}: Failed to parse '{}', using default {}",
                context,
                s,
                default
            );
            default
        })
    }

    /// 安全获取字符串切片
    pub fn safe_str_slice(
        s: &str,
        start: usize,
        end: usize,
        default: &str,
        context: &str,
    ) -> String {
        s.get(start..end).map(|s| s.to_string()).unwrap_or_else(|| {
            log::warn!(
                "{}: Invalid slice range {}..{} for string '{}'",
                context,
                start,
                end,
                s
            );
            default.to_string()
        })
    }

    /// 安全获取 JSON 值
    pub fn safe_json_value<T>(json: &serde_json::Value, key: &str, default: T, context: &str) -> T
    where
        T: serde::de::DeserializeOwned + std::fmt::Display,
    {
        json.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| {
                log::warn!(
                    "{}: Failed to get or parse key '{}' from JSON",
                    context,
                    key
                );
                default
            })
    }

    /// 安全获取 JSON 数值
    pub fn safe_json_number(
        json: &serde_json::Value,
        key: &str,
        default: u64,
        context: &str,
    ) -> u64 {
        json.get(key).and_then(|v| v.as_u64()).unwrap_or_else(|| {
            log::warn!("{}: Failed to get number key '{}' from JSON", context, key);
            default
        })
    }

    /// 安全获取 JSON 字符串
    pub fn safe_json_string(
        json: &serde_json::Value,
        key: &str,
        default: &str,
        context: &str,
    ) -> String {
        json.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                log::warn!("{}: Failed to get string key '{}' from JSON", context, key);
                default.to_string()
            })
    }

    /// 安全获取 JSON 布尔值
    pub fn safe_json_bool(
        json: &serde_json::Value,
        key: &str,
        default: bool,
        context: &str,
    ) -> bool {
        json.get(key).and_then(|v| v.as_bool()).unwrap_or_else(|| {
            log::warn!("{}: Failed to get bool key '{}' from JSON", context, key);
            default
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_safe_unwrap_or_log() {
        let result: std::result::Result<i32, &str> = Ok(42);
        assert_eq!(
            crate::utils::error_handling::SafeResult::unwrap_or_log(result, 0, "test"),
            42
        );

        let result: std::result::Result<i32, &str> = Err("error");
        assert_eq!(
            crate::utils::error_handling::SafeResult::unwrap_or_log(result, 0, "test"),
            0
        );
    }

    #[test]
    fn test_safe_some_or_log() {
        let option = Some(42);
        assert_eq!(
            crate::utils::error_handling::SafeResult::some_or_log(option, 0, "test"),
            42
        );

        let option: Option<i32> = None;
        assert_eq!(
            crate::utils::error_handling::SafeResult::some_or_log(option, 0, "test"),
            0
        );
    }

    #[test]
    fn test_convenience_safe_parse_number() {
        assert_eq!(
            crate::utils::error_handling::convenience::safe_parse_number("42", 0, "test"),
            42
        );
        assert_eq!(
            crate::utils::error_handling::convenience::safe_parse_number("abc", 0, "test"),
            0
        );
    }
}
