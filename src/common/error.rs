use thiserror::Error;

/// 应用程序的统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误: {message}")]
    Config { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Git 操作错误: {message}")]
    Git { 
        message: String, 
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
        command: Option<String>,
    },

    #[error("AI 服务错误: {message}")]
    AI { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("AST-Grep 分析错误: {message}")]
    Analysis { 
        message: String,
        analysis_type: Option<String>,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("翻译服务错误: {message}")]
    Translation { 
        message: String,
        language: Option<String>,
    },

    #[error("DevOps 集成错误: {message}")]
    DevOps { 
        message: String,
        status_code: Option<u16>,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("文件操作错误: {message}")]
    IO { 
        message: String,
        path: Option<String>,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("网络错误: {message}")]
    Network { 
        message: String,
        url: Option<String>,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("CLI 参数错误: {message}")]
    CLI { 
        message: String,
        argument: Option<String>,
    },

    #[error("通用错误: {message}")]
    Generic { 
        message: String,
        category: Option<String>,
    },
}

impl AppError {
    // 配置错误构建器
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config { 
            message: msg.into(), 
            source: None 
        }
    }

    pub fn config_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Config { 
            message: msg.into(), 
            source: Some(Box::new(err)) 
        }
    }

    // Git 错误构建器
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git { 
            message: msg.into(), 
            exit_code: None,
            stdout: None,
            stderr: None,
            command: None,
        }
    }

    pub fn git_with_exit_code(msg: impl Into<String>, code: i32) -> Self {
        Self::Git { 
            message: msg.into(), 
            exit_code: Some(code),
            stdout: None,
            stderr: None,
            command: None,
        }
    }

    pub fn git_command_failed(
        cmd: impl Into<String>, 
        exit_code: Option<i32>, 
        stdout: impl Into<String>, 
        stderr: impl Into<String>
    ) -> Self {
        let cmd_str = cmd.into();
        let stderr_str = stderr.into();
        let stdout_str = stdout.into();
        
        let mut message = format!("Git 命令 '{}' 失败", cmd_str);
        if let Some(code) = exit_code {
            message.push_str(&format!(" (退出码: {})", code));
        }
        if !stderr_str.is_empty() {
            message.push_str(&format!(" - {}", stderr_str));
        }

        Self::Git {
            message,
            exit_code,
            stdout: Some(stdout_str),
            stderr: Some(stderr_str),
            command: Some(cmd_str),
        }
    }

    // AI 错误构建器
    pub fn ai(msg: impl Into<String>) -> Self {
        Self::AI { 
            message: msg.into(), 
            source: None 
        }
    }

    pub fn ai_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::AI { 
            message: msg.into(), 
            source: Some(Box::new(err)) 
        }
    }

    // 分析错误构建器
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis { 
            message: msg.into(), 
            analysis_type: None,
            source: None,
        }
    }

    pub fn analysis_with_type(msg: impl Into<String>, analysis_type: impl Into<String>) -> Self {
        Self::Analysis { 
            message: msg.into(), 
            analysis_type: Some(analysis_type.into()),
            source: None,
        }
    }

    pub fn analysis_with_source(
        msg: impl Into<String>, 
        analysis_type: Option<String>,
        err: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Analysis { 
            message: msg.into(), 
            analysis_type,
            source: Some(Box::new(err)),
        }
    }

    // 翻译错误构建器
    pub fn translation(msg: impl Into<String>) -> Self {
        Self::Translation { 
            message: msg.into(), 
            language: None,
        }
    }

    pub fn translation_with_language(msg: impl Into<String>, lang: impl Into<String>) -> Self {
        Self::Translation { 
            message: msg.into(), 
            language: Some(lang.into()),
        }
    }

    // DevOps 错误构建器
    pub fn devops(msg: impl Into<String>) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: None,
            source: None,
        }
    }

    pub fn devops_with_status(msg: impl Into<String>, status_code: u16) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: Some(status_code),
            source: None,
        }
    }

    pub fn devops_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: None,
            source: Some(Box::new(err)),
        }
    }

    // IO 错误构建器
    pub fn io(msg: impl Into<String>) -> Self {
        Self::IO { 
            message: msg.into(), 
            path: None,
            source: None,
        }
    }

    pub fn io_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self {
        Self::IO { 
            message: msg.into(), 
            path: Some(path.into()),
            source: None,
        }
    }

    pub fn io_with_source(
        msg: impl Into<String>, 
        path: Option<String>,
        err: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::IO { 
            message: msg.into(), 
            path,
            source: Some(Box::new(err)),
        }
    }

    // 网络错误构建器
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network { 
            message: msg.into(), 
            url: None,
            source: None,
        }
    }

    pub fn network_with_url(msg: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Network { 
            message: msg.into(), 
            url: Some(url.into()),
            source: None,
        }
    }

    pub fn network_with_source(
        msg: impl Into<String>, 
        url: Option<String>,
        err: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Network { 
            message: msg.into(), 
            url,
            source: Some(Box::new(err)),
        }
    }

    // CLI 错误构建器
    pub fn cli(msg: impl Into<String>) -> Self {
        Self::CLI { 
            message: msg.into(), 
            argument: None,
        }
    }

    pub fn cli_with_argument(msg: impl Into<String>, arg: impl Into<String>) -> Self {
        Self::CLI { 
            message: msg.into(), 
            argument: Some(arg.into()),
        }
    }

    // 通用错误构建器
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic { 
            message: msg.into(), 
            category: None,
        }
    }

    pub fn generic_with_category(msg: impl Into<String>, category: impl Into<String>) -> Self {
        Self::Generic { 
            message: msg.into(), 
            category: Some(category.into()),
        }
    }

    /// 获取错误的退出码
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Git { exit_code: Some(code), .. } => *code,
            AppError::Config { .. } => 2,
            AppError::CLI { .. } => 2,
            AppError::DevOps { status_code: Some(code), .. } => (*code as i32).max(1),
            _ => 1,
        }
    }

    /// 检查是否为特定类型的错误
    pub fn is_config_error(&self) -> bool {
        matches!(self, AppError::Config { .. })
    }

    pub fn is_git_error(&self) -> bool {
        matches!(self, AppError::Git { .. })
    }

    pub fn is_ai_error(&self) -> bool {
        matches!(self, AppError::AI { .. })
    }

    pub fn is_network_error(&self) -> bool {
        matches!(self, AppError::Network { .. })
    }

    /// 获取详细的错误信息（包括上下文）
    pub fn detailed_message(&self) -> String {
        match self {
            AppError::Git { message, stdout, stderr, command, .. } => {
                let mut details = message.clone();
                if let Some(cmd) = command {
                    details.push_str(&format!("\n命令: {}", cmd));
                }
                if let Some(stderr) = stderr {
                    if !stderr.is_empty() {
                        details.push_str(&format!("\n错误输出: {}", stderr));
                    }
                }
                if let Some(stdout) = stdout {
                    if !stdout.is_empty() {
                        details.push_str(&format!("\n标准输出: {}", stdout));
                    }
                }
                details
            },
            AppError::IO { message, path, .. } => {
                let mut details = message.clone();
                if let Some(p) = path {
                    details.push_str(&format!(" (路径: {})", p));
                }
                details
            },
            AppError::Network { message, url, .. } => {
                let mut details = message.clone();
                if let Some(u) = url {
                    details.push_str(&format!(" (URL: {})", u));
                }
                details
            },
            AppError::Analysis { message, analysis_type, .. } => {
                let mut details = message.clone();
                if let Some(t) = analysis_type {
                    details.push_str(&format!(" (分析类型: {})", t));
                }
                details
            },
            AppError::Translation { message, language, .. } => {
                let mut details = message.clone();
                if let Some(lang) = language {
                    details.push_str(&format!(" (语言: {})", lang));
                }
                details
            },
            AppError::DevOps { message, status_code, .. } => {
                let mut details = message.clone();
                if let Some(code) = status_code {
                    details.push_str(&format!(" (状态码: {})", code));
                }
                details
            },
            AppError::CLI { message, argument, .. } => {
                let mut details = message.clone();
                if let Some(arg) = argument {
                    details.push_str(&format!(" (参数: {})", arg));
                }
                details
            },
            AppError::Generic { message, category, .. } => {
                let mut details = message.clone();
                if let Some(cat) = category {
                    details.push_str(&format!(" (类别: {})", cat));
                }
                details
            },
            _ => self.to_string(),
        }
    }
}

