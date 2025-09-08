#![allow(dead_code, clippy::uninlined_format_args, clippy::print_stdout, clippy::unnecessary_cast, clippy::io_other_error)]
//! DI容器v2的性能基准测试

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures_util::future;
use gitai::infrastructure::container::v2::ServiceContainer;
use tokio::runtime::Runtime;

/// 测试用的简单服务
#[derive(Clone)]
struct SimpleService {
    value: i32,
}

/// 测试用的复杂服务（包含多个字段）
struct ComplexService {
    id: u64,
    name: String,
    config: ServiceConfig,
    dependencies: Vec<String>,
}

#[derive(Clone)]
struct ServiceConfig {
    timeout: u64,
    max_retries: u32,
    enabled: bool,
}

/// 基准测试配置
struct BenchConfig {
    service_count: usize,
    resolution_count: usize,
    concurrent_resolutions: usize,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            service_count: 100,
            resolution_count: 1000,
            concurrent_resolutions: 10,
        }
    }
}

/// 基准测试：简单服务解析
fn bench_simple_service_resolution(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("simple_service_resolution");

    for service_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &service_count| {
                b.iter(|| {
                    runtime.block_on(async {
                        let container = ServiceContainer::new();

                        // 注册多个简单服务
                        for i in 0..service_count {
                            container.register(move |_| Ok(SimpleService { value: i as i32 }));
                        }

                        // 解析服务
                        let mut results = Vec::new();
for _ in 0..100 {
                            let service = container.resolve::<SimpleService>().await.unwrap();
                            results.push(service.value);
                        }

                        black_box(results)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：复杂服务解析
fn bench_complex_service_resolution(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("complex_service_resolution");

    group.bench_function("single_complex_service", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();

                // 注册复杂服务
                container.register(|_| {
                    Ok(ComplexService {
                        id: 12345,
                        name: "TestService".to_string(),
                        config: ServiceConfig {
                            timeout: 30,
                            max_retries: 3,
                            enabled: true,
                        },
                        dependencies: vec![
                            "logger".to_string(),
                            "config".to_string(),
                            "database".to_string(),
                        ],
                    })
                });

                // 多次解析
                let mut services = Vec::new();
                for _ in 0..100 {
                    let service = container.resolve::<ComplexService>().await.unwrap();
                    services.push(service.id);
                }

                black_box(services)
            })
        });
    });

    group.finish();
}

/// 基准测试：缓存性能
fn bench_cache_performance(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_performance");

    // 测试缓存命中vs未命中
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                container.register(|_| Ok(SimpleService { value: 42 }));

                // 第一次解析 - 缓存未命中
                let service = container.resolve::<SimpleService>().await.unwrap();
                black_box(service.value)
            })
        });
    });

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                container.register(|_| Ok(SimpleService { value: 42 }));

                // 预热缓存
                let _ = container.resolve::<SimpleService>().await.unwrap();

                // 第二次解析 - 缓存命中
                let service = container.resolve::<SimpleService>().await.unwrap();
                black_box(service.value)
            })
        });
    });

    // 测试连续缓存命中
    group.bench_function("consecutive_cache_hits", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                container.register(|_| Ok(SimpleService { value: 42 }));

                // 预热
                let _ = container.resolve::<SimpleService>().await.unwrap();

                // 连续100次缓存命中
                let mut sum = 0;
                for _ in 0..100 {
                    let service = container.resolve::<SimpleService>().await.unwrap();
                    sum += service.value;
                }

                black_box(sum)
            })
        });
    });

    group.finish();
}

