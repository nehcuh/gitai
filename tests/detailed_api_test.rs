//! 详细的API对比测试

#![allow(clippy::uninlined_format_args, clippy::print_stdout)]

use gitai::infrastructure::container::{ContainerError, ServiceContainer, ServiceProvider};
use std::any::TypeId;

#[derive(Clone, Debug, PartialEq)]
struct TestService {
    value: i32,
}

struct ManualProvider;

impl ServiceProvider for ManualProvider {
    type Service = TestService;

    fn create(&self, _container: &ServiceContainer) -> Result<Self::Service, ContainerError> {
        Ok(TestService { value: 100 })
    }
}

#[tokio::test]
async fn test_direct_closure_vs_simple() {
    println!("=== 直接对比测试 ===");

    // 测试1: 直接闭包方法（已知工作）
    {
        println!("测试1: 直接闭包方法");
        let container = ServiceContainer::new();

        let provider = |_container: &ServiceContainer| -> Result<TestService, ContainerError> {
            println!("  闭包provider被调用");
            Ok(TestService { value: 111 })
        };

        container.register_singleton(provider).await;
        println!("  注册完成");

        match container.resolve::<TestService>().await {
            Ok(service) => println!("  ✓ 成功: {:?}", service),
            Err(e) => println!("  ✗ 失败: {:?}", e),
        }
    }

    // 测试2: 闭包语法（推荐方式）
    {
        println!("\n测试2: 闭包语法");
        let container = ServiceContainer::new();

        container
            .register_singleton_simple(|| {
                println!("  闭包工厂被调用");
                Ok::<_, ContainerError>(TestService { value: 222 })
            })
            .await;
        println!("  注册完成");

        match container.resolve::<TestService>().await {
            Ok(service) => println!("  ✓ 成功: {:?}", service),
            Err(e) => println!("  ✗ 失败: {:?}", e),
        }
    }

    // 测试3: 手动创建provider闭包
    {
        println!("\n测试3: 手动创建provider闭包");
        let container = ServiceContainer::new();

        let factory = || {
            println!("  manual factory被调用");
            Ok::<_, ContainerError>(TestService { value: 333 })
        };

        let provider = move |_container: &ServiceContainer| -> Result<TestService, ContainerError> {
            println!("  manual provider被调用");
            factory()
        };

        container.register_singleton(provider).await;
        println!("  注册完成");

        match container.resolve::<TestService>().await {
            Ok(service) => println!("  ✓ 成功: {:?}", service),
            Err(e) => println!("  ✗ 失败: {:?}", e),
        }
    }
}

#[tokio::test]
async fn test_service_type_registration() {
    println!("=== 服务类型注册测试 ===");

    let container = ServiceContainer::new();

    // 检查注册前的状态
    println!("注册前服务类型ID: {:?}", TypeId::of::<TestService>());

    // 使用闭包语法注册（推荐方式）
    container
        .register_singleton_simple(|| Ok::<_, ContainerError>(TestService { value: 555 }))
        .await;

    println!("使用闭包语法注册完成");

    // 检查是否能解析
    match container.resolve::<TestService>().await {
        Ok(service) => println!("✓ 解析成功: {:?}", service),
        Err(e) => println!("✗ 解析失败: {:?}", e),
    }

    // 对比手动provider
    let container2 = ServiceContainer::new();
    container2.register_singleton(ManualProvider).await;

    match container2.resolve::<TestService>().await {
        Ok(service) => println!("✓ 手动provider解析成功: {:?}", service),
        Err(e) => println!("✗ 手动provider解析失败: {:?}", e),
    }
}
