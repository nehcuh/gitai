// MCP (Model Context Protocol) 服务化模块
// 
// 该模块使用 rmcp 库实现 GitAI 的 MCP 服务化架构，包括：
// - 服务注册和发现
// - 服务实现
// - 客户端集成
// - 传输协议支持

pub mod services;
pub mod registry;
pub mod client;
pub mod utils;
pub mod rmcp_compat; // API 兼容性适配层

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// 重新导出 rmcp 的核心类型
pub use rmcp::{
    handler::server::ServerHandler,
    service::ServiceError,
    model::{Tool, Resource, ServerInfo, InitializeResult},
    transport,
};

// 类型别名
pub type RmcpResult<T> = Result<T, ServiceError>;
pub type RmcpError = ServiceError;

/// GitAI MCP 服务管理器
pub struct GitAiMcpManager {
    /// 服务注册表
    registry: registry::McpServiceRegistry,
    /// 活跃的服务实例
    active_services: HashMap<String, Box<dyn McpService + Send + Sync>>,
    /// 服务配置
    config: GitAiMcpConfig,
}

/// GitAI MCP 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAiMcpConfig {
    /// 启用的服务列表
    pub enabled_services: Vec<String>,
    /// 服务端口配置
    pub service_ports: HashMap<String, u16>,
    /// 传输协议配置
    pub transport_config: TransportConfig,
    /// 服务发现配置
    pub discovery_config: DiscoveryConfig,
}

/// 传输协议配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// 默认传输协议
    pub default_transport: String,
    /// STDIO 传输配置
    pub stdio_config: StdioConfig,
    /// SSE 传输配置
    pub sse_config: Option<SseConfig>,
}

/// STDIO 传输配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioConfig {
    /// 是否启用
    pub enabled: bool,
    /// 缓冲区大小
    pub buffer_size: usize,
}

/// SSE 传输配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseConfig {
    /// 是否启用
    pub enabled: bool,
    /// 监听地址
    pub bind_address: String,
    /// 端口
    pub port: u16,
}

/// 服务发现配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// 是否启用服务发现
    pub enabled: bool,
    /// 服务注册超时时间
    pub registration_timeout: u64,
    /// 健康检查间隔
    pub health_check_interval: u64,
}

/// GitAI MCP 服务抽象
/// 
/// 为了保持 trait object 安全性，我们使用同步方法来管理服务状态
pub trait McpService: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;
    
    /// 获取服务版本
    fn version(&self) -> &str;
    
    /// 获取服务描述
    fn description(&self) -> &str;
    
    /// 启动服务（同步方法）
    fn start_sync(&mut self) -> RmcpResult<()>;
    
    /// 停止服务（同步方法）
    fn stop_sync(&mut self) -> RmcpResult<()>;
    
    /// 健康检查（同步方法）
    fn health_check_sync(&self) -> RmcpResult<bool>;
    
    /// 获取服务的 MCP 服务器处理器 (返回处理器名称，由于trait object限制)
    fn get_handler_info(&self) -> String;
}

impl GitAiMcpManager {
    /// 创建新的 MCP 管理器
    pub fn new(config: GitAiMcpConfig) -> Self {
        Self {
            registry: registry::McpServiceRegistry::new(),
            active_services: HashMap::new(),
            config,
        }
    }

    /// 注册服务
    pub async fn register_service(&mut self, service: Box<dyn McpService + Send + Sync>) -> RmcpResult<()> {
        let service_name = service.name().to_string();
        
        // 检查服务是否已启用
        if !self.config.enabled_services.contains(&service_name) {
            tracing::info!("服务 '{}' 未启用，跳过注册", service_name);
            return Ok(());
        }

        // 注册到服务注册表
        self.registry.register(service_name.clone()).await?;
        
        // 启动服务
        let mut service = service;
        service.start_sync()?;
        
        // 添加到活跃服务列表
        self.active_services.insert(service_name.clone(), service);
        
        tracing::info!("服务 '{}' 注册成功", service_name);
        Ok(())
    }

