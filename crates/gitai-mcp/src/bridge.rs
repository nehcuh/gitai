//! MCP 桥接模块 - stdio 真实实现

use std::collections::HashMap;
use std::io::{self, Write};

use gitai_core::config::Config;
use serde_json::{json, Map, Value};
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::error::{execution_failed_error, McpResult};
use crate::services::{McpService, ServiceFactory};
use crate::GitAiMcpManager;

/// 启动 MCP 服务器（stdio）：实现 JSON-RPC initialize / tools/list / tools/call
pub async fn start_mcp_server(config: Config) -> McpResult<()> {
    // 打印基础信息到 stderr，避免干扰 JSON-RPC stdout
    eprintln!("🚀 GitAI MCP Server (stdio) starting...");
    eprintln!("🔌 Transport: stdio");

    // 构建服务集合与工具映射
    let config = std::sync::Arc::new(config);
    let services: Vec<Box<dyn McpService>> = ServiceFactory::create_services(config.clone());

    // 工具名 -> 服务索引的映射（固定映射到具体服务）
    let mut tool_to_service: HashMap<&'static str, usize> = HashMap::new();
    tool_to_service.insert("execute_review", 0);
    tool_to_service.insert("execute_scan", 1);
    tool_to_service.insert("execute_commit", 2);
    tool_to_service.insert("execute_analysis", 3);
    tool_to_service.insert("execute_dependency_graph", 4);
    tool_to_service.insert("analyze_deviation", 5);
    tool_to_service.insert("summarize_graph", 6);
    tool_to_service.insert("query_call_chain", 7);

    // 预构建工具列表与输入 schema（与 Warp / MCP 客户端对齐）
    let tools_listing = build_tools_listing();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // 安全写 JSON 响应
    fn write_response(stdout: &mut impl Write, response: &Value) -> McpResult<()> {
        writeln!(stdout, "{}", response)
            .map_err(|e| execution_failed_error(format!("Failed to write response: {}", e)))?;
        stdout
            .flush()
            .map_err(|e| execution_failed_error(format!("Failed to flush stdout: {}", e)))?;
        Ok(())
    }

    // 主循环：逐行读取 JSON-RPC 请求
    loop {
        let mut buffer = String::new();
        match stdin.read_line(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let parsed: Result<Value, _> = serde_json::from_str(trimmed);
                let msg = match parsed {
                    Ok(v) => v,
                    Err(e) => {
                        // 非法 JSON，忽略或返回错误
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": null,
                            "error": {"code": -32700, "message": format!("Parse error: {}", e)}
                        });
                        let _ = write_response(&mut stdout, &response);
                        continue;
                    }
                };

                let method = msg.get("method").and_then(|m| m.as_str());
                match method {
                    Some("initialize") => {
                        // 回显客户端的 protocolVersion，避免版本不匹配
                        let client_protocol = msg
                            .get("params")
                            .and_then(|p| p.get("protocolVersion"))
                            .cloned()
                            .unwrap_or_else(|| Value::String("2025-03-26".to_string()));

                        // 服务器信息（配置中未提供详细字段时使用默认值）
                        let (server_name, server_version) =
                            ("gitai".to_string(), "1.0.0".to_string());

                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": {
                                "protocolVersion": client_protocol,
                                "capabilities": {
                                    "tools": { "listChanged": true }
                                },
                                "serverInfo": {
                                    "name": server_name,
                                    "version": server_version
                                }
                            }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("tools/list") => {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": { "tools": tools_listing }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("tools/call") => {
                        // 解析工具名和参数
                        let tool_name = msg
                            .get("params")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("");
                        let arguments = msg
                            .get("params")
                            .and_then(|p| p.get("arguments"))
                            .cloned()
                            .unwrap_or_else(|| Value::Object(Map::new()));

                        // 路由到具体服务
                        match tool_to_service.get(tool_name) {
                            Some(&idx) => {
                                if let Some(svc) = services.get(idx) {
                                    let start = std::time::Instant::now();
                                match svc.execute(arguments).await {
                                    Ok(result) => {
                                        // Convert JSON result to text format for MCP protocol compliance
                                        let text_content = serde_json::to_string_pretty(&result)
                                            .unwrap_or_else(|_| result.to_string());
                                        let response = json!({
                                            "jsonrpc": "2.0",
                                            "id": msg.get("id"),
                                            "result": {
                                                "content": [
                                                    {"type": "text", "text": text_content}
                                                ],
                                                "usage": {
                                                    "duration_ms": start.elapsed().as_millis()
                                                }
                                            }
                                        });
                                        write_response(&mut stdout, &response)?;
                                        }
                                        Err(e) => {
                                            let response = json!({
                                                "jsonrpc": "2.0",
                                                "id": msg.get("id"),
                                                "error": {
                                                    "code": -32000,
                                                    "message": format!("Tool execution failed: {}", e),
                                                    "data": {"type": "ExecutionFailed"}
                                                }
                                            });
                                            write_response(&mut stdout, &response)?;
                                        }
                                    }
                                } else {
                                    let response = json!({
                                        "jsonrpc": "2.0",
                                        "id": msg.get("id"),
                                        "error": {"code": -32601, "message": format!("Service not found for tool: {}", tool_name)}
                                    });
                                    write_response(&mut stdout, &response)?;
                                }
                            }
                            None => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "id": msg.get("id"),
                                    "error": {"code": -32601, "message": format!("Unknown tool: {}", tool_name)}
                                });
                                write_response(&mut stdout, &response)?;
                            }
                        }
                    }
                    Some("resources/list") => {
                        // Return empty resources list for now - MCP clients should handle this gracefully
                        // In the future, we could expose project files or Git resources
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": { "resources": [] }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("resources/list_disabled") => {
                        // Original implementation (disabled for now to avoid issues)
                        let (path, limit) = {
                            let p = msg
                                .get("params")
                                .and_then(|p| p.get("path"))
                                .and_then(|v| v.as_str())
                                .unwrap_or(".");
                            let l = msg
                                .get("params")
                                .and_then(|p| p.get("limit"))
                                .and_then(|v| v.as_u64())
                                .unwrap_or(200);
                            (p.to_string(), l as usize)
                        };
                        let mut files: Vec<String> = Vec::new();
                        fn is_ignored_dir(name: &str) -> bool {
                            matches!(
                                name,
                                ".git"
                                    | "target"
                                    | "node_modules"
                                    | ".cache"
                                    | ".idea"
                                    | ".vscode"
                                    | "dist"
                                    | "build"
                            )
                        }
                        fn walk_dir(
                            dir: &std::path::Path,
                            out: &mut Vec<String>,
                            left: &mut usize,
                        ) {
                            if *left == 0 {
                                return;
                            }
                            if let Ok(entries) = std::fs::read_dir(dir) {
                                for entry in entries.flatten() {
                                    if *left == 0 {
                                        break;
                                    }
                                    let p = entry.path();
                                    if p.is_dir() {
                                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                                            if is_ignored_dir(name) {
                                                continue;
                                            }
                                        }
                                        walk_dir(&p, out, left);
                                    } else if p.is_file() {
                                        if let Some(s) = p.to_str() {
                                            out.push(s.to_string());
                                        }
                                        if *left > 0 {
                                            *left -= 1;
                                        }
                                    }
                                }
                            }
                        }
                        let mut left = limit;
                        walk_dir(std::path::Path::new(&path), &mut files, &mut left);
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": { "resources": files }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("resources/read") => {
                        let (path, max_bytes) = {
                            let p = msg
                                .get("params")
                                .and_then(|p| p.get("path"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let m = msg
                                .get("params")
                                .and_then(|p| p.get("max_bytes"))
                                .and_then(|v| v.as_u64())
                                .unwrap_or(200_000);
                            (p.to_string(), m as usize)
                        };
                        let data = match std::fs::read(&path) {
                            Ok(mut bytes) => {
                                if bytes.len() > max_bytes {
                                    bytes.truncate(max_bytes);
                                }
                                match String::from_utf8(bytes) {
                                    Ok(s) => s,
                                    Err(_) => String::new(),
                                }
                            }
                            Err(_) => String::new(),
                        };
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": {
                                "content": [ {"type": "text", "text": data } ]
                            }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("resources/describe") => {
                        let p = msg
                            .get("params")
                            .and_then(|p| p.get("path"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let meta = std::fs::metadata(p);
                        let (exists, is_file, size) = match meta {
                            Ok(m) => (true, m.is_file(), m.len()),
                            Err(_) => (false, false, 0),
                        };
                        let hash = if exists && is_file {
                            match std::fs::read(p) {
                                Ok(bytes) => format!("md5:{}", format_md5(&bytes)),
                                Err(_) => String::new(),
                            }
                        } else {
                            String::new()
                        };
                        let mime = guess_mime(p);
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": { "exists": exists, "is_file": is_file, "size": size, "hash": hash, "mime": mime }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some("health") | Some("server/health") => {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "result": { "ok": true }
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    Some(other) => {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "error": {"code": -32601, "message": format!("Method not found: {}", other)}
                        });
                        write_response(&mut stdout, &response)?;
                    }
                    None => {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "id": msg.get("id"),
                            "error": {"code": -32600, "message": "Invalid Request: missing method"}
                        });
                        write_response(&mut stdout, &response)?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    eprintln!("👋 GitAI MCP Server (stdio) shutting down");
    Ok(())
}

