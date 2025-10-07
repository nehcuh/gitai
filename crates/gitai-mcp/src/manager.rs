//! MCP 服务管理器 - 简化版本

use crate::error::McpResult;
use log::info;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let config = gitai_core::config::Config::default();
        let manager = GitAiMcpManager::new(config).await;

        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_manager_start() {
        let config = gitai_core::config::Config::default();
        let manager = GitAiMcpManager::new(config).await.unwrap();

        // 启动管理器
        let result = manager.start().await;
        assert!(result.is_ok());

        // 验证管理器状态
        assert!(manager.is_running());
    }

    #[tokio::test]
    async fn test_manager_double_start() {
        let config = gitai_core::config::Config::default();
        let manager = GitAiMcpManager::new(config).await.unwrap();

        // 第一次启动
        let result1 = manager.start().await;
        assert!(result1.is_ok());

        // 第二次启动应该成功但只是警告
        let result2 = manager.start().await;
        assert!(result2.is_ok());

        // 验证管理器仍在运行
        assert!(manager.is_running());
    }
}

/// GitAI MCP 服务管理器 - 简化版本
pub struct GitAiMcpManager {
    /// 运行状态
    is_running: std::sync::atomic::AtomicBool,
}

impl GitAiMcpManager {
    /// 创建新的 MCP 服务管理器
    pub async fn new(_config: gitai_core::config::Config) -> McpResult<Self> {
        info!("🔧 初始化 GitAI MCP 服务管理器");
        Ok(Self {
            is_running: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// 启动管理器
    pub async fn start(&self) -> McpResult<()> {
        if self
            .is_running
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            log::warn!("⚠️ 管理器已经在运行");
            return Ok(());
        }

        info!("🚀 启动 GitAI MCP 服务管理器");
        info!("✅ GitAI MCP 服务管理器启动完成");
        Ok(())
    }

    /// 停止管理器
    pub async fn stop(&self) -> McpResult<()> {
        if !self
            .is_running
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            log::warn!("⚠️ 管理器未运行");
            return Ok(());
        }

        info!("⏹️ 停止 GitAI MCP 服务管理器");
        info!("✅ GitAI MCP 服务管理器已停止");
        Ok(())
    }

    /// 检查管理器状态
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::Relaxed)
    }
}
