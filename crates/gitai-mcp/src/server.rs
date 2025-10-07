//! MCP 服务器模块 - 简化版本

use log::info;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = McpServer::new();
        // 验证服务器初始状态
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_server_start() {
        let server = McpServer::new();

        // 启动服务器
        let result = server.start().await;
        assert!(result.is_ok());

        // 验证服务器状态
        assert!(server.is_running());
    }

    #[tokio::test]
    async fn test_server_double_start() {
        let server = McpServer::new();

        // 第一次启动
        let result1 = server.start().await;
        assert!(result1.is_ok());

        // 第二次启动应该成功但只是警告
        let result2 = server.start().await;
        assert!(result2.is_ok());

        // 验证服务器仍在运行
        assert!(server.is_running());
    }

    #[tokio::test]
    async fn test_server_stop() {
        let server = McpServer::new();

        // 启动服务器
        let _ = server.start().await;
        assert!(server.is_running());

        // 停止服务器
        let result = server.stop().await;
        assert!(result.is_ok());

        // 验证服务器已停止
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_server_stop_when_not_running() {
        let server = McpServer::new();
        assert!(!server.is_running());

        // 停止未运行的服务器应该成功但只是警告
        let result = server.stop().await;
        assert!(result.is_ok());

        // 验证服务器仍然未运行
        assert!(!server.is_running());
    }
}

/// MCP 服务器实现 - 简化版本
pub struct McpServer {
    /// 运行状态
    is_running: std::sync::atomic::AtomicBool,
}

impl McpServer {
    /// 创建新的 MCP 服务器
    pub fn new() -> Self {
        Self {
            is_running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self
            .is_running
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            log::warn!("⚠️ 服务器已经在运行");
            return Ok(());
        }

        info!("🚀 启动 GitAI MCP 服务器");
        info!("✅ GitAI MCP 服务器启动完成（模拟模式）");
        Ok(())
    }

    /// 停止服务器
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self
            .is_running
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            log::warn!("⚠️ 服务器未运行");
            return Ok(());
        }

        info!("⏹️ 停止 GitAI MCP 服务器");
        info!("✅ GitAI MCP 服务器已停止");
        Ok(())
    }

    /// 检查服务器状态
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// 传输协议类型
#[derive(Debug, Clone)]
pub enum TransportProtocol {
    /// 标准输入输出
    Stdio,
    /// HTTP
    Http,
    /// WebSocket
    WebSocket,
    /// TCP
    Tcp,
}

impl std::fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportProtocol::Stdio => write!(f, "stdio"),
            TransportProtocol::Http => write!(f, "http"),
            TransportProtocol::WebSocket => write!(f, "websocket"),
            TransportProtocol::Tcp => write!(f, "tcp"),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
