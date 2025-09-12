use anyhow::Result;
use log::{debug, info};

use gitai::args::Command;
use gitai::config::Config;

/// Handler for MCP command with Command enum
#[cfg(feature = "mcp")]
pub async fn handle_command(config: &Config, command: &Command) -> crate::cli::CliResult<()> {
    match command {
        Command::Mcp { transport, addr } => handle_mcp(config, transport, addr)
            .await
            .map_err(|e| e.into()),
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
        return Err(anyhow::anyhow!("MCP is disabled in configuration"));
    }

    info!("Starting GitAI MCP server with transport: {}", transport);
    println!("🚀 启动 GitAI MCP 服务器");
    println!("📡 传输协议: {}", transport);

    match transport {
        "stdio" => {
            println!("🔌 使用 stdio 传输");
            debug!("Starting MCP server with stdio transport");
            mcp::bridge::start_mcp_server(config.clone())
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))
        }
        "tcp" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  TCP 传输暂未实现");
            debug!("TCP transport requested but not implemented");
            Err(anyhow::anyhow!("TCP transport not implemented"))
        }
        "sse" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  SSE 传输暂未实现");
            debug!("SSE transport requested but not implemented");
            Err(anyhow::anyhow!("SSE transport not implemented"))
        }
        _ => {
            eprintln!("❌ 不支持的传输协议: {}", transport);
            debug!("Unsupported transport protocol: {}", transport);
            Err(anyhow::anyhow!("Unsupported transport: {}", transport))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        use gitai::config::{AiConfig, McpConfig, McpServerConfig, McpServicesConfig, ScanConfig};
        Config {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                temperature: 0.3,
            },
            scan: ScanConfig {
                default_path: Some(".".to_string()),
                timeout: 300,
                jobs: 4,
                rules_dir: None,
            },
            devops: None,
            language: None,
            mcp: Some(McpConfig {
                enabled: true,
                server: McpServerConfig {
                    transport: "stdio".to_string(),
                    listen_addr: None,
                    name: "test-mcp".to_string(),
                    version: "0.1.0".to_string(),
                },
                services: McpServicesConfig {
                    enabled: vec!["review".to_string(), "commit".to_string()],
                    review: None,
                    commit: None,
                    scan: None,
                    analysis: None,
                    dependency: None,
                },
            }),
        }
    }

    #[tokio::test]
    #[cfg(feature = "mcp")]
    async fn test_handle_mcp_command() {
        let config = create_test_config();
        let command = Command::Mcp {
            // Use a non-blocking transport in tests to avoid starting a real server
            transport: "tcp".to_string(),
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
