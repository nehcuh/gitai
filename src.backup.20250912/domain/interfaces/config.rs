//! 配置服务接口定义

use super::{ConfigurableInterface, HealthCheckInterface, VersionedInterface};
use crate::domain::errors::ConfigError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 配置提供者接口
/// 负责管理和提供应用程序的各种配置
#[async_trait]
pub trait ConfigProvider:
    VersionedInterface + ConfigurableInterface + HealthCheckInterface + Send + Sync
{
    /// 从文件加载配置
    async fn load_from_file(&self, path: &Path) -> Result<(), ConfigError>;

    /// 从环境变量加载配置
    async fn load_from_env(&self) -> Result<(), ConfigError>;

    /// 获取AI配置
    async fn get_ai_config(&self) -> Result<AiConfig, ConfigError>;

    /// 获取扫描配置
    async fn get_scan_config(&self) -> Result<ScanConfig, ConfigError>;

    /// 获取DevOps配置
    async fn get_devops_config(&self) -> Result<Option<DevOpsConfig>, ConfigError>;

    /// 获取MCP配置
    async fn get_mcp_config(&self) -> Result<Option<McpConfig>, ConfigError>;

    /// 获取缓存配置
    async fn get_cache_config(&self) -> Result<CacheConfig, ConfigError>;

    /// 获取日志配置
    async fn get_logging_config(&self) -> Result<LoggingConfig, ConfigError>;

    /// 获取功能开关配置
    async fn get_feature_flags(&self) -> Result<FeatureFlags, ConfigError>;

    /// 保存配置到文件
    async fn save_to_file(&self, path: &Path) -> Result<(), ConfigError>;

    /// 重置配置到默认值
    async fn reset_to_defaults(&self) -> Result<(), ConfigError>;

    /// 获取配置的JSON表示
    async fn to_json(&self) -> Result<serde_json::Value, ConfigError>;

    /// 从JSON更新配置
    async fn update_from_json(&mut self, json: serde_json::Value) -> Result<(), ConfigError>;

    /// 订阅配置变更通知
    fn subscribe_config_changes(&self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 获取配置变更历史
    async fn get_config_history(&self) -> Result<Vec<ConfigChange>, ConfigError>;
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// API端点URL
    pub api_url: String,
    /// 使用的模型名称
    pub model: String,
    /// API密钥（可选）
    pub api_key: Option<String>,
    /// 温度参数（0.0-1.0）
    pub temperature: f32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 请求超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大令牌数
    pub max_tokens: Option<u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            temperature: 0.7,
            max_retries: 3,
            timeout_seconds: 60,
            max_tokens: Some(2048),
        }
    }
}

impl AiConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.api_url.trim().is_empty() {
            return Err(ConfigError::Missing("AI API URL".to_string()));
        }

        if !self.api_url.starts_with("http://") && !self.api_url.starts_with("https://") {
            return Err(ConfigError::InvalidFormat(
                "AI API URL must start with http:// or https://".to_string(),
            ));
        }

        if self.model.trim().is_empty() {
            return Err(ConfigError::Missing("AI model name".to_string()));
        }

        if self.temperature < 0.0 || self.temperature > 1.0 {
            return Err(ConfigError::ValidationFailed(format!(
                "AI temperature must be between 0.0 and 1.0, got {}",
                self.temperature
            )));
        }

        if self.max_retries == 0 {
            return Err(ConfigError::ValidationFailed(
                "AI max retries must be greater than 0".to_string(),
            ));
        }

        if self.timeout_seconds == 0 {
            return Err(ConfigError::ValidationFailed(
                "AI timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// 扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// 默认扫描路径
    pub default_path: String,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 并发任务数
    pub max_concurrency: usize,
    /// 规则目录路径
    pub rules_dir: Option<String>,
    /// 是否自动安装缺失的工具
    pub auto_install_tools: bool,
    /// 扫描结果缓存时间（秒）
    pub cache_ttl_seconds: u64,
    /// 要排除的文件模式
    pub exclude_patterns: Vec<String>,
    /// 要包含的文件扩展名
    pub include_extensions: Vec<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            default_path: ".".to_string(),
            timeout_seconds: 300,
            max_concurrency: 4,
            rules_dir: None,
            auto_install_tools: true,
            cache_ttl_seconds: 3600,
            exclude_patterns: vec![
                "*/.git/*".to_string(),
                "*/target/*".to_string(),
                "*/node_modules/*".to_string(),
                "*/__pycache__/*".to_string(),
            ],
            include_extensions: vec![
                "rs".to_string(),
                "java".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "go".to_string(),
                "c".to_string(),
                "cpp".to_string(),
            ],
        }
    }
}

