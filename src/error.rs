// GitAI 错误处理模块
//
// 提供用户友好的错误信息和详细的日志记录

use std::fmt;
use log::{error, warn, debug};

/// GitAI 统一错误类型
#[derive(Debug)]
pub enum GitAiError {
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

impl GitAiError {
    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            GitAiError::Config(msg) => {
                format!("❌ 配置错误: {}\n💡 提示: 请检查 ~/.config/gitai/config.toml 配置文件", msg)
            }
            GitAiError::Git(msg) => {
                format!("❌ Git 操作失败: {}\n💡 提示: 确保您在 Git 仓库中，并且有相应的权限", msg)
            }
            GitAiError::FileSystem(msg) => {
                format!("❌ 文件系统错误: {}\n💡 提示: 检查文件路径和权限设置", msg)
            }
            GitAiError::Network(msg) => {
                format!("❌ 网络连接错误: {}\n💡 提示: 检查网络连接和代理设置", msg)
            }
            GitAiError::ScanTool(msg) => {
                format!("❌ 扫描工具错误: {}\n💡 提示: 使用 'gitai scan --auto-install' 安装所需工具", msg)
            }
            GitAiError::AiService(msg) => {
                format!("❌ AI 服务错误: {}\n💡 提示: 检查 AI 服务配置和 API 密钥", msg)
            }
            GitAiError::Parse(msg) => {
                format!("❌ 解析错误: {}\n💡 提示: 数据格式可能不正确，请检查输入", msg)
            }
            GitAiError::MissingDependency(dep) => {
                format!("❌ 缺少依赖: {}\n💡 提示: 请先安装所需的依赖工具", dep)
            }
            GitAiError::UserCancelled => {
                "⚠️ 操作已取消".to_string()
            }
            GitAiError::Unknown(msg) => {
                format!("❌ 未知错误: {}\n💡 提示: 请查看日志文件获取更多信息", msg)
            }
        }
    }
    
    /// 记录错误到日志
    pub fn log(&self) {
        match self {
            GitAiError::Config(_) | GitAiError::MissingDependency(_) => {
                error!("{:?}", self);
            }
            GitAiError::UserCancelled => {
                debug!("User cancelled operation");
            }
            _ => {
                warn!("{:?}", self);
            }
        }
    }
}

impl fmt::Display for GitAiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for GitAiError {}

// 便捷转换实现
impl From<std::io::Error> for GitAiError {
    fn from(err: std::io::Error) -> Self {
        GitAiError::FileSystem(err.to_string())
    }
}

impl From<serde_json::Error> for GitAiError {
    fn from(err: serde_json::Error) -> Self {
        GitAiError::Parse(format!("JSON 解析失败: {}", err))
    }
}

impl From<reqwest::Error> for GitAiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GitAiError::Network("请求超时".to_string())
        } else if err.is_connect() {
            GitAiError::Network("连接失败".to_string())
        } else {
            GitAiError::Network(err.to_string())
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, GitAiError>;

/// 错误上下文扩展 trait
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn context(self, msg: &str) -> Result<T>;
    
    /// 添加用户友好的提示
    fn with_hint(self, hint: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E> 
where
    E: std::error::Error + Send + Sync + 'static
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            let error = GitAiError::Unknown(format!("{}: {}", msg, e));
            error.log();
            error
        })
    }
    
    fn with_hint(self, hint: &str) -> Result<T> {
        self.map_err(|e| {
            let mut error_msg = e.to_string();
            error_msg.push_str(&format!("\n💡 {}", hint));
            GitAiError::Unknown(error_msg)
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
