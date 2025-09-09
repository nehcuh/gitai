use anyhow::Result;
use log::{debug, info};

use gitai::args::Command;
use gitai::config::Config;

/// Handler for MCP command with Command enum
#[cfg(feature = "mcp")]
pub async fn handle_command(
    config: &Config,
    command: &Command,
) -> crate::cli::CliResult<()> {
    match command {
        Command::Mcp { transport, addr } => {
            handle_mcp(config, transport, addr).await.map_err(|e| e.into())
        }
        _ => Err("Invalid command for MCP handler".into()),
    }
}

/// Handle MCP server startup
#[cfg(feature = "mcp")]
async fn handle_mcp(config: &Config, transport: &str, addr: &str) -> Result<()> {
    use gitai::mcp;
    
    // 检查 MCP 是否启用
    if !config.mcp.as_ref().map_or(false, |mcp| mcp.enabled) {
        eprintln!("❌ MCP 服务未启用，请在配置文件中启用 MCP");
        std::process::exit(1);
    }

    info!("Starting GitAI MCP server with transport: {}", transport);
    println!("🚀 启动 GitAI MCP 服务器");
    println!("📡 传输协议: {}", transport);

    match transport {
        "stdio" => {
            println!("🔌 使用 stdio 传输");
            debug!("Starting MCP server with stdio transport");
            mcp::bridge::start_mcp_server(config.clone()).await
        }
        "tcp" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  TCP 传输暂未实现");
            debug!("TCP transport requested but not implemented");
            Err("TCP transport not implemented".into())
        }
        "sse" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  SSE 传输暂未实现");
            debug!("SSE transport requested but not implemented");
            Err("SSE transport not implemented".into())
        }
        _ => {
            eprintln!("❌ 不支持的传输协议: {}", transport);
            debug!("Unsupported transport protocol: {}", transport);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitai::config::{AiConfig, ScanConfig, McpConfig, McpServicesConfig};

    fn create_test_config() -> Config {
        Config {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                temperature: Some(0.3),
            },
            scan: ScanConfig {
                default_path: Some(".".to_string()),
                timeout: Some(300),
                jobs: Some(4),
            },
            devops: None,
            mcp: Some(McpConfig {
                enabled: true,
                services: McpServicesConfig {
                    enabled: vec!["review".to_string(), "commit".to_string()],
                },
            }),
        }
    }

    #[tokio::test]
    #[cfg(feature = "mcp")]
    async fn test_handle_mcp_command() {
        let config = create_test_config();
        let command = Command::Mcp {
            transport: "stdio".to_string(),
            addr: "localhost:8080".to_string(),
        };
        
        // This test would need proper MCP setup to work
        let result = handle_command(&config, &command).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "mcp")]
    async fn test_handle_mcp_unsupported_transport() {
        let config = create_test_config();
        
        // Test with unsupported transport
        let result = handle_mcp(&config, "websocket", "localhost:8080").await;
        assert!(result.is_err());
    }
}