impl ScanConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.timeout_seconds == 0 {
            return Err(ConfigError::ValidationFailed(
                "Scan timeout must be greater than 0".to_string(),
            ));
        }

        if self.timeout_seconds > 3600 {
            return Err(ConfigError::ValidationFailed(
                "Scan timeout cannot exceed 1 hour".to_string(),
            ));
        }

        if self.max_concurrency == 0 {
            return Err(ConfigError::ValidationFailed(
                "Scan max concurrency must be greater than 0".to_string(),
            ));
        }

        if self.max_concurrency > 32 {
            return Err(ConfigError::ValidationFailed(
                "Scan max concurrency cannot exceed 32".to_string(),
            ));
        }

        if self.cache_ttl_seconds == 0 {
            return Err(ConfigError::ValidationFailed(
                "Cache TTL must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// DevOps配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevOpsConfig {
    /// 平台类型（coding, github, gitlab等）
    pub platform: String,
    /// API基础URL
    pub api_base_url: String,
    /// 访问令牌
    pub access_token: Option<String>,
    /// 项目ID
    pub project_id: Option<u64>,
    /// API版本
    pub api_version: String,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for DevOpsConfig {
    fn default() -> Self {
        Self {
            platform: "coding".to_string(),
            api_base_url: "https://coding.net".to_string(),
            access_token: None,
            project_id: None,
            api_version: "v1".to_string(),
            timeout_seconds: 30,
        }
    }
}

/// MCP配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// 是否启用MCP
    pub enabled: bool,
    /// 服务器名称
    pub server_name: String,
    /// 服务器版本
    pub server_version: String,
    /// 监听地址
    pub listen_address: String,
    /// 监听端口
    pub listen_port: u16,
    /// 传输协议（stdio, tcp, sse）
    pub transport_protocol: String,
    /// 启用的服务
    pub enabled_services: Vec<String>,
    /// 最大并发连接数
    pub max_connections: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_name: "gitai-mcp".to_string(),
            server_version: "1.0.0".to_string(),
            listen_address: "127.0.0.1".to_string(),
            listen_port: 8080,
            transport_protocol: "stdio".to_string(),
            enabled_services: vec![
                "review".to_string(),
                "scan".to_string(),
                "commit".to_string(),
            ],
            max_connections: 100,
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 缓存类型（memory, file, redis）
    pub cache_type: String,
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 默认TTL（秒）
    pub default_ttl_seconds: u64,
    /// 缓存目录（文件缓存时使用）
    pub cache_dir: Option<String>,
    /// Redis连接字符串（Redis缓存时使用）
    pub redis_url: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: "file".to_string(),
            max_entries: 10000,
            default_ttl_seconds: 3600,
            cache_dir: None,
            redis_url: None,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub log_level: String,
    /// 日志格式
    pub log_format: String,
    /// 日志文件路径
    pub log_file: Option<String>,
    /// 是否启用文件日志
    pub enable_file_logging: bool,
    /// 日志轮转配置
    pub rotation_config: Option<RotationConfig>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_format: "pretty".to_string(),
            log_file: None,
            enable_file_logging: false,
            rotation_config: None,
        }
    }
}

/// 日志轮转配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// 轮转大小（字节）
    pub max_size_bytes: u64,
    /// 保留的日志文件数
    pub max_files: usize,
    /// 轮转周期（天）
    pub rotation_days: u32,
}

