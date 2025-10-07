/// TASK-003-3-1-2-3: 性能优化测试模块
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;
use crate::ai::{CircuitBreakerManager, CircuitBreakerConfig};

/// 测试 DashMap 相对于 HashMap+RwLock 的性能提升
pub async fn test_dashmap_performance() {
    println!("=== DashMap Performance Test ===");

    let manager = Arc::new(CircuitBreakerManager::new(CircuitBreakerConfig::default()));

    // 重置性能统计
    manager.reset_performance_stats();

    let num_concurrent_tasks = 100;
    let operations_per_task = 50;
    let num_unique_instances = 20;

    println!("Starting {} concurrent tasks, {} ops each",
             num_concurrent_tasks, operations_per_task);

    let start_time = Instant::now();
    let mut task_set = JoinSet::new();

    // 启动多个并发任务
    for _task_id in 0..num_concurrent_tasks {
        let manager_clone = manager.clone();
        task_set.spawn(async move {
            for i in 0..operations_per_task {
                let instance_name = format!("instance_{}", i % num_unique_instances);
                let _breaker = manager_clone.get_or_create_breaker(&instance_name);

                // 模拟一些异步工作
                tokio::time::sleep(tokio::time::Duration::from_nanos(100)).await;
            }
        });
    }

    // 等待所有任务完成
    while let Some(result) = task_set.join_next().await {
        result.unwrap();
    }

    let total_time = start_time.elapsed();
    let total_operations = num_concurrent_tasks * operations_per_task;
    let throughput = total_operations as f64 / total_time.as_secs_f64();

    // 获取性能统计
    let stats = manager.get_performance_stats();

    // 输出结果
    println!("Performance Results:");
    println!("  Total operations: {}", total_operations);
    println!("  Total time: {:?}", total_time);
    println!("  Throughput: {:.2} ops/sec", throughput);
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Hit rate: {:.2}%", (stats.cache_hits as f64 / total_operations as f64) * 100.0);
    println!("  Avg latency: {:.2} μs", stats.avg_access_latency_us);
    println!("  Max concurrent: {}", stats.max_concurrent_accesses);
    println!("  Instance count: {}", manager.instance_count());

    // 性能断言
    assert!(throughput > 5000.0, "Expected throughput > 5000 ops/sec, got: {:.2}", throughput);
    assert!(stats.cache_hits > 0, "Should have cache hits");
    assert!(stats.cache_misses > 0, "Should have cache misses");
    assert_eq!(manager.instance_count(), num_unique_instances);

    println!("✅ Performance test passed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashmap_performance_concurrent() {
        test_dashmap_performance().await;
    }
}