/// 基准测试：并发解析性能
fn bench_concurrent_resolution(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_resolution");

    for concurrent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrent_count),
            concurrent_count,
            |b, &concurrent_count| {
                b.iter(|| {
                    runtime.block_on(async {
                        let container = ServiceContainer::new();
                        container.register(|_| Ok(SimpleService { value: 100 }));

                        // 预热缓存
                        let _ = container.resolve::<SimpleService>().await.unwrap();

                        // 并发解析
                        let mut handles = Vec::new();
                        for _ in 0..concurrent_count {
                            let container_clone = container.clone();
                            handles.push(tokio::spawn(async move {
                                let service =
                                    container_clone.resolve::<SimpleService>().await.unwrap();
                                service.value
                            }));
                        }

                        // 等待所有任务完成
let results = future::join_all(handles).await;
                        let sum: i32 = results.into_iter().map(|r| r.unwrap()).sum();

                        black_box(sum)
                    })
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：服务注册性能
fn bench_service_registration(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("service_registration");

    for service_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(service_count),
            service_count,
            |b, &service_count| {
                b.iter(|| {
                    let container = ServiceContainer::new();

                    // 批量注册服务
                    runtime.block_on(async {
                        for i in 0..service_count {
                            container.register(move |_| Ok(SimpleService { value: i as i32 }));
                        }
                    });

                    black_box(container)
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：内存效率
fn bench_memory_efficiency(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_efficiency");

    group.bench_function("large_service_set", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();

                // 注册大量不同类型的服务
                for i in 0..1000 {
                    container.register(move |_| {
                        Ok(ComplexService {
                            id: i as u64,
                            name: format!("Service{}", i),
                            config: ServiceConfig {
                                timeout: 30 + (i % 60),
                                max_retries: 3,
                                enabled: i % 2 == 0,
                            },
                            dependencies: vec![
                                format!("dep1_{}", i),
                                format!("dep2_{}", i),
                                format!("dep3_{}", i),
                            ],
                        })
                    });
                }

                // 解析一些服务
                let mut services = Vec::new();
                for i in (0..1000).step_by(10) {
                    // 这里简化实现，实际应该使用类型擦除
                    // 为了测试目的，我们使用SimpleService
                    container.register(move |_| Ok(SimpleService { value: i as i32 }));
                    let service = container.resolve::<SimpleService>().await.unwrap();
                    services.push(service.value);
                }

                black_box(services)
            })
        });
    });

    group.finish();
}

/// 基准测试：错误处理性能
fn bench_error_handling(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("error_handling");

    group.bench_function("service_not_found", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                // 不注册任何服务，直接解析

                match container.resolve::<SimpleService>().await {
                    Ok(_) => panic!("Expected error"),
                    Err(e) => black_box(e),
                }
            })
        });
    });

    group.bench_function("service_creation_failed", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();

                // 注册总是失败的服务
container.register::<SimpleService, _>(|_| {
                    Err::<SimpleService, Box<dyn std::error::Error + Send + Sync>>(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Creation failed",
                    )))
                });

                match container.resolve::<SimpleService>().await {
                    Ok(_) => panic!("Expected error"),
                    Err(e) => black_box(e),
                }
            })
        });
    });

    group.finish();
}

/// 基准测试：统计信息收集性能
fn bench_stats_collection(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("stats_collection");

    group.bench_function("get_stats", |b| {
        let container = ServiceContainer::new();
        container.register(|_| Ok(SimpleService { value: 42 }));

        // 进行一些解析操作
        runtime.block_on(async {
            for _ in 0..1000 {
                let _ = container.resolve::<SimpleService>().await.unwrap();
            }
        });

        b.iter(|| {
            let stats = container.get_stats();
            black_box(stats)
        });
    });

    group.bench_function("get_cache_hit_rate", |b| {
        let container = ServiceContainer::new();
        container.register(|_| Ok(SimpleService { value: 42 }));

        runtime.block_on(async {
            for _ in 0..1000 {
                let _ = container.resolve::<SimpleService>().await.unwrap();
            }
        });

        b.iter(|| {
            let hit_rate = container.get_cache_hit_rate();
            black_box(hit_rate)
        });
    });

    group.finish();
}

/// 基准测试：与旧版本对比（模拟）
fn bench_comparison_with_legacy(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("comparison_with_legacy");

    group.bench_function("new_container_single_resolution", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                container.register(|_| Ok(SimpleService { value: 42 }));

                let service = container.resolve::<SimpleService>().await.unwrap();
                black_box(service.value)
            })
        });
    });

    group.bench_function("new_container_cached_resolution", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let container = ServiceContainer::new();
                container.register(|_| Ok(SimpleService { value: 42 }));

                // 预热
                let _ = container.resolve::<SimpleService>().await.unwrap();

                // 缓存命中
                let service = container.resolve::<SimpleService>().await.unwrap();
                black_box(service.value)
            })
        });
    });

    group.finish();
}

// 基准测试组配置
criterion_group!(
    benches,
    bench_simple_service_resolution,
    bench_complex_service_resolution,
    bench_cache_performance,
    bench_concurrent_resolution,
    bench_service_registration,
    bench_memory_efficiency,
    bench_error_handling,
    bench_stats_collection,
    bench_comparison_with_legacy
);

criterion_main!(benches);