fn format_md5(bytes: &[u8]) -> String {
    let digest = md5::compute(bytes);
    format!("{:032x}", digest)
}

fn guess_mime(path: &str) -> String {
    if let Some(ext) = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        match ext.to_ascii_lowercase().as_str() {
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" => "application/javascript",
            "ts" => "application/typescript",
            "java" => "text/x-java",
            "go" => "text/x-go",
            "c" | "h" => "text/x-c",
            "cpp" | "hpp" | "cc" | "cxx" | "hxx" => "text/x-c++",
            "json" => "application/json",
            "toml" => "application/toml",
            "yml" | "yaml" => "application/yaml",
            "md" => "text/markdown",
            "txt" => "text/plain",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "pdf" => "application/pdf",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            _ => "application/octet-stream",
        }
        .to_string()
    } else {
        "application/octet-stream".to_string()
    }
}

/// Start a simple TCP-based MCP server that forwards JSON-RPC lines to the shared processor.
pub async fn start_mcp_tcp_server(config: Config, addr: &str) -> McpResult<()> {
    eprintln!("🚀 GitAI MCP Server (tcp) starting on {}...", addr);
    let config = std::sync::Arc::new(config);
    let _services: Vec<Box<dyn McpService>> = ServiceFactory::create_services(config.clone());
    let tools_listing = build_tools_listing();

    // 共享服务管理器
    let mcp_manager = GitAiMcpManager::new((*config).clone())
        .await
        .map_err(|e| execution_failed_error(format!("init manager failed: {}", e)))?;
    let manager = std::sync::Arc::new(tokio::sync::RwLock::new(mcp_manager));

    let listener = TcpListener::bind(addr)
        .map_err(|e| execution_failed_error(format!("bind failed: {}", e)))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tools = tools_listing.clone();
                let manager = manager.clone();
                let config = config.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_tcp_client(stream, tools, manager, &config) {
                        eprintln!("[tcp] client error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("[tcp] accept error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_tcp_client(
    stream: TcpStream,
    tools_listing: Vec<Value>,
    manager: std::sync::Arc<tokio::sync::RwLock<GitAiMcpManager>>,
    config: &std::sync::Arc<Config>,
) -> Result<(), String> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    eprintln!("[tcp] client connected: {}", peer);
    let mut writer = stream.try_clone().map_err(|e| e.to_string())?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let resp = json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":format!("Parse error: {}", e)}});
                writeln!(&mut writer, "{}", resp).map_err(|e| e.to_string())?;
                continue;
            }
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let response = rt.block_on(process_message(&msg, &tools_listing, &manager, config));
        writeln!(&mut writer, "{}", response).map_err(|e| e.to_string())?;
    }
    eprintln!("[tcp] client disconnected: {}", peer);
    Ok(())
}