/// 功能开关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// 是否启用AI功能
    pub ai_enabled: bool,
    /// 是否启用安全扫描
    pub security_scan_enabled: bool,
    /// 是否启用Tree-sitter分析
    pub tree_sitter_enabled: bool,
    /// 是否启用MCP服务器
    pub mcp_enabled: bool,
    /// 是否启用度量收集
    pub metrics_enabled: bool,
    /// 是否启用自动更新
    pub auto_update_enabled: bool,
    /// 是否启用缓存
    pub cache_enabled: bool,
    /// 是否启用离线模式
    pub offline_mode: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            ai_enabled: true,
            security_scan_enabled: cfg!(feature = "security"),
            tree_sitter_enabled: true,
            mcp_enabled: cfg!(feature = "mcp"),
            metrics_enabled: cfg!(feature = "metrics"),
            auto_update_enabled: true,
            cache_enabled: true,
            offline_mode: false,
        }
    }
}

/// 配置变更记录
#[derive(Debug, Clone)]
pub struct ConfigChange {
    /// 变更时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 变更的配置项
    pub config_key: String,
    /// 旧值
    pub old_value: Option<String>,
    /// 新值
    pub new_value: Option<String>,
    /// 变更来源
    pub change_source: String,
}

/// 配置验证器trait
pub trait ConfigValidator {
    /// 验证配置
    fn validate(&self) -> Result<(), ConfigError>;
}

/// 配置默认值trait
pub trait ConfigDefault {
    /// 获取默认值
    fn default_config() -> Self;
}

/// 模块配置trait
pub trait ModuleConfig: ConfigValidator + ConfigDefault + Send + Sync {
    /// 获取模块名称
    fn module_name(&self) -> &str;

    /// 获取配置版本
    fn config_version(&self) -> &str;

    /// 验证模块特定的业务规则
    fn validate_business_rules(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

/// 环境感知配置trait
pub trait EnvironmentAwareConfig {
    /// 根据环境调整配置
    fn adjust_for_environment(&mut self, environment: &str) -> Result<(), ConfigError>;

    /// 获取当前环境
    fn get_environment(&self) -> &str;
}

/// 可观察配置trait
#[async_trait]
pub trait ObservableConfig {
    /// 获取配置指标
    async fn get_config_metrics(&self) -> Result<ConfigMetrics, ConfigError>;

    /// 获取配置变更事件流
    async fn get_config_events(&self) -> Result<Vec<ConfigEvent>, ConfigError>;
}

/// 配置指标
#[derive(Debug, Clone)]
pub struct ConfigMetrics {
    /// 配置项总数
    pub total_config_items: usize,
    /// 已验证的配置项数
    pub validated_config_items: usize,
    /// 配置加载时间
    pub load_time_ms: u64,
    /// 配置验证时间
    pub validation_time_ms: u64,
    /// 最后更新时间
    pub last_update_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// 配置事件
#[derive(Debug, Clone)]
pub struct ConfigEvent {
    /// 事件类型
    pub event_type: ConfigEventType,
    /// 事件时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 配置键
    pub config_key: String,
    /// 事件详情
    pub details: Option<serde_json::Value>,
}

/// 配置事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigEventType {
    /// 配置加载
    Loaded,
    /// 配置更新
    Updated,
    /// 配置验证
    Validated,
    /// 配置重置
    Reset,
    /// 配置错误
    Error,
}

impl std::fmt::Display for ConfigEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigEventType::Loaded => write!(f, "loaded"),
            ConfigEventType::Updated => write!(f, "updated"),
            ConfigEventType::Validated => write!(f, "validated"),
            ConfigEventType::Reset => write!(f, "reset"),
            ConfigEventType::Error => write!(f, "error"),
        }
    }
}

/// 配置提供者默认实现
pub struct DefaultConfigProvider {
    ai_config: AiConfig,
    scan_config: ScanConfig,
    devops_config: Option<DevOpsConfig>,
    mcp_config: Option<McpConfig>,
    cache_config: CacheConfig,
    logging_config: LoggingConfig,
    feature_flags: FeatureFlags,
}

