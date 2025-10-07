/// TASK-003-3-1-4: 断路器指标收集和导出模块
///
/// 这个模块提供了断路器运行时指标的收集、导出和监控功能，支持：
/// - Prometheus 格式指标导出
/// - 实时指标收集
/// - 自定义指标注册
/// - 指标过期清理
///
/// # 使用示例
///
/// ```rust
/// use gitai_core::metrics::{MetricsManager};
/// use gitai_core::ai::{CircuitBreakerManager, CircuitBreakerConfig};
/// use std::time::Duration;
///
/// // 创建管理器和指标系统
/// let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());
/// let metrics_manager = MetricsManager::new(Duration::from_secs(10));
///
/// // 初始化默认指标
/// metrics_manager.init_default_metrics();
///
/// // 收集指标
/// metrics_manager.collect_from_manager(&manager).await;
///
/// // 导出为 Prometheus 格式
/// let prometheus_output = metrics_manager.export_prometheus();
/// println!("{}", prometheus_output);
/// ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// Re-export from parent module
use super::ai::{
    CircuitBreakerManager, CircuitState, HealthStatus,
};

/// TASK-003-3-1-4: 指标类型定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器类型（单调递增）
    Counter,
    /// 仪表盘类型（可增可减）
    Gauge,
    /// 直方图类型
    Histogram,
    /// 摘要类型
    Summary,
}

/// TASK-003-3-1-4: 指标单位
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricUnit {
    /// 无单位
    None,
    /// 毫秒
    Milliseconds,
    /// 秒
    Seconds,
    /// 字节
    Bytes,
    /// 次数
    Count,
    /// 百分比
    Percent,
    /// 每秒请求数
    RequestsPerSecond,
}

/// TASK-003-3-1-4: 指标元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricMetadata {
    /// 指标名称
    pub name: String,
    /// 指标描述
    pub description: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标单位
    pub unit: MetricUnit,
    /// 标签键
    pub label_keys: Vec<String>,
}

/// TASK-003-3-1-4: 指标值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// 指标名称
    pub name: String,
    /// 标签值
    pub labels: HashMap<String, String>,
    /// 数值
    pub value: f64,
    /// 时间戳（毫秒）
    pub timestamp_ms: u64,
}

/// TASK-003-3-1-4: 指标家族
#[derive(Debug, Clone)]
pub struct MetricFamily {
    /// 元数据
    pub metadata: MetricMetadata,
    /// 所有指标值
    pub values: Vec<MetricValue>,
}

/// TASK-003-3-1-4: 断路器指标收集器
#[derive(Debug)]
pub struct CircuitBreakerMetricsCollector {
    /// 指标存储
    metrics: Arc<RwLock<HashMap<String, MetricFamily>>>,
    /// 收集间隔
    collection_interval: Duration,
    /// 是否启用收集
    enabled: Arc<RwLock<bool>>,
    /// 最后收集时间
    last_collection: Arc<RwLock<Instant>>,
}

impl CircuitBreakerMetricsCollector {
    /// 创建新的指标收集器
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
            enabled: Arc::new(RwLock::new(true)),
            last_collection: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// 启用/禁用指标收集
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.write() = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// 注册指标家族
    pub fn register_metric(&self, metadata: MetricMetadata) {
        let mut metrics = self.metrics.write();
        metrics.insert(metadata.name.clone(), MetricFamily {
            metadata,
            values: Vec::new(),
        });
    }

    /// 更新指标值
    pub fn update_metric(&self, name: &str, labels: HashMap<String, String>, value: f64) {
        let mut metrics = self.metrics.write();
        if let Some(family) = metrics.get_mut(name) {
            // 查找现有指标
            if let Some(metric) = family.values.iter_mut()
                .find(|m| m.labels == labels) {
                metric.value = value;
                metric.timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
            } else {
                // 创建新指标
                family.values.push(MetricValue {
                    name: name.to_string(),
                    labels,
                    value,
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                });
            }
        }
    }

    /// 递增计数器指标
    pub fn increment_counter(&self, name: &str, labels: HashMap<String, String>) {
        let mut metrics = self.metrics.write();
        if let Some(family) = metrics.get_mut(name) {
            if let Some(metric) = family.values.iter_mut()
                .find(|m| m.labels == labels) {
                metric.value += 1.0;
                metric.timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
            } else {
                family.values.push(MetricValue {
                    name: name.to_string(),
                    labels,
                    value: 1.0,
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                });
            }
        }
    }

