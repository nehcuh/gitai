// GitAI 集成测试套件
//
// 这个测试套件验证 GitAI 的各个组件是否能够正常协同工作
//
// 包含以下测试：
// 1. 完整的 MCP 服务生命周期测试
// 2. CLI 与 MCP 服务的集成测试
// 3. 多服务协同工作测试
// 4. 错误处理和恢复测试
// 5. 性能和负载测试

use std::time::Duration;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::sleep;

use gitai::config::AppConfig;
use gitai::mcp::{
    GitAiMcpConfig, 
    init_gitai_mcp_manager,
    services::{
        TreeSitterService,
        AiAnalysisService,
        DevOpsService,
        ScannerService,
        RuleManagementService,
    },
    registry::{McpServiceRegistry, ServiceType, HealthStatus},
};

/// 测试 MCP 服务的完整生命周期
#[tokio::test]
async fn test_mcp_service_lifecycle() {
    // 创建默认配置
    let config = Arc::new(AppConfig::default());
    
    // 初始化 MCP 管理器
    let mcp_config = GitAiMcpConfig::default();
    let mut manager = init_gitai_mcp_manager(Some(mcp_config)).await.unwrap();
    
    // 1. 注册所有服务
    let treesitter_service = TreeSitterService::new(config.tree_sitter.clone());
    manager.register_service(Box::new(treesitter_service)).await.unwrap();
    
    let ai_service = AiAnalysisService::new();
    manager.register_service(Box::new(ai_service)).await.unwrap();
    
    let devops_service = DevOpsService::new();
    manager.register_service(Box::new(devops_service)).await.unwrap();
    
    let scanner_service = ScannerService::new();
    manager.register_service(Box::new(scanner_service)).await.unwrap();
    
    let rule_service = RuleManagementService::new();
    manager.register_service(Box::new(rule_service)).await.unwrap();
    
    // 2. 验证服务注册
    let active_services = manager.list_active_services();
    assert_eq!(active_services.len(), 5);
    
    // 3. 执行健康检查
    let health_status = manager.health_check_all();
    assert_eq!(health_status.len(), 5);
    
    // 4. 验证服务可以正常停止
    manager.stop_all_services().await.unwrap();
    
    // 5. 验证服务可以重新启动
    manager.start_all_services().await.unwrap();
    
    println!("✅ MCP 服务生命周期测试通过");
}

/// 测试服务注册表的高级功能
#[tokio::test]
async fn test_service_registry_advanced_features() {
    let registry = McpServiceRegistry::new();
    
    // 1. 注册不同类型的服务
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "2.0.0".to_string());
    metadata.insert("author".to_string(), "GitAI Team".to_string());
    
    registry.register_with_details(
        "treesitter-advanced".to_string(),
        "2.0.0".to_string(),
        ServiceType::TreeSitter,
        vec!["config-service".to_string()],
        metadata.clone(),
    ).await.unwrap();
    
    registry.register_with_details(
        "ai-analysis-advanced".to_string(),
        "1.5.0".to_string(),
        ServiceType::AiAnalysis,
        vec!["treesitter-advanced".to_string()],
        metadata.clone(),
    ).await.unwrap();
    
    // 2. 测试服务发现
    let ts_services = registry.discover_services_by_type(ServiceType::TreeSitter).await;
    assert_eq!(ts_services.len(), 1);
    assert_eq!(ts_services[0].name, "treesitter-advanced");
    
    let ai_services = registry.discover_services_by_type(ServiceType::AiAnalysis).await;
    assert_eq!(ai_services.len(), 1);
    assert_eq!(ai_services[0].name, "ai-analysis-advanced");
    
    // 3. 测试版本发现
    let v2_services = registry.discover_services_by_version("2.").await;
    assert_eq!(v2_services.len(), 1);
    
    let v1_services = registry.discover_services_by_version("1.").await;
    assert_eq!(v1_services.len(), 1);
    
    // 4. 测试依赖检查
    let missing_deps = registry.check_dependencies("ai-analysis-advanced").await.unwrap();
    assert_eq!(missing_deps.len(), 0); // treesitter-advanced 应该已经注册
    
    // 5. 测试健康检查
    registry.perform_health_check("treesitter-advanced").await.unwrap();
    registry.perform_health_check("ai-analysis-advanced").await.unwrap();
    
    let healthy_services = registry.get_healthy_services().await;
    assert_eq!(healthy_services.len(), 2);
    
    // 6. 测试服务指标
    registry.update_service_metrics(
        "treesitter-advanced",
        10,
        1,
        Some(Duration::from_millis(150)),
    ).await.unwrap();
    
    registry.update_service_metrics(
        "ai-analysis-advanced",
        5,
        0,
        Some(Duration::from_millis(300)),
    ).await.unwrap();
    
    let stats = registry.get_service_stats().await;
    assert_eq!(stats.total_services, 2);
    assert_eq!(stats.active_services, 2);
    assert_eq!(stats.total_requests, 15);
    assert_eq!(stats.total_failures, 1);
    assert!((stats.success_rate - 93.33).abs() < 0.1);
    
    println!("✅ 服务注册表高级功能测试通过");
}

