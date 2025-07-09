// 探索 rmcp 0.2.1 的真实 API 结构

use std::collections::HashMap;

// 测试 rmcp 各种类型和结构
fn main() {
    println!("探索 rmcp 0.2.1 API...");
    
    // 1. 测试 ServiceError 的实际结构
    test_service_error();
    
    // 2. 测试 ServerHandler 的实际方法
    test_server_handler();
    
    // 3. 测试 Tool 和 Resource 的实际结构
    test_tool_and_resource();
    
    // 4. 测试其他 MCP 相关类型
    test_mcp_types();
}

fn test_service_error() {
    use rmcp::service::ServiceError;
    
    println!("\n=== ServiceError API 测试 ===");
    
    // 尝试不同的构造方式
    let error1 = ServiceError::InvalidParams;
    let error2 = ServiceError::InternalError;
    let error3 = ServiceError::ParseError;
    
    // 检查是否有其他变体
    match &error1 {
        ServiceError::InvalidParams => println!("✓ InvalidParams 变体存在"),
        ServiceError::InternalError => println!("✓ InternalError 变体存在"),
        ServiceError::ParseError => println!("✓ ParseError 变体存在"),
        ServiceError::MethodNotFound { method } => println!("✓ MethodNotFound 变体存在: {}", method),
        _ => println!("? 未知的 ServiceError 变体"),
    }
    
    println!("ServiceError display: {}", error1);
    println!("ServiceError debug: {:?}", error1);
}

fn test_server_handler() {
    use rmcp::handler::server::ServerHandler;
    use rmcp::model::{Tool, Resource};
    
    println!("\n=== ServerHandler 特征测试 ===");
    
    // 创建一个测试实现来查看需要的方法
    struct TestHandler;
    
    impl ServerHandler for TestHandler {
        fn list_tools(&self) -> Vec<Tool> {
            println!("✓ list_tools 方法是必需的");
            Vec::new()
        }
        
        fn list_resources(&self) -> Vec<Resource> {
            println!("✓ list_resources 方法是必需的");
            Vec::new()
        }
        
        fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, rmcp::service::ServiceError> {
            println!("✓ call_tool 方法是必需的: {}", name);
            Ok(serde_json::json!({"result": "test"}))
        }
        
        fn read_resource(&self, uri: &str) -> Result<String, rmcp::service::ServiceError> {
            println!("✓ read_resource 方法是必需的: {}", uri);
            Ok(format!("Resource content for: {}", uri))
        }
    }
    
    let handler = TestHandler;
    let tools = handler.list_tools();
    let resources = handler.list_resources();
    
    println!("Tools count: {}", tools.len());
    println!("Resources count: {}", resources.len());
}

fn test_tool_and_resource() {
    use rmcp::model::{Tool, Resource};
    use std::sync::Arc;
    
    println!("\n=== Tool 和 Resource 结构测试 ===");
    
    // 测试 Tool 结构
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
    
    let tool = Tool {
        name: "test_tool".into(),
        description: Some("测试工具".into()),
        input_schema: Arc::new(schema),
        annotations: None,
    };
    
    println!("✓ Tool 创建成功: {}", tool.name);
    println!("  Description: {:?}", tool.description);
    println!("  Schema type: Arc<Map<String, Value>>");
    println!("  Annotations: {:?}", tool.annotations);
    
    // 测试 Resource 结构 - 需要查看实际的字段
    // 这里可能会有编译错误，用来发现正确的字段名
    
    println!("Tool 结构测试完成");
}

fn test_mcp_types() {
    use rmcp::model::{ServerInfo, InitializeResult};
    
    println!("\n=== 其他 MCP 类型测试 ===");
    
    // 测试 ServerInfo
    let server_info = ServerInfo {
        name: "test_server".into(),
        version: "1.0.0".into(),
    };
    
    println!("✓ ServerInfo 创建成功: {} v{}", server_info.name, server_info.version);
    
    // 测试 InitializeResult - 查看实际字段
    // 这里可能会有编译错误，用来发现正确的字段名
    
    println!("MCP 类型测试完成");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::service::ServiceError;
    
    #[test]
    fn test_service_error_construction() {
        let error = ServiceError::InvalidParams;
        assert!(matches!(error, ServiceError::InvalidParams));
        
        let error = ServiceError::MethodNotFound { method: "test".to_string() };
        if let ServiceError::MethodNotFound { method } = error {
            assert_eq!(method, "test");
        } else {
            panic!("MethodNotFound 构造失败");
        }
    }
    
    #[test]
    fn test_tool_creation() {
        use rmcp::model::Tool;
        use std::sync::Arc;
        
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
        
        let tool = Tool {
            name: "test".into(),
            description: Some("test".into()),
            input_schema: Arc::new(schema),
            annotations: None,
        };
        
        assert_eq!(tool.name, "test");
    }
}