    /// 设置仪表盘指标
    pub fn set_gauge(&self, name: &str, labels: HashMap<String, String>, value: f64) {
        self.update_metric(name, labels, value);
    }

    /// 记录直方图指标
    pub fn observe_histogram(&self, name: &str, labels: HashMap<String, String>, value: f64) {
        // 简化实现：记录为计数和总和
        let count_labels = {
            let mut count_labels = labels.clone();
            count_labels.insert("_type".to_string(), "count".to_string());
            count_labels
        };

        let sum_labels = {
            let mut sum_labels = labels;
            sum_labels.insert("_type".to_string(), "sum".to_string());
            sum_labels
        };

        self.increment_counter(name, count_labels);

        let mut metrics = self.metrics.write();
        if let Some(family) = metrics.get_mut(name) {
            if let Some(metric) = family.values.iter_mut()
                .find(|m| m.labels == sum_labels) {
                metric.value += value;
                metric.timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
            } else {
                family.values.push(MetricValue {
                    name: name.to_string(),
                    labels: sum_labels,
                    value,
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                });
            }
        }
    }

    /// 从断路器管理器收集指标
    pub async fn collect_from_manager(&self, manager: &CircuitBreakerManager) {
        if !self.is_enabled() {
            return;
        }

        let now = Instant::now();
        let should_collect = {
            let last_collection = *self.last_collection.read();
            now.duration_since(last_collection) >= self.collection_interval
        };

        if !should_collect {
            return;
        }

        // 收集管理器级别的指标
        self.collect_manager_metrics(manager).await;

        // 收集实例级别的指标
        self.collect_instance_metrics(manager).await;

        *self.last_collection.write() = now;
    }

    /// 收集管理器级别指标
    async fn collect_manager_metrics(&self, manager: &CircuitBreakerManager) {
        let stats = manager.get_performance_stats();
        let health = manager.get_manager_health();

        // 管理器性能指标
        let manager_labels = HashMap::from([
            ("manager_id".to_string(), manager.manager_id().to_string()),
        ]);

        // 访问指标
        self.set_gauge("circuit_breaker_total_accesses",
                     manager_labels.clone(),
                     stats.total_accesses as f64);

        self.set_gauge("circuit_breaker_cache_hits",
                     manager_labels.clone(),
                     stats.cache_hits as f64);

        self.set_gauge("circuit_breaker_cache_misses",
                     manager_labels.clone(),
                     stats.cache_misses as f64);

        // 计算缓存命中率
        let hit_rate = if stats.total_accesses > 0 {
            stats.cache_hits as f64 / stats.total_accesses as f64
        } else {
            0.0
        };
        self.set_gauge("circuit_breaker_cache_hit_rate",
                     manager_labels.clone(),
                     hit_rate);

        // 并发指标
        self.set_gauge("circuit_breaker_current_concurrent_accesses",
                     manager_labels.clone(),
                     stats.current_concurrent_accesses as f64);

        self.set_gauge("circuit_breaker_max_concurrent_accesses",
                     manager_labels.clone(),
                     stats.max_concurrent_accesses as f64);

        // 健康状态指标
        let health_value = match health.overall_status {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Warning(_) => 0.5,
            HealthStatus::Unhealthy(_) => 0.0,
            HealthStatus::Unknown(_) => -1.0,
        };

        self.set_gauge("circuit_breaker_manager_health",
                     manager_labels.clone(),
                     health_value);

        // 实例数量指标
        self.set_gauge("circuit_breaker_total_instances",
                     manager_labels.clone(),
                     health.total_instances as f64);

        self.set_gauge("circuit_breaker_healthy_instances",
                     manager_labels.clone(),
                     health.healthy_instances as f64);

        self.set_gauge("circuit_breaker_warning_instances",
                     manager_labels.clone(),
                     health.warning_instances as f64);

        self.set_gauge("circuit_breaker_unhealthy_instances",
                     manager_labels.clone(),
                     health.unhealthy_instances as f64);
    }