/// Process a JSON-RPC request and return a JSON-RPC response value.
pub async fn process_message(
    msg: &Value,
    tools_listing: &Vec<Value>,
    _manager: &std::sync::Arc<tokio::sync::RwLock<GitAiMcpManager>>,
    _config: &std::sync::Arc<Config>,
) -> Value {
    if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
        match method {
            "initialize" => json!({
                "jsonrpc":"2.0","id": msg.get("id"),
                "result":{
                    "protocolVersion": msg.get("params").and_then(|p| p.get("protocolVersion")).cloned().unwrap_or(json!("2025-03-26")),
                    "capabilities": {"tools": {"listChanged": true}},
                    "serverInfo": {"name":"gitai","version":"1.0.0"}
                }
            }),
            "tools/list" => {
                json!({"jsonrpc":"2.0","id": msg.get("id"),"result":{"tools": tools_listing }})
            }
            "tools/call" => {
                let tool_name = msg
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let arguments = msg
                    .get("params")
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(json!({}));
                // Map known tools to index
                let name_to_idx = [
                    ("execute_review", 0),
                    ("execute_scan", 1),
                    ("execute_commit", 2),
                    ("execute_analysis", 3),
                    ("execute_dependency_graph", 4),
                    ("analyze_deviation", 5),
                    ("summarize_graph", 6),
                    ("query_call_chain", 7),
                ]
                .into_iter()
                .collect::<std::collections::HashMap<_, _>>();
                if let Some(idx) = name_to_idx.get(tool_name) {
                    let start = std::time::Instant::now();
                    let services =
                        crate::services::ServiceFactory::create_services(_config.clone());
                    let result = if let Some(svc) = services.get(*idx) {
                        svc.execute(arguments).await
                    } else {
                        Err(crate::error::execution_failed_error(format!(
                            "Service not found for tool: {}",
                            tool_name
                        )))
                    };
                    match result {
                        Ok(val) => {
                            // Convert JSON result to text format for MCP protocol compliance
                            let text_content = serde_json::to_string_pretty(&val)
                                .unwrap_or_else(|_| val.to_string());
                            json!({
                                "jsonrpc":"2.0","id": msg.get("id"),
                                "result": {
                                    "content":[{"type":"text","text": text_content}], 
                                    "usage": {"duration_ms": start.elapsed().as_millis()}
                                }
                            })
                        },
                        Err(e) => json!({
                            "jsonrpc":"2.0","id": msg.get("id"),
                            "error": {"code": -32000, "message": format!("Tool execution failed: {}", e), "data": {"type":"ExecutionFailed"}}
                        }),
                    }
                } else {
                    json!({"jsonrpc":"2.0","id": msg.get("id"),"error":{"code":-32601,"message":format!("Unknown tool: {}", tool_name)}})
                }
            }
            _ => {
                json!({"jsonrpc":"2.0","id": msg.get("id"),"error":{"code":-32601,"message":format!("Method not found: {}", method)}})
            }
        }
    } else {
        json!({"jsonrpc":"2.0","id": msg.get("id"),"error":{"code":-32600,"message":"Invalid Request: missing method"}})
    }
}

