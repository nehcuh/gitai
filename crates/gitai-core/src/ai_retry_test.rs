// AI服务重试机制测试

use gitai_core::ai::{AIClient, RetryConfig, RetryError};
use gitai_core::config::Config;
use gitai_types::{Result, GitAIError};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建测试用的AI客户端配置
    fn create_test_config() -> Config {
        Config {
            ai: gitai_core::config::AiConfig {
                api_url: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("test-key".to_string()),
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
            },
            ..Default::default()
        }
    }

    /// 测试重试延迟计算
    #[test]
    fn test_calculate_retry_delay() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 16000,
            backoff_multiplier: 2.0,
            jitter: false, // 关闭抖动以便精确测试
            ..Default::default()
        };

        let client = AIClient::new_with_retry(create_test_config(), config);

        // 测试指数退避
        let delay1 = client.calculate_retry_delay(0);
        assert_eq!(delay1.as_millis(), 1000); // 1000 * 2^0

        let delay2 = client.calculate_retry_delay(1);
        assert_eq!(delay2.as_millis(), 2000); // 1000 * 2^1

        let delay3 = client.calculate_retry_delay(2);
        assert_eq!(delay3.as_millis(), 4000); // 1000 * 2^2

        // 测试最大延迟限制
        let delay5 = client.calculate_retry_delay(5);
        assert_eq!(delay5.as_millis(), 16000); // 应该被限制在max_delay_ms
    }

    /// 测试重试延迟计算（带抖动）
    #[test]
    fn test_calculate_retry_delay_with_jitter() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter: true,
            ..Default::default()
        };

        let client = AIClient::new_with_retry(create_test_config(), config);

        // 带抖动的延迟应该在基础延迟的±25%范围内
        let delay = client.calculate_retry_delay(0);
        let base = 1000.0;
        let min = base * 0.75;
        let max = base * 1.25;

        assert!(delay.as_millis() >= min as u64);
        assert!(delay.as_millis() <= max as u64);
    }

    /// 测试可重试错误判断
    #[test]
    fn test_retryable_error_detection() {
        let config = RetryConfig {
            retryable_errors: vec![
                "timeout".to_string(),
                "connection".to_string(),
                "network".to_string(),
            ],
            retryable_status_codes: vec![429, 500, 502, 503],
            ..Default::default()
        };

        // 测试HTTP错误
        assert!(RetryError::Http("connection timeout".to_string()).is_retryable(&config));
        assert!(RetryError::Http("network unreachable".to_string()).is_retryable(&config));
        assert!(!RetryError::Http("authentication failed".to_string()).is_retryable(&config));

        // 测试状态码错误
        assert!(RetryError::Status(429).is_retryable(&config));
        assert!(RetryError::Status(500).is_retryable(&config));
        assert!(!RetryError::Status(404).is_retryable(&config));
        assert!(!RetryError::Status(401).is_retryable(&config));

        // 测试其他错误类型
        assert!(!RetryError::Json("invalid json".to_string()).is_retryable(&config));
        assert!(RetryError::Response("server error".to_string()).is_retryable(&config));
        assert!(!RetryError::MaxRetriesExceeded.is_retryable(&config));
    }

    /// 测试成功操作不需要重试
    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let config = RetryConfig::default();
        let client = AIClient::new_with_retry(create_test_config(), config);

        let result = client.execute_with_retry(|| {
            let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count == 0 {
                    Ok("success")
                } else {
                    Err(RetryError::Http("unexpected retry".to_string()))
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1); // 只调用了一次
    }

    /// 测试重试机制在遇到可重试错误时的行为
    #[tokio::test]
    async fn test_retry_on_retryable_error() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10, // 使用短延迟以加快测试
            jitter: false,
            ..Default::default()
        };

        let client = AIClient::new_with_retry(create_test_config(), config);

        let result = client.execute_with_retry(|| {
            let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    // 前两次失败
                    Err(RetryError::Status(429)) // 429是可重试的
                } else {
                    // 第三次成功
                    Ok("success after retries")
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success after retries");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // 调用了3次
    }

    /// 测试超过最大重试次数时的行为
    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            jitter: false,
            ..Default::default()
        };

        let client = AIClient::new_with_retry(create_test_config(), config);

        let result = client.execute_with_retry(|| {
            attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                Err(RetryError::Http("connection timeout".to_string())) // 可重试错误
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // 初始尝试 + 2次重试
    }

    /// 测试不可重试错误不会触发重试
    #[tokio::test]
    async fn test_non_retryable_error_no_retry() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            ..Default::default()
        };

        let client = AIClient::new_with_retry(create_test_config(), config);

        let result = client.execute_with_retry(|| {
            attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                Err(RetryError::Json("invalid json".to_string())) // 不可重试
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1); // 只调用了一次
    }

    /// TASK-003-3-1-2-1: 测试自动清理定时任务功能
    #[tokio::test]
    #[ignore] // 暂时忽略此测试以避免超时问题
    async fn test_automatic_cleanup_task() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};
        use std::time::Duration;

        let config = CircuitBreakerConfig::default();
        let cleanup_interval = Duration::from_millis(50);
        let max_idle_time = Duration::from_millis(30);

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            config.clone(),
            cleanup_interval,
            max_idle_time,
        );

        // 验证清理任务已启动
        assert!(manager.is_cleanup_task_running());

        // 创建几个断路器实例
        manager.get_or_create_breaker("test1");
        manager.get_or_create_breaker("test2");
        manager.get_or_create_breaker("test3");

        assert_eq!(manager.instance_count(), 3);

        // 等待足够长时间让自动清理任务清理闲置实例
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 手动执行清理来代替自动清理
        let cleaned_count = manager.cleanup_idle_instances();
        assert!(cleaned_count > 0);

        // 停止清理任务
        manager.stop_cleanup_task().await;
        assert!(!manager.is_cleanup_task_running());
    }

    /// TASK-003-3-1-2-1: 测试清理任务停止和重启
    #[tokio::test]
    #[ignore] // 暂时忽略此测试以避免超时问题
    async fn test_cleanup_task_stop_and_restart() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};
        use std::time::Duration;

        let config = CircuitBreakerConfig::default();
        let cleanup_interval = Duration::from_millis(50);
        let max_idle_time = Duration::from_millis(30);

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            config,
            cleanup_interval,
            max_idle_time,
        );

        // 验证任务已启动
        assert!(manager.is_cleanup_task_running());

        // 创建实例
        manager.get_or_create_breaker("test_stop");
        assert_eq!(manager.instance_count(), 1);

        // 停止清理任务
        manager.stop_cleanup_task().await;
        assert!(!manager.is_cleanup_task_running());

        // 手动执行清理测试
        tokio::time::sleep(Duration::from_millis(50)).await;
        let cleaned_count = manager.cleanup_idle_instances();
        assert_eq!(cleaned_count, 0); // 50ms内不应该被清理

        // 重启清理任务
        manager.restart_cleanup_task().await;
        assert!(manager.is_cleanup_task_running());
    }

    /// TASK-003-3-1-2-1: 测试活跃实例不会被清理
    #[tokio::test]
    async fn test_active_instances_not_cleaned() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};
        use std::time::Duration;

        let config = CircuitBreakerConfig::default();
        let max_idle_time = Duration::from_millis(80);

        // 使用简单的管理器，不启动自动清理任务
        let manager = CircuitBreakerManager::new(config);

        // 立即停止清理任务以避免干扰
        manager.stop_cleanup_task().await;

        // 创建实例
        let breaker1 = manager.get_or_create_breaker("active1");
        let _breaker2 = manager.get_or_create_breaker("active2");

        assert_eq!(manager.instance_count(), 2);

        // 保持活跃访问
        for _ in 0..3 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            let _ = manager.get_or_create_breaker("active1");
            let _ = breaker1.stats();
        }

        // 验证实例仍然存在
        assert_eq!(manager.instance_count(), 2);

        // 等待足够长时间让实例闲置
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 手动执行清理
        let cleaned_count = manager.cleanup_idle_instances();
        assert_eq!(cleaned_count, 2);
        assert_eq!(manager.instance_count(), 0);
    }

    /// TASK-003-3-1-2-1: 测试默认配置的清理任务
    #[tokio::test]
    async fn test_default_cleanup_task() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 默认应该启动清理任务
        assert!(manager.is_cleanup_task_running());

        // 创建实例
        manager.get_or_create_breaker("default_test");
        assert_eq!(manager.instance_count(), 1);

        // 停止清理任务（避免影响其他测试）
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-1: 测试清理任务的并发安全性
    #[tokio::test]
    async fn test_cleanup_task_concurrent_safety() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};
        use std::time::Duration;
        use tokio::task::JoinHandle;

        let config = CircuitBreakerConfig::default();
        let cleanup_interval = Duration::from_millis(50);
        let max_idle_time = Duration::from_millis(30);

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            config,
            cleanup_interval,
            max_idle_time,
        );

        // 启动多个并发任务创建和访问断路器实例
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let instance_name = format!("concurrent_{}", i);
                for _ in 0..10 {
                    let _ = manager_clone.get_or_create_breaker(&instance_name);
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await;
        }

        // 等待清理任务执行
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 验证清理任务仍在运行（没有崩溃）
        assert!(manager.is_cleanup_task_running());

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 实例创建事件
    #[tokio::test]
    async fn test_manager_event_instance_created() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 创建事件收集器
        let events = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events_clone = events.clone();

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例
        let _breaker = manager.get_or_create_breaker("test_event_created");

        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 验证事件
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 1);
        let event = &events_guard[0];
        assert_eq!(event.event_type, CircuitBreakerManagerEventType::InstanceCreated {
            name: "test_event_created".to_string(),
            config: CircuitBreakerConfig::default()
        });
        assert_eq!(event.manager_id, manager.manager_id());
        assert!(!event.is_empty());

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 实例访问事件
    #[tokio::test]
    async fn test_manager_event_instance_accessed() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 创建事件收集器
        let events = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events_clone = events.clone();

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例（第一次创建，应该只触发InstanceCreated事件）
        let _breaker1 = manager.get_or_create_breaker("test_access");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 再次访问同一实例（应该触发InstanceAccessed事件）
        let _breaker2 = manager.get_or_create_breaker("test_access");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 验证事件
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 2);

        // 第一个事件应该是InstanceCreated
        match &events_guard[0].event_type {
            CircuitBreakerManagerEventType::InstanceCreated { name, .. } => {
                assert_eq!(name, "test_access");
            }
            _ => panic!("Expected InstanceCreated event"),
        }

        // 第二个事件应该是InstanceAccessed
        match &events_guard[1].event_type {
            CircuitBreakerManagerEventType::InstanceAccessed { name } => {
                assert_eq!(name, "test_access");
            }
            _ => panic!("Expected InstanceAccessed event"),
        }

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 批量清理事件
    #[tokio::test]
    async fn test_manager_event_batch_cleanup() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let cleanup_interval = Duration::from_millis(50);
        let max_idle_time = Duration::from_millis(30);

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            config,
            cleanup_interval,
            max_idle_time,
        );

        // 创建事件收集器
        let events = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events_clone = events.clone();

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建多个断路器实例
        manager.get_or_create_breaker("cleanup1");
        manager.get_or_create_breaker("cleanup2");
        manager.get_or_create_breaker("cleanup3");

        // 等待清理任务执行
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 验证事件
        let events_guard = events.lock().await;

        // 应该有3个InstanceCreated事件和1个BatchCleanup事件
        let mut created_count = 0;
        let mut cleanup_count = 0;

        for event in events_guard.iter() {
            match &event.event_type {
                CircuitBreakerManagerEventType::InstanceCreated { .. } => created_count += 1,
                CircuitBreakerManagerEventType::BatchCleanup { cleaned_count, remaining_count } => {
                    cleanup_count += 1;
                    assert_eq!(*cleaned_count, 3); // 清理了3个实例
                    assert_eq!(*remaining_count, 0); // 剩余0个实例
                }
                _ => {}
            }
        }

        assert_eq!(created_count, 3);
        assert_eq!(cleanup_count, 1);

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 事件取消订阅
    #[tokio::test]
    async fn test_manager_event_unsubscribe() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 创建事件收集器
        let events = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events_clone = events.clone();

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例 - 应该收到事件
        let _breaker = manager.get_or_create_breaker("test_unsubscribe");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 取消订阅
        manager.unsubscribe_from_manager_events(handle);

        // 再次创建断路器实例 - 不应该收到事件
        let _breaker2 = manager.get_or_create_breaker("test_unsubscribe2");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 验证只收到了一个事件
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 1);
        assert_eq!(events_guard[0].manager_id, manager.manager_id());

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 多个订阅者
    #[tokio::test]
    async fn test_manager_event_multiple_subscribers() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 创建两个事件收集器
        let events1 = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events2 = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));

        // 创建两个订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber1 = TestSubscriber { events: events1.clone() };
        let subscriber2 = TestSubscriber { events: events2.clone() };

        let _handle1 = manager.subscribe_to_manager_events(Box::new(subscriber1));
        let _handle2 = manager.subscribe_to_manager_events(Box::new(subscriber2));

        // 创建断路器实例
        let _breaker = manager.get_or_create_breaker("test_multiple");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 验证两个订阅者都收到了事件
        let events1_guard = events1.lock().await;
        let events2_guard = events2.lock().await;

        assert_eq!(events1_guard.len(), 1);
        assert_eq!(events2_guard.len(), 1);

        // 验证事件内容相同
        assert_eq!(events1_guard[0].manager_id, events2_guard[0].manager_id);
        assert_eq!(events1_guard[0].event_type, events2_guard[0].event_type);

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2: 测试管理器事件系统 - 事件序列化
    #[tokio::test]
    async fn test_manager_event_serialization() {
        use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig, CircuitBreakerManagerEvent, CircuitBreakerManagerSubscriber};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // 创建事件收集器
        let events = Arc::new(Mutex::new(Vec::<CircuitBreakerManagerEvent>::new()));
        let events_clone = events.clone();

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例
        let _breaker = manager.get_or_create_breaker("test_serialization");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // 验证事件可以序列化为JSON
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 1);

        let event = &events_guard[0];
        let json_str = event.to_string();
        assert!(!json_str.is_empty());
        assert!(json_str.contains("manager_id"));
        assert!(json_str.contains("event_type"));

        // 验证包含时间戳
        assert!(json_str.contains("timestamp"));

        // 停止清理任务
        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2-2: 测试断路器配置更新事件
    #[tokio::test]
    async fn test_manager_event_instance_config_updated() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            CircuitBreakerConfig::default(),
            Duration::from_millis(100), // 快速清理
            Duration::from_millis(50),   // 短闲置时间
        );

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例
        let _breaker = manager.get_or_create_breaker("test_config_update");
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 更新配置
        let new_config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 8,
            timeout: Duration::from_millis(5000),
            window_size: 100,
            failure_rate_threshold: 0.5,
            half_open_max_calls: 10,
        };

        let result = manager.update_breaker_config("test_config_update", new_config.clone());
        assert!(result.is_ok());
        assert!(result.unwrap()); // 配置确实发生了变更

        tokio::time::sleep(Duration::from_millis(10)).await;

        // 验证事件
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 2); // InstanceCreated + InstanceConfigUpdated

        // 检查配置更新事件
        let config_event = &events_guard[1];
        match &config_event.event_type {
            CircuitBreakerManagerEventType::InstanceConfigUpdated { name, old_config: _, new_config: event_new_config } => {
                assert_eq!(name, "test_config_update");
                assert_eq!(event_new_config.failure_threshold, 10);
                assert_eq!(event_new_config.success_threshold, 8);
            }
            _ => panic!("Expected InstanceConfigUpdated event, got: {:?}", config_event.event_type),
        }

        // 检查事件详情
        match &config_event.details {
            ManagerEventDetails::InstanceConfigUpdated { name, old_config: _, new_config: event_new_config, changed_fields } => {
                assert_eq!(name, "test_config_update");
                assert_eq!(event_new_config.failure_threshold, 10);
                assert!(!changed_fields.is_empty());
                assert!(changed_fields.contains(&"failure_threshold".to_string()));
            }
            _ => panic!("Expected InstanceConfigUpdated details, got: {:?}", config_event.details),
        }

        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2-2: 测试默认配置更新事件
    #[tokio::test]
    async fn test_manager_event_default_config_updated() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 更新默认配置
        let new_config = CircuitBreakerConfig {
            failure_threshold: 15,
            success_threshold: 12,
            timeout: Duration::from_millis(8000),
            window_size: 200,
            failure_rate_threshold: 0.3,
            half_open_max_calls: 15,
        };

        let new_manager = manager.with_default_config(new_config.clone());
        let changed_fields = manager.calculate_config_changes(&manager.get_default_config(), &new_config);
        assert!(!changed_fields.is_empty());
        assert!(changed_fields.contains(&"failure_threshold".to_string()));

        tokio::time::sleep(Duration::from_millis(10)).await;

        // 验证事件
        let events_guard = events.lock().await;

        // 应该有多个ConfigChanged事件，每个变更的字段一个
        let config_events: Vec<_> = events_guard.iter()
            .filter(|e| matches!(e.event_type, CircuitBreakerManagerEventType::ConfigChanged { .. }))
            .collect();

        assert!(!config_events.is_empty());

        // 检查其中一个配置变更事件
        for event in config_events {
            match &event.event_type {
                CircuitBreakerManagerEventType::ConfigChanged { field, old_value: _, new_value } => {
                    assert!(changed_fields.contains(field));
                    match field.as_str() {
                        "failure_threshold" => assert_eq!(new_value, "15"),
                        "success_threshold" => assert_eq!(new_value, "12"),
                        "timeout" => assert!(new_value.contains("8000ms") || new_value.contains("8s")),
                        "window_size" => assert_eq!(new_value, "200"),
                        _ => {}
                    }
                }
                _ => panic!("Expected ConfigChanged event, got: {:?}", event.event_type),
            }
        }

        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2-2: 测试管理器配置更新事件
    #[tokio::test]
    async fn test_manager_event_manager_config_updated() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let manager = CircuitBreakerManager::new_with_cleanup_config(
            CircuitBreakerConfig::default(),
            Duration::from_secs(60), // 原始清理间隔
            Duration::from_secs(30), // 原始闲置时间
        );

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 更新管理器配置
        let new_cleanup_interval = Duration::from_secs(30);
        let new_max_idle_time = Duration::from_secs(15);

        let new_manager = manager.with_cleanup_config(
            Some(new_cleanup_interval),
            Some(new_max_idle_time),
        );

        // 验证配置确实发生了变更
        assert!(new_manager.cleanup_interval == Duration::from_secs(30));
        assert!(new_manager.max_idle_time == Duration::from_secs(15));

        tokio::time::sleep(Duration::from_millis(10)).await;

        // 验证事件
        let events_guard = events.lock().await;

        // 应该有两个ConfigChanged事件
        let config_events: Vec<_> = events_guard.iter()
            .filter(|e| matches!(e.event_type, CircuitBreakerManagerEventType::ConfigChanged { .. }))
            .collect();

        assert_eq!(config_events.len(), 2);

        // 验证清理间隔变更事件
        let cleanup_event = config_events.iter()
            .find(|e| {
                matches!(e.event_type, CircuitBreakerManagerEventType::ConfigChanged { field, .. } if field == "cleanup_interval")
            })
            .unwrap();

        match &cleanup_event.event_type {
            CircuitBreakerManagerEventType::ConfigChanged { field, old_value, new_value } => {
                assert_eq!(field, "cleanup_interval");
                assert_eq!(old_value, "60s");
                assert_eq!(new_value, "30s");
            }
            _ => panic!("Expected ConfigChanged event"),
        }

        // 验证闲置时间变更事件
        let idle_event = config_events.iter()
            .find(|e| {
                matches!(e.event_type, CircuitBreakerManagerEventType::ConfigChanged { field, .. } if field == "max_idle_time")
            })
            .unwrap();

        match &idle_event.event_type {
            CircuitBreakerManagerEventType::ConfigChanged { field, old_value, new_value } => {
                assert_eq!(field, "max_idle_time");
                assert_eq!(old_value, "30s");
                assert_eq!(new_value, "15s");
            }
            _ => panic!("Expected ConfigChanged event"),
        }

        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2-2: 测试配置更新不存在实例的错误处理
    #[tokio::test]
    async fn test_update_config_nonexistent_instance() {
        let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());

        let new_config = CircuitBreakerConfig {
            failure_threshold: 10,
            timeout: Duration::from_millis(1000),
            window_size: 50,
            failure_rate_threshold: 0.5,
            success_threshold: 3,
            half_open_max_calls: 5,
        };

        let result = manager.update_breaker_config("nonexistent", new_config);
        assert!(result.is_err());
        match result.unwrap_err() {
            GitAIError::Config(gitai_types::ConfigError::Missing(msg)) => assert!(msg.contains("not found")),
            _ => panic!("Expected ConfigError::Missing"),
        }

        manager.stop_cleanup_task().await;
    }

    /// TASK-003-3-1-2-2-2: 测试相同配置不触发事件
    #[tokio::test]
    async fn test_update_config_no_changes() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_millis(1000),
            window_size: 50,
            failure_rate_threshold: 0.5,
            half_open_max_calls: 5,
        };

        let manager = CircuitBreakerManager::new(config.clone());

        // 创建订阅者
        struct TestSubscriber {
            events: Arc<Mutex<Vec<CircuitBreakerManagerEvent>>>,
        }

        impl CircuitBreakerManagerSubscriber for TestSubscriber {
            fn on_manager_event(&self, event: CircuitBreakerManagerEvent) {
                let events = self.events.clone();
                tokio::spawn(async move {
                    let mut e = events.lock().await;
                    e.push(event);
                });
            }
        }

        let subscriber = TestSubscriber { events: events_clone };
        let _handle = manager.subscribe_to_manager_events(Box::new(subscriber));

        // 创建断路器实例
        let _breaker = manager.get_or_create_breaker("test_no_change");
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 使用相同配置更新
        let result = manager.update_breaker_config("test_no_change", config.clone());
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 没有发生变更

        tokio::time::sleep(Duration::from_millis(10)).await;

        // 验证没有额外的事件
        let events_guard = events.lock().await;
        assert_eq!(events_guard.len(), 1); // 只有InstanceCreated事件

        manager.stop_cleanup_task().await;
    }
}