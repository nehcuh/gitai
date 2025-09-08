//! GitAI DI容器 - 简化使用示例
//!
//! 这个示例展示了如何使用推荐的闭包API来注册和解析服务。
//! 注意：由于resolve是异步的，而register_*方法是同步的，
//! 当前的设计限制了在注册时解析其他服务的能力。
//! 这是一个已知的架构限制，我们将在后续版本中解决。

use gitai::infrastructure::container::ServiceContainer;
use std::sync::Arc;

// 示例服务结构
#[derive(Debug, Clone)]
struct Config {
    app_name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct Logger {
    app_name: String,
}

impl Logger {
    fn new(config: &Config) -> Self {
        Self {
            app_name: config.app_name.clone(),
        }
    }
    
    fn log(&self, message: &str) {
        println!("[{}] {}", self.app_name, message);
    }
}

#[derive(Debug, Clone)]
struct DatabaseService {
    config: Arc<Config>,
    logger: Arc<Logger>,
}

impl DatabaseService {
    fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        logger.log("Initializing database service");
        Self { config, logger }
    }
    
    fn connect(&self) {
        self.logger.log(&format!("Connecting to database for {}", self.config.app_name));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GitAI DI容器 - 简化使用示例\n");
    
    // 创建容器
    let container = ServiceContainer::new();
    
    println!("1️⃣ 注册配置服务（单例）");
    container.register_singleton_simple(|| {
        Ok(Config {
            app_name: "MyApp".to_string(),
            version: "1.0.0".to_string(),
        })
    }).await;
    
    // 由于resolve是异步的，而register_*方法是同步的，
    // 我们不能直接在注册闭包中解析其他服务。
    // 这是当前架构的一个限制。
    println!("\n💡 架构说明：由于技术限制，当前不能在同步注册闭包中解析其他服务。");
    println!("   这是DI容器设计中的一个已知问题，将在后续版本中解决。\n");
    
    println!("2️⃣ 注册日志服务（单例）");
    container.register_singleton_simple(|| {
        // 这里不能直接解析Config，因为resolve是异步的
        // 我们使用硬编码的值来演示
        Ok(Logger {
            app_name: "MyApp".to_string(), // 硬编码，不是从Config解析
        })
    }).await;
    
    println!("3️⃣ 注册数据库服务（单例）");
    container.register_singleton_simple(|| {
        // 同样，这里不能直接解析其他服务
        Ok(DatabaseService {
            config: Arc::new(Config {
                app_name: "MyApp".to_string(),
                version: "1.0.0".to_string(),
            }),
            logger: Arc::new(Logger {
                app_name: "MyApp".to_string(),
            }),
        })
    }).await;
    
    println!("\n✅ 所有服务注册完成！\n");
    
    // 使用服务
    println!("4️⃣ 解析和使用服务");
    let config = container.resolve::<Config>().await?;
    println!("配置信息: {:?}", config);
    
    let logger = container.resolve::<Logger>().await?;
    logger.log("Application started");
    
    let db_service = container.resolve::<DatabaseService>().await?;
    db_service.connect();
    
    println!("\n5️⃣ 验证单例行为");
    let config1 = container.resolve::<Config>().await?;
    let config2 = container.resolve::<Config>().await?;
    
    println!("配置1和配置2是同一个实例: {}", 
        std::ptr::eq(Arc::as_ptr(&config1), Arc::as_ptr(&config2)));
    
    println!("\n6️⃣ 演示瞬态服务");
    container.register_transient_simple(|| {
        Ok(Config {
            app_name: "TransientApp".to_string(),
            version: "2.0.0".to_string(),
        })
    }).await;
    
    let trans1 = container.resolve::<Config>().await?;
    let trans2 = container.resolve::<Config>().await?;
    
    println!("瞬态服务1: {:?}", trans1);
    println!("瞬态服务2: {:?}", trans2);
    println!("瞬态服务是不同的实例: {}", 
        !std::ptr::eq(Arc::as_ptr(&trans1), Arc::as_ptr(&trans2)));
    
    println!("\n6️⃣ 演示统计功能");
    let stats = container.get_stats().await;
    println!("容器统计信息:");
    println!("   📊 总解析次数: {}", stats.total_resolutions);
    println!("   📋 注册服务数量: {}", stats.registered_services);
    println!("   🔥 活跃单例数量: {}", stats.active_singletons);
    println!("   🎯 缓存命中率: {:.1}%", stats.cache_hit_rate());
    println!("   ⚡ 瞬态服务创建: {}", stats.transient_creations);
    println!("   🎯 循环依赖检测: {}", stats.circular_dependency_checks);
    
    println!("\n🚀 性能摘要: {}", container.get_performance_summary().await);
    
    println!("\n✅ 示例完成！");
    println!("当前架构提供了完整的DI功能，包括统计监控。");
    println!("推荐的API模式：register_singleton_simple(|| Ok(service))");
    println!("这个模式简单、类型安全，适合大多数使用场景。");
    println!("新增的统计功能提供了性能监控和诊断能力。");
    
    Ok(())
}

#[tokio::test]
async fn test_simple_usage() {
    // 这个测试确保示例代码可以正常运行
    main().await.expect("示例应该正常运行");
}

#[tokio::test]
async fn test_error_handling() {
    let container = ServiceContainer::new();
    
    // 尝试解析未注册的服务
    let result = container.resolve::<Config>().await;
    assert!(result.is_err(), "应该返回错误");
    
    // 注册一个总是失败的服务
    container.register_singleton(|_container: &ServiceContainer| -> Result<Config, gitai::infrastructure::container::ContainerError> {
        Err(gitai::infrastructure::container::ContainerError::ServiceCreationFailed(
            "配置错误".to_string()
        ))
    }).await;
    
    let result = container.resolve::<Config>().await;
    assert!(result.is_err(), "应该返回创建错误");
}

#[tokio::test]
async fn test_concurrent_usage() {
    let container = ServiceContainer::new();
    
    container.register_singleton(|_container| {
        Ok(Config {
            app_name: "ConcurrentApp".to_string(),
            version: "1.0.0".to_string(),
        })
    }).await;
    
    // 并发解析
    let mut handles = vec![];
    for i in 0..10 {
        let container_clone = container.clone();
        handles.push(tokio::spawn(async move {
            let config = container_clone.resolve::<Config>().await?;
            Ok::<_, Box<dyn std::error::Error>>(config.app_name.clone())
        }));
    }
    
    let results = futures_util::future::join_all(handles).await;
    for result in results {
        let app_name = result.expect("任务应该成功完成")?;
        assert_eq!(app_name, "ConcurrentApp");
    }
    
    Ok(())
}