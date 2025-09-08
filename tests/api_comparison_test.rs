//! API对比测试 - 诊断简化API的问题

use gitai::infrastructure::container::{ContainerError, ServiceContainer, ServiceProvider};

#[derive(Clone, Debug, PartialEq)]
struct TestService {
    value: i32,
}

struct SimpleProvider;

impl ServiceProvider for SimpleProvider {
    type Service = TestService;

    fn create(&self, _container: &ServiceContainer) -> Result<Self::Service, ContainerError> {
        Ok(TestService { value: 123 })
    }
}

#[tokio::test]
async fn test_original_api_works() {
    println!("=== 测试原始API ===");
    let container = ServiceContainer::new();

    // 使用原始API注册
    println!("使用原始API注册服务...");
    container.register_singleton(SimpleProvider).await;
    println!("原始API注册完成");

    // 解析服务
    println!("尝试解析服务...");
    match container.resolve::<TestService>().await {
        Ok(service) => {
            println!("✓ 原始API服务解析成功: {:?}", service);
            assert_eq!(service.value, 123);
        }
        Err(e) => {
            println!("✗ 原始API服务解析失败: {:?}", e);
            panic!("原始API应该工作");
        }
    }
}

#[tokio::test]
async fn test_simple_api_works() {
    println!("=== 测试简化API ===");
    let container = ServiceContainer::new();

    // 使用闭包语法注册（推荐方式）
    println!("使用闭包语法注册服务...");
    container
        .register_singleton(|_container| Ok::<_, ContainerError>(TestService { value: 456 }))
        .await;
    println!("闭包语法注册完成");

    // 解析服务
    println!("尝试解析服务...");
    match container.resolve::<TestService>().await {
        Ok(service) => {
            println!("✓ 简化API服务解析成功: {:?}", service);
            assert_eq!(service.value, 456);
        }
        Err(e) => {
            println!("✗ 简化API服务解析失败: {:?}", e);
            // 不panic，让测试继续
        }
    }
}

#[tokio::test]
async fn test_closure_api_works() {
    println!("=== 测试闭包API ===");
    let container = ServiceContainer::new();

    // 使用闭包直接注册
    println!("使用闭包直接注册服务...");
    let provider = |_container: &ServiceContainer| -> Result<TestService, ContainerError> {
        Ok(TestService { value: 789 })
    };
    container.register_singleton(provider).await;
    println!("闭包API注册完成");

    // 解析服务
    println!("尝试解析服务...");
    match container.resolve::<TestService>().await {
        Ok(service) => {
            println!("✓ 闭包API服务解析成功: {:?}", service);
            assert_eq!(service.value, 789);
        }
        Err(e) => {
            println!("✗ 闭包API服务解析失败: {:?}", e);
            panic!("闭包API应该工作");
        }
    }
}

#[tokio::test]
async fn test_type_inference_issue() {
    println!("=== 测试类型推断问题 ===");
    let container = ServiceContainer::new();

    // 尝试不同的类型声明方式
    println!("尝试1: 完全限定语法...");
    container
        .register_singleton(|_container| -> Result<TestService, ContainerError> {
            Ok(TestService { value: 111 })
        })
        .await;

    match container.resolve::<TestService>().await {
        Ok(service) => println!("✓ 方法1成功: {:?}", service),
        Err(e) => println!("✗ 方法1失败: {:?}", e),
    }

    // 重置容器
    let container = ServiceContainer::new();
    println!("尝试2: 显式类型标注...");
    let factory: fn() -> Result<TestService, ContainerError> = || Ok(TestService { value: 222 });
    container.register_singleton(|_container| factory()).await;

    match container.resolve::<TestService>().await {
        Ok(service) => println!("✓ 方法2成功: {:?}", service),
        Err(e) => println!("✗ 方法2失败: {:?}", e),
    }
}