    /// 收集实例级别指标
    async fn collect_instance_metrics(&self, manager: &CircuitBreakerManager) {
        let instance_health = manager.get_all_instances_health();

        for health in instance_health {
            let instance_labels = HashMap::from([
                ("instance_name".to_string(), health.name.clone()),
                ("manager_id".to_string(), manager.manager_id().to_string()),
            ]);

            // 实例状态指标
            let state_value = match health.metrics.current_state {
                CircuitState::Closed => 1.0,
                CircuitState::Open => 0.0,
                CircuitState::HalfOpen => 0.5,
            };

            self.set_gauge("circuit_breaker_instance_state",
                         instance_labels.clone(),
                         state_value);

            // 实例健康指标
            let health_value = match health.status {
                HealthStatus::Healthy => 1.0,
                HealthStatus::Warning(_) => 0.5,
                HealthStatus::Unhealthy(_) => 0.0,
                HealthStatus::Unknown(_) => -1.0,
            };

            self.set_gauge("circuit_breaker_instance_health",
                         instance_labels.clone(),
                         health_value);

            // 请求统计指标
            self.set_gauge("circuit_breaker_instance_requests",
                         instance_labels.clone(),
                         health.metrics.total_requests as f64);

            self.set_gauge("circuit_breaker_instance_successes",
                         instance_labels.clone(),
                         health.metrics.successful_requests as f64);

            self.set_gauge("circuit_breaker_instance_failures",
                         instance_labels.clone(),
                         health.metrics.failed_requests as f64);

            self.set_gauge("circuit_breaker_instance_failure_rate",
                         instance_labels.clone(),
                         health.metrics.failure_rate);

            // 平均响应时间指标（毫秒）
            let avg_response_time_ms = health.metrics.avg_response_time_us / 1000.0;
            self.set_gauge("circuit_breaker_instance_avg_response_time_ms",
                         instance_labels.clone(),
                         avg_response_time_ms);

            // 状态持续时间指标（秒）
            let state_duration_secs = health.metrics.state_duration.as_secs_f64();
            self.set_gauge("circuit_breaker_instance_state_duration_seconds",
                         instance_labels.clone(),
                         state_duration_secs);
        }
    }

    /// 获取所有指标
    pub fn get_all_metrics(&self) -> Vec<MetricFamily> {
        self.metrics.read().values().cloned().collect()
    }

    /// 清除所有指标
    pub fn clear_metrics(&self) {
        self.metrics.write().clear();
    }

    /// 清除过期指标
    pub fn clear_expired_metrics(&self, max_age: Duration) {
        let _now = Instant::now();
        let mut metrics = self.metrics.write();

        for family in metrics.values_mut() {
            family.values.retain(|metric| {
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64).saturating_sub(metric.timestamp_ms) <= max_age.as_millis() as u64
            });
        }

        // 移除空的指标家族
        metrics.retain(|_, family| !family.values.is_empty());
    }
}

impl Default for CircuitBreakerMetricsCollector {
    fn default() -> Self {
        Self::new(Duration::from_secs(10)) // 默认10秒收集间隔
    }
}

/// TASK-003-3-1-4: Prometheus 格式导出器
#[derive(Debug)]
pub struct PrometheusExporter {
    collector: Arc<CircuitBreakerMetricsCollector>,
}

impl PrometheusExporter {
    /// 创建新的 Prometheus 导出器
    pub fn new(collector: Arc<CircuitBreakerMetricsCollector>) -> Self {
        Self { collector }
    }

    /// 导出为 Prometheus 格式
    pub fn export(&self) -> String {
        let metrics = self.collector.get_all_metrics();
        let mut output = String::new();

        for family in metrics {
            // 输出 TYPE 和 HELP 注释
            output.push_str(&format!("# HELP {} {}\n",
                family.metadata.name,
                family.metadata.description));

            let type_str = match family.metadata.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", family.metadata.name, type_str));

            // 输出指标值
            for metric in &family.values {
                let label_str = if metric.labels.is_empty() {
                    String::new()
                } else {
                    let labels: Vec<String> = metric.labels
                        .iter()
                        .map(|(k, v)| format!("{}=\"{}\"", k, v))
                        .collect();
                    format!("{{{}}}", labels.join(","))
                };

                output.push_str(&format!("{}{} {}\n",
                    metric.name,
                    label_str,
                    metric.value));
            }

            output.push('\n');
        }

        output
    }
}

/// TASK-003-3-1-4: 指标管理器
#[derive(Debug)]
pub struct MetricsManager {
    collector: Arc<CircuitBreakerMetricsCollector>,
    exporter: PrometheusExporter,
}

impl MetricsManager {
    /// 创建新的指标管理器
    pub fn new(collection_interval: Duration) -> Self {
        let collector = Arc::new(CircuitBreakerMetricsCollector::new(collection_interval));
        let exporter = PrometheusExporter::new(collector.clone());

        Self { collector, exporter }
    }

