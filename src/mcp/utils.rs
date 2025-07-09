use std::collections::HashMap;
use serde_json::Value;
use rmcp::{
    model::{ServerInfo, Tool, Resource},
};
use crate::mcp::rmcp_compat::ServiceError;

/// MCP 工具类
pub struct McpUtils;

impl McpUtils {
    /// 验证工具参数
    pub fn validate_tool_args(args: &Value, schema: &Value) -> Result<(), ServiceError> {
        // 简化的参数验证逻辑
        if args.is_null() {
            return Err(ServiceError::invalid_params("工具参数不能为空".to_string()));
        }
        
        // 这里可以添加更复杂的 JSON Schema 验证
        tracing::debug!("验证工具参数: {:?}", args);
        Ok(())
    }

    /// 格式化错误消息
    pub fn format_error_message(error: &ServiceError) -> String {
        // rmcp 0.2.1 ServiceError 使用不同的结构，使用 Display 或 Debug 格式
        format!("MCP 错误: {}", error)
    }

    /// 创建标准响应
    pub fn create_success_response(data: Value) -> Value {
        serde_json::json!({
            "success": true,
            "data": data,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// 创建错误响应
    pub fn create_error_response(error: &ServiceError) -> Value {
        serde_json::json!({
            "success": false,
            "error": Self::format_error_message(error),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }

    /// 解析服务配置
    pub fn parse_service_config(config_str: &str) -> Result<HashMap<String, Value>, ServiceError> {
        serde_json::from_str(config_str)
            .map_err(|e| ServiceError::parse_error(format!("配置解析失败: {}", e)))
    }

    /// 生成工具描述
    pub fn generate_tool_description(name: &str, description: &str, parameters: &[&str]) -> Tool {
        let mut properties = serde_json::Map::new();
        for p in parameters {
            properties.insert(p.to_string(), serde_json::json!({
                "type": "string",
                "description": format!("参数 {}", p)
            }));
        }
        
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
        schema.insert("properties".to_string(), serde_json::Value::Object(properties));
        
        Tool {
            name: name.to_string().into(),
            description: Some(description.to_string().into()),
            input_schema: std::sync::Arc::new(schema),
            annotations: None,
        }
    }

    /// 生成资源描述
    pub fn generate_resource_description(uri: &str, name: &str, description: &str) -> Resource {
        // rmcp 0.2.1 uses different Resource structure
        // This is a placeholder - actual implementation depends on rmcp 0.2.1 API
        unimplemented!("Resource generation needs to be updated for rmcp 0.2.1")
    }

    /// 检查服务健康状态
    pub async fn check_service_health(service_name: &str) -> Result<bool, ServiceError> {
        // 这里是占位符实现
        // 在真实实现中，这里应该检查具体的服务状态
        tracing::debug!("检查服务健康状态: {}", service_name);
        Ok(true)
    }

    /// 格式化服务统计信息
    pub fn format_service_stats(stats: &HashMap<String, Value>) -> String {
        let mut result = String::new();
        
        result.push_str("服务统计信息:\n");
        for (key, value) in stats {
            result.push_str(&format!("  {}: {}\n", key, value));
        }
        
        result
    }

    /// 验证资源 URI
    pub fn validate_resource_uri(uri: &str) -> Result<(), ServiceError> {
        if uri.is_empty() {
            return Err(ServiceError::invalid_params("资源 URI 不能为空".to_string()));
        }

        // 简单的 URI 格式验证
        if !uri.contains("://") {
            return Err(ServiceError::invalid_params("资源 URI 格式无效".to_string()));
        }

        Ok(())
    }

    /// 创建默认的服务器信息
    pub fn create_default_server_info(name: &str, version: &str) -> ServerInfo {
        use rmcp::model::{Implementation, ServerCapabilities, ProtocolVersion};
        
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: name.to_string(),
                version: version.to_string(),
            },
            instructions: None,
        }
    }

    /// 合并配置
    pub fn merge_config(base: &mut HashMap<String, Value>, overlay: HashMap<String, Value>) {
        for (key, value) in overlay {
            base.insert(key, value);
        }
    }

    /// 生成唯一的请求 ID
    pub fn generate_request_id() -> String {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("req_{}", timestamp)
    }
}

/// 配置构建器
pub struct ConfigBuilder {
    config: HashMap<String, Value>,
}

impl ConfigBuilder {
    /// 创建新的配置构建器
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    /// 设置配置项
    pub fn set<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.config.insert(key.to_string(), value.into());
        self
    }

    /// 构建配置
    pub fn build(self) -> HashMap<String, Value> {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tool_args() {
        let args = serde_json::json!({"param": "value"});
        let schema = serde_json::json!({});
        
        let result = McpUtils::validate_tool_args(&args, &schema);
        assert!(result.is_ok());
        
        let null_args = serde_json::json!(null);
        let result = McpUtils::validate_tool_args(&null_args, &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_error_message() {
        let error = ServiceError::InvalidParams;
        
        let formatted = McpUtils::format_error_message(&error);
        assert!(formatted.contains("MCP"));
    }

    #[test]
    fn test_create_success_response() {
        let data = serde_json::json!({"result": "success"});
        let response = McpUtils::create_success_response(data);
        
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["result"], "success");
        assert!(response["timestamp"].is_string());
    }

    #[test]
    fn test_create_error_response() {
        let error = ServiceError::InvalidParams;
        
        let response = McpUtils::create_error_response(&error);
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
        assert!(response["timestamp"].is_string());
    }

    #[test]
    fn test_parse_service_config() {
        let config_str = r#"{"name": "test", "version": "1.0.0"}"#;
        let config = McpUtils::parse_service_config(config_str);
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config["name"], "test");
        assert_eq!(config["version"], "1.0.0");
    }

    #[test]
    fn test_generate_tool_description() {
        let tool = McpUtils::generate_tool_description(
            "test_tool",
            "测试工具",
            &["param1", "param2"]
        );
        
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, Some("测试工具".to_string()));
        assert!(tool.input_schema.is_object());
    }

    #[test]
    #[should_panic]
    fn test_generate_resource_description() {
        let _resource = McpUtils::generate_resource_description(
            "test://resource",
            "测试资源",
            "测试资源描述"
        );
        // Should panic because it's unimplemented
    }

    #[test]
    fn test_validate_resource_uri() {
        // 有效的 URI
        assert!(McpUtils::validate_resource_uri("test://resource").is_ok());
        
        // 无效的 URI
        assert!(McpUtils::validate_resource_uri("").is_err());
        assert!(McpUtils::validate_resource_uri("invalid-uri").is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .set("name", "test")
            .set("version", "1.0.0")
            .set("enabled", true)
            .build();
        
        assert_eq!(config["name"], "test");
        assert_eq!(config["version"], "1.0.0");
        assert_eq!(config["enabled"], true);
    }

    #[test]
    fn test_merge_config() {
        let mut base = HashMap::new();
        base.insert("name".to_string(), "base".into());
        base.insert("version".to_string(), "1.0.0".into());
        
        let overlay = HashMap::from([
            ("name".to_string(), "overlay".into()),
            ("description".to_string(), "测试".into()),
        ]);
        
        McpUtils::merge_config(&mut base, overlay);
        
        assert_eq!(base["name"], "overlay");
        assert_eq!(base["version"], "1.0.0");
        assert_eq!(base["description"], "测试");
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = McpUtils::generate_request_id();
        let id2 = McpUtils::generate_request_id();
        
        assert!(id1.starts_with("req_"));
        assert!(id2.starts_with("req_"));
        assert_ne!(id1, id2);
    }
}