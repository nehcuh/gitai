//! 领域层错误定义

use std::fmt;

/// 领域层通用错误
#[derive(Debug)]
pub enum DomainError {
    /// 配置错误
    Configuration(String),
    /// 验证失败
    Validation(String),
    /// 业务规则违反
    BusinessRule(String),
    /// 资源未找到
    NotFound(String),
    /// 权限不足
    Unauthorized(String),
    /// 服务不可用
    ServiceUnavailable(String),
    /// 外部依赖错误
    ExternalDependency(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            DomainError::Validation(msg) => write!(f, "Validation failed: {msg}"),
            DomainError::BusinessRule(msg) => write!(f, "Business rule violation: {msg}"),
            DomainError::NotFound(msg) => write!(f, "Resource not found: {msg}"),
            DomainError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            DomainError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {msg}"),
            DomainError::ExternalDependency(msg) => write!(f, "External dependency error: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}

/// 配置相关错误
#[derive(Debug)]
pub enum ConfigError {
    /// 配置项缺失
    Missing(String),
    /// 配置格式错误
    InvalidFormat(String),
    /// 配置验证失败
    ValidationFailed(String),
    /// 配置文件未找到
    FileNotFound(String),
    /// 配置加载失败
    LoadFailed(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "Missing configuration: {key}"),
            ConfigError::InvalidFormat(msg) => write!(f, "Invalid configuration format: {msg}"),
            ConfigError::ValidationFailed(msg) => {
                write!(f, "Configuration validation failed: {msg}")
            }
            ConfigError::FileNotFound(path) => write!(f, "Configuration file not found: {path}"),
            ConfigError::LoadFailed(msg) => write!(f, "Failed to load configuration: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Git操作错误
#[derive(Debug)]
pub enum GitError {
    /// Git命令执行失败
    CommandFailed(String),
    /// 仓库未找到
    RepositoryNotFound(String),
    /// 分支未找到
    BranchNotFound(String),
    /// 提交未找到
    CommitNotFound(String),
    /// 工作区状态异常
    WorkingDirectoryDirty(String),
    /// 权限不足
    PermissionDenied(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::CommandFailed(msg) => write!(f, "Git command failed: {msg}"),
            GitError::RepositoryNotFound(path) => write!(f, "Git repository not found: {path}"),
            GitError::BranchNotFound(name) => write!(f, "Git branch not found: {name}"),
            GitError::CommitNotFound(hash) => write!(f, "Git commit not found: {hash}"),
            GitError::WorkingDirectoryDirty(msg) => {
                write!(f, "Working directory is dirty: {msg}")
            }
            GitError::PermissionDenied(msg) => write!(f, "Git permission denied: {msg}"),
        }
    }
}

impl std::error::Error for GitError {}

/// AI服务错误
#[derive(Debug)]
pub enum AiError {
    /// API调用失败
    ApiCallFailed(String),
    /// 模型不可用
    ModelUnavailable(String),
    /// 请求超时
    RequestTimeout(u64),
    /// 响应解析失败
    ResponseParseFailed(String),
    /// 认证失败
    AuthenticationFailed(String),
    /// 配额不足
    QuotaExceeded(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::ApiCallFailed(msg) => write!(f, "AI API call failed: {msg}"),
            AiError::ModelUnavailable(model) => write!(f, "AI model unavailable: {model}"),
            AiError::RequestTimeout(seconds) => {
                write!(f, "AI request timeout: {seconds} seconds")
            }
            AiError::ResponseParseFailed(msg) => write!(f, "AI response parse failed: {msg}"),
            AiError::AuthenticationFailed(msg) => write!(f, "AI authentication failed: {msg}"),
            AiError::QuotaExceeded(msg) => write!(f, "AI quota exceeded: {msg}"),
        }
    }
}

impl std::error::Error for AiError {}

/// 缓存错误
#[derive(Debug)]
pub enum CacheError {
    /// 缓存键未找到
    KeyNotFound(String),
    /// 缓存已满
    CacheFull,
    /// 序列化失败
    SerializationFailed(String),
    /// 反序列化失败
    DeserializationFailed(String),
    /// 存储错误
    StorageError(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::KeyNotFound(key) => write!(f, "Cache key not found: {key}"),
            CacheError::CacheFull => write!(f, "Cache is full"),
            CacheError::SerializationFailed(msg) => {
                write!(f, "Cache serialization failed: {msg}")
            }
            CacheError::DeserializationFailed(msg) => {
                write!(f, "Cache deserialization failed: {msg}")
            }
            CacheError::StorageError(msg) => write!(f, "Cache storage error: {msg}"),
        }
    }
}

impl std::error::Error for CacheError {}

/// 扫描错误
#[derive(Debug)]
pub enum ScanError {
    /// 扫描工具未找到
    ToolNotFound(String),
    /// 扫描执行失败
    ScanExecutionFailed(String),
    /// 规则加载失败
    RuleLoadFailed(String),
    /// 结果解析失败
    ResultParseFailed(String),
    /// 文件访问失败
    FileAccessFailed(String),
    /// 超时
    Timeout(u64),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::ToolNotFound(name) => write!(f, "Scan tool not found: {name}"),
            ScanError::ScanExecutionFailed(msg) => write!(f, "Scan execution failed: {msg}"),
            ScanError::RuleLoadFailed(msg) => write!(f, "Rule load failed: {msg}"),
            ScanError::ResultParseFailed(msg) => write!(f, "Result parse failed: {msg}"),
            ScanError::FileAccessFailed(msg) => write!(f, "File access failed: {msg}"),
            ScanError::Timeout(seconds) => write!(f, "Scan timeout: {seconds} seconds"),
        }
    }
}

impl std::error::Error for ScanError {}

/// 命令错误
#[derive(Debug)]
pub enum CommandError {
    /// 命令未找到
    CommandNotFound(String),
    /// 参数验证失败
    ArgumentValidationFailed(String),
    /// 命令执行失败
    ExecutionFailed(String),
    /// 权限不足
    InsufficientPermissions(String),
    /// 超时
    Timeout(u64),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::CommandNotFound(name) => write!(f, "Command not found: {name}"),
            CommandError::ArgumentValidationFailed(msg) => {
                write!(f, "Argument validation failed: {msg}")
            }
            CommandError::ExecutionFailed(msg) => write!(f, "Command execution failed: {msg}"),
            CommandError::InsufficientPermissions(msg) => {
                write!(f, "Insufficient permissions: {msg}")
            }
            CommandError::Timeout(seconds) => write!(f, "Command timeout: {seconds} seconds"),
        }
    }
}

impl std::error::Error for CommandError {}

/// 通用的结果类型别名
/// 领域层统一结果类型
pub type DomainResult<T> = Result<T, DomainError>;
/// 配置相关结果类型
pub type ConfigResult<T> = Result<T, ConfigError>;
/// Git 相关结果类型
pub type GitResult<T> = Result<T, GitError>;
/// AI 相关结果类型
pub type AiResult<T> = Result<T, AiError>;
/// 缓存相关结果类型
pub type CacheResult<T> = Result<T, CacheError>;
/// 安全扫描相关结果类型
pub type ScanResult<T> = Result<T, ScanError>;
/// 外部命令相关结果类型
pub type CommandResult<T> = Result<T, CommandError>;
