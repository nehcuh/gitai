//! 简单的DI容器测试
//!
//! 验证新的DI容器实现的基本功能

#![allow(clippy::uninlined_format_args, clippy::print_stdout)]

use gitai::infrastructure::container::v2::{ContainerError, ServiceContainer};
use std::sync::Arc;

/// 测试用的简单服务
#[derive(Clone, Debug, PartialEq)]
struct SimpleService {
    value: i32,
}

/// 测试用的复杂服务
#[derive(Clone, Debug, PartialEq)]
struct ComplexService {
    name: String,
    id: u64,
}

// 不再使用 ServiceProvider trait，改用简化 API 注册

#[tokio::test]
async fn test_basic_service_registration() {
    let container = ServiceContainer::new();

    // 注册一个简单的服务
    container
        .register_singleton_simple(|| Ok::<_, ContainerError>(SimpleService { value: 42 }))
        .await;

    // 解析服务
    let service = container.resolve::<SimpleService>().await.unwrap();

    // 验证服务正常工作
    assert_eq!(service.value, 42);
}

#[tokio::test]
async fn test_singleton_behavior() {
    let container = ServiceContainer::new();

    // 注册单例服务
    container
        .register_singleton_simple(|| Ok::<_, ContainerError>(SimpleService { value: 42 }))
        .await;

    // 多次解析服务
    let service1 = container.resolve::<SimpleService>().await.unwrap();
    let service2 = container.resolve::<SimpleService>().await.unwrap();

    // 验证所有服务实例相同（Arc 指针相同）
    assert!(Arc::ptr_eq(&service1, &service2));
    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
}

#[tokio::test]
async fn test_multiple_service_types() {
    let container = ServiceContainer::new();

    // 注册多个不同类型的服务
    container
        .register_singleton_simple(|| Ok::<_, ContainerError>(SimpleService { value: 42 }))
        .await;
    container
        .register_singleton_simple(|| {
            Ok::<_, ContainerError>(ComplexService {
                name: "TestService".to_string(),
                id: 456,
            })
        })
        .await;

    // 解析服务
    let simple = container.resolve::<SimpleService>().await.unwrap();
    let complex = container.resolve::<ComplexService>().await.unwrap();

    // 验证服务正常工作
    assert_eq!(simple.value, 42);
    assert_eq!(complex.name, "TestService");
    assert_eq!(complex.id, 456);
}

#[tokio::test]
async fn test_service_not_found() {
    let container = ServiceContainer::new();

    // 尝试解析未注册的服务
    let result = container.resolve::<SimpleService>().await;

    // 验证返回错误
    assert!(result.is_err());
}