/// 测试服务故障恢复
#[tokio::test]
async fn test_service_failure_recovery() {
    let registry = McpServiceRegistry::new();
    
    // 1. 注册服务
    registry.register("resilient-service".to_string()).await.unwrap();
    
    // 2. 模拟服务故障
    registry.update_service_status("resilient-service", false).await.unwrap();
    
    // 3. 验证服务状态
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 0);
    
    // 4. 模拟服务恢复
    registry.update_service_status("resilient-service", true).await.unwrap();
    
    // 5. 验证服务恢复
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 1);
    
    // 6. 执行健康检查
    let health = registry.perform_health_check("resilient-service").await.unwrap();
    assert_eq!(health, HealthStatus::Unknown);
    
    println!("✅ 服务故障恢复测试通过");
}

/// 测试并发服务操作
#[tokio::test]
async fn test_concurrent_service_operations() {
    let registry = Arc::new(McpServiceRegistry::new());
    
    // 并发注册多个服务
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let registry = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let service_name = format!("concurrent-service-{}", i);
            registry.register(service_name).await.unwrap();
        });
        handles.push(handle);
    }
    
    // 等待所有服务注册完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 验证所有服务都已注册
    let services = registry.list_services().await;
    assert_eq!(services.len(), 10);
    
    // 并发更新服务状态
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let registry = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let service_name = format!("concurrent-service-{}", i);
            registry.update_service_status(&service_name, i % 2 == 0).await.unwrap();
        });
        handles.push(handle);
    }
    
    // 等待所有操作完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 验证服务状态
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 5); // 一半服务应该是活跃的
    
    println!("✅ 并发服务操作测试通过");
}

/// 测试服务指标统计
#[tokio::test]
async fn test_service_metrics_tracking() {
    let registry = McpServiceRegistry::new();
    
    // 注册测试服务
    registry.register("metrics-service".to_string()).await.unwrap();
    
    // 模拟服务请求
    let request_counts = vec![5, 3, 8, 2, 10];
    let failure_counts = vec![0, 1, 0, 0, 2];
    let response_times = vec![100, 200, 150, 80, 300];
    
    for (i, &requests) in request_counts.iter().enumerate() {
        registry.update_service_metrics(
            "metrics-service",
            requests,
            failure_counts[i],
            Some(Duration::from_millis(response_times[i])),
        ).await.unwrap();
    }
    
    // 验证指标
    let service = registry.get_service("metrics-service").await.unwrap();
    assert_eq!(service.metrics.total_requests, 28);
    assert_eq!(service.metrics.failed_requests, 3);
    assert_eq!(service.metrics.last_response_time, Some(Duration::from_millis(300)));
    
    let stats = registry.get_service_stats().await;
    assert_eq!(stats.total_requests, 28);
    assert_eq!(stats.total_failures, 3);
    assert!((stats.success_rate - 89.29).abs() < 0.1);
    
    println!("✅ 服务指标统计测试通过");
}

