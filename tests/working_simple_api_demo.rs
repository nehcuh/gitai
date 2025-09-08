//! 演示：如何使用简化的API（基于已验证的工作模式）

use gitai::infrastructure::container::{ContainerError, ServiceContainer};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
struct SimpleService {
    value: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct ComplexService {
    name: String,
    id: u64,
}

#[tokio::test]
async fn demo_simple_singleton() {
    println!("=== 演示：简单单例服务 ===");

    let container = ServiceContainer::new();

    // 方法1: 使用直接的闭包语法（推荐）
    container
        .register_singleton(|_container| Ok(SimpleService { value: 42 }))
        .await;

    // 解析服务
    let service = container.resolve::<SimpleService>().await.unwrap();
    println!("单例服务: {:?}", service);
    assert_eq!(service.value, 42);

    // 验证单例行为
    let service2 = container.resolve::<SimpleService>().await.unwrap();
    assert!(Arc::ptr_eq(&service, &service2));
}

#[tokio::test]
async fn demo_simple_transient() {
    println!("=== 演示：简单瞬态服务 ===");

    let container = ServiceContainer::new();

    // 使用直接的闭包语法
    container
        .register_transient(|_container| Ok(SimpleService { value: 100 }))
        .await;

    // 解析服务（每次都应该得到新实例）
    let service1 = container.resolve::<SimpleService>().await.unwrap();
    let service2 = container.resolve::<SimpleService>().await.unwrap();

    println!("瞬态服务1: {:?}", service1);
    println!("瞬态服务2: {:?}", service2);

    assert_eq!(service1.value, 100);
    assert_eq!(service2.value, 100);
    // 瞬态服务应该是不同的实例
    assert!(!Arc::ptr_eq(&service1, &service2));
}

#[tokio::test]
async fn demo_complex_service() {
    println!("=== 演示：复杂服务 ===");

    let container = ServiceContainer::new();

    // 注册复杂服务
    container
        .register_singleton(|_container| {
            Ok(ComplexService {
                name: "DatabaseService".to_string(),
                id: 12345,
            })
        })
        .await;

    let service = container.resolve::<ComplexService>().await.unwrap();
    println!("复杂服务: {:?}", service);

    assert_eq!(service.name, "DatabaseService");
    assert_eq!(service.id, 12345);
}

#[tokio::test]
async fn demo_multiple_services() {
    println!("=== 演示：多服务注册 ===");

    let container = ServiceContainer::new();

    // 注册多个服务
    container
        .register_singleton(|_container| Ok(SimpleService { value: 1 }))
        .await;

    container
        .register_transient(|_container| {
            Ok(ComplexService {
                name: "TransientService".to_string(),
                id: 999,
            })
        })
        .await;

    // 解析所有服务
    let simple = container.resolve::<SimpleService>().await.unwrap();
    let complex = container.resolve::<ComplexService>().await.unwrap();

    println!("简单服务: {:?}", simple);
    println!("复杂服务: {:?}", complex);

    assert_eq!(simple.value, 1);
    assert_eq!(complex.name, "TransientService");
}

#[tokio::test]
async fn demo_concurrent_usage() {
    println!("=== 演示：并发使用 ===");

    let container = ServiceContainer::new();

    // 注册单例服务
    container
        .register_singleton(|_container| Ok(SimpleService { value: 777 }))
        .await;

    // 并发解析
    let mut handles = vec![];
    for i in 0..5 {
        let container_clone = container.clone();
        handles.push(tokio::spawn(async move {
            let service = container_clone.resolve::<SimpleService>().await.unwrap();
            println!("并发任务 {}: {:?}", i, service);
            service.value
        }));
    }

    let results = futures_util::future::join_all(handles).await;
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("任务 {} 完成，值: {}", i, value),
            Err(e) => println!("任务 {} 失败: {:?}", i, e),
        }
    }

    // 所有值应该相同（单例）
    let values: Vec<i32> = results.into_iter().filter_map(|r| r.ok()).collect();
    assert!(values.iter().all(|&v| v == 777));
}

#[tokio::test]
async fn demo_error_handling() {
    println!("=== 演示：错误处理 ===");

    let container = ServiceContainer::new();

    // 不注册任何服务，尝试解析
    match container.resolve::<SimpleService>().await {
        Ok(_) => panic!("应该失败"),
        Err(e) => println!("预期的错误: {:?}", e),
    }

    // 注册一个总是失败的服务
    container
        .register_singleton(|_container| -> Result<SimpleService, ContainerError> {
            Err(ContainerError::ServiceCreationFailed(
                "配置错误".to_string(),
            ))
        })
        .await;

    match container.resolve::<SimpleService>().await {
        Ok(_) => panic!("应该失败"),
        Err(e) => println!("创建失败错误: {:?}", e),
    }
}

#[tokio::main]
async fn main() {
    println!("🚀 GitAI DI容器简化API演示\n");

    println!("这个演示展示了如何使用推荐的闭包语法来注册服务。");
    println!("我们使用register_singleton方法配合闭包，");
    println!("这提供了简单、直观且类型安全的API。\n");

    demo_simple_singleton().await;
    println!();
    demo_simple_transient().await;
    println!();
    demo_complex_service().await;
    println!();
    demo_multiple_services().await;
    println!();
    demo_concurrent_usage().await;
    println!();
    demo_error_handling().await;

    println!("\n✅ 演示完成！这种API设计既简单又保持了类型安全。");
}
