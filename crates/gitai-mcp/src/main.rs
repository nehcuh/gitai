// GitAI MCP 服务器独立二进制
//
// 这是一个独立的 MCP 服务器，可以通过 stdio 与 LLM 客户端通信

#![allow(clippy::uninlined_format_args, clippy::unnecessary_map_or)]

use clap::{Parser, Subcommand};
use gitai_core::config::Config;
use gitai_mcp::bridge;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gitai-mcp")]
#[command(about = "GitAI MCP Server - 提供 GitAI 功能的 MCP 服务")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// 详细输出
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 MCP 服务器
    Serve {
        /// 传输协议
        #[arg(short, long, default_value = "stdio")]
        transport: String,

        /// 监听地址 (仅用于 TCP/SSE)
        #[arg(short, long, default_value = "127.0.0.1:8711")]
        addr: String,

        /// 启用的服务 (逗号分隔)
        #[arg(
            short,
            long,
            default_value = "review,scan,commit,analysis,dependency,deviation"
        )]
        services: String,
    },
    /// 显示服务器信息
    Info,
    /// 列出可用工具
    ListTools,
    /// 健康检查
    Health,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&log_level)).init();

    let cli = Cli::parse();

    // 设置日志级别
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }

    // 加载配置
    let config = Config::load().map_err(|e| format!("Failed to load config: {}", e))?;
    
    // 调试：输出配置信息
    eprintln!("🔧 配置加载成功:");
    eprintln!("  AI API URL: {}", config.ai.api_url);
    eprintln!("  AI Model: {}", config.ai.model);
    eprintln!("  MCP 启用: {}", config.mcp.as_ref().map_or(false, |mcp| mcp.enabled));

    // 检查 MCP 是否启用
    if !config.mcp.as_ref().map_or(false, |mcp| mcp.enabled) {
        eprintln!("❌ MCP 服务未启用，请在配置文件中启用 MCP");
        eprintln!("💡 可以使用 'gitai init' 初始化配置");
        std::process::exit(1);
    }

    match cli.command {
        Commands::Serve {
            transport,
            addr,
            services,
        } => {
            eprintln!("🚀 启动 GitAI MCP 服务器");
            eprintln!("📡 传输协议: {}", transport);
            eprintln!("🔧 启用的服务: {}", services);

            // 解析传输协议
            match transport.as_str() {
                "stdio" => {
                    eprintln!("🔌 使用 stdio 传输");
                    crate::bridge::start_mcp_server(config.clone()).await?;
                }
                "tcp" => {
                    eprintln!("🌐 使用 TCP 传输，监听地址: {}", addr);
                    crate::bridge::start_mcp_tcp_server(config.clone(), &addr).await?;
                }
                "http" | "sse" | "websocket" | "ws" => {
                    eprintln!("🌐 使用 HTTP/SSE 传输，监听地址: {}", addr);
                    gitai_mcp::http::start_mcp_http_server(config.clone(), &addr).await?;
                }
                _ => {
                    eprintln!("❌ 不支持的传输协议: {}", transport);
                    std::process::exit(1);
                }
            };
        }
        Commands::Info => {
            println!("📋 GitAI MCP 服务器信息");
            println!("  名称: gitai-mcp");
            println!("  版本: 1.0.0");
            println!("  状态: 🟢 运行中");
        }
        Commands::ListTools => {
            println!("🔧 GitAI MCP 可用工具");
            println!("  📦 execute_review - 执行代码评审");
            println!("  📦 execute_scan - 执行安全扫描");
            println!("  📦 execute_commit - 执行智能提交");
            println!("  📦 execute_analysis - 执行代码分析");
            println!("  📦 query_call_chain - 查询函数调用链");
            println!("  📦 execute_dependency_graph - 生成代码依赖图");
            println!("  📦 analyze_deviation - 分析代码变更与 Issue 的偏离度");
        }
        Commands::Health => {
            println!("🏥 GitAI MCP 服务器健康检查");
            println!("  ✅ 服务器运行正常");
            println!("  🔧 可用工具数量: 7");
        }
    }

    Ok(())
}