    /// 注销服务
    pub async fn unregister_service(&mut self, service_name: &str) -> RmcpResult<()> {
        if let Some(mut service) = self.active_services.remove(service_name) {
            service.stop_sync()?;
            self.registry.unregister(service_name).await?;
            tracing::info!("服务 '{}' 注销成功", service_name);
        }
        Ok(())
    }

    /// 获取服务
    pub fn get_service(&self, service_name: &str) -> Option<&(dyn McpService + Send + Sync)> {
        self.active_services.get(service_name).map(|s| s.as_ref())
    }

    /// 列出所有活跃服务
    pub fn list_active_services(&self) -> Vec<&str> {
        self.active_services.keys().map(|s| s.as_str()).collect()
    }

    /// 启动所有服务
    pub async fn start_all_services(&mut self) -> RmcpResult<()> {
        for service_name in self.config.enabled_services.clone() {
            if let Some(service) = self.active_services.get_mut(&service_name) {
                service.start_sync()?;
                tracing::info!("服务 '{}' 启动成功", service_name);
            }
        }
        Ok(())
    }

    /// 停止所有服务
    pub async fn stop_all_services(&mut self) -> RmcpResult<()> {
        for (service_name, service) in self.active_services.iter_mut() {
            service.stop_sync()?;
            tracing::info!("服务 '{}' 停止成功", service_name);
        }
        Ok(())
    }

    /// 健康检查所有服务
    pub fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        
        for (service_name, service) in &self.active_services {
            let health = service.health_check_sync().unwrap_or(false);
            results.insert(service_name.clone(), health);
        }
        
        results
    }
}

/// 默认配置
impl Default for GitAiMcpConfig {
    fn default() -> Self {
        Self {
            enabled_services: vec![
                "gitai-treesitter-service".to_string(),
                "gitai-ai-analysis-service".to_string(),
                "gitai-devops-service".to_string(),
                "gitai-rule-management-service".to_string(),
                "gitai-scanner-service".to_string(),
                "gitai-git-service".to_string(),
            ],
            service_ports: HashMap::new(),
            transport_config: TransportConfig {
                default_transport: "stdio".to_string(),
                stdio_config: StdioConfig {
                    enabled: true,
                    buffer_size: 8192,
                },
                sse_config: None,
            },
            discovery_config: DiscoveryConfig {
                enabled: true,
                registration_timeout: 30,
                health_check_interval: 60,
            },
        }
    }
}

/// 创建 MCP 服务
pub async fn create_mcp_service<H: ServerHandler + 'static>(
    handler: H,
    transport_type: &str,
) -> RmcpResult<()> {
    match transport_type {
        "stdio" => {
            // Use rmcp 0.2.1 API structure - service creation is handled differently
            tracing::info!("MCP 服务已配置为使用 stdio 传输");
            Ok(())
        }
        _ => {
            return Err(rmcp_compat::ServiceError::invalid_params(
                format!("不支持的传输协议: {}", transport_type)
            ).into());
        }
    }
}

/// 初始化 GitAI MCP 服务管理器
pub async fn init_gitai_mcp_manager(config: Option<GitAiMcpConfig>) -> RmcpResult<GitAiMcpManager> {
    let config = config.unwrap_or_default();
    let mut manager = GitAiMcpManager::new(config);

    // 注册默认服务
    // 注意：这里需要具体的服务实现，我们稍后会创建
    
    tracing::info!("GitAI MCP 服务管理器初始化完成");
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GitAiMcpConfig::default();
        assert_eq!(config.enabled_services.len(), 5);
        assert!(config.enabled_services.contains(&"gitai-treesitter-service".to_string()));
        assert!(config.transport_config.stdio_config.enabled);
        assert!(config.discovery_config.enabled);
    }

    #[test]
    fn test_manager_creation() {
        let config = GitAiMcpConfig::default();
        let manager = GitAiMcpManager::new(config);
        assert_eq!(manager.list_active_services().len(), 0);
    }

    #[tokio::test]
    async fn test_init_manager() {
        let result = init_gitai_mcp_manager(None).await;
        assert!(result.is_ok());
    }
}