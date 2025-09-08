// GitAI 错误处理模块
//
// 提供用户友好的错误信息和详细的日志记录

use log::{debug, error, warn};
use std::fmt;

/// GitAI 统一错误类型
#[derive(Debug)]
pub enum GitAIError {
    /// 配置错误
    Config(ConfigError),
    /// Git 操作错误
    Git(GitError),
    /// 文件系统错误
    FileSystem(FileSystemError),
    /// 网络错误
    Network(NetworkError),
    /// 扫描工具错误
    ScanTool(ScanError),
    /// AI 服务错误
    AiService(AiError),
    /// 解析错误
    Parse(ParseError),
    /// 依赖缺失
    MissingDependency(String),
    /// 用户取消
    UserCancelled,
    /// 容器错误
    Container(ContainerError),
    /// 更新错误
    Update(UpdateError),
    /// MCP 错误
    Mcp(McpError),
    /// 未知错误
    Unknown(String),
}

/// 配置错误详情
#[derive(Debug)]
pub enum ConfigError {
    /// 配置文件未找到
    FileNotFound(String),
    /// 配置格式错误
    InvalidFormat(String),
    /// 配置验证失败
    ValidationFailed(String),
    /// 配置加载失败
    LoadFailed(String),
    /// 配置项缺失
    Missing(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "配置文件未找到: {}", path),
            ConfigError::InvalidFormat(msg) => write!(f, "配置格式错误: {}", msg),
            ConfigError::ValidationFailed(msg) => write!(f, "配置验证失败: {}", msg),
            ConfigError::LoadFailed(msg) => write!(f, "配置加载失败: {}", msg),
            ConfigError::Missing(key) => write!(f, "配置项缺失: {}", key),
        }
    }
}

/// Git 操作错误详情
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
            GitError::CommandFailed(msg) => write!(f, "Git命令执行失败: {}", msg),
            GitError::RepositoryNotFound(path) => write!(f, "仓库未找到: {}", path),
            GitError::BranchNotFound(branch) => write!(f, "分支未找到: {}", branch),
            GitError::CommitNotFound(commit) => write!(f, "提交未找到: {}", commit),
            GitError::WorkingDirectoryDirty(msg) => write!(f, "工作区状态异常: {}", msg),
            GitError::PermissionDenied(msg) => write!(f, "权限不足: {}", msg),
        }
    }
}

/// 文件系统错误详情
#[derive(Debug)]
pub enum FileSystemError {
    /// 文件未找到
    FileNotFound(String),
    /// 目录遍历失败
    DirectoryTraversal(String),
    /// 权限不足
    PermissionDenied(String),
    /// IO 操作失败
    Io(String),
    /// 路径无效
    InvalidPath(String),
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::FileNotFound(path) => write!(f, "文件未找到: {}", path),
            FileSystemError::DirectoryTraversal(msg) => write!(f, "目录遍历失败: {}", msg),
            FileSystemError::PermissionDenied(msg) => write!(f, "权限不足: {}", msg),
            FileSystemError::Io(msg) => write!(f, "IO 操作失败: {}", msg),
            FileSystemError::InvalidPath(path) => write!(f, "路径无效: {}", path),
        }
    }
}

/// 网络错误详情
#[derive(Debug)]
pub enum NetworkError {
    /// 连接失败
    ConnectionFailed(String),
    /// 请求超时
    Timeout(String),
    /// DNS 解析失败
    DnsFailed(String),
    /// SSL 错误
    Ssl(String),
    /// HTTP 错误
    Http(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionFailed(msg) => write!(f, "连接失败: {}", msg),
            NetworkError::Timeout(msg) => write!(f, "请求超时: {}", msg),
            NetworkError::DnsFailed(msg) => write!(f, "DNS 解析失败: {}", msg),
            NetworkError::Ssl(msg) => write!(f, "SSL 错误: {}", msg),
            NetworkError::Http(msg) => write!(f, "HTTP 错误: {}", msg),
        }
    }
}

