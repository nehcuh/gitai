// GitAI MCP 服务器独立二进制
//
// 这是一个独立的 MCP 服务器，可以通过 stdio 与 LLM 客户端通信

#![allow(clippy::uninlined_format_args, clippy::unnecessary_map_or)]

use clap::{Parser, Subcommand};
use gitai::config::Config;
use gitai::mcp::bridge;
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
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 MCP 服务器
    Serve {
        /// 传输协议
        #[arg(short, long, default_value = "stdio")]
        transport: String,

        /// 监听地址 (仅用于 TCP/SSE)
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        addr: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // 加载配置
    let config = Config::load()?;

    // 检查 MCP 是否启用
    if !config.mcp.as_ref().map_or(false, |mcp| mcp.enabled) {
        eprintln!("❌ MCP 服务未启用，请在配置文件中启用 MCP");
        std::process::exit(1);
    }

    match cli.command {
        Commands::Serve { transport, addr } => {
            println!("🚀 启动 GitAI MCP 服务器");
            println!("📡 传输协议: {}", transport);

            match transport.as_str() {
                "stdio" => {
                    println!("🔌 使用 stdio 传输");
                    bridge::start_mcp_server(config).await?;
                }
                "tcp" => {
                    println!("🌐 监听地址: {}", addr);
                    eprintln!("⚠️  TCP 传输暂未实现");
                }
                "sse" => {
                    println!("🌐 监听地址: {}", addr);
                    eprintln!("⚠️  SSE 传输暂未实现");
                }
                _ => {
                    eprintln!("❌ 不支持的传输协议: {}", transport);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