    /// 初始化默认指标
    pub fn init_default_metrics(&self) {
        let metrics = vec![
            MetricMetadata {
                name: "circuit_breaker_total_accesses".to_string(),
                description: "Total number of circuit breaker accesses".to_string(),
                metric_type: MetricType::Counter,
                unit: MetricUnit::Count,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_cache_hits".to_string(),
                description: "Number of circuit breaker cache hits".to_string(),
                metric_type: MetricType::Counter,
                unit: MetricUnit::Count,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_cache_misses".to_string(),
                description: "Number of circuit breaker cache misses".to_string(),
                metric_type: MetricType::Counter,
                unit: MetricUnit::Count,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_cache_hit_rate".to_string(),
                description: "Circuit breaker cache hit rate".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::Percent,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_manager_health".to_string(),
                description: "Circuit breaker manager health status".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::None,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_total_instances".to_string(),
                description: "Total number of circuit breaker instances".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::Count,
                label_keys: vec!["manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_state".to_string(),
                description: "Circuit breaker instance state".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::None,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_health".to_string(),
                description: "Circuit breaker instance health status".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::None,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_requests".to_string(),
                description: "Total requests handled by circuit breaker instance".to_string(),
                metric_type: MetricType::Counter,
                unit: MetricUnit::Count,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_failure_rate".to_string(),
                description: "Circuit breaker instance failure rate".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::Percent,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_avg_response_time_ms".to_string(),
                description: "Circuit breaker instance average response time in milliseconds".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::Milliseconds,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
            MetricMetadata {
                name: "circuit_breaker_instance_uptime_seconds".to_string(),
                description: "Circuit breaker instance uptime in seconds".to_string(),
                metric_type: MetricType::Gauge,
                unit: MetricUnit::Seconds,
                label_keys: vec!["instance_name".to_string(), "manager_id".to_string()],
            },
        ];

        for metric in metrics {
            self.collector.register_metric(metric);
        }
    }

    /// 获取指标收集器
    pub fn collector(&self) -> &Arc<CircuitBreakerMetricsCollector> {
        &self.collector
    }

    /// 获取导出器
    pub fn exporter(&self) -> &PrometheusExporter {
        &self.exporter
    }

    /// 从管理器收集指标
    pub async fn collect_from_manager(&self, manager: &CircuitBreakerManager) {
        self.collector.collect_from_manager(manager).await;
    }

    /// 导出 Prometheus 格式指标
    pub fn export_prometheus(&self) -> String {
        self.exporter.export()
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collector_basic() {
        let collector = CircuitBreakerMetricsCollector::new(Duration::from_millis(100));

        // 注册测试指标
        let metadata = MetricMetadata {
            name: "test_counter".to_string(),
            description: "Test counter".to_string(),
            metric_type: MetricType::Counter,
            unit: MetricUnit::Count,
            label_keys: vec!["test_label".to_string()],
        };

        collector.register_metric(metadata);

        // 更新指标
        let mut labels = HashMap::new();
        labels.insert("test_label".to_string(), "test_value".to_string());

        collector.increment_counter("test_counter", labels.clone());
        collector.increment_counter("test_counter", labels.clone());

        // 验证指标
        let metrics = collector.get_all_metrics();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].values.len(), 1);
        assert_eq!(metrics[0].values[0].value, 2.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = Arc::new(CircuitBreakerMetricsCollector::new(Duration::from_millis(100)));
        let exporter = PrometheusExporter::new(collector.clone());

        // 注册测试指标
        let metadata = MetricMetadata {
            name: "test_gauge".to_string(),
            description: "Test gauge".to_string(),
            metric_type: MetricType::Gauge,
            unit: MetricUnit::None,
            label_keys: vec!["service".to_string()],
        };

        collector.register_metric(metadata);

        // 设置指标值
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "test".to_string());
        collector.set_gauge("test_gauge", labels, 42.0);

        // 导出并验证格式
        let exported = exporter.export();
        assert!(exported.contains("# HELP test_gauge Test gauge"));
        assert!(exported.contains("# TYPE test_gauge gauge"));
        assert!(exported.contains("test_gauge{service=\"test\"} 42"));
    }

    #[tokio::test]
    async fn test_metrics_manager() {
        let manager = MetricsManager::new(Duration::from_millis(100));
        manager.init_default_metrics();

        // 验证指标已初始化
        let metrics = manager.collector().get_all_metrics();
        assert!(!metrics.is_empty());

        // 验证导出功能
        let exported = manager.export_prometheus();
        assert!(!exported.is_empty());
        assert!(exported.contains("# HELP"));
        assert!(exported.contains("# TYPE"));
    }

}