/// 解析错误详情
#[derive(Debug)]
pub enum ParseError {
    /// JSON 解析失败
    Json(String),
    /// TOML 解析失败
    Toml(String),
    /// 其他格式解析失败
    Other(String, String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Json(msg) => write!(f, "JSON 解析失败: {}", msg),
            ParseError::Toml(msg) => write!(f, "TOML 解析失败: {}", msg),
            ParseError::Other(format, msg) => write!(f, "{} 解析失败: {}", format, msg),
        }
    }
}

/// 更新错误详情
#[derive(Debug)]
pub enum UpdateError {
    /// 网络错误
    Network(String),
    /// IO 错误
    Io(String),
    /// 配置错误
    Config(String),
    /// 下载错误
    Download(String),
    /// 解析错误
    Parse(String),
    /// 版本错误
    Version(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Network(msg) => write!(f, "网络错误: {}", msg),
            UpdateError::Io(msg) => write!(f, "IO错误: {}", msg),
            UpdateError::Config(msg) => write!(f, "配置错误: {}", msg),
            UpdateError::Download(msg) => write!(f, "下载错误: {}", msg),
            UpdateError::Parse(msg) => write!(f, "解析错误: {}", msg),
            UpdateError::Version(msg) => write!(f, "版本错误: {}", msg),
        }
    }
}

/// 扫描工具错误详情
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
            ScanError::ToolNotFound(name) => write!(f, "扫描工具未找到: {}", name),
            ScanError::ScanExecutionFailed(msg) => write!(f, "扫描执行失败: {}", msg),
            ScanError::RuleLoadFailed(msg) => write!(f, "规则加载失败: {}", msg),
            ScanError::ResultParseFailed(msg) => write!(f, "结果解析失败: {}", msg),
            ScanError::FileAccessFailed(msg) => write!(f, "文件访问失败: {}", msg),
            ScanError::Timeout(seconds) => write!(f, "扫描超时: {} 秒", seconds),
        }
    }
}

/// AI 服务错误详情
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
            AiError::ApiCallFailed(msg) => write!(f, "AI API调用失败: {}", msg),
            AiError::ModelUnavailable(model) => write!(f, "AI模型不可用: {}", model),
            AiError::RequestTimeout(seconds) => write!(f, "AI请求超时: {} 秒", seconds),
            AiError::ResponseParseFailed(msg) => write!(f, "AI响应解析失败: {}", msg),
            AiError::AuthenticationFailed(msg) => write!(f, "AI认证失败: {}", msg),
            AiError::QuotaExceeded(msg) => write!(f, "AI配额不足: {}", msg),
        }
    }
}

/// 容器错误详情
#[derive(Debug)]
pub enum ContainerError {
    /// 服务未注册
    ServiceNotRegistered {
        type_name: String,
        available_services: Vec<String>,
        suggestion: Option<String>,
    },
    /// 循环依赖
    CircularDependency {
        service_chain: Vec<String>,
        cycle_point: String,
    },
    /// 服务创建失败
    ServiceCreationFailed {
        service_type: String,
        reason: String,
    },
    /// 类型转换失败
    TypeCastFailed {
        expected: String,
        actual: String,
    },
    /// 作用域错误
    ScopeError {
        operation: String,
        reason: String,
    },
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::ServiceNotRegistered { type_name, available_services, suggestion } => {
                write!(f, "服务未注册: {}", type_name)?;
                if !available_services.is_empty() {
                    write!(f, "，可用服务: {:?}", available_services)?;
                }
                if let Some(suggestion) = suggestion {
                    write!(f, "，建议: {}", suggestion)?;
                }
                Ok(())
            }
            ContainerError::CircularDependency { service_chain, cycle_point } => {
                write!(f, "循环依赖检测: 依赖链 {:?}，循环点 {}", service_chain, cycle_point)
            }
            ContainerError::ServiceCreationFailed { service_type, reason } => {
                write!(f, "服务创建失败: {}，原因: {}", service_type, reason)
            }
            ContainerError::TypeCastFailed { expected, actual } => {
                write!(f, "类型转换失败: 期望 {}，实际 {}", expected, actual)
            }
            ContainerError::ScopeError { operation, reason } => {
                write!(f, "作用域错误: 操作 {}，原因: {}", operation, reason)
            }
        }
    }
}

