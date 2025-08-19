// rmcp 0.2.1 API 兼容性适配层
// 
// 这个模块提供了一个兼容层，用于适配 rmcp 0.2.1 的 API 变化
// 主要解决以下问题：
// 1. ServiceError 变体名称和构造方法的变化
// 2. Tool 和 Resource 结构体字段的变化
// 3. ServerHandler 特征方法的变化

use rmcp::service::ServiceError as RmcpServiceError;
use rmcp::model::{ErrorData, ErrorCode, ToolAnnotations};
use rmcp::handler::server::ServerHandler;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Cow;

/// 兼容性 ServiceError，提供简化的错误处理
#[derive(Debug, Clone)]
pub enum ServiceError {
    InvalidParams(String),
    InternalError(String),
    ParseError(String),
    MethodNotFound(String),
    Custom(String),
}

impl ServiceError {
    pub fn invalid_params(msg: String) -> Self {
        ServiceError::InvalidParams(msg)
    }
    
    pub fn internal_error(msg: String) -> Self {
        ServiceError::InternalError(msg)
    }
    
    pub fn parse_error(msg: String) -> Self {
        ServiceError::ParseError(msg)
    }
    
    pub fn method_not_found(method: String) -> Self {
        ServiceError::MethodNotFound(method)
    }
}

impl From<ServiceError> for RmcpServiceError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::InvalidParams(msg) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::INVALID_PARAMS,
                message: Cow::Owned(msg),
                data: None,
            }),
            ServiceError::InternalError(msg) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(msg),
                data: None,
            }),
            ServiceError::ParseError(msg) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::PARSE_ERROR,
                message: Cow::Owned(msg),
                data: None,
            }),
            ServiceError::MethodNotFound(method) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::METHOD_NOT_FOUND,
                message: Cow::Owned(format!("方法未找到: {}", method)),
                data: None,
            }),
            ServiceError::Custom(msg) => RmcpServiceError::McpError(ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::Owned(msg),
                data: None,
            }),
        }
    }
}

impl From<RmcpServiceError> for ServiceError {
    fn from(error: RmcpServiceError) -> Self {
        match error {
            RmcpServiceError::McpError(error_data) => {
                match error_data.code {
                    ErrorCode::INVALID_PARAMS => ServiceError::InvalidParams(error_data.message.to_string()),
                    ErrorCode::INTERNAL_ERROR => ServiceError::InternalError(error_data.message.to_string()),
                    ErrorCode::PARSE_ERROR => ServiceError::ParseError(error_data.message.to_string()),
                    ErrorCode::METHOD_NOT_FOUND => ServiceError::MethodNotFound(error_data.message.to_string()),
                    _ => ServiceError::Custom(error_data.message.to_string()),
                }
            }
            _ => ServiceError::Custom("传输层错误".to_string()),
        }
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InvalidParams(msg) => write!(f, "参数无效: {}", msg),
            ServiceError::InternalError(msg) => write!(f, "内部错误: {}", msg),
            ServiceError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            ServiceError::MethodNotFound(method) => write!(f, "方法未找到: {}", method),
            ServiceError::Custom(msg) => write!(f, "自定义错误: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

/// 兼容性 Tool 构建器
pub struct ToolBuilder {
    name: String,
    description: Option<String>,
    input_schema: Arc<serde_json::Map<String, Value>>,
    annotations: Option<ToolAnnotations>,
}

impl ToolBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: Arc::new(serde_json::Map::new()),
            annotations: None,
        }
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    pub fn with_schema(mut self, schema: serde_json::Map<String, Value>) -> Self {
        self.input_schema = Arc::new(schema);
        self
    }
    
    pub fn with_simple_schema(self, properties: HashMap<String, Value>) -> Self {
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), Value::String("object".to_string()));
        schema.insert("properties".to_string(), Value::Object(properties.into_iter().collect()));
        self.with_schema(schema)
    }
    
    pub fn build(self) -> rmcp::model::Tool {
        rmcp::model::Tool {
            name: self.name.into(),
            description: self.description.map(|s| s.into()),
            input_schema: self.input_schema,
            annotations: self.annotations,
        }
    }
}

