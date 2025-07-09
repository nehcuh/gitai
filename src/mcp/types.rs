use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入参数 schema
    pub input_schema: serde_json::Value,
}

/// MCP 资源定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// 资源 URI
    pub uri: String,
    /// 资源描述
    pub description: String,
    /// 资源类型
    pub resource_type: Option<String>,
    /// 是否可读
    pub readable: bool,
    /// 是否可写
    pub writable: bool,
}

/// MCP 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServiceInfo {
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: String,
    /// 服务描述
    pub description: String,
    /// 支持的工具
    pub tools: Vec<McpTool>,
    /// 提供的资源
    pub resources: Vec<McpResource>,
    /// 服务状态
    pub status: ServiceStatus,
    /// 服务元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 服务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 启动中
    Starting,
    /// 停止中
    Stopping,
    /// 错误状态
    Error,
    /// 未知状态
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "运行中"),
            ServiceStatus::Stopped => write!(f, "已停止"),
            ServiceStatus::Starting => write!(f, "启动中"),
            ServiceStatus::Stopping => write!(f, "停止中"),
            ServiceStatus::Error => write!(f, "错误"),
            ServiceStatus::Unknown => write!(f, "未知"),
        }
    }
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// 工具名称
    pub name: String,
    /// 输入参数
    pub arguments: HashMap<String, serde_json::Value>,
    /// 请求 ID
    pub request_id: Option<String>,
    /// 超时时间（秒）
    pub timeout: Option<u64>,
}

/// 工具调用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// 响应 ID
    pub request_id: Option<String>,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time: Option<u64>,
}

/// 资源访问请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// 资源 URI
    pub uri: String,
    /// 访问类型
    pub access_type: ResourceAccessType,
    /// 请求参数
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 资源访问类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceAccessType {
    /// 读取
    Read,
    /// 写入
    Write,
    /// 列表
    List,
    /// 删除
    Delete,
}

/// 资源访问响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    /// 是否成功
    pub success: bool,
    /// 资源内容
    pub content: Option<serde_json::Value>,
    /// 资源元数据
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// 错误信息
    pub error: Option<String>,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// 是否健康
    pub healthy: bool,
    /// 状态信息
    pub status: String,
    /// 检查时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 详细信息
    pub details: HashMap<String, serde_json::Value>,
}

/// 服务发现请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryRequest {
    /// 服务名称模式
    pub name_pattern: Option<String>,
    /// 服务类型
    pub service_type: Option<String>,
    /// 版本要求
    pub version_requirement: Option<String>,
    /// 是否仅返回可用服务
    pub available_only: bool,
}

/// 服务发现响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryResponse {
    /// 找到的服务
    pub services: Vec<McpServiceInfo>,
    /// 搜索时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 服务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// 服务名称
    pub service_name: String,
    /// 总调用次数
    pub total_calls: u64,
    /// 成功调用次数
    pub successful_calls: u64,
    /// 失败调用次数
    pub failed_calls: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 最后调用时间
    pub last_call_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 服务启动时间
    pub startup_time: chrono::DateTime<chrono::Utc>,
}

/// 批量操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// 操作列表
    pub operations: Vec<BatchOperation>,
    /// 是否并行执行
    pub parallel: bool,
    /// 失败时是否继续
    pub continue_on_error: bool,
}

/// 批量操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// 操作 ID
    pub id: String,
    /// 操作类型
    pub operation_type: BatchOperationType,
    /// 操作数据
    pub data: serde_json::Value,
}

/// 批量操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchOperationType {
    /// 工具调用
    ToolCall,
    /// 资源访问
    ResourceAccess,
    /// 健康检查
    HealthCheck,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// 操作结果
    pub results: Vec<BatchOperationResult>,
    /// 总体成功率
    pub success_rate: f64,
    /// 执行时间（毫秒）
    pub execution_time: u64,
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    /// 操作 ID
    pub id: String,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time: u64,
}

