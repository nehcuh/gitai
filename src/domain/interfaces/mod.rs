//! 领域层接口定义
//! 
//! 定义所有外部依赖的抽象接口，实现依赖倒置原则

use std::sync::Arc;

pub mod config;
pub mod git;
pub mod ai;
pub mod cache;
pub mod scan;
pub mod review;
pub mod devops;


use async_trait::async_trait;

/// 版本化接口trait
/// 用于接口版本管理和兼容性检查
pub trait VersionedInterface {
    /// 获取当前接口版本
    fn interface_version(&self) -> &'static str;
    
    /// 获取支持的版本列表
    fn supported_versions(&self) -> Vec<&'static str> {
        vec![self.interface_version()]
    }
    
    /// 检查是否支持指定版本
    fn supports_version(&self, version: &str) -> bool {
        self.supported_versions().contains(&version)
    }
}

/// 可配置接口trait
/// 用于支持动态配置的接口
#[async_trait]
pub trait ConfigurableInterface {
    /// 验证配置
    async fn validate_config(&self) -> Result<(), crate::domain::errors::ConfigError>;
    
    /// 更新配置
    async fn update_config(&self, config: serde_json::Value) -> Result<(), crate::domain::errors::ConfigError>;
    
    /// 获取当前配置
    async fn get_config(&self) -> Result<serde_json::Value, crate::domain::errors::ConfigError>;
}

/// 健康检查接口trait
#[async_trait]
pub trait HealthCheckInterface {
    /// 执行健康检查
    async fn health_check(&self) -> HealthCheckResult;
}

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub is_healthy: bool,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl HealthCheckResult {
    pub fn healthy() -> Self {
        Self {
            is_healthy: true,
            status: HealthStatus::Healthy,
            message: None,
            details: None,
        }
    }
    
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            is_healthy: false,
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            details: None,
        }
    }
    
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            is_healthy: true,
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            details: None,
        }
    }
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 降级
    Degraded,
}

/// 监控指标接口trait
#[async_trait]
pub trait MetricsInterface {
    /// 记录指标
    async fn record_metric(&self, name: &str, value: f64, tags: Vec<(&str, &str)>);
    
    /// 增加计数器
    async fn increment_counter(&self, name: &str, tags: Vec<(&str, &str)>);
    
    /// 记录直方图
    async fn record_histogram(&self, name: &str, value: f64, tags: Vec<(&str, &str)>);
}

/// 日志接口trait
pub trait LoggingInterface {
    /// 记录调试日志
    fn log_debug(&self, message: &str);
    
    /// 记录信息日志
    fn log_info(&self, message: &str);
    
    /// 记录警告日志
    fn log_warning(&self, message: &str);
    
    /// 记录错误日志
    fn log_error(&self, message: &str);
}

/// 可清理资源接口trait
#[async_trait]
pub trait DisposableInterface {
    /// 清理资源
    async fn dispose(&self) -> Result<(), crate::domain::errors::DomainError>;
    
    /// 检查是否需要清理
    fn needs_disposal(&self) -> bool;
}

/// 服务状态接口trait
#[async_trait]
pub trait ServiceStatusInterface {
    /// 获取服务状态
    async fn get_status(&self) -> ServiceStatus;
    
    /// 获取服务统计信息
    async fn get_statistics(&self) -> ServiceStatistics;
}

/// 服务状态
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub is_running: bool,
    pub uptime: Option<std::time::Duration>,
    pub last_error: Option<String>,
    pub request_count: u64,
    pub error_count: u64,
}

impl ServiceStatus {
    pub fn new() -> Self {
        Self {
            is_running: false,
            uptime: None,
            last_error: None,
            request_count: 0,
            error_count: 0,
        }
    }
}

impl Default for ServiceStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// 服务统计信息
#[derive(Debug, Clone)]
pub struct ServiceStatistics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Option<std::time::Duration>,
    pub percentile_95_response_time: Option<std::time::Duration>,
    pub percentile_99_response_time: Option<std::time::Duration>,
}

impl ServiceStatistics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: None,
            percentile_95_response_time: None,
            percentile_99_response_time: None,
        }
    }
}

impl Default for ServiceStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// 服务注册表trait
pub trait ServiceRegistry {
    /// 注册服务
    fn register_service<T: VersionedInterface + Send + Sync + 'static>(
        &mut self, 
        name: &str, 
        service: Arc<T>
    ) -> Result<(), String>;
    
    /// 获取服务
    fn get_service<T: VersionedInterface + Send + Sync + 'static>(
        &self, 
        name: &str
    ) -> Option<Arc<T>>;
    
    /// 列出所有服务
    fn list_services(&self
    ) -> Vec<(&str, &str, &str)>; // (name, version, interface_type)
}

/// 服务发现trait
#[async_trait]
pub trait ServiceDiscovery {
    /// 发现服务
    async fn discover_service(&self, 
        name: &str, 
        version_requirement: Option<&str>
    ) -> Result<ServiceEndpoint, DiscoveryError>;
    
    /// 健康检查
    async fn health_check(&self, 
        endpoint: &ServiceEndpoint
    ) -> Result<HealthCheckResult, DiscoveryError>;
}

/// 服务端点信息
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub version: String,
    pub address: String,
    pub port: u16,
    pub protocol: String,
    pub metadata: Option<serde_json::Value>,
}

/// 服务发现错误
#[derive(Debug)]
pub enum DiscoveryError {
    ServiceNotFound(String),
    NoHealthyInstances(String),
    NetworkError(String),
    Timeout,
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::ServiceNotFound(name) => write!(f, "Service not found: {}", name),
            DiscoveryError::NoHealthyInstances(name) => write!(f, "No healthy instances for service: {}", name),
            DiscoveryError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DiscoveryError::Timeout => write!(f, "Discovery timeout"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// 结果类型别名
pub type ConfigResult<T> = Result<T, crate::domain::errors::ConfigError>;
pub type GitResult<T> = Result<T, crate::domain::errors::GitError>;
pub type AiResult<T> = Result<T, crate::domain::errors::AiError>;
pub type CacheResult<T> = Result<T, crate::domain::errors::CacheError>;
pub type ScanResult<T> = Result<T, crate::domain::errors::ScanError>;
pub type CommandResult<T> = Result<T, crate::domain::errors::CommandError>;
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;