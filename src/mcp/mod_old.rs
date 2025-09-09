// GitAI MCP 服务模块
//
// 该模块提供 GitAI 核心功能的 MCP (Model Context Protocol) 服务实现，
// 使得 GitAI 可以作为 MCP 服务器被 LLM 调用

pub mod bridge;
pub mod registry;
pub mod services;

use log::{debug, error, info, warn};
use registry::{ServiceRegistry, ServiceEventListener, ServiceEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 性能统计结构
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// 工具调用次数
    pub tool_calls: u64,
    /// 成功的工具调用次数
    pub successful_calls: u64,
    /// 失败的工具调用次数
    pub failed_calls: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
    /// 各工具的调用统计
    pub tool_stats: HashMap<String, ToolStats>,
}

/// 单个工具的统计信息
#[derive(Debug, Clone)]
pub struct ToolStats {
    /// 调用次数
    pub calls: u64,
    /// 成功次数
    pub successful_calls: u64,
    /// 失败次数
    pub failed_calls: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 最短执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最长执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub average_execution_time_ms: f64,
}

/// 性能统计收集器
#[derive(Debug)]
pub struct PerformanceCollector {
    /// 总调用次数
    total_calls: AtomicU64,
    /// 成功调用次数
    successful_calls: AtomicU64,
    /// 失败调用次数
    failed_calls: AtomicU64,
    /// 总执行时间（毫秒）
    total_execution_time_ms: AtomicU64,
    /// 各工具的统计信息
    tool_stats: Arc<parking_lot::RwLock<HashMap<String, ToolStats>>>,
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            successful_calls: AtomicU64::new(0),
            failed_calls: AtomicU64::new(0),
            total_execution_time_ms: AtomicU64::new(0),
            tool_stats: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
}

impl PerformanceCollector {
    /// 创建新的性能统计收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录工具调用开始
    pub fn record_call_start(&self, _tool_name: &str) -> Instant {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        Instant::now()
    }

    /// 记录工具调用成功
    pub fn record_call_success(&self, tool_name: &str, execution_time_ms: u64) {
        self.successful_calls.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms
            .fetch_add(execution_time_ms, Ordering::Relaxed);

        let mut tool_stats = self.tool_stats.write();
        let stats = tool_stats
            .entry(tool_name.to_string())
            .or_insert(ToolStats {
                calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
                average_execution_time_ms: 0.0,
            });

        stats.calls += 1;
        stats.successful_calls += 1;
        stats.total_execution_time_ms += execution_time_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(execution_time_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(execution_time_ms);
        stats.average_execution_time_ms = stats.total_execution_time_ms as f64 / stats.calls as f64;
    }

    /// 记录工具调用失败
    pub fn record_call_failure(&self, tool_name: &str, execution_time_ms: u64) {
        self.failed_calls.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms
            .fetch_add(execution_time_ms, Ordering::Relaxed);

        let mut tool_stats = self.tool_stats.write();
        let stats = tool_stats
            .entry(tool_name.to_string())
            .or_insert(ToolStats {
                calls: 0,
                successful_calls: 0,
                failed_calls: 0,
                total_execution_time_ms: 0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
                average_execution_time_ms: 0.0,
            });

        stats.calls += 1;
        stats.failed_calls += 1;
        stats.total_execution_time_ms += execution_time_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(execution_time_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(execution_time_ms);
        stats.average_execution_time_ms = stats.total_execution_time_ms as f64 / stats.calls as f64;
    }

    /// 获取性能统计
    pub fn get_stats(&self) -> PerformanceStats {
        let total_calls = self.total_calls.load(Ordering::Relaxed);
        let successful_calls = self.successful_calls.load(Ordering::Relaxed);
        let failed_calls = self.failed_calls.load(Ordering::Relaxed);
        let total_execution_time_ms = self.total_execution_time_ms.load(Ordering::Relaxed);

        let average_execution_time_ms = if total_calls > 0 {
            total_execution_time_ms as f64 / total_calls as f64
        } else {
            0.0
        };

        let tool_stats = self.tool_stats.read().clone();

        PerformanceStats {
            tool_calls: total_calls,
            successful_calls,
            failed_calls,
            total_execution_time_ms,
            average_execution_time_ms,
            tool_stats,
        }
    }

    /// 重置统计信息
    pub fn reset(&self) {
        self.total_calls.store(0, Ordering::Relaxed);
        self.successful_calls.store(0, Ordering::Relaxed);
        self.failed_calls.store(0, Ordering::Relaxed);
        self.total_execution_time_ms.store(0, Ordering::Relaxed);
        self.tool_stats.write().clear();
    }
}

// 重新导出核心类型
pub use rmcp::{
    model::{Implementation, Tool},
    service::ServiceError,
};

// 类型别名
pub type McpResult<T> = Result<T, McpError>;

/// GitAI MCP 错误类型
#[derive(Debug, Clone)]
pub enum McpError {
    /// 参数验证错误
    InvalidParameters(String),
    /// 服务执行错误
    ExecutionFailed(String),
    /// 配置错误
    ConfigurationError(String),
    /// 文件操作错误
    FileOperationError(String),
    /// 网络错误
    NetworkError(String),
    /// 外部工具错误
    ExternalToolError(String),
    /// 权限错误
    PermissionError(String),
    /// 超时错误
    TimeoutError(String),
    /// 未知错误
    Unknown(String),
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            McpError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            McpError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            McpError::FileOperationError(msg) => write!(f, "File operation error: {}", msg),
            McpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            McpError::ExternalToolError(msg) => write!(f, "External tool error: {}", msg),
            McpError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
            McpError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            McpError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for McpError {}

impl From<McpError> for ServiceError {
    fn from(err: McpError) -> Self {
        ServiceError::Transport(std::io::Error::other(err.to_string()))
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::InvalidParameters(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                McpError::FileOperationError(format!("File not found: {}", err))
            }
            std::io::ErrorKind::PermissionDenied => {
                McpError::PermissionError(format!("Permission denied: {}", err))
            }
            std::io::ErrorKind::TimedOut => McpError::TimeoutError(format!("Timeout: {}", err)),
            _ => McpError::FileOperationError(format!("IO error: {}", err)),
        }
    }
}

impl From<tokio::time::error::Elapsed> for McpError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        McpError::TimeoutError(format!("Operation timeout: {}", err))
    }
}

