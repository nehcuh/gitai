// rmcp 0.2.1 Error 结构分析
// 
// 这个文件分析了 rmcp 0.2.1 中 ServiceError 和 Error (McpError) 的关系

fn main() {
    println!("=== rmcp 0.2.1 Error 结构分析 ===\n");

    // 1. ServiceError 的实际结构
    println!("1. ServiceError 的变体：");
    print_service_error_variants();
    
    // 2. McpError (即 ErrorData) 的结构
    println!("\n2. McpError (ErrorData) 的结构：");
    print_mcp_error_structure();
    
    // 3. ErrorCode 的标准常量
    println!("\n3. ErrorCode 的标准 JSON-RPC 错误码：");
    print_error_codes();
    
    // 4. ServiceError 与 McpError 的关系
    println!("\n4. ServiceError 与 McpError 的关系：");
    print_error_relationship();
}

fn print_service_error_variants() {
    println!("ServiceError 包含以下变体：");
    println!("- McpError(McpError): 包装 MCP 协议错误");
    println!("- TransportSend(Box<dyn std::error::Error + Send + Sync>): 传输发送错误");
    println!("- TransportClosed: 传输连接已关闭");
    println!("- UnexpectedResponse: 意外的响应类型");
    println!("- Cancelled {{ reason: Option<String> }}: 请求被取消");
    println!("- Timeout {{ timeout: Duration }}: 请求超时");
    
    println!("\n注意：ServiceError 不直接包含 InvalidParams、InternalError 等变体");
    println!("这些错误通过 McpError 变体中的 ErrorData 来表示");
}

fn print_mcp_error_structure() {
    println!("McpError 类型别名为 ErrorData，包含：");
    println!("- code: ErrorCode (i32 包装器)");
    println!("- message: Cow<'static, str> (错误消息)");
    println!("- data: Option<serde_json::Value> (可选的额外数据)");
}

fn print_error_codes() {
    println!("ErrorCode 的标准 JSON-RPC 错误码常量：");
    println!("- PARSE_ERROR: -32700");
    println!("- INVALID_REQUEST: -32600");
    println!("- METHOD_NOT_FOUND: -32601");
    println!("- INVALID_PARAMS: -32602");
    println!("- INTERNAL_ERROR: -32603");
    println!("- RESOURCE_NOT_FOUND: -32002");
    
    println!("\n示例 InvalidParams 错误：");
    println!("code: -32602, message: \"参数无效\"");
}

fn print_error_relationship() {
    println!("ServiceError 与 McpError 的关系：");
    println!("1. ServiceError::McpError(error) 包装了 MCP 协议层的错误");
    println!("2. MCP 协议错误（如 InvalidParams, InternalError）通过 ErrorData 表示");
    println!("3. ErrorData 使用 ErrorCode 来区分不同类型的错误");
    println!("4. 项目中的兼容层 (rmcp_compat.rs) 提供了简化的 ServiceError 包装");
    
    println!("\n实际使用中：");
    println!("- 要创建 InvalidParams 错误，需要创建 ErrorData 并设置 code 为 INVALID_PARAMS");
    println!("- 然后将其包装在 ServiceError::McpError() 中");
    println!("- 兼容层提供了便捷方法来简化这个过程");
}

// 演示如何创建各种标准错误的辅助函数
#[allow(dead_code)]
fn create_standard_errors() {
    println!("标准错误创建示例：");
    println!("- 创建 InvalidParams: ErrorData {{ code: ErrorCode::INVALID_PARAMS, message: \"参数无效\", data: None }}");
    println!("- 创建 InternalError: ErrorData {{ code: ErrorCode::INTERNAL_ERROR, message: \"内部错误\", data: None }}");
    println!("- 创建 MethodNotFound: ErrorData {{ code: ErrorCode::METHOD_NOT_FOUND, message: \"方法未找到\", data: Some(method_data) }}");
    println!("- 然后包装在 ServiceError::McpError() 中");
}