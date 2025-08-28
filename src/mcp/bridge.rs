// MCP 桥接模块
//
// 提供 GitAI MCP 服务器的桥接层，将 GitAI 核心功能封装为 MCP 兼容接口

use crate::config::Config;
use crate::mcp::McpResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// GitAI MCP 服务器处理器
#[derive(Clone)]
#[allow(dead_code)]
pub struct GitAiMcpServer {
    /// MCP 服务管理器
    manager: Arc<RwLock<crate::mcp::GitAiMcpManager>>,
}

impl GitAiMcpServer {
    /// 创建新的 MCP 服务器
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        let manager = Arc::new(RwLock::new(crate::mcp::GitAiMcpManager::new(config)));
        Self { manager }
    }
}

// 暂时注释掉 ServerHandler 实现，因为 rmcp API 比较复杂
// #[async_trait::async_trait]
// impl ServerHandler for GitAiMcpServer {
//     async fn initialize(&self, params: InitializeRequestParam, _ctx: RequestContext<RoleServer>) -> McpResult<InitializeResult> {
//         let manager = self.manager.read().await;
//         let server_info = manager.get_server_info();
//         
//         Ok(InitializeResult {
//             protocol_version: params.protocol_version,
//             capabilities: ServerCapabilities {
//                 tools: Some(ToolsCapability {
//                     list_changed: Some(true),
//                 }),
//                 ..Default::default()
//             },
//             server_info,
//             instructions: Some("GitAI MCP Server - 提供代码评审、提交、扫描和分析功能".to_string()),
//         })
//     }
//
//     async fn list_tools(&self, _params: Option<PaginatedRequestParamInner>, _ctx: RequestContext<RoleServer>) -> McpResult<ListToolsResult> {
//         let manager = self.manager.read().await;
//         let tools = manager.get_all_tools();
//         
//         Ok(ListToolsResult { tools, next_cursor: None })
//     }
//
//     async fn call_tool(&self, params: CallToolRequestParam, _ctx: RequestContext<RoleServer>) -> McpResult<CallToolResult> {
//         let manager = self.manager.read().await;
//         
//         match manager.handle_tool_call(&params.name, serde_json::Value::Object(params.arguments.unwrap_or_default())).await {
//             Ok(result) => {
//                 // 将结果转换为 CallToolResult
//                 let content = Content::text(serde_json::to_string_pretty(&result).unwrap_or_default());
//                 Ok(CallToolResult {
//                     content: vec![content],
//                     is_error: None,
//                 })
//             }
//             Err(e) => {
//                 let error_msg = format!("Tool call failed: {}", e);
//                 let content = Content::text(error_msg);
//                 Ok(CallToolResult {
//                     content: vec![content],
//                     is_error: Some(true),
//                 })
//             }
//         }
//     }
// }