/// 测试服务依赖管理
#[tokio::test]
async fn test_service_dependency_management() {
    let registry = McpServiceRegistry::new();
    
    // 注册基础服务
    registry.register_with_details(
        "base-service".to_string(),
        "1.0.0".to_string(),
        ServiceType::Custom("base".to_string()),
        Vec::new(),
        HashMap::new(),
    ).await.unwrap();
    
    // 注册依赖服务
    registry.register_with_details(
        "dependent-service".to_string(),
        "1.0.0".to_string(),
        ServiceType::Custom("dependent".to_string()),
        vec!["base-service".to_string(), "missing-service".to_string()],
        HashMap::new(),
    ).await.unwrap();
    
    // 检查依赖
    let missing_deps = registry.check_dependencies("dependent-service").await.unwrap();
    assert_eq!(missing_deps.len(), 1);
    assert_eq!(missing_deps[0], "missing-service");
    
    // 注册缺失的依赖
    registry.register("missing-service".to_string()).await.unwrap();
    
    // 再次检查依赖
    let missing_deps = registry.check_dependencies("dependent-service").await.unwrap();
    assert_eq!(missing_deps.len(), 0);
    
    // 停用依赖服务
    registry.update_service_status("base-service", false).await.unwrap();
    
    // 检查依赖（应该检测到非活跃依赖）
    let missing_deps = registry.check_dependencies("dependent-service").await.unwrap();
    assert_eq!(missing_deps.len(), 1);
    assert_eq!(missing_deps[0], "base-service");
    
    println!("✅ 服务依赖管理测试通过");
}

/// 测试服务健康检查时序
#[tokio::test]
async fn test_service_health_check_timing() {
    let registry = McpServiceRegistry::new();
    
    // 注册服务
    registry.register("health-timing-service".to_string()).await.unwrap();
    
    // 第一次健康检查
    let health1 = registry.perform_health_check("health-timing-service").await.unwrap();
    assert_eq!(health1, HealthStatus::Unknown);
    
    // 短暂等待
    sleep(Duration::from_millis(100)).await;
    
    // 第二次健康检查
    let health2 = registry.perform_health_check("health-timing-service").await.unwrap();
    assert_eq!(health2, HealthStatus::Healthy);
    
    // 验证健康服务列表
    let healthy_services = registry.get_healthy_services().await;
    assert_eq!(healthy_services.len(), 1);
    assert_eq!(healthy_services[0], "health-timing-service");
    
    println!("✅ 服务健康检查时序测试通过");
}

/// 测试大规模服务注册
#[tokio::test]
async fn test_large_scale_service_registration() {
    let registry = McpServiceRegistry::new();
    
    // 注册大量服务
    for i in 0..100 {
        let service_name = format!("scale-service-{:03}", i);
        registry.register(service_name).await.unwrap();
    }
    
    // 验证服务数量
    let services = registry.list_services().await;
    assert_eq!(services.len(), 100);
    
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 100);
    
    // 随机停用一些服务
    for i in (0..100).step_by(10) {
        let service_name = format!("scale-service-{:03}", i);
        registry.update_service_status(&service_name, false).await.unwrap();
    }
    
    // 验证活跃服务数量
    let active_services = registry.get_active_services().await;
    assert_eq!(active_services.len(), 90);
    
    // 获取统计信息
    let stats = registry.get_service_stats().await;
    assert_eq!(stats.total_services, 100);
    assert_eq!(stats.active_services, 90);
    
    println!("✅ 大规模服务注册测试通过");
}

/// 性能基准测试
#[tokio::test]
async fn test_performance_benchmarks() {
    let registry = McpServiceRegistry::new();
    
    // 注册一些服务用于性能测试
    for i in 0..50 {
        let service_name = format!("perf-service-{}", i);
        registry.register(service_name).await.unwrap();
    }
    
    // 测试服务查找性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let service_name = format!("perf-service-{}", i % 50);
        registry.is_registered(&service_name).await;
    }
    let lookup_time = start.elapsed();
    
    // 测试健康检查性能
    let start = std::time::Instant::now();
    for i in 0..50 {
        let service_name = format!("perf-service-{}", i);
        registry.perform_health_check(&service_name).await.unwrap();
    }
    let health_check_time = start.elapsed();
    
    // 验证性能基准
    assert!(lookup_time < Duration::from_millis(100), "服务查找性能不达标");
    assert!(health_check_time < Duration::from_millis(500), "健康检查性能不达标");
    
    println!("✅ 性能基准测试通过");
    println!("   - 1000次服务查找耗时: {:?}", lookup_time);
    println!("   - 50次健康检查耗时: {:?}", health_check_time);
}