/// MCP 错误详情
#[derive(Debug)]
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

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::InvalidParameters(msg) => write!(f, "参数验证错误: {}", msg),
            McpError::ExecutionFailed(msg) => write!(f, "服务执行错误: {}", msg),
            McpError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
            McpError::FileOperationError(msg) => write!(f, "文件操作错误: {}", msg),
            McpError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            McpError::ExternalToolError(msg) => write!(f, "外部工具错误: {}", msg),
            McpError::PermissionError(msg) => write!(f, "权限错误: {}", msg),
            McpError::TimeoutError(msg) => write!(f, "超时错误: {}", msg),
            McpError::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl GitAIError {
    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            GitAIError::Config(msg) => {
                format!("❌ 配置错误: {msg}\n💡 提示: 请检查 ~/.config/gitai/config.toml 配置文件")
            }
            GitAIError::Git(msg) => {
                format!("❌ Git 操作失败: {msg}\n💡 提示: 确保您在 Git 仓库中，并且有相应的权限")
            }
            GitAIError::FileSystem(msg) => {
                format!("❌ 文件系统错误: {msg}\n💡 提示: 检查文件路径和权限设置")
            }
            GitAIError::Network(msg) => {
                format!("❌ 网络连接错误: {msg}\n💡 提示: 检查网络连接和代理设置")
            }
            GitAIError::ScanTool(msg) => {
                format!(
                    "❌ 扫描工具错误: {msg}\n💡 提示: 使用 'gitai scan --auto-install' 安装所需工具"
                )
            }
            GitAIError::AiService(msg) => {
                format!("❌ AI 服务错误: {msg}\n💡 提示: 检查 AI 服务配置和 API 密钥")
            }
            GitAIError::Parse(msg) => {
                format!("❌ 解析错误: {msg}\n💡 提示: 数据格式可能不正确，请检查输入")
            }
            GitAIError::MissingDependency(dep) => {
                format!("❌ 缺少依赖: {dep}\n💡 提示: 请先安装所需的依赖工具")
            }
            GitAIError::Container(msg) => {
                format!("❌ 容器错误: {msg}\n💡 提示: 检查服务注册和依赖注入配置")
            }
            GitAIError::Update(msg) => {
                format!("❌ 更新错误: {msg}\n💡 提示: 检查网络连接和更新权限")
            }
            GitAIError::Mcp(msg) => {
                format!("❌ MCP 错误: {msg}\n💡 提示: 检查 MCP 服务配置和参数")
            }
            GitAIError::UserCancelled => "⚠️ 操作已取消".to_string(),
            GitAIError::Unknown(msg) => {
                format!("❌ 未知错误: {msg}\n💡 提示: 请查看日志文件获取更多信息")
            }
        }
    }

    /// 记录错误到日志
    pub fn log(&self) {
        match self {
            GitAIError::Config(_) 
            | GitAIError::MissingDependency(_) 
            | GitAIError::Container(_)
            | GitAIError::Update(_)
            | GitAIError::Mcp(_) => {
                error!("{self:?}");
            }
            GitAIError::UserCancelled => {
                debug!("User cancelled operation");
            }
            _ => {
                warn!("{self:?}");
            }
        }
    }
}

impl fmt::Display for GitAIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for GitAIError {}

// 便捷转换实现
impl From<std::io::Error> for GitAIError {
    fn from(err: std::io::Error) -> Self {
        GitAIError::FileSystem(FileSystemError::Io(err.to_string()))
    }
}

