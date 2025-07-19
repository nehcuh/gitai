// GitAI 集成测试套件
//
// 简化版测试，只测试实际可用的功能

use std::sync::Arc;
use std::collections::HashMap;

use gitai::config::AppConfig;
use gitai::mcp::{
    GitAiMcpConfig, 
    init_gitai_mcp_manager,
    registry::McpServiceRegistry,
};

/// 测试基本的 MCP 服务注册表功能
#[tokio::test]
async fn test_mcp_service_registry_basic() {
    let registry = McpServiceRegistry::new();
    
    // 1. 注册服务
    registry.register("test-service".to_string()).await.unwrap();
    
    // 2. 验证服务注册
    assert!(registry.is_registered("test-service").await);
    
    // 3. 检查活跃服务
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 1);
    assert_eq!(active_services[0], "test-service");
    
    // 4. 列出所有服务
    let all_services = registry.list_services().await;
    assert_eq!(all_services.len(), 1);
    assert_eq!(all_services[0].name, "test-service");
    assert!(all_services[0].active);
    
    // 5. 更新服务状态
    registry.update_service_status("test-service", false).await.unwrap();
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 0);
    
    // 6. 恢复服务状态
    registry.update_service_status("test-service", true).await.unwrap();
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 1);
    
    // 7. 注销服务
    registry.unregister("test-service").await.unwrap();
    assert!(!registry.is_registered("test-service").await);
    
    println!("✅ MCP 服务注册表基本功能测试通过");
}

/// 测试应用配置创建
#[tokio::test]
async fn test_app_config_creation() {
    // 使用 from_partial_and_env 创建配置
    let config = AppConfig::from_partial_and_env(
        None, 
        HashMap::new(),
        HashMap::new()
    );
    
    assert!(config.is_ok());
    
    println!("✅ 应用配置创建测试通过");
}

/// 测试 MCP 管理器初始化
#[tokio::test]
async fn test_mcp_manager_initialization() {
    let mcp_config = GitAiMcpConfig::default();
    let manager = init_gitai_mcp_manager(Some(mcp_config)).await;
    
    assert!(manager.is_ok());
    
    println!("✅ MCP 管理器初始化测试通过");
}