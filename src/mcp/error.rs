use std::fmt;
use serde::{Deserialize, Serialize};

/// MCP 服务错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpError {
    /// 服务连接错误
    ConnectionError {
        service: String,
        message: String,
    },
    /// 服务未找到
    ServiceNotFound(String),
    /// 服务不可用
    ServiceUnavailable {
        service: String,
        reason: String,
    },
    /// 请求超时
    Timeout {
        service: String,
        duration: u64,
    },
    /// 序列化/反序列化错误
    SerializationError {
        message: String,
    },
    /// 工具调用错误
    ToolCallError {
        service: String,
        tool: String,
        message: String,
    },
    /// 资源访问错误
    ResourceError {
        service: String,
        resource: String,
        message: String,
    },
    /// 认证错误
    AuthenticationError {
        service: String,
        message: String,
    },
    /// 授权错误
    AuthorizationError {
        service: String,
        message: String,
    },
    /// 配置错误
    ConfigError {
        message: String,
    },
    /// 网络错误
    NetworkError {
        message: String,
    },
    /// 协议错误
    ProtocolError {
        message: String,
    },
    /// 内部错误
    InternalError {
        message: String,
    },
    /// 其他错误
    Other {
        message: String,
    },
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::ConnectionError { service, message } => {
                write!(f, "连接服务 '{}' 失败: {}", service, message)
            }
            McpError::ServiceNotFound(service) => {
                write!(f, "服务 '{}' 未找到", service)
            }
            McpError::ServiceUnavailable { service, reason } => {
                write!(f, "服务 '{}' 不可用: {}", service, reason)
            }
            McpError::Timeout { service, duration } => {
                write!(f, "服务 '{}' 请求超时 ({}s)", service, duration)
            }
            McpError::SerializationError { message } => {
                write!(f, "序列化错误: {}", message)
            }
            McpError::ToolCallError { service, tool, message } => {
                write!(f, "服务 '{}' 工具 '{}' 调用失败: {}", service, tool, message)
            }
            McpError::ResourceError { service, resource, message } => {
                write!(f, "服务 '{}' 资源 '{}' 访问失败: {}", service, resource, message)
            }
            McpError::AuthenticationError { service, message } => {
                write!(f, "服务 '{}' 认证失败: {}", service, message)
            }
            McpError::AuthorizationError { service, message } => {
                write!(f, "服务 '{}' 授权失败: {}", service, message)
            }
            McpError::ConfigError { message } => {
                write!(f, "配置错误: {}", message)
            }
            McpError::NetworkError { message } => {
                write!(f, "网络错误: {}", message)
            }
            McpError::ProtocolError { message } => {
                write!(f, "协议错误: {}", message)
            }
            McpError::InternalError { message } => {
                write!(f, "内部错误: {}", message)
            }
            McpError::Other { message } => {
                write!(f, "其他错误: {}", message)
            }
        }
    }
}

impl std::error::Error for McpError {}