impl From<serde_json::Error> for GitAIError {
    fn from(err: serde_json::Error) -> Self {
        GitAIError::Parse(ParseError::Json(err.to_string()))
    }
}

impl From<reqwest::Error> for GitAIError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GitAIError::Network(NetworkError::Timeout("请求超时".to_string()))
        } else if err.is_connect() {
            GitAIError::Network(NetworkError::ConnectionFailed("连接失败".to_string()))
        } else {
            GitAIError::Network(NetworkError::Http(err.to_string()))
        }
    }
}

impl From<toml::de::Error> for GitAIError {
    fn from(err: toml::de::Error) -> Self {
        GitAIError::Parse(ParseError::Toml(err.to_string()))
    }
}

impl From<toml::ser::Error> for GitAIError {
    fn from(err: toml::ser::Error) -> Self {
        GitAIError::Parse(ParseError::Toml(err.to_string()))
    }
}

impl From<walkdir::Error> for GitAIError {
    fn from(err: walkdir::Error) -> Self {
        GitAIError::FileSystem(FileSystemError::DirectoryTraversal(err.to_string()))
    }
}

// 支持从anyhow::Error转换
impl From<anyhow::Error> for GitAIError {
    fn from(err: anyhow::Error) -> Self {
        GitAIError::Unknown(err.to_string())
    }
}

// 支持从字符串转换
impl From<&str> for GitAIError {
    fn from(msg: &str) -> Self {
        GitAIError::Unknown(msg.to_string())
    }
}

impl From<String> for GitAIError {
    fn from(msg: String) -> Self {
        GitAIError::Unknown(msg)
    }
}

// 支持从Box<dyn Error>转换
impl From<Box<dyn std::error::Error + Send + Sync>> for GitAIError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        GitAIError::Unknown(err.to_string())
    }
}

// 子错误类型转换
impl From<ConfigError> for GitAIError {
    fn from(err: ConfigError) -> Self {
        GitAIError::Config(err)
    }
}

impl From<GitError> for GitAIError {
    fn from(err: GitError) -> Self {
        GitAIError::Git(err)
    }
}

impl From<FileSystemError> for GitAIError {
    fn from(err: FileSystemError) -> Self {
        GitAIError::FileSystem(err)
    }
}

impl From<NetworkError> for GitAIError {
    fn from(err: NetworkError) -> Self {
        GitAIError::Network(err)
    }
}

impl From<ScanError> for GitAIError {
    fn from(err: ScanError) -> Self {
        GitAIError::ScanTool(err)
    }
}

impl From<AiError> for GitAIError {
    fn from(err: AiError) -> Self {
        GitAIError::AiService(err)
    }
}

impl From<ContainerError> for GitAIError {
    fn from(err: ContainerError) -> Self {
        GitAIError::Container(err)
    }
}

impl From<UpdateError> for GitAIError {
    fn from(err: UpdateError) -> Self {
        GitAIError::Update(err)
    }
}