/// 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpEvent {
    /// 服务启动
    ServiceStarted {
        service_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// 服务停止
    ServiceStopped {
        service_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// 服务错误
    ServiceError {
        service_name: String,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// 工具调用
    ToolCalled {
        service_name: String,
        tool_name: String,
        success: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// 资源访问
    ResourceAccessed {
        service_name: String,
        resource_uri: String,
        access_type: ResourceAccessType,
        success: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// 事件监听器
pub trait McpEventListener: Send + Sync {
    /// 处理事件
    fn handle_event(&self, event: McpEvent);
}

// 实现默认值
impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Unknown
    }
}

impl Default for ToolCallResponse {
    fn default() -> Self {
        Self {
            request_id: None,
            success: false,
            result: None,
            error: None,
            execution_time: None,
        }
    }
}

impl Default for ResourceResponse {
    fn default() -> Self {
        Self {
            success: false,
            content: None,
            metadata: None,
            error: None,
        }
    }
}

impl Default for HealthCheckResponse {
    fn default() -> Self {
        Self {
            healthy: false,
            status: "未知".to_string(),
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        }
    }
}

impl Default for ServiceStats {
    fn default() -> Self {
        Self {
            service_name: "unknown".to_string(),
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            avg_response_time: 0.0,
            last_call_time: None,
            startup_time: chrono::Utc::now(),
        }
    }
}

// 辅助函数
impl ToolCallResponse {
    /// 创建成功响应
    pub fn success(result: serde_json::Value) -> Self {
        Self {
            success: true,
            result: Some(result),
            ..Default::default()
        }
    }

    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            ..Default::default()
        }
    }
}

impl ResourceResponse {
    /// 创建成功响应
    pub fn success(content: serde_json::Value) -> Self {
        Self {
            success: true,
            content: Some(content),
            ..Default::default()
        }
    }

    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            ..Default::default()
        }
    }
}

impl HealthCheckResponse {
    /// 创建健康响应
    pub fn healthy(status: String) -> Self {
        Self {
            healthy: true,
            status,
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        }
    }

    /// 创建不健康响应
    pub fn unhealthy(status: String) -> Self {
        Self {
            healthy: false,
            status,
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_display() {
        assert_eq!(format!("{}", ServiceStatus::Running), "运行中");
        assert_eq!(format!("{}", ServiceStatus::Stopped), "已停止");
        assert_eq!(format!("{}", ServiceStatus::Error), "错误");
    }

    #[test]
    fn test_tool_call_response_success() {
        let response = ToolCallResponse::success(serde_json::json!({"result": "ok"}));
        assert!(response.success);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_tool_call_response_error() {
        let response = ToolCallResponse::error("测试错误".to_string());
        assert!(!response.success);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_resource_response_success() {
        let response = ResourceResponse::success(serde_json::json!({"data": "test"}));
        assert!(response.success);
        assert!(response.content.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_resource_response_error() {
        let response = ResourceResponse::error("资源不存在".to_string());
        assert!(!response.success);
        assert!(response.content.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_health_check_response_healthy() {
        let response = HealthCheckResponse::healthy("服务正常".to_string());
        assert!(response.healthy);
        assert_eq!(response.status, "服务正常");
    }

    #[test]
    fn test_health_check_response_unhealthy() {
        let response = HealthCheckResponse::unhealthy("服务异常".to_string());
        assert!(!response.healthy);
        assert_eq!(response.status, "服务异常");
    }

    #[test]
    fn test_service_status_default() {
        let status = ServiceStatus::default();
        assert_eq!(status, ServiceStatus::Unknown);
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "测试工具".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: McpTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test_tool");
        assert_eq!(deserialized.description, "测试工具");
    }

    #[test]
    fn test_mcp_resource_serialization() {
        let resource = McpResource {
            uri: "test://resource".to_string(),
            description: "测试资源".to_string(),
            resource_type: Some("data".to_string()),
            readable: true,
            writable: false,
        };

        let json = serde_json::to_string(&resource).unwrap();
        let deserialized: McpResource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.uri, "test://resource");
        assert_eq!(deserialized.description, "测试资源");
        assert!(deserialized.readable);
        assert!(!deserialized.writable);
    }
}