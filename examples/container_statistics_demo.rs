//! GitAI DI容器统计功能演示
//!
//! 这个示例展示了如何使用容器的统计和性能监控功能。

use gitai::infrastructure::container::ServiceContainer;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct DatabaseService {
    connection_string: String,
}

#[derive(Clone, Debug)]
struct CacheService {
    size: usize,
}

#[derive(Clone, Debug)]
struct LoggerService {
    app_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GitAI DI容器统计功能演示\n");

    // 创建容器
    let container = ServiceContainer::new();

    println!("1️⃣ 注册服务并查看初始统计");

    // 注册数据库服务（单例）
    container
        .register_singleton_simple(|| {
            println!("📦 创建数据库服务实例");
            Ok(DatabaseService {
                connection_string: "postgresql://localhost:5432/myapp".to_string(),
            })
        })
        .await;

    // 注册缓存服务（单例）
    container
        .register_singleton_simple(|| {
            println!("📦 创建缓存服务实例");
            Ok(CacheService { size: 1024 })
        })
        .await;

    // 注册日志服务（瞬态）
    container
        .register_transient_simple(|| {
            println!("📦 创建日志服务实例");
            Ok(LoggerService {
                app_name: "GitAI".to_string(),
            })
        })
        .await;

    // 查看初始统计
    let initial_stats = container.get_stats().await;
    println!("✅ 初始统计信息:");
    println!("   - 注册服务数量: {}", initial_stats.registered_services);
    println!("   - 总解析次数: {}", initial_stats.total_resolutions);
    println!("   - 活跃单例数量: {}", initial_stats.active_singletons);
    println!(
        "   - 活跃作用域实例: {}",
        initial_stats.active_scoped_instances
    );

    println!("\n2️⃣ 解析服务并观察统计变化");

    // 首次解析数据库服务（应该缓存未命中）
    println!("\n🔍 首次解析数据库服务...");
    let _db1 = container.resolve::<DatabaseService>().await?;

    let after_first = container.get_stats().await;
    println!(
        "   统计更新: 总解析={}, 缓存命中={}, 缓存未命中={}",
        after_first.total_resolutions,
        after_first.singleton_cache_hits,
        after_first.singleton_cache_misses
    );

    // 再次解析数据库服务（应该缓存命中）
    println!("\n🔍 再次解析数据库服务（缓存命中）...");
    let _db2 = container.resolve::<DatabaseService>().await?;

    let after_second = container.get_stats().await;
    println!(
        "   统计更新: 总解析={}, 缓存命中={}, 缓存未命中={}",
        after_second.total_resolutions,
        after_second.singleton_cache_hits,
        after_second.singleton_cache_misses
    );

    // 解析缓存服务（另一个单例）
    println!("\n🔍 解析缓存服务...");
    let _cache = container.resolve::<CacheService>().await?;

    let after_cache = container.get_stats().await;
    println!("   活跃单例数量: {}", after_cache.active_singletons);

    // 多次解析瞬态服务
    println!("\n🔍 多次解析瞬态日志服务...");
    let _logger1 = container.resolve::<LoggerService>().await?;
    let _logger2 = container.resolve::<LoggerService>().await?;
    let _logger3 = container.resolve::<LoggerService>().await?;

    let after_transients = container.get_stats().await;
    println!(
        "   瞬态服务创建次数: {}",
        after_transients.transient_creations
    );

    println!("\n3️⃣ 查看详细统计和性能摘要");

    let final_stats = container.get_stats().await;
    println!("\n📊 详细统计信息:");
    println!("   📈 总解析次数: {}", final_stats.total_resolutions);
    println!("   🎯 缓存命中率: {:.1}%", final_stats.cache_hit_rate());
    println!("   📋 注册服务数量: {}", final_stats.registered_services);
    println!("   🔥 活跃单例数量: {}", final_stats.active_singletons);
    println!("   📦 单例缓存命中: {}", final_stats.singleton_cache_hits);
    println!(
        "   ❌ 单例缓存未命中: {}",
        final_stats.singleton_cache_misses
    );
    println!("   ⚡ 瞬态服务创建: {}", final_stats.transient_creations);
    println!(
        "   🎯 循环依赖检测: {}",
        final_stats.circular_dependency_checks
    );

    // 服务创建分布
    let (singletons, transients, scoped) = final_stats.service_creation_distribution();
    println!("\n📊 服务创建分布:");
    println!(
        "   单例: {}, 瞬态: {}, 作用域: {}",
        singletons, transients, scoped
    );

    // 性能摘要
    let performance_summary = container.get_performance_summary().await;
    println!("\n🚀 性能摘要: {}", performance_summary);

    println!("\n4️⃣ 演示统计重置功能");

    // 重置统计
    container.reset_stats().await;
    let reset_stats = container.get_stats().await;

    println!("🔄 重置后的统计:");
    println!("   总解析次数: {}", reset_stats.total_resolutions);
    println!("   缓存命中率: {:.1}%", reset_stats.cache_hit_rate());
    println!("   注册服务数量: {}", reset_stats.registered_services);

    // 注意：重置统计不会影响已注册的服务和活跃实例
    println!(
        "   活跃单例数量: {} (不受影响)",
        reset_stats.active_singletons
    );

    println!("\n✅ 演示完成！");
    println!("容器统计功能提供了丰富的性能监控指标，包括:");
    println!("- 总解析次数和缓存命中率");
    println!("- 不同类型服务的创建统计");
    println!("- 活跃实例数量跟踪");
    println!("- 循环依赖检测统计");
    println!("- 性能摘要和详细指标");
    println!("这些统计信息对于性能调优和问题诊断非常有价值。");

    Ok(())
}