// 错误创建辅助函数
#[allow(dead_code)]
pub fn invalid_parameters_error<T: Into<String>>(msg: T) -> McpError {
    McpError::InvalidParameters(msg.into())
}

#[allow(dead_code)]
pub fn execution_failed_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ExecutionFailed(msg.into())
}

#[allow(dead_code)]
pub fn configuration_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ConfigurationError(msg.into())
}

#[allow(dead_code)]
pub fn file_operation_error<T: Into<String>>(msg: T) -> McpError {
    McpError::FileOperationError(msg.into())
}

#[allow(dead_code)]
pub fn network_error<T: Into<String>>(msg: T) -> McpError {
    McpError::NetworkError(msg.into())
}

#[allow(dead_code)]
pub fn external_tool_error<T: Into<String>>(msg: T) -> McpError {
    McpError::ExternalToolError(msg.into())
}

#[allow(dead_code)]
pub fn permission_error<T: Into<String>>(msg: T) -> McpError {
    McpError::PermissionError(msg.into())
}

#[allow(dead_code)]
pub fn timeout_error<T: Into<String>>(msg: T) -> McpError {
    McpError::TimeoutError(msg.into())
}

#[allow(dead_code)]
pub fn unknown_error<T: Into<String>>(msg: T) -> McpError {
    McpError::Unknown(msg.into())
}

// 向后兼容的辅助函数
#[allow(dead_code)]
pub fn service_error(msg: String) -> ServiceError {
    ServiceError::Transport(std::io::Error::other(msg))
}

/// GitAI MCP 服务管理器
pub struct GitAiMcpManager {
    /// GitAI 配置
    #[allow(dead_code)]
    config: crate::config::Config,
    /// 服务注册表
    registry: ServiceRegistry,
    /// 性能统计收集器
    performance_collector: Arc<PerformanceCollector>,
}

/// 服务事件监听器实现
struct McpServiceEventListener {
    name: String,
}

impl McpServiceEventListener {
    fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl ServiceEventListener for McpServiceEventListener {
    async fn on_service_event(&self, event: ServiceEvent) {
        match event {
            ServiceEvent::Registered { service_id, metadata } => {
                info!("🎉 服务已注册: {} (ID: {}, 版本: {})", 
                      metadata.name, service_id, metadata.version);
            }
            ServiceEvent::Unregistered { service_id, reason } => {
                info!("📤 服务已注销: {} (原因: {})", service_id, reason);
            }
            ServiceEvent::StatusChanged { service_id, old_status, new_status } => {
                info!("🔄 服务状态变更: {} {:?} -> {:?}", 
                      service_id, old_status, new_status);
            }
            ServiceEvent::HealthCheckCompleted { service_id, healthy, response_time } => {
                if healthy {
                    debug!("💚 服务健康检查通过: {} (响应时间: {:?})", 
                           service_id, response_time);
                } else {
                    warn!("💔 服务健康检查失败: {} (响应时间: {:?})", 
                          service_id, response_time);
                }
            }
        }
    }
}

/// GitAI MCP 服务 trait
#[async_trait::async_trait]
pub trait GitAiMcpService: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;

