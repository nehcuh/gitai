//! MCP 桥接模块 - 简化版本

use log::info;

/// 启动 MCP 服务器 - 简化实现
pub async fn start_mcp_server(_config: gitai_core::config::Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🚀 启动 GitAI MCP 服务器");
    
    // 简化实现，避免复杂的 rmcp 依赖
    info!("✅ MCP 服务器启动完成（模拟模式）");
    
    Ok(())
}