impl From<McpError> for GitAIError {
    fn from(err: McpError) -> Self {
        GitAIError::Mcp(err)
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, GitAIError>;

/// 错误上下文扩展 trait
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn context(self, msg: &str) -> Result<T>;

    /// 添加用户友好的提示
    fn with_hint(self, hint: &str) -> Result<T>;
    
    /// 添加文件路径上下文
    fn with_file(self, file: &str) -> Result<T>;
    
    /// 添加函数上下文
    fn with_function(self, function: &str) -> Result<T>;
    
    /// 添加行号上下文
    fn with_line(self, line: u32) -> Result<T>;
    
    /// 添加自定义键值对上下文
    fn with_context(self, key: &str, value: &str) -> Result<T>;
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContextInfo {
    /// 文件路径
    pub file: Option<String>,
    /// 函数名
    pub function: Option<String>,
    /// 行号
    pub line: Option<u32>,
    /// 自定义上下文
    pub custom_context: std::collections::HashMap<String, String>,
    /// 时间戳
    pub timestamp: std::time::SystemTime,
}

impl Default for ErrorContextInfo {
    fn default() -> Self {
        Self {
            file: None,
            function: None,
            line: None,
            custom_context: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

impl ErrorContextInfo {
    /// 创建新的错误上下文
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            ..Default::default()
        }
    }
    
    /// 设置文件路径
    pub fn file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }
    
    /// 设置函数名
    pub fn function(mut self, function: &str) -> Self {
        self.function = Some(function.to_string());
        self
    }
    
    /// 设置行号
    pub fn line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }
    
    /// 添加自定义上下文
    pub fn add_context(mut self, key: &str, value: &str) -> Self {
        self.custom_context.insert(key.to_string(), value.to_string());
        self
    }
    
    /// 格式化上下文信息
    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref file) = self.file {
            parts.push(format!("文件: {}", file));
        }
        
        if let Some(ref function) = self.function {
            parts.push(format!("函数: {}", function));
        }
        
        if let Some(line) = self.line {
            parts.push(format!("行号: {}", line));
        }
        
        for (key, value) in &self.custom_context {
            parts.push(format!("{}: {}", key, value));
        }
        
        if parts.is_empty() {
            String::new()
        } else {
            format!("[{}]", parts.join(", "))
        }
    }
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let error = GitAIError::Unknown(format!("{msg}: {e}"));
            error.log();
            error
        })
    }

    fn with_hint(self, hint: &str) -> Result<T> {
        self.map_err(|e| {
            let mut error_msg = e.to_string();
            error_msg.push_str(&format!("\n💡 {hint}"));
            GitAIError::Unknown(error_msg)
        })
    }
    
    fn with_file(self, file: &str) -> Result<T> {
        self.map_err(|e| {
            let error_msg = format!("{} [文件: {}]", e, file);
            GitAIError::Unknown(error_msg)
        })
    }
    
    fn with_function(self, function: &str) -> Result<T> {
        self.map_err(|e| {
            let error_msg = format!("{} [函数: {}]", e, function);
            GitAIError::Unknown(error_msg)
        })
    }
    
    fn with_line(self, line: u32) -> Result<T> {
        self.map_err(|e| {
            let error_msg = format!("{} [行号: {}]", e, line);
            GitAIError::Unknown(error_msg)
        })
    }
    
    fn with_context(self, key: &str, value: &str) -> Result<T> {
        self.map_err(|e| {
            let error_msg = format!("{} [{}: {}]", e, key, value);
            GitAIError::Unknown(error_msg)
        })
    }
}

/// 错误上下文宏
#[macro_export]
macro_rules! with_error_context {
    ($result:expr, $file:expr, $function:expr, $line:expr) => {
        $result
            .with_file($file)
            .with_function($function)
            .with_line($line)
    };
}

/// 错误上下文宏（带自定义上下文）
#[macro_export]
macro_rules! with_error_context_and {
    ($result:expr, $file:expr, $function:expr, $line:expr, $($key:expr => $value:expr),*) => {
        {
            let mut result = $result
                .with_file($file)
                .with_function($function)
                .with_line($line);
            $(
                result = result.with_context($key, $value);
            )*
            result
        }
    };
}

/// 日志辅助宏
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        log::error!("❌ {}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::error!("❌ {}", format!($fmt, $($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        log::warn!("⚠️ {}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::warn!("⚠️ {}", format!($fmt, $($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        log::info!("ℹ️ {}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::info!("ℹ️ {}", format!($fmt, $($arg)*));
    };
}

#[macro_export]
macro_rules! log_success {
    ($msg:expr) => {
        log::info!("✅ {}", $msg);
    };
    ($fmt:expr, $($arg:tt)*) => {
        log::info!("✅ {}", format!($fmt, $($arg)*));
    };
}
