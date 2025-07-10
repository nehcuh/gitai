use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// 错误严重程度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 严重错误 - 应用程序无法继续
    Critical,
    /// 高级错误 - 功能无法完成
    High,
    /// 中等错误 - 功能受限但可以继续
    Medium,
    /// 低级错误 - 轻微影响
    Low,
}

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 配置错误
    Configuration,
    /// 网络错误
    Network,
    /// 认证错误
    Authentication,
    /// 业务逻辑错误
    Business,
    /// 系统错误
    System,
    /// 外部服务错误
    External,
}

/// 错误上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// 操作名称
    pub operation: String,
    /// 发生位置
    pub location: String,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 附加信息
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(operation: &str, location: &str) -> Self {
        Self {
            operation: operation.to_string(),
            location: location.to_string(),
            timestamp: SystemTime::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// 结构化错误消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// 错误代码
    pub code: String,
    /// 用户友好的错误消息
    pub message: String,
    /// 详细信息（仅用于调试）
    pub details: Option<String>,
    /// 错误严重程度
    pub severity: ErrorSeverity,
    /// 错误分类
    pub category: ErrorCategory,
    /// 上下文信息
    pub context: Option<ErrorContext>,
}

impl ErrorMessage {
    /// 创建新的错误消息
    pub fn new(
        code: &str,
        message: &str,
        severity: ErrorSeverity,
        category: ErrorCategory,
    ) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            severity,
            category,
            context: None,
        }
    }
    
    /// 添加详细信息
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
    
    /// 添加上下文
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }
}

/// 错误码定义
pub mod error_codes {
    // 配置错误
    pub const CONFIG_FILE_NOT_FOUND: &str = "CONFIG_001";
    pub const CONFIG_PARSE_ERROR: &str = "CONFIG_002";
    pub const CONFIG_VALIDATION_ERROR: &str = "CONFIG_003";
    
    // Git 错误
    pub const GIT_COMMAND_FAILED: &str = "GIT_001";
    pub const GIT_NOT_REPOSITORY: &str = "GIT_002";
    pub const GIT_NO_STAGED_CHANGES: &str = "GIT_003";
    
    // AI 错误
    pub const AI_REQUEST_FAILED: &str = "AI_001";
    pub const AI_RESPONSE_PARSE_ERROR: &str = "AI_002";
    pub const AI_API_ERROR: &str = "AI_003";
    pub const AI_EMPTY_RESPONSE: &str = "AI_004";
    
    // TreeSitter 错误
    pub const TREESITTER_UNSUPPORTED_LANGUAGE: &str = "TS_001";
    pub const TREESITTER_PARSE_ERROR: &str = "TS_002";
    pub const TREESITTER_QUERY_ERROR: &str = "TS_003";
    
    // DevOps 错误
    pub const DEVOPS_NETWORK_ERROR: &str = "DEVOPS_001";
    pub const DEVOPS_AUTH_ERROR: &str = "DEVOPS_002";
    pub const DEVOPS_NOT_FOUND: &str = "DEVOPS_003";
    pub const DEVOPS_RATE_LIMIT: &str = "DEVOPS_004";
    
    // MCP 错误
    pub const MCP_CONNECTION_ERROR: &str = "MCP_001";
    pub const MCP_PROTOCOL_ERROR: &str = "MCP_002";
    pub const MCP_SERVICE_ERROR: &str = "MCP_003";
    
    // 系统错误
    pub const SYSTEM_IO_ERROR: &str = "SYS_001";
    pub const SYSTEM_NETWORK_ERROR: &str = "SYS_002";
    pub const SYSTEM_TIMEOUT: &str = "SYS_003";
}

/// 创建配置错误的便捷函数
pub fn config_error(code: &str, message: &str) -> ErrorMessage {
    ErrorMessage::new(code, message, ErrorSeverity::High, ErrorCategory::Configuration)
}

/// 创建网络错误的便捷函数
pub fn network_error(code: &str, message: &str) -> ErrorMessage {
    ErrorMessage::new(code, message, ErrorSeverity::Medium, ErrorCategory::Network)
}

/// 创建认证错误的便捷函数
pub fn auth_error(code: &str, message: &str) -> ErrorMessage {
    ErrorMessage::new(code, message, ErrorSeverity::High, ErrorCategory::Authentication)
}

/// 创建业务逻辑错误的便捷函数
pub fn business_error(code: &str, message: &str) -> ErrorMessage {
    ErrorMessage::new(code, message, ErrorSeverity::Medium, ErrorCategory::Business)
}

/// 创建系统错误的便捷函数
pub fn system_error(code: &str, message: &str) -> ErrorMessage {
    ErrorMessage::new(code, message, ErrorSeverity::Critical, ErrorCategory::System)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("git_commit", "src/handlers/git.rs:123")
            .with_metadata("command", "git commit -m 'test'")
            .with_metadata("working_dir", "/tmp/test");
        
        assert_eq!(context.operation, "git_commit");
        assert_eq!(context.location, "src/handlers/git.rs:123");
        assert_eq!(context.metadata.get("command"), Some(&"git commit -m 'test'".to_string()));
        assert_eq!(context.metadata.get("working_dir"), Some(&"/tmp/test".to_string()));
    }

    #[test]
    fn test_error_message_creation() {
        let error = ErrorMessage::new(
            "TEST_001",
            "Test error message",
            ErrorSeverity::High,
            ErrorCategory::Business,
        )
        .with_details("Additional details about the error")
        .with_context(ErrorContext::new("test_operation", "test_location"));
        
        assert_eq!(error.code, "TEST_001");
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.severity, ErrorSeverity::High);
        assert_eq!(error.category, ErrorCategory::Business);
        assert!(error.details.is_some());
        assert!(error.context.is_some());
    }

    #[test]
    fn test_convenience_functions() {
        let config_err = config_error("CONFIG_001", "Config file not found");
        assert_eq!(config_err.category, ErrorCategory::Configuration);
        assert_eq!(config_err.severity, ErrorSeverity::High);
        
        let network_err = network_error("NET_001", "Network timeout");
        assert_eq!(network_err.category, ErrorCategory::Network);
        assert_eq!(network_err.severity, ErrorSeverity::Medium);
        
        let auth_err = auth_error("AUTH_001", "Invalid credentials");
        assert_eq!(auth_err.category, ErrorCategory::Authentication);
        assert_eq!(auth_err.severity, ErrorSeverity::High);
    }

    #[test]
    fn test_error_serialization() {
        let error = ErrorMessage::new(
            "TEST_001",
            "Test error",
            ErrorSeverity::Medium,
            ErrorCategory::Business,
        );
        
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ErrorMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.severity, deserialized.severity);
        assert_eq!(error.category, deserialized.category);
    }
}