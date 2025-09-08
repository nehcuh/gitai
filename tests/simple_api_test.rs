//! 测试推荐的闭包API接口
//!
//! 验证推荐的闭包注册方法是否正常工作

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

#[tokio::test]
async fn test_simple_transient_registration() {
    let container = ServiceContainer::new();

    // 使用简单的瞬态服务注册（推荐方式）
    container
        .register_transient_simple(|| {
Ok::<_, ContainerError>(SimpleService { value: 42 })
        })
        .await;

    // 解析服务
    let service1 = container.resolve::<SimpleService>().await.unwrap();
    let service2 = container.resolve::<SimpleService>().await.unwrap();

    // 验证瞬态行为：不同的实例，相同的值
    assert!(!Arc::ptr_eq(&service1, &service2));
    assert_eq!(service1.value, 42);
    assert_eq!(service2.value, 42);
}

#[tokio::test]
async fn test_simple_singleton_registration() {
    let container = ServiceContainer::new();

    // 使用简单的单例服务注册（推荐方式）
    container
        .register_singleton_simple(|| {
Ok::<_, ContainerError>(SimpleService { value: 100 })
        })
        .await;

    // 解析服务
    let service1 = container.resolve::<SimpleService>().await.unwrap();
    let service2 = container.resolve::<SimpleService>().await.unwrap();

    // 验证单例行为：相同的实例
    assert!(Arc::ptr_eq(&service1, &service2));
    assert_eq!(service1.value, 100);
    assert_eq!(service2.value, 100);
}

#[tokio::test]
async fn test_simple_scoped_registration() {
    let container = ServiceContainer::new();

    // 使用简单的作用域服务注册（推荐方式）
    container
        .register_scoped_simple(|| {
Ok::<_, ContainerError>(SimpleService { value: 200 })
        })
        .await;

    // 启动一个作用域以解析作用域服务
    container.begin_scope().await;

    // 解析服务
    let service1 = container.resolve::<SimpleService>().await.unwrap();
    let service2 = container.resolve::<SimpleService>().await.unwrap();

    // 验证作用域行为：在当前实现中，作用域服务在同一作用域内应保持一致的值
    assert_eq!(service1.value, service2.value);
    assert_eq!(service1.value, 200);
}

#[tokio::test]
async fn test_simple_api_with_complex_service() {
    let container = ServiceContainer::new();

    // 使用简单的单例服务注册复杂服务（推荐方式）
    container
        .register_singleton_simple(|| {
Ok::<_, ContainerError>(ComplexService {
                name: "TestService".to_string(),
                id: 12345,
            })
        })
        .await;

    // 解析服务
    let service = container.resolve::<ComplexService>().await.unwrap();

    // 验证服务正确创建
    assert_eq!(service.name, "TestService");
    assert_eq!(service.id, 12345);
}

#[tokio::test]
async fn test_simple_api_error_handling() {
    let container = ServiceContainer::new();

    // 注册一个总是失败的服务（使用简单语法）
    container
        .register_singleton_simple(|| {
            Err::<SimpleService, _>(
                ContainerError::CreationFailed("Test error".to_string()),
            )
        })
        .await;

    // 尝试解析服务
    let result = container.resolve::<SimpleService>().await;

    // 验证错误处理
    assert!(result.is_err());
}

#[tokio::test]
async fn test_simple_api_multiple_services() {
    let container = ServiceContainer::new();

    // 使用闭包语法注册多个不同类型的服务（推荐方式）
    container
        .register_singleton_simple(|| {
Ok::<_, ContainerError>(SimpleService { value: 42 })
        })
        .await;

    container
        .register_transient_simple(|| {
Ok::<_, ContainerError>(ComplexService {
                name: "TransientService".to_string(),
                id: 999,
            })
        })
        .await;

    // 解析两种服务
    let simple = container.resolve::<SimpleService>().await.unwrap();
    let complex = container.resolve::<ComplexService>().await.unwrap();

    // 验证服务正常工作
    assert_eq!(simple.value, 42);
    assert_eq!(complex.name, "TransientService");
    assert_eq!(complex.id, 999);
}

#[tokio::test]
async fn test_simple_api_concurrent() {
    let container = ServiceContainer::new();

    // 使用简单的单例服务注册（推荐方式）
    container
        .register_singleton_simple(|| {
Ok::<_, ContainerError>(SimpleService { value: 777 })
        })
        .await;

    // 并发解析服务
    let mut handles = vec![];
    for _ in 0..10 {
        let container_clone = container.clone();
        handles.push(tokio::spawn(async move {
            container_clone.resolve::<SimpleService>().await
        }));
    }

    // 等待所有任务完成
    let results = futures_util::future::join_all(handles).await;

    // 验证所有解析成功且值相同
    assert_eq!(results.len(), 10);
    for result in results {
        let service = result.unwrap().unwrap();
        assert_eq!(service.value, 777);
    }
}
