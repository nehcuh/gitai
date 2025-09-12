//! MCP 命令处理器
//!
//! 处理 MCP 服务器相关的命令

use crate::args::Command;

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn parse_jsonc(input: &str) -> Result<serde_json::Value, String> {
    // First try strict JSON
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(input) {
        return Ok(v);
    }
    // Naive stripper: remove block comments, then line comments, then trailing commas patterns.
    fn strip_jsonc_loosely(mut s: String) -> String {
        // Remove block comments /* ... */ (non-nested, naive)
        loop {
            if let Some(start) = s.find("/*") {
                if let Some(end) = s[start + 2..].find("*/") {
                    let _end_idx = start + 2 + end + 2 - 2; // compute correct slice end
                    let end_abs = start + 2 + end;
                    // remove from start to end_abs+2
                    s.replace_range(start..end_abs + 2, "");
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // Remove line comments // ... (naive: up to end of line)
        let mut out = String::with_capacity(s.len());
        for line in s.lines() {
            if let Some(idx) = line.find("//") {
                out.push_str(&line[..idx]);
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        s = out;
        // Remove trailing commas before ] or } (skip whitespace)
        let mut out2 = String::with_capacity(s.len());
        let mut it = s.chars().peekable();
        let mut in_str = false;
        let mut escape = false;
        while let Some(c) = it.next() {
            if in_str {
                out2.push(c);
                if escape {
                    escape = false;
                    continue;
                }
                match c {
                    '\\' => escape = true,
                    '"' => in_str = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => {
                    out2.push(c);
                    in_str = true;
                }
                ',' => {
                    // look ahead skipping whitespace
                    let mut j = it.clone();
                    let mut skip = false;
                    while let Some(nc) = j.next() {
                        if nc.is_whitespace() {
                            continue;
                        }
                        if nc == ']' || nc == '}' {
                            skip = true;
                        }
                        break;
                    }
                    if !skip {
                        out2.push(',');
                    }
                }
                _ => out2.push(c),
            }
        }
        out2
    }

    let s1 = strip_jsonc_loosely(input.to_string());
    serde_json::from_str::<serde_json::Value>(&s1).map_err(|e| format!("JSONC 解析失败: {}", e))
}

/// 处理 mcp 命令
pub async fn handle_command(
    config: &gitai_core::config::Config,
    command: &Command,
) -> HandlerResult<()> {
    match command {
        Command::Mcp { transport, addr } => {
            eprintln!("🚀 启动 GitAI MCP 服务器");
            eprintln!("📡 传输协议: {}", transport);
            match transport.as_str() {
                "stdio" => {
                    eprintln!("🔌 使用 stdio 传输");
                    gitai_mcp::bridge::start_mcp_server(config.clone()).await?;
                    Ok(())
                }
                "tcp" => {
                    eprintln!("🌐 使用 TCP 传输，监听地址: {}", addr);
                    gitai_mcp::bridge::start_mcp_tcp_server(config.clone(), addr).await?;
                    Ok(())
                }
                "http" | "sse" => {
                    eprintln!("🌐 使用 HTTP/SSE 传输，监听地址: {}", addr);
                    gitai_mcp::http::start_mcp_http_server(config.clone(), addr).await?;
                    Ok(())
                }
                other => {
                    eprintln!("❌ 不支持的传输协议: {}", other);
                    Err(format!("Unsupported transport: {}", other).into())
                }
            }
        }
        Command::McpHealth { url } => {
            let url = url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:8711".to_string());
            let endpoint = format!("{}/health", url.trim_end_matches('/'));
            let res = reqwest::get(&endpoint).await?;
            if res.status().is_success() {
                let v: serde_json::Value = res.json().await?;
                println!("{}", serde_json::to_string_pretty(&v)?);
                Ok(())
            } else {
                Err(format!("HTTP {}: {}", res.status(), endpoint).into())
            }
        }
        Command::McpTools { url } => {
            let url = url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:8711".to_string());
            let endpoint = format!("{}/tools", url.trim_end_matches('/'));
            let res = reqwest::get(&endpoint).await?;
            if res.status().is_success() {
                let v: serde_json::Value = res.json().await?;
                println!("{}", serde_json::to_string_pretty(&v)?);
                Ok(())
            } else {
                Err(format!("HTTP {}: {}", res.status(), endpoint).into())
            }
        }
        Command::McpInfo { url } => {
            let url = url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:8711".to_string());
            let base = url.trim_end_matches('/');
            let health_ep = format!("{}/health", base);
            let tools_ep = format!("{}/tools", base);
            let (health, tools) = tokio::join!(reqwest::get(&health_ep), reqwest::get(&tools_ep));
            let mut info = serde_json::Map::new();
            info.insert(
                "server".into(),
                serde_json::json!({"name":"gitai-mcp","version":"1.0.0"}),
            );
            match health {
                Ok(resp) if resp.status().is_success() => {
                    let v: serde_json::Value = resp
                        .json()
                        .await
                        .unwrap_or(serde_json::json!({"ok": false}));
                    info.insert("health".into(), v);
                }
                Ok(resp) => {
                    info.insert(
                        "health".into(),
                        serde_json::json!({"ok": false, "status": resp.status().as_u16()}),
                    );
                }
                Err(e) => {
                    info.insert(
                        "health".into(),
                        serde_json::json!({"ok": false, "error": e.to_string()}),
                    );
                }
            }
            match tools {
                Ok(resp) if resp.status().is_success() => {
                    let v: serde_json::Value = resp
                        .json()
                        .await
                        .unwrap_or(serde_json::json!({"tools": []}));
                    info.insert("tools".into(), v);
                }
                Ok(resp) => {
                    info.insert(
                        "tools".into(),
                        serde_json::json!({"error": format!("HTTP {}", resp.status())}),
                    );
                }
                Err(e) => {
                    info.insert("tools".into(), serde_json::json!({"error": e.to_string()}));
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::Value::Object(info))?
            );
            Ok(())
        }
        Command::McpCall {
            url,
            name,
            args,
            args_file,
            output,
            raw,
            minify,
            allow_jsonc,
        } => {
            let url = url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:8711".to_string());
            let base = url.trim_end_matches('/');
            let rpc_ep = format!("{}/rpc", base);
            let arguments: serde_json::Value = if let Some(s) = args {
                if !s.trim().is_empty() {
                    if *allow_jsonc {
                        parse_jsonc(s)?
                    } else {
                        serde_json::from_str(s).map_err(|e| format!("args JSON 解析失败: {}", e))?
                    }
                } else {
                    serde_json::json!({})
                }
            } else if let Some(p) = args_file {
                let data = std::fs::read_to_string(p)?;
                if !data.trim().is_empty() {
                    if *allow_jsonc {
                        parse_jsonc(&data)?
                    } else {
                        serde_json::from_str(&data)
                            .map_err(|e| format!("args_file JSON 解析失败: {}", e))?
                    }
                } else {
                    serde_json::json!({})
                }
            } else {
                serde_json::json!({})
            };
            let payload = serde_json::json!({
                "jsonrpc":"2.0",
                "id": 1,
                "method":"tools/call",
                "params": {"name": name, "arguments": arguments}
            });
            let client = reqwest::Client::new();
            let res = client.post(&rpc_ep).json(&payload).send().await?;
            if res.status().is_success() {
                let v: serde_json::Value = res.json().await?;
                let out_val = if *raw {
                    if let Some(contents) = v
                        .get("result")
                        .and_then(|r| r.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        let mut items: Vec<serde_json::Value> = Vec::new();
                        for it in contents {
                            if let Some(j) = it.get("json").cloned() {
                                items.push(j);
                            } else if let Some(t) = it.get("text").cloned() {
                                items.push(t);
                            }
                        }
                        if items.len() == 1 {
                            items.remove(0)
                        } else {
                            serde_json::Value::Array(items)
                        }
                    } else {
                        v.clone()
                    }
                } else {
                    v.clone()
                };
                let out = if *minify {
                    serde_json::to_string(&out_val)?
                } else {
                    serde_json::to_string_pretty(&out_val)?
                };
                if let Some(path) = output.clone() {
                    std::fs::write(&path, out)?;
                } else {
                    println!("{}", out);
                }
                Ok(())
            } else {
                Err(format!("HTTP {}: {}", res.status(), rpc_ep).into())
            }
        }
        Command::McpBatch {
            url,
            file,
            output,
            raw,
            minify,
            concurrency,
            retries,
            allow_jsonc,
        } => {
            let url = url
                .clone()
                .unwrap_or_else(|| "http://127.0.0.1:8711".to_string());
            let base = url.trim_end_matches('/');
            let rpc_ep = format!("{}/rpc", base);
            let data = std::fs::read_to_string(file)?;
            let v = if *allow_jsonc {
                parse_jsonc(&data)?
            } else {
                serde_json::from_str(&data).map_err(|e| format!("batch 文件解析失败: {}", e))?
            };
            let calls: Vec<serde_json::Value> = v
                .as_array()
                .cloned()
                .ok_or_else(|| "batch 文件不是数组".to_string())?;
            // 支持两种格式：数组的 {name, arguments} 或数组的完整 JSON-RPC 请求
            let client = reqwest::Client::new();
            // outputs with metadata (index, name, value)
            let mut outputs: Vec<(usize, String, serde_json::Value)> = Vec::new();
            outputs.resize_with(calls.len(), || {
                (0, String::new(), serde_json::json!({"error":"pending"}))
            });
            use futures_util::stream::{self, StreamExt};
            let stream = stream::iter(calls.into_iter().enumerate())
                .map(|(i, item)| {
                    let client = client.clone();
                    let rpc_ep = rpc_ep.clone();
                    async move {
                        let (payload, name_str) = if item.get("jsonrpc").is_some() {
                            let name = item
                                .get("params")
                                .and_then(|p| p.get("name"))
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            (item, name)
                        } else {
                            let name = item
                                .get("name")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            let arguments = item
                                .get("arguments")
                                .cloned()
                                .unwrap_or(serde_json::json!({}));
                            (
                                serde_json::json!({
                                    "jsonrpc":"2.0","id": i + 1, "method":"tools/call",
                                    "params": {"name": name, "arguments": arguments}
                                }),
                                name,
                            )
                        };
                        let mut attempt = 0usize;
                        let result = loop {
                            let resp = client.post(&rpc_ep).json(&payload).send().await;
                            match resp {
                                Ok(res) => {
                                    if res.status().is_success() {
                                        match res.json::<serde_json::Value>().await {
                                            Ok(v) => break Ok(v),
                                            Err(e) => {
                                                break Err(format!("resp json 解析失败: {}", e))
                                            }
                                        }
                                    } else {
                                        if attempt >= *retries {
                                            break Err(format!("HTTP {}", res.status()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    if attempt >= *retries {
                                        break Err(e.to_string());
                                    }
                                }
                            }
                            attempt += 1;
                        };
                        (i, name_str, result)
                    }
                })
                .buffer_unordered(*concurrency);
            tokio::pin!(stream);
            while let Some((i, name, result)) = stream.next().await {
                match result {
                    Ok(v) => outputs[i] = (i, name, v),
                    Err(e) => outputs[i] = (i, name, serde_json::json!({"error": e})),
                }
            }
            let out_val = if *raw {
                // 合并每个响应的 content 主体到数组，携带 index/name 元数据
                let mut items: Vec<serde_json::Value> = Vec::new();
                for (i, name, v) in &outputs {
                    let mut extracted: Vec<serde_json::Value> = Vec::new();
                    if let Some(contents) = v
                        .get("result")
                        .and_then(|r| r.get("content"))
                        .and_then(|c| c.as_array())
                    {
                        for it in contents {
                            if let Some(j) = it.get("json").cloned() {
                                extracted.push(j);
                            } else if let Some(t) = it.get("text").cloned() {
                                extracted.push(t);
                            }
                        }
                    }
                    let content_val = if extracted.len() == 1 {
                        extracted.remove(0)
                    } else {
                        serde_json::Value::Array(extracted)
                    };
                    items.push(
                        serde_json::json!({"index": i, "name": name, "content": content_val}),
                    );
                }
                serde_json::Value::Array(items)
            } else {
                // 完整响应，携带 index/name 元数据
                let mut items: Vec<serde_json::Value> = Vec::new();
                for (i, name, v) in outputs {
                    items.push(serde_json::json!({"index": i, "name": name, "response": v}));
                }
                serde_json::Value::Array(items)
            };
            let out = if *minify {
                serde_json::to_string(&out_val)?
            } else {
                serde_json::to_string_pretty(&out_val)?
            };
            if let Some(path) = output.clone() {
                std::fs::write(&path, out)?;
            } else {
                println!("{}", out);
            }
            Ok(())
        }
        _ => Err("Invalid command for mcp handler".into()),
    }
}