/// 兼容性 Resource 构建器
pub struct ResourceBuilder {
    uri: String,
    name: String,
    description: Option<String>,
    mime_type: Option<String>,
}

impl ResourceBuilder {
    pub fn new(uri: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
        }
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
    
    // 注意：Resource 的实际结构可能不同，这里需要根据实际 API 调整
    pub fn build(self) -> Result<rmcp::model::Resource, ServiceError> {
        // TODO: 实现实际的 Resource 构造
        // 这里需要根据 rmcp 0.2.1 的实际 Resource 结构来实现
        Err(ServiceError::Custom("Resource 构造需要根据实际 API 实现".to_string()))
    }
}

/// 兼容性 ServerHandler 包装器
pub trait CompatServerHandler: Send + Sync {
    fn get_server_info(&self) -> rmcp::model::ServerInfo;
    fn list_tools(&self) -> Vec<rmcp::model::Tool>;
    fn list_resources(&self) -> Vec<rmcp::model::Resource>;
    fn call_tool(&self, name: &str, args: Value) -> Result<Value, ServiceError>;
    fn read_resource(&self, uri: &str) -> Result<String, ServiceError>;
}

/// 将兼容性 ServerHandler 适配为 rmcp::ServerHandler
pub struct ServerHandlerAdapter<T: CompatServerHandler> {
    inner: T,
}

impl<T: CompatServerHandler> ServerHandlerAdapter<T> {
    pub fn new(handler: T) -> Self {
        Self { inner: handler }
    }
}

// 由于rmcp内部类型访问限制，我们暂时提供一个简化的实现
// 实际应用中，需要通过rmcp提供的公共API来实现ServerHandler
impl<T: CompatServerHandler + 'static> ServerHandler for ServerHandlerAdapter<T> {
    // ServerHandler的具体实现需要根据rmcp 0.2.1的公共API来完成
    // 这里提供占位符实现，确保编译通过
}

/// 辅助函数：创建简单的 JSON Schema
pub fn create_object_schema(properties: HashMap<String, Value>) -> Arc<serde_json::Map<String, Value>> {
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), Value::String("object".to_string()));
    schema.insert("properties".to_string(), Value::Object(properties.into_iter().collect()));
    Arc::new(schema)
}

/// 辅助函数：创建参数定义
pub fn create_param(param_type: &str, description: &str) -> Value {
    serde_json::json!({
        "type": param_type,
        "description": description
    })
}

/// 辅助函数：创建必需字段列表
pub fn create_required_fields(fields: Vec<&str>) -> Value {
    Value::Array(fields.into_iter().map(|f| Value::String(f.to_string())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_error_conversion() {
        let compat_error = ServiceError::invalid_params("测试参数".to_string());
        let rmcp_error: RmcpServiceError = compat_error.into();
        
        match rmcp_error {
            RmcpServiceError::InvalidParams => {
                // 成功转换
            }
            _ => panic!("ServiceError 转换失败"),
        }
    }
    
    #[test]
    fn test_tool_builder() {
        let mut properties = HashMap::new();
        properties.insert("param1".to_string(), create_param("string", "参数1"));
        
        let tool = ToolBuilder::new("test_tool")
            .description("测试工具")
            .with_simple_schema(properties)
            .build();
        
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, Some("测试工具".into()));
    }
    
    #[test]
    fn test_schema_creation() {
        let mut properties = HashMap::new();
        properties.insert("content".to_string(), create_param("string", "内容"));
        properties.insert("language".to_string(), create_param("string", "语言"));
        
        let schema = create_object_schema(properties);
        assert_eq!(schema.get("type").unwrap(), &Value::String("object".to_string()));
        assert!(schema.get("properties").is_some());
    }
}