//! HTTP/SSE server for GitAI MCP

use std::convert::Infallible;
use std::sync::Arc;

use futures_util::stream;
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;
use warp::{http::StatusCode, Filter};

use crate::bridge::{build_tools_listing, process_message};
use crate::error::{execution_failed_error, McpResult};
use crate::GitAiMcpManager;
use gitai_core::config::Config;

/// Start an HTTP/SSE MCP server using warp.
pub async fn start_mcp_http_server(config: Config, addr: &str) -> McpResult<()> {
    eprintln!("🚀 GitAI MCP Server (http) starting on {}...", addr);

    let config = Arc::new(config);
    let tools_listing = build_tools_listing();
    let tools_listing_arc = Arc::new(tools_listing);

    // Initialize manager (shared)
    let mcp_manager = GitAiMcpManager::new((*config).clone())
        .await
        .map_err(|e| execution_failed_error(format!("init manager failed: {}", e)))?;
    let manager: Arc<RwLock<GitAiMcpManager>> = Arc::new(RwLock::new(mcp_manager));

    // Inject shared state via warp filters
    let with_tools = {
        let tools = tools_listing_arc.clone();
        warp::any().map(move || tools.clone())
    };
    let with_manager = {
        let manager = manager.clone();
        warp::any().map(move || manager.clone())
    };
    let with_config = {
        let cfg = config.clone();
        warp::any().map(move || cfg.clone())
    };

    // POST /rpc -> JSON-RPC proxy
    let rpc = warp::path("rpc")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_tools.clone())
        .and(with_manager.clone())
        .and(with_config.clone())
        .and_then(
            |msg: Value,
             tools: Arc<Vec<Value>>,
             manager: Arc<RwLock<GitAiMcpManager>>,
             cfg: Arc<Config>| async move {
                if let Some(arr) = msg.as_array() {
                    // JSON-RPC batch
                    let mut out: Vec<Value> = Vec::with_capacity(arr.len());
                    for m in arr {
                        let resp = process_message(m, &tools, &manager, &cfg).await;
                        out.push(resp);
                    }
                    Ok::<_, Infallible>(warp::reply::with_status(
                        warp::reply::json(&Value::Array(out)),
                        StatusCode::OK,
                    ))
                } else {
                    let resp = process_message(&msg, &tools, &manager, &cfg).await;
                    Ok::<_, Infallible>(warp::reply::with_status(
                        warp::reply::json(&resp),
                        StatusCode::OK,
                    ))
                }
            },
        );

    // GET /health
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&json!({"ok": true})));

    // GET /tools
    let tools = warp::path("tools")
        .and(warp::get())
        .and(with_tools.clone())
        .map(|tools: Arc<Vec<Value>>| warp::reply::json(&json!({"tools": *tools })));

    // GET /events (SSE) - immediate ready event, then heartbeat every 15s
    let events = warp::path("events").and(warp::get()).map(|| {
        let ready = stream::once(async {
            Ok::<_, Infallible>(
                warp::sse::Event::default()
                    .event("ready")
                    .data("{\"ok\":true}"),
            )
        });
        let heartbeats = IntervalStream::new(interval(Duration::from_secs(15))).map(|_| {
            Ok::<_, Infallible>(warp::sse::Event::default().event("heartbeat").data("ok"))
        });
        let stream = ready.chain(heartbeats);
        warp::sse::reply(warp::sse::keep_alive().stream(stream))
    });

    let routes = rpc
        .or(health)
        .or(tools)
        .or(events)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(["content-type"])
                .allow_methods(["GET", "POST"]),
        )
        .with(warp::log("gitai_mcp_http"));

    let sock_addr: std::net::SocketAddr = addr
        .parse()
        .map_err(|e| execution_failed_error(format!("Invalid addr {}: {}", addr, e)))?;
    warp::serve(routes).run(sock_addr).await;
    eprintln!("👋 GitAI MCP Server (http) shutting down");
    Ok(())
}