/// 启动 MCP 服务器
pub async fn start_mcp_server(config: Config) -> McpResult<()> {
    use std::io::{self, Write};
    use serde_json::{Value, json};
    
    // Helper function to safely write JSON response to stdout
    fn write_response(stdout: &mut impl Write, response: &Value) -> McpResult<()> {
        writeln!(stdout, "{}", response)
            .map_err(|e| crate::mcp::execution_failed_error(format!("Failed to write response: {}", e)))?;
        stdout.flush()
            .map_err(|e| crate::mcp::execution_failed_error(format!("Failed to flush stdout: {}", e)))?;
        Ok(())
    }
    
    eprintln!("🚀 GitAI MCP Server starting...");
    eprintln!("📡 Available services:");
    eprintln!("   - review: Code review with tree-sitter and security scan");
    eprintln!("   - commit: Smart commit with AI-generated messages");
    eprintln!("   - scan: Security scanning with OpenGrep");
    eprintln!("   - analysis: Code structure analysis");
    eprintln!("🔌 Listening on stdio...");
    
    // 创建并初始化服务管理器
    let manager = std::sync::Arc::new(tokio::sync::RwLock::new(crate::mcp::GitAiMcpManager::new(config.clone())));
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // 简单的 MCP 协议处理循环
    loop {
        let mut buffer = String::new();
        match stdin.read_line(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // 尝试解析 JSON 消息
                if let Ok(msg) = serde_json::from_str::<Value>(&buffer.trim()) {
                    if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                        match method {
                            "initialize" => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": msg.get("id"),
                                    "result": {
                                        "protocolVersion": "2024-11-05",
                                        "capabilities": {
                                            "tools": {
                                                "listChanged": true
                                            }
                                        },
                                        "serverInfo": {
                                            "name": "gitai",
                                            "version": "0.1.0"
                                        }
                                    }
                                });
                                write_response(&mut stdout, &response)?;
                            }
                            "tools/list" => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": msg.get("id"),
                                    "result": {
                                        "tools": [
                                            {
                                                "name": "execute_review",
                                                "description": "Execute code review",
                                                "inputSchema": {
                                                    "type": "object",
                                                    "properties": {
                                                        "tree_sitter": {"type": "boolean"},
                                                        "security_scan": {"type": "boolean"}
                                                    }
                                                }
                                            },
                                            {
                                                "name": "execute_commit",
                                                "description": "Execute smart commit",
                                                "inputSchema": {
                                                    "type": "object",
                                                    "properties": {
                                                        "message": {"type": "string"},
                                                        "issue_ids": {"type": "array", "items": {"type": "string"}}
                                                    }
                                                }
                                            },
                                            {
                                                "name": "execute_scan",
                                                "description": "Execute security scan",
                                                "inputSchema": {
                                                    "type": "object",
                                                    "properties": {
                                                        "path": {"type": "string"},
                                                        "tool": {"type": "string"}
                                                    },
                                                    "required": ["path"]
                                                }
                                            },
                                            {
                                                "name": "execute_analysis",
                                                "description": "Execute code analysis",
                                                "inputSchema": {
                                                    "type": "object",
                                                    "properties": {
                                                        "path": {"type": "string"},
                                                        "language": {"type": "string"}
                                                    },
                                                    "required": ["path"]
                                                }
                                            }
                                        ]
                                    }
                                });
                                write_response(&mut stdout, &response)?;
                            }
                            "tools/call" => {
                                let tool_name = msg.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                                let arguments = msg.get("params").and_then(|p| p.get("arguments")).cloned().unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                                
                                // 使用共享的服务管理器处理工具调用
                                let manager_read = manager.read().await;
                                match manager_read.handle_tool_call(tool_name, arguments).await {
                                    Ok(result) => {
                                        let response = json!({
                                            "jsonrpc": "2.0",
                                            "id": msg.get("id"),
                                            "result": {
                                                "content": [
                                                    {
                                                        "type": "text",
                                                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Error formatting result".to_string())
                                                    }
                                                ]
                                            }
                                        });
                                        write_response(&mut stdout, &response)?;
                                    }
                                    Err(e) => {
                                        let error_type = match e {
                                            crate::mcp::McpError::InvalidParameters(_) => "InvalidParameters",
                                            crate::mcp::McpError::ExecutionFailed(_) => "ExecutionFailed",
                                            crate::mcp::McpError::ConfigurationError(_) => "ConfigurationError",
                                            crate::mcp::McpError::FileOperationError(_) => "FileOperationError",
                                            crate::mcp::McpError::NetworkError(_) => "NetworkError",
                                            crate::mcp::McpError::ExternalToolError(_) => "ExternalToolError",
                                            crate::mcp::McpError::PermissionError(_) => "PermissionError",
                                            crate::mcp::McpError::TimeoutError(_) => "TimeoutError",
                                            crate::mcp::McpError::Unknown(_) => "Unknown",
                                        };
                                        
                                        let (error_code, error_message) = match e {
                                            crate::mcp::McpError::InvalidParameters(msg) => (-32602, msg),
                                            crate::mcp::McpError::ExecutionFailed(msg) => (-32000, msg),
                                            crate::mcp::McpError::ConfigurationError(msg) => (-32603, msg),
                                            crate::mcp::McpError::FileOperationError(msg) => (-32001, msg),
                                            crate::mcp::McpError::NetworkError(msg) => (-32002, msg),
                                            crate::mcp::McpError::ExternalToolError(msg) => (-32003, msg),
                                            crate::mcp::McpError::PermissionError(msg) => (-32004, msg),
                                            crate::mcp::McpError::TimeoutError(msg) => (-32005, msg),
                                            crate::mcp::McpError::Unknown(msg) => (-32603, msg),
                                        };
                                        
                                        let response = json!({
                                            "jsonrpc": "2.0",
                                            "id": msg.get("id"),
                                            "error": {
                                                "code": error_code,
                                                "message": error_message,
                                                "data": {
                                                    "type": error_type
                                                }
                                            }
                                        });
                                        write_response(&mut stdout, &response)?;
                                    }
                                }
                            }
                            _ => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": msg.get("id"),
                                    "error": {
                                        "code": -32601,
                                        "message": format!("Method not found: {}", method)
                                    }
                                });
                                write_response(&mut stdout, &response)?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    eprintln!("👋 GitAI MCP Server shutting down");
    Ok(())
}

/// 启动 TCP MCP 服务器
#[allow(dead_code)]
pub async fn start_mcp_tcp_server(_config: Config, _addr: &str) -> McpResult<()> {
    // TCP 传输需要额外的实现，目前 rmcp 主要支持 stdio
    eprintln!("⚠️  TCP transport not fully implemented in current rmcp version");
    eprintln!("   Please use stdio transport instead");
    Ok(())
}

/// 启动 SSE MCP 服务器
#[allow(dead_code)]
pub async fn start_mcp_websocket_server(_config: Config, _addr: &str) -> McpResult<()> {
    eprintln!("⚠️  SSE transport not fully implemented in current rmcp version");
    eprintln!("   Please use stdio transport instead");
    Ok(())
}