    /// 服务描述
    fn description(&self) -> &str;

    /// 获取服务提供的工具列表
    fn tools(&self) -> Vec<Tool>;

    /// 处理工具调用
    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value>;
}

impl GitAiMcpManager {
    /// 创建新的 MCP 服务管理器
    pub async fn new(config: crate::config::Config) -> McpResult<Self> {
        info!("🔧 初始化 GitAI MCP 服务管理器");
        
        let performance_collector = Arc::new(PerformanceCollector::new());
        let registry = ServiceRegistry::new();
        
        // 添加事件监听器
        let event_listener = Arc::new(McpServiceEventListener::new("GitAI-MCP".to_string()));
        registry.add_event_listener(event_listener).await;
        
        let mut manager = Self {
            config: config.clone(),
            registry,
            performance_collector,
        };
        
        // 根据配置初始化服务
        if let Some(mcp_config) = &config.mcp {
            if mcp_config.enabled {
                info!("📋 启用 MCP 服务: {:?}", mcp_config.services.enabled);
                for service_name in &mcp_config.services.enabled {
                    if let Err(e) = manager.register_service_by_name(service_name, &config).await {
                        error!("❌ 服务 '{$1}' $2", service_name, e);
                    }
                }
                
                let services = manager.registry.list_services().await;
                info!(
                    "🎯 MCP 服务管理器初始化完成，共注册 {} 个服务",
                    services.len()
                );
            } else {
                info!("ℹ️  MCP 服务已禁用");
            }
        } else {
            info!("ℹ️  未找到 MCP 配置");
        }
        
        Ok(manager)
    }
    
    /// 根据服务名称注册服务
    async fn register_service_by_name(
        &mut self,
        service_name: &str,
        config: &crate::config::Config,
    ) -> McpResult<()> {
        debug!("🔧 正在初始化服务: {}", service_name);
        
        let service_config = serde_json::json!({
            "service_name": service_name,
            "enabled": true
        });
        
        match service_name {
            "review" => {
                let service = services::ReviewService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create review service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            "commit" => {
                let service = services::CommitService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create commit service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            "scan" => {
                let service = services::ScanService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create scan service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            "analysis" => {
                let service = services::AnalysisService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create analysis service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            "dependency" => {
                let service = services::DependencyService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create dependency service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            "deviation" => {
                let service = services::DeviationService::new(config.clone())
                    .map_err(|e| configuration_error(format!("Failed to create deviation service: {}", e)))?;
                self.registry
                    .register_service(Arc::new(service), service_config)
                    .await?;
                info!("✅ 服务 '{$1}' $2");
            }
            _ => {
                warn!("⚠️  未知的服务名称: {}", service_name);
                return Err(configuration_error(format!("Unknown service: {}", service_name)));
            }
        }
        
        Ok(())
    }", mcp_config.services.enabled);
                for service_name in &mcp_config.services.enabled {
                    debug!("🔧 正在初始化服务: {}", service_name);
                    match service_name.as_str() {
                        "review" => match services::ReviewService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "review".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        "commit" => match services::CommitService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "commit".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        "scan" => match services::ScanService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "scan".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        "analysis" => match services::AnalysisService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "analysis".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        "dependency" => match services::DependencyService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "dependency".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        "deviation" => match services::DeviationService::new(config.clone()) {
                            Ok(service) => {
                                services.insert(
                                    "deviation".to_string(),
                                    Box::new(service) as Box<dyn GitAiMcpService + Send + Sync>,
                                );
                                info!("✅ 服务 '{$1}' $2");
                            }
                            Err(e) => {
                                error!("❌ 服务 '{$1}' $2", e);
                            }
                        },
                        _ => {
                            warn!("⚠️  未知的服务名称: {}", service_name);
                        }
                    }
                }
                info!(
                    "🎯 MCP 服务管理器初始化完成，共注册 {} 个服务",
                    services.len()
                );
            } else {
                info!("ℹ️  MCP 服务已禁用");
            }
        } else {
            info!("ℹ️  未找到 MCP 配置");
        }

        Self {
            config,
            services,
            performance_collector,
        }
    }

    /// 获取所有工具
    #[allow(dead_code)]
    pub async fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = Vec::new();
        let services = self.registry.list_services().await;
        
        for metadata in services {
            if let Some(service) = self.registry.get_service(&metadata.id).await {
                tools.extend(service.tools());
            }
        }
        tools
    }
    
    /// 动态注册服务
    #[allow(dead_code)]
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<()> {
        self.registry.register_service(service, config).await
    }
    
    /// 动态注销服务
    #[allow(dead_code)]
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()> {
        self.registry.unregister_service(service_id, reason).await
    }
    
    /// 获取所有服务列表
    #[allow(dead_code)]
    pub async fn list_services(&self) -> Vec<registry::ServiceMetadata> {
        self.registry.list_services().await
    }
    
    /// 获取健康的服务列表
    #[allow(dead_code)]
    pub async fn get_healthy_services(&self) -> Vec<registry::ServiceMetadata> {
        self.registry.get_healthy_services().await
    }
        tools
    }

    /// 处理工具调用
    pub async fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value> {
        debug!("🔧 处理工具调用: {}", tool_name);
        debug!(
            "📋 工具参数: {}",
            serde_json::to_string_pretty(&arguments).unwrap_or_default()
        );

        // 记录调用开始
        let start_time = self.performance_collector.record_call_start(tool_name);

        // 查找处理该工具的服务
        if let Some(service) = self.registry.find_service_by_tool(tool_name).await {
            debug!("🎯 找到处理服务: {}", service.name());

            let result = service.handle_tool_call(tool_name, arguments).await;

            // 记录调用结果
            let duration = start_time.elapsed();
            let duration_ms = duration.as_millis() as u64;

            match &result {
                Ok(_) => {
                    self.performance_collector
                        .record_call_success(tool_name, duration_ms);
                    info!("✅ 工具调用成功: {} (耗时: {:?})", tool_name, duration);
                }
                Err(e) => {
                    self.performance_collector
                        .record_call_failure(tool_name, duration_ms);
                    warn!(
                        "❌ 工具调用失败: {} (耗时: {:?}, 错误: {})",
                        tool_name, duration, e
                    );
                }
            }

            return result;
        }

        error!("❌ 未找到处理工具的服务: {}", tool_name);
        Err(invalid_parameters_error(format!(
            "Unknown tool: {}",
            tool_name
        )))
    }", tool_name);
        debug!(
            "📋 工具参数: {}",
            serde_json::to_string_pretty(&arguments).unwrap_or_default()
        );

        // 记录调用开始
        let start_time = self.performance_collector.record_call_start(tool_name);

        // 查找处理该工具的服务
        for service in self.services.values() {
            let tools = service.tools();
            if tools.iter().any(|tool| tool.name == tool_name) {
                debug!("🎯 找到处理服务: {}", service.name());

                let result = service.handle_tool_call(tool_name, arguments).await;

                // 记录调用结果
                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis() as u64;

                match &result {
                    Ok(_) => {
                        self.performance_collector
                            .record_call_success(tool_name, duration_ms);
                        info!("✅ 工具调用成功: {} (耗时: {:?})", tool_name, duration);
                    }
                    Err(e) => {
                        self.performance_collector
                            .record_call_failure(tool_name, duration_ms);
                        warn!(
                            "❌ 工具调用失败: {} (耗时: {:?}, 错误: {})",
                            tool_name, duration, e
                        );
                    }
                }

                return result;
            }
        }

        error!("❌ 未找到处理工具的服务: {}", tool_name);
        Err(invalid_parameters_error(format!(
            "Unknown tool: {}",
            tool_name
        )))
    }

    /// 获取性能统计
    #[allow(dead_code)]
    pub fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_collector.get_stats()
    }

    /// 重置性能统计
    #[allow(dead_code)]
    pub fn reset_performance_stats(&self) {
        self.performance_collector.reset();
        info!("📊 性能统计已重置");
    }

    /// 获取服务器信息
    #[allow(dead_code)]
    pub fn get_server_info(&self) -> Option<Implementation> {
        self.config.mcp.as_ref().map(|config| Implementation {
            name: config.server.name.clone(),
            version: config.server.version.clone(),
        })
    }
}

// =============================================================================
// MCP Error Conversion Helpers - Eliminate repetition following Linus's taste
// =============================================================================

/// Convert parameter parsing error to MCP error
/// Linus principle: eliminate the pattern "Failed to parse XXX parameters: {}"
pub fn parse_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    invalid_parameters_error(format!(
        "Failed to parse {} parameters: {}",
        service_name, e
    ))
}

/// Convert execution error to MCP error
/// Linus principle: eliminate the pattern "XXX execution failed: {}"
pub fn execution_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    execution_failed_error(format!("{} execution failed: {}", service_name, e))
}

/// Convert serialization error to MCP error
/// Linus principle: eliminate the pattern "Failed to serialize XXX result: {}"
pub fn serialize_error(service_name: &str, e: impl std::fmt::Display) -> McpError {
    execution_failed_error(format!(
        "Failed to serialize {} result: {}",
        service_name, e
    ))
}
