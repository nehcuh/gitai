// GitAI 错误处理模块
//
// 提供用户友好的错误信息和详细的日志记录

use log::{debug, error, warn};
use std::fmt;

/// GitAI 统一错误类型
#[derive(Debug)]
pub enum GitAIError {
    /// 配置错误
    Config(String),
    /// Git 操作错误
    Git(String),
    /// 文件系统错误
    FileSystem(String),
    /// 网络错误
    Network(String),
    /// 扫描工具错误
    ScanTool(String),
    /// AI 服务错误
    AiService(String),
    /// 解析错误
    Parse(String),
    /// 依赖缺失
    MissingDependency(String),
    /// 用户取消
    UserCancelled,
    /// 未知错误
    Unknown(String),
}

impl GitAIError {
    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            GitAIError::Config(msg) => {
                format!(
                    "❌ 配置错误: {}\n💡 提示: 请检查 ~/.config/gitai/config.toml 配置文件",
                    msg
                )
            }
            GitAIError::Git(msg) => {
                format!(
                    "❌ Git 操作失败: {}\n💡 提示: 确保您在 Git 仓库中，并且有相应的权限",
                    msg
                )
            }
            GitAIError::FileSystem(msg) => {
                format!("❌ 文件系统错误: {}\n💡 提示: 检查文件路径和权限设置", msg)
            }
            GitAIError::Network(msg) => {
                format!("❌ 网络连接错误: {}\n💡 提示: 检查网络连接和代理设置", msg)
            }
            GitAIError::ScanTool(msg) => {
                format!(
                    "❌ 扫描工具错误: {}\n💡 提示: 使用 'gitai scan --auto-install' 安装所需工具",
                    msg
                )
            }
            GitAIError::AiService(msg) => {
                format!(
                    "❌ AI 服务错误: {}\n💡 提示: 检查 AI 服务配置和 API 密钥",
                    msg
                )
            }
            GitAIError::Parse(msg) => {
                format!(
                    "❌ 解析错误: {}\n💡 提示: 数据格式可能不正确，请检查输入",
                    msg
                )
            }
            GitAIError::MissingDependency(dep) => {
                format!("❌ 缺少依赖: {}\n💡 提示: 请先安装所需的依赖工具", dep)
            }
            GitAIError::UserCancelled => "⚠️ 操作已取消".to_string(),
            GitAIError::Unknown(msg) => {
                format!("❌ 未知错误: {}\n💡 提示: 请查看日志文件获取更多信息", msg)
            }
        }
    }

    /// 记录错误到日志
    pub fn log(&self) {
        match self {
            GitAIError::Config(_) | GitAIError::MissingDependency(_) => {
                error!("{:?}", self);
            }
            GitAIError::UserCancelled => {
                debug!("User cancelled operation");
            }
            _ => {
                warn!("{:?}", self);
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
        GitAIError::FileSystem(err.to_string())
    }
}

impl From<serde_json::Error> for GitAIError {
    fn from(err: serde_json::Error) -> Self {
        GitAIError::Parse(format!("JSON 解析失败: {}", err))
    }
}

impl From<reqwest::Error> for GitAIError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GitAIError::Network("请求超时".to_string())
        } else if err.is_connect() {
            GitAIError::Network("连接失败".to_string())
        } else {
            GitAIError::Network(err.to_string())
        }
    }
}

impl From<toml::de::Error> for GitAIError {
    fn from(err: toml::de::Error) -> Self {
        GitAIError::Parse(format!("TOML 解析失败: {}", err))
    }
}

impl From<toml::ser::Error> for GitAIError {
    fn from(err: toml::ser::Error) -> Self {
        GitAIError::Parse(format!("TOML 序列化失败: {}", err))
    }
}

impl From<walkdir::Error> for GitAIError {
    fn from(err: walkdir::Error) -> Self {
        GitAIError::FileSystem(format!("目录遍历失败: {}", err))
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

/// 结果类型别名
pub type Result<T> = std::result::Result<T, GitAIError>;

/// 错误上下文扩展 trait
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn context(self, msg: &str) -> Result<T>;

    /// 添加用户友好的提示
    fn with_hint(self, hint: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let error = GitAIError::Unknown(format!("{}: {}", msg, e));
            error.log();
            error
        })
    }

    fn with_hint(self, hint: &str) -> Result<T> {
        self.map_err(|e| {
            let mut error_msg = e.to_string();
            error_msg.push_str(&format!("\n💡 {}", hint));
            GitAIError::Unknown(error_msg)
        })
    }
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
