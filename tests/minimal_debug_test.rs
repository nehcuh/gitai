//! 最小化调试测试

use gitai::infrastructure::container::{ServiceContainer, ServiceProvider, ContainerError};

#[derive(Clone, Debug)]
struct TestService {
    value: i32,
}

static mut CALL_COUNT: u32 = 0;

#[tokio::test]
async fn test_minimal_simple_api() {
    let container = ServiceContainer::new();
    
    // 使用静态变量跟踪调用
    unsafe { CALL_COUNT = 0; }
    
    println!("注册服务...");
    container.register_singleton(|_container| {
        unsafe { CALL_COUNT += 1; }
        println!("工厂函数被调用，次数: {}", unsafe { CALL_COUNT });
        Ok::<_, ContainerError>(TestService { value: 42 })
    }).await;
    
    println!("注册完成，调用次数: {}", unsafe { CALL_COUNT });
    
    println!("解析服务...");
    let result = container.resolve::<TestService>().await;
    
    println!("解析结果: {:?}, 总调用次数: {}", result, unsafe { CALL_COUNT });
    
    match result {
        Ok(service) => {
            println!("成功解析: {:?}", service);
            assert_eq!(service.value, 42);
        }
        Err(e) => {
            println!("解析失败: {:?}", e);
            // 不panic，继续测试
        }
    }
}

#[tokio::test]
async fn test_direct_comparison() {
    println!("=== 直接对比测试 ===");
    
    // 测试1: 直接provider（应该工作）
    {
        println!("测试1: 直接provider");
        let container = ServiceContainer::new();
        
        let mut call_count = 0;
        let provider = move |_container: &ServiceContainer| -> Result<TestService, ContainerError> {
            call_count += 1;
            println!("直接provider被调用: {}", call_count);
            Ok(TestService { value: 100 })
        };
        
        container.register_singleton(provider).await;
        println!("直接provider注册完成");
        
        match container.resolve::<TestService>().await {
            Ok(service) => println!("✓ 直接provider成功: {:?}", service),
            Err(e) => println!("✗ 直接provider失败: {:?}", e),
        }
        println!("直接provider调用次数: {}", call_count);
    }
    
    // 测试2: 闭包语法（推荐方式）
    {
        println!("\n测试2: 闭包语法");
        let container = ServiceContainer::new();
        
        let mut call_count = 0;
        let captured_value = 200; // 捕获的值
        container.register_singleton(move |_container| {
            call_count += 1;
            println!("闭包工厂被调用: {}", call_count);
            Ok::<_, ContainerError>(TestService { value: captured_value })
        }).await;
        println!("闭包语法注册完成");
        
        match container.resolve::<TestService>().await {
            Ok(service) => println!("✓ 闭包语法成功: {:?}", service),
            Err(e) => println!("✗ 闭包语法失败: {:?}", e),
        }
        println!("闭包语法调用次数: {}", call_count);
    }
}

#[tokio::test]
async fn test_factory_capture() {
    println!("=== 工厂捕获测试 ===");
    
    let container = ServiceContainer::new();
    
    // 创建一个简单的值
    let value = 42;
    
    println!("使用捕获的值创建工厂...");
    container.register_singleton(move |_container| {
        println!("工厂执行，使用值: {}", value);
        Ok::<_, ContainerError>(TestService { value })
    }).await;
    
    println!("工厂注册完成，解析服务...");
    match container.resolve::<TestService>().await {
        Ok(service) => println!("✓ 捕获测试成功: {:?}", service),
        Err(e) => println!("✗ 捕获测试失败: {:?}", e),
    }
}

#[tokio::test]
async fn test_result_type_explicit() {
    println!("=== 显式结果类型测试 ===");
    
    let container = ServiceContainer::new();
    
    // 尝试最显式的类型声明（使用闭包语法）
    container.register_singleton(|_container| -> Result<TestService, ContainerError> {
        println!("显式工厂被调用");
        Ok(TestService { value: 999 })
    }).await;
    
    println!("显式注册完成，解析服务...");
    match container.resolve::<TestService>().await {
        Ok(service) => println!("✓ 显式测试成功: {:?}", service),
        Err(e) => println!("✗ 显式测试失败: {:?}", e),
    }
}

#[tokio::test]
async fn test_with_println() {
    println!("=== 带输出的测试 ===");
    
    let container = ServiceContainer::new();
    
    println!("注册前");
    container.register_singleton(|_container| {
        println!("工厂函数内部 - 开始");
        let service = TestService { value: 555 };
        println!("工厂函数内部 - 创建服务: {:?}", service);
        let result = Ok::<_, ContainerError>(service);
        println!("工厂函数内部 - 返回结果: {:?}", result);
        result
    }).await;
    println!("注册后");
    
    println!("解析前");
    let result = container.resolve::<TestService>().await;
    println!("解析后: {:?}", result);
    
    match result {
        Ok(service) => println!("最终成功: {:?}", service),
        Err(e) => println!("最终失败: {:?}", e),
    }
}

#[tokio::main]
async fn main() {
    println!("开始最小化调试测试...");
    test_minimal_simple_api().await;
    println!("\n开始直接对比测试...");
    test_direct_comparison().await;
    println!("\n开始工厂捕获测试...");
    test_factory_capture().await;
    println!("\n开始显式结果类型测试...");
    test_result_type_explicit().await;
    println!("\n开始带输出测试...");
    test_with_println().await;
    println!("\n所有测试完成！");
}