impl McpError {
    /// 创建连接错误
    pub fn connection_error(service: &str, message: &str) -> Self {
        Self::ConnectionError {
            service: service.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建服务未找到错误
    pub fn service_not_found(service: &str) -> Self {
        Self::ServiceNotFound(service.to_string())
    }

    /// 创建服务不可用错误
    pub fn service_unavailable(service: &str, reason: &str) -> Self {
        Self::ServiceUnavailable {
            service: service.to_string(),
            reason: reason.to_string(),
        }
    }

    /// 创建超时错误
    pub fn timeout(service: &str, duration: u64) -> Self {
        Self::Timeout {
            service: service.to_string(),
            duration,
        }
    }

    /// 创建序列化错误
    pub fn serialization_error(message: &str) -> Self {
        Self::SerializationError {
            message: message.to_string(),
        }
    }

    /// 创建工具调用错误
    pub fn tool_call_error(service: &str, tool: &str, message: &str) -> Self {
        Self::ToolCallError {
            service: service.to_string(),
            tool: tool.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建资源访问错误
    pub fn resource_error(service: &str, resource: &str, message: &str) -> Self {
        Self::ResourceError {
            service: service.to_string(),
            resource: resource.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建认证错误
    pub fn authentication_error(service: &str, message: &str) -> Self {
        Self::AuthenticationError {
            service: service.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建授权错误
    pub fn authorization_error(service: &str, message: &str) -> Self {
        Self::AuthorizationError {
            service: service.to_string(),
            message: message.to_string(),
        }
    }

    /// 创建配置错误
    pub fn config_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
        }
    }

    /// 创建网络错误
    pub fn network_error(message: &str) -> Self {
        Self::NetworkError {
            message: message.to_string(),
        }
    }

    /// 创建协议错误
    pub fn protocol_error(message: &str) -> Self {
        Self::ProtocolError {
            message: message.to_string(),
        }
    }

    /// 创建内部错误
    pub fn internal_error(message: &str) -> Self {
        Self::InternalError {
            message: message.to_string(),
        }
    }

    /// 创建其他错误
    pub fn other(message: &str) -> Self {
        Self::Other {
            message: message.to_string(),
        }
    }

    /// 获取错误的服务名称（如果有）
    pub fn service_name(&self) -> Option<&str> {
        match self {
            McpError::ConnectionError { service, .. } => Some(service),
            McpError::ServiceNotFound(service) => Some(service),
            McpError::ServiceUnavailable { service, .. } => Some(service),
            McpError::Timeout { service, .. } => Some(service),
            McpError::ToolCallError { service, .. } => Some(service),
            McpError::ResourceError { service, .. } => Some(service),
            McpError::AuthenticationError { service, .. } => Some(service),
            McpError::AuthorizationError { service, .. } => Some(service),
            _ => None,
        }
    }

    /// 检查是否为可重试错误
    pub fn is_retryable(&self) -> bool {
        match self {
            McpError::ConnectionError { .. } => true,
            McpError::ServiceUnavailable { .. } => true,
            McpError::Timeout { .. } => true,
            McpError::NetworkError { .. } => true,
            McpError::InternalError { .. } => true,
            _ => false,
        }
    }

    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            McpError::ConnectionError { .. } => ErrorSeverity::High,
            McpError::ServiceNotFound(_) => ErrorSeverity::High,
            McpError::ServiceUnavailable { .. } => ErrorSeverity::Medium,
            McpError::Timeout { .. } => ErrorSeverity::Medium,
            McpError::SerializationError { .. } => ErrorSeverity::Medium,
            McpError::ToolCallError { .. } => ErrorSeverity::Medium,
            McpError::ResourceError { .. } => ErrorSeverity::Medium,
            McpError::AuthenticationError { .. } => ErrorSeverity::High,
            McpError::AuthorizationError { .. } => ErrorSeverity::High,
            McpError::ConfigError { .. } => ErrorSeverity::High,
            McpError::NetworkError { .. } => ErrorSeverity::Medium,
            McpError::ProtocolError { .. } => ErrorSeverity::High,
            McpError::InternalError { .. } => ErrorSeverity::High,
            McpError::Other { .. } => ErrorSeverity::Low,
        }
    }
}

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 低严重程度
    Low,
    /// 中等严重程度
    Medium,
    /// 高严重程度
    High,
    /// 严重
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "低"),
            ErrorSeverity::Medium => write!(f, "中"),
            ErrorSeverity::High => write!(f, "高"),
            ErrorSeverity::Critical => write!(f, "严重"),
        }
    }
}

/// 错误结果类型别名
pub type McpResult<T> = Result<T, McpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = McpError::connection_error("test-service", "连接失败");
        assert!(matches!(err, McpError::ConnectionError { .. }));
        assert_eq!(err.service_name(), Some("test-service"));
        assert!(err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_service_not_found() {
        let err = McpError::service_not_found("unknown-service");
        assert!(matches!(err, McpError::ServiceNotFound(_)));
        assert_eq!(err.service_name(), Some("unknown-service"));
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_timeout_error() {
        let err = McpError::timeout("slow-service", 30);
        assert!(matches!(err, McpError::Timeout { .. }));
        assert_eq!(err.service_name(), Some("slow-service"));
        assert!(err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_tool_call_error() {
        let err = McpError::tool_call_error("ai-service", "analyze", "参数错误");
        assert!(matches!(err, McpError::ToolCallError { .. }));
        assert_eq!(err.service_name(), Some("ai-service"));
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_display() {
        let err = McpError::connection_error("test-service", "连接失败");
        let display = format!("{}", err);
        assert!(display.contains("test-service"));
        assert!(display.contains("连接失败"));
    }

    #[test]
    fn test_error_severity_display() {
        assert_eq!(format!("{}", ErrorSeverity::Low), "低");
        assert_eq!(format!("{}", ErrorSeverity::Medium), "中");
        assert_eq!(format!("{}", ErrorSeverity::High), "高");
        assert_eq!(format!("{}", ErrorSeverity::Critical), "严重");
    }

    #[test]
    fn test_config_error() {
        let err = McpError::config_error("配置文件格式错误");
        assert!(matches!(err, McpError::ConfigError { .. }));
        assert_eq!(err.service_name(), None);
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_serialization_error() {
        let err = McpError::serialization_error("JSON 解析失败");
        assert!(matches!(err, McpError::SerializationError { .. }));
        assert_eq!(err.service_name(), None);
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), ErrorSeverity::Medium);
    }
}