/// Build the static tools listing used by MCP clients.
pub fn build_tools_listing() -> Vec<Value> {
    vec![
        json!({
            "name": "execute_review",
            "description": "执行代码评审（可选 Issue 关联）",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "issue_ids": {"type": "array", "items": {"type": "string"}},
                    "format": {"type": "string", "enum": ["text","json","markdown"]},
                    "tree_sitter": {"type": "boolean"},
                    "security_scan": {"type": "boolean"}
                },
                "required": []
            }
        }),
        json!({
            "name": "execute_scan",
            "description": "执行安全扫描（多语言）",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "lang": {"type": "string"},
                    "timeout": {"type": "integer"}
                },
                "required": []
            }
        }),
        json!({
            "name": "execute_commit",
            "description": "智能提交（AI 生成信息）",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "add_all": {"type": "boolean"},
                    "dry_run": {"type": "boolean"},
                    "message": {"type": "string"}
                },
                "required": []
            }
        }),
        json!({
            "name": "execute_analysis",
            "description": "多语言代码结构分析",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "language": {"type": "string", "enum": ["rust","java","c","cpp","python","go","javascript","typescript"]},
                    "verbosity": {"type": "integer", "minimum": 0, "maximum": 2}
                },
                "required": []
            }
        }),
        json!({
            "name": "execute_dependency_graph",
            "description": "生成完整代码依赖图（注意：大型项目输出可能非常庞大，建议使用 summarize_graph）",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "format": {"type": "string", "enum": ["json","dot","svg","mermaid","ascii"]},
                    "include_calls": {"type": "boolean"},
                    "include_imports": {"type": "boolean"}
                },
                "required": []
            }
        }),
        json!({
            "name": "analyze_deviation",
            "description": "分析代码变更与 Issue 的偏离度",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "diff": {"type": "string"},
                    "issue_ids": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["issue_ids"]
            }
        }),
        json!({
            "name": "summarize_graph",
            "description": "依赖图智能摘要（推荐：默认使用此工具进行依赖分析，支持预算控制和自适应裁剪）",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "扫描目录（默认 .）"},
                    "radius": {"type": "integer", "minimum": 1, "description": "从种子出发的邻域半径（默认1）"},
                    "top_k": {"type": "integer", "minimum": 1, "description": "Top节点上限（默认200）"},
                    "budget_tokens": {"type": "integer", "minimum": 0, "description": "预算token用于自适应裁剪（默认3000）"},
                    "format": {"type": "string", "enum": ["json","text"], "description": "输出格式（默认json）"},
                    "community": {"type": "boolean", "description": "启用社区压缩（v1）"},
                    "comm_alg": {"type": "string", "enum": ["labelprop"], "description": "社区检测算法（默认labelprop）"},
                    "max_communities": {"type": "integer", "minimum": 1, "description": "社区数量上限（默认50）"},
                    "max_nodes_per_community": {"type": "integer", "minimum": 1, "description": "每个社区展示节点上限（默认10）"},
                    "with_paths": {"type": "boolean", "description": "启用路径采样（v2）"},
                    "path_samples": {"type": "integer", "minimum": 0, "description": "路径样本数量（默认5）"},
                    "path_max_hops": {"type": "integer", "minimum": 1, "description": "单条路径最大跳数（默认5）"},
                    "seeds_from_diff": {"type": "boolean", "description": "从 git diff 推导变更种子（默认false）"}
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "query_call_chain",
            "description": "查询函数调用链（上游/下游），可设定最大深度与路径数量",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "扫描目录（默认 .）"},
                    "start": {"type": "string", "description": "起始函数名（必需）"},
                    "end": {"type": "string", "description": "结束函数名（可选）"},
                    "direction": {"type": "string", "enum": ["downstream","upstream"], "description": "方向：下游(被调用方)/上游(调用方)，默认 downstream"},
                    "max_depth": {"type": "integer", "minimum": 1, "maximum": 32, "description": "最大深度，默认 8"},
                    "max_paths": {"type": "integer", "minimum": 1, "maximum": 100, "description": "最多返回路径数，默认 20"}
                },
                "required": ["start"]
            }
        }),
    ]
}