// 实现从常见错误类型的转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::io_with_source("文件操作失败".to_string(), None, err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        let url = err.url().map(|u| u.to_string());
        Self::network_with_source("网络请求失败".to_string(), url, err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::generic_with_category(
            format!("JSON 解析错误: {}", err),
            "serialization"
        )
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        Self::config_with_source("TOML 配置解析错误".to_string(), err)
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(err: toml::ser::Error) -> Self {
        Self::config_with_source("TOML 配置序列化错误".to_string(), err)
    }
}

// 从旧错误系统的转换支持
impl From<crate::errors::AppError> for AppError {
    fn from(old_err: crate::errors::AppError) -> Self {
        match old_err {
            crate::errors::AppError::Config(config_err) => {
                Self::config_with_source("配置错误".to_string(), config_err)
            },
            crate::errors::AppError::Git(git_err) => {
                match git_err {
                    crate::errors::GitError::CommandFailed { command, status_code, stdout, stderr } => {
                        Self::git_command_failed(command, status_code, stdout, stderr)
                    },
                    crate::errors::GitError::PassthroughFailed { command, status_code } => {
                        Self::git_with_exit_code(
                            format!("Git 透传命令失败: {}", command), 
                            status_code.unwrap_or(1)
                        )
                    },
                    crate::errors::GitError::NotARepository => {
                        Self::git("不是 Git 仓库".to_string())
                    },
                    crate::errors::GitError::NoStagedChanges => {
                        Self::git("没有暂存的变更".to_string())
                    },
                    crate::errors::GitError::DiffError(io_err) => {
                        Self::git_with_exit_code(format!("获取差异失败: {}", io_err), 1)
                    },
                    crate::errors::GitError::Other(msg) => {
                        Self::git(msg)
                    },
                }
            },
            crate::errors::AppError::AI(ai_err) => {
                Self::ai_with_source("AI 服务错误".to_string(), ai_err)
            },
            crate::errors::AppError::Analysis(analysis_err) => {
                Self::analysis_with_source("分析错误".to_string(), None, analysis_err)
            },
            crate::errors::AppError::DevOps(devops_err) => {
                Self::devops_with_source("DevOps 集成错误".to_string(), devops_err)
            },
            crate::errors::AppError::IO(context, io_err) => {
                Self::io_with_source(format!("IO 错误: {}", context), None, io_err)
            },
            crate::errors::AppError::Generic(msg) => {
                Self::generic(msg)
            },
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;