impl DefaultConfigProvider {
    pub fn new() -> Self {
        Self {
            ai_config: AiConfig::default(),
            scan_config: ScanConfig::default(),
            devops_config: None,
            mcp_config: None,
            cache_config: CacheConfig::default(),
            logging_config: LoggingConfig::default(),
            feature_flags: FeatureFlags::default(),
        }
    }
}

impl Default for DefaultConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionedInterface for DefaultConfigProvider {
    fn interface_version(&self) -> &'static str {
        "1.0.0"
    }
}

#[async_trait]
impl ConfigurableInterface for DefaultConfigProvider {
    async fn validate_config(&self) -> Result<(), crate::domain::errors::ConfigError> {
        self.ai_config.validate()?;
        self.scan_config.validate()?;

        if let Some(_devops_config) = &self.devops_config {
            // TODO: 验证DevOps配置
        }

        if let Some(_mcp_config) = &self.mcp_config {
            // TODO: 验证MCP配置
        }

        Ok(())
    }

    async fn update_config(
        &self,
        _config: serde_json::Value,
    ) -> Result<(), crate::domain::errors::ConfigError> {
        // TODO: 实现配置更新逻辑
        Ok(())
    }

    async fn get_config(&self) -> Result<serde_json::Value, crate::domain::errors::ConfigError> {
        Ok(serde_json::json!({
            "ai": self.ai_config,
            "scan": self.scan_config,
            "devops": self.devops_config,
            "mcp": self.mcp_config,
            "cache": self.cache_config,
            "logging": self.logging_config,
            "features": self.feature_flags,
        }))
    }
}

#[async_trait]
impl HealthCheckInterface for DefaultConfigProvider {
    async fn health_check(&self) -> super::HealthCheckResult {
        match self.validate_config().await {
            Ok(_) => super::HealthCheckResult::healthy(),
            Err(e) => {
                super::HealthCheckResult::unhealthy(format!("Config validation failed: {}", e))
            }
        }
    }
}

#[async_trait]
impl ConfigProvider for DefaultConfigProvider {
    async fn load_from_file(&self, _path: &Path) -> Result<(), ConfigError> {
        // TODO: 实现从文件加载配置
        Ok(())
    }

    async fn load_from_env(&self) -> Result<(), ConfigError> {
        // TODO: 实现从环境变量加载配置
        Ok(())
    }

    async fn get_ai_config(&self) -> Result<AiConfig, ConfigError> {
        Ok(self.ai_config.clone())
    }

    async fn get_scan_config(&self) -> Result<ScanConfig, ConfigError> {
        Ok(self.scan_config.clone())
    }

    async fn get_devops_config(&self) -> Result<Option<DevOpsConfig>, ConfigError> {
        Ok(self.devops_config.clone())
    }

    async fn get_mcp_config(&self) -> Result<Option<McpConfig>, ConfigError> {
        Ok(self.mcp_config.clone())
    }

    async fn get_cache_config(&self) -> Result<CacheConfig, ConfigError> {
        Ok(self.cache_config.clone())
    }

    async fn get_logging_config(&self) -> Result<LoggingConfig, ConfigError> {
        Ok(self.logging_config.clone())
    }

    async fn get_feature_flags(&self) -> Result<FeatureFlags, ConfigError> {
        Ok(self.feature_flags.clone())
    }

    async fn save_to_file(&self, _path: &Path) -> Result<(), ConfigError> {
        // TODO: 实现保存配置到文件
        Ok(())
    }

    async fn reset_to_defaults(&self) -> Result<(), ConfigError> {
        // TODO: 实现重置到默认配置
        Ok(())
    }

    async fn to_json(&self) -> Result<serde_json::Value, ConfigError> {
        self.get_config().await
    }

    async fn update_from_json(&mut self, _json: serde_json::Value) -> Result<(), ConfigError> {
        // TODO: 实现从JSON更新配置
        Ok(())
    }

    fn subscribe_config_changes(&self, _handler: Box<dyn Fn(&str) + Send + Sync>) {
        // TODO: 实现配置变更订阅
    }

    async fn get_config_history(&self) -> Result<Vec<ConfigChange>, ConfigError> {
        // TODO: 实现获取配置变更历史
        Ok(Vec::new())
    }
}
