//! # GitAI MCP 服务器
//!
//! 独立的 MCP 服务器，提供 GitAI 的所有核心功能作为 MCP 服务
//!
//! ## 运行方式
//!
//! ```bash
//! # 使用 stdio 传输
//! cargo run --bin mcp_server
//!
//! # 使用 tcp 传输
//! cargo run --bin mcp_server -- --transport tcp --listen-addr 127.0.0.1:8080
//! ```

use clap::Parser;
use gitai::{config::AppConfig, mcp_bridge::GitAiMcpBridge};
use rmcp::model::*;
use serde_json;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 传输协议
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// 监听地址 (仅 tcp 模式)
    #[arg(long, default_value = "127.0.0.1:8080")]
    listen_addr: String,
}

/// 创建一个 Send + Sync 兼容的错误处理包装器
#[derive(Debug)]
struct SendSafeError(String);

impl From<gitai::errors::AppError> for SendSafeError {
    fn from(e: gitai::errors::AppError) -> Self {
        SendSafeError(e.to_string())
    }
}

/// 处理 MCP 消息
async fn handle_mcp_message(bridge: &GitAiMcpBridge, message: &str) -> Result<String, String> {
    let request: serde_json::Value = serde_json::from_str(message).map_err(|e| e.to_string())?;
    
    // 检查是否是初始化请求
    if let Some(method) = request.get("method").and_then(|m| m.as_str()) {
        match method {
            "initialize" => {
                let response = InitializeResult {
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability {
                            list_changed: None,
                        }),
                        ..Default::default()
                    },
                    protocol_version: ProtocolVersion::V_2024_11_05,
                    server_info: Implementation {
                        name: "gitai-mcp-server".into(),
                        version: "0.1.0".into(),
                    },
                    instructions: None,
                };
                
                let json_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": response
                });
                
                return Ok(serde_json::to_string(&json_response).map_err(|e| e.to_string())?);
            }
            "tools/list" => {
                let tools = bridge.get_tools();
                let json_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "tools": tools
                    }
                });
                
                return Ok(serde_json::to_string(&json_response).map_err(|e| e.to_string())?);
            }
            "tools/call" => {
                // 解析工具调用请求
                if let Ok(tool_request) = serde_json::from_value::<CallToolRequest>(request.get("params").unwrap_or(&serde_json::Value::Null).clone()) {
                    match bridge.handle_tool_call(tool_request).await {
                        Ok(result) => {
                            let json_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": request.get("id"),
                                "result": result
                            });
                            
                            return Ok(serde_json::to_string(&json_response).map_err(|e| e.to_string())?);
                        }
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": request.get("id"),
                                "error": {
                                    "code": -32603,
                                    "message": format!("Tool call failed: {}", e)
                                }
                            });
                            
                            return Ok(serde_json::to_string(&error_response).map_err(|e| e.to_string())?);
                        }
                    }
                }
            }
            _ => {
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                });
                
                return Ok(serde_json::to_string(&error_response).map_err(|e| e.to_string())?);
            }
        }
    }
    
    Ok("{}".to_string())
}

/// 处理 stdio 传输
async fn handle_stdio(bridge: GitAiMcpBridge) -> Result<(), String> {
    info!("📡 启动 stdio MCP 服务器");
    
    let stdin = tokio::io::stdin();
    let mut reader = TokioBufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await.map_err(|e| e.to_string()) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                match handle_mcp_message(&bridge, line).await {
                    Ok(response) => {
                        stdout.write_all(response.as_bytes()).await.map_err(|e| e.to_string())?;
                        stdout.write_all(b"\n").await.map_err(|e| e.to_string())?;
                        stdout.flush().await.map_err(|e| e.to_string())?;
                    }
                    Err(e) => {
                        error!("处理消息错误: {}", e);
                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": "Internal error"
                            }
                        });
                        stdout.write_all(serde_json::to_string(&error_response).map_err(|e| e.to_string())?.as_bytes()).await.map_err(|e| e.to_string())?;
                        stdout.write_all(b"\n").await.map_err(|e| e.to_string())?;
                        stdout.flush().await.map_err(|e| e.to_string())?;
                    }
                }
            }
            Err(e) => {
                error!("读取输入错误: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

/// 处理 TCP 连接
async fn handle_tcp_connection(
    bridge: GitAiMcpBridge,
    mut stream: tokio::net::TcpStream,
) -> Result<(), String> {
    let (reader, mut writer) = stream.split();
    let mut reader = TokioBufReader::new(reader);
    
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await.map_err(|e| e.to_string()) {
            Ok(0) => break, // 连接关闭
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                match handle_mcp_message(&bridge, line).await {
                    Ok(response) => {
                        writer.write_all(response.as_bytes()).await.map_err(|e| e.to_string())?;
                        writer.write_all(b"\n").await.map_err(|e| e.to_string())?;
                        writer.flush().await.map_err(|e| e.to_string())?;
                    }
                    Err(e) => {
                        error!("处理消息错误: {}", e);
                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": "Internal error"
                            }
                        });
                        writer.write_all(serde_json::to_string(&error_response).map_err(|e| e.to_string())?.as_bytes()).await.map_err(|e| e.to_string())?;
                        writer.write_all(b"\n").await.map_err(|e| e.to_string())?;
                        writer.flush().await.map_err(|e| e.to_string())?;
                    }
                }
            }
            Err(e) => {
                error!("读取连接错误: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("🚀 启动 GitAI MCP 服务器");

    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            error!("❌ 无法加载配置: {}", e);
            return Err(e.to_string());
        }
    };

    let bridge = GitAiMcpBridge::new(config);

    info!("📡 使用 {} 传输协议", args.transport);

    match args.transport.as_str() {
        "stdio" => {
            handle_stdio(bridge).await?;
        }
        "tcp" => {
            use tokio::net::TcpListener;
            info!("👂 服务器正在监听: {}", args.listen_addr);
            let listener = TcpListener::bind(&args.listen_addr).await.map_err(|e| e.to_string())?;
            
            loop {
                let (stream, addr) = listener.accept().await.map_err(|e| e.to_string())?;
                info!("🔗 接受连接来自: {}", addr);
                let bridge_clone = bridge.clone();
                
                // 将错误转换为字符串以确保 Send 兼容性
                tokio::spawn(async move {
                    let bridge = bridge_clone;
                    if let Err(e) = handle_tcp_connection(bridge, stream).await {
                        error!("❌ 处理连接失败: {}", e);
                    }
                });
            }
        }
        _ => {
            error!("❌ 不支持的传输协议: {}", args.transport);
            return Err("不支持的传输协议".into());
        }
    }

    Ok(())
}