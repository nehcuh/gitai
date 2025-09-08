//! 演示：如何使用简化的API（基于已验证的工作模式）

#![allow(clippy::uninlined_format_args, clippy::print_stdout)]

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
        .register_singleton_simple(|| Ok(SimpleService { value: 42 }))
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
        .register_transient_simple(|| Ok(SimpleService { value: 100 }))
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
        .register_singleton_simple(|| {
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
        .register_singleton_simple(|| Ok(SimpleService { value: 1 }))
        .await;

container
        .register_transient_simple(|| {
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
        .register_singleton_simple(|| Ok(SimpleService { value: 777 }))
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
        .register_singleton_simple(|| -> Result<SimpleService, ContainerError> {
            Err(ContainerError::ServiceCreationFailed {
                service_type: "SimpleService".to_string(),
                service_name: Some("SimpleService".to_string()),
                reason: "配置错误".to_string(),
                source_error: None,
            })
        })
        .await;

    match container.resolve::<SimpleService>().await {
        Ok(_) => panic!("应该失败"),
        Err(e) => println!("创建失败错误: {:?}", e),
    }
}

