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
    /// Creates a configuration error with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::config("Missing configuration file");
    /// ```
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config { 
            message: msg.into(), 
            source: None 
        }
    }

    /// Creates a configuration error with a message and an associated source error.
    ///
    /// # Parameters
    ///
    /// - `msg`: The error message describing the configuration issue.
    /// - `err`: The underlying source error to associate with this configuration error.
    ///
    /// # Returns
    ///
    /// An `AppError::Config` variant containing the provided message and source error.
    pub fn config_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Config { 
            message: msg.into(), 
            source: Some(Box::new(err)) 
        }
    }

    // Git 错误构建器
    /// Creates a new Git-related error with the provided message.
    ///
    /// This constructor initializes a `Git` error variant without additional context such as exit code, command, or output streams. Use this for general Git errors where detailed information is not available.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::git("Failed to fetch repository");
    /// assert!(matches!(err, AppError::Git { .. }));
    /// ```
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git { 
            message: msg.into(), 
            exit_code: None,
            stdout: None,
            stderr: None,
            command: None,
        }
    }

    /// Creates a Git-related error with a message and exit code.
    ///
    /// This constructor is used when a Git operation fails and an exit code is available, but no additional context such as command, stdout, or stderr is provided.
    ///
    /// # Parameters
    /// - `msg`: Description of the Git error.
    /// - `code`: Exit code returned by the Git command.
    ///
    /// # Returns
    /// An `AppError` variant representing a Git error with the specified message and exit code.
    pub fn git_with_exit_code(msg: impl Into<String>, code: i32) -> Self {
        Self::Git { 
            message: msg.into(), 
            exit_code: Some(code),
            stdout: None,
            stderr: None,
            command: None,
        }
    }

    /// Constructs a `Git` error variant representing a failed Git command, including command details, exit code, and captured output.
    ///
    /// # Parameters
    /// - `cmd`: The Git command that was executed.
    /// - `exit_code`: Optional exit code returned by the command.
    /// - `stdout`: Standard output captured from the command.
    /// - `stderr`: Standard error captured from the command.
    ///
    /// # Returns
    /// An `AppError::Git` variant containing a descriptive message and contextual information about the failed command.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::git_command_failed("git status", Some(1), "", "fatal: not a git repository");
    /// assert!(matches!(err, AppError::Git { .. }));
    /// ```
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
    /// Creates an AI service error with the provided message.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::ai("AI model failed to respond");
    /// ```
    pub fn ai(msg: impl Into<String>) -> Self {
        Self::AI { 
            message: msg.into(), 
            source: None 
        }
    }

    /// Creates an AI service error with a message and an associated source error.
    ///
    /// # Examples
    ///
    /// ```
    /// let source = std::io::Error::new(std::io::ErrorKind::Other, "underlying error");
    /// let err = AppError::ai_with_source("AI request failed", source);
    /// match err {
    ///     AppError::AI { message, source: Some(_) } => assert_eq!(message, "AI request failed"),
    ///     _ => panic!("Expected AI error variant"),
    /// }
    /// ```
    pub fn ai_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::AI { 
            message: msg.into(), 
            source: Some(Box::new(err)) 
        }
    }

    // 分析错误构建器
    /// Creates an AST-Grep analysis error with the given message and no additional context.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::analysis("Failed to analyze syntax tree");
    /// match err {
    ///     AppError::Analysis { message, .. } => assert_eq!(message, "Failed to analyze syntax tree"),
    ///     _ => panic!("Expected Analysis variant"),
    /// }
    /// ```
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis { 
            message: msg.into(), 
            analysis_type: None,
            source: None,
        }
    }

    /// Creates an AST-Grep analysis error with a specified message and analysis type.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::analysis_with_type("Pattern not found", "syntax-check");
    /// ```
    pub fn analysis_with_type(msg: impl Into<String>, analysis_type: impl Into<String>) -> Self {
        Self::Analysis { 
            message: msg.into(), 
            analysis_type: Some(analysis_type.into()),
            source: None,
        }
    }

    /// Creates an AST-Grep analysis error with a message, optional analysis type, and a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// let source_err = std::io::Error::new(std::io::ErrorKind::Other, "parse failed");
    /// let err = AppError::analysis_with_source("AST analysis failed", Some("syntax".to_string()), source_err);
    /// ```
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
    /// Creates a translation service error with the given message and no specified language.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::translation("Translation service unavailable");
    /// match err {
    ///     AppError::Translation { message, language } => {
    ///         assert_eq!(message, "Translation service unavailable");
    ///         assert!(language.is_none());
    ///     }
    ///     _ => panic!("Expected Translation variant"),
    /// }
    /// ```
    pub fn translation(msg: impl Into<String>) -> Self {
        Self::Translation { 
            message: msg.into(), 
            language: None,
        }
    }

    /// Creates a translation service error with a specified message and language code.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::translation_with_language("Translation failed", "fr");
    /// ```
    pub fn translation_with_language(msg: impl Into<String>, lang: impl Into<String>) -> Self {
        Self::Translation { 
            message: msg.into(), 
            language: Some(lang.into()),
        }
    }

    // DevOps 错误构建器
    /// Creates a DevOps integration error with the given message.
    ///
    /// The error will not include a status code or source error.
    pub fn devops(msg: impl Into<String>) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: None,
            source: None,
        }
    }

    /// Creates a DevOps error with a message and an associated status code.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::devops_with_status("Deployment failed", 502);
    /// if let AppError::DevOps { status_code: Some(code), .. } = err {
    ///     assert_eq!(code, 502);
    /// }
    /// ```
    pub fn devops_with_status(msg: impl Into<String>, status_code: u16) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: Some(status_code),
            source: None,
        }
    }

    /// Creates a DevOps integration error with a message and a source error.
    ///
    /// The status code is set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let source = std::io::Error::new(std::io::ErrorKind::Other, "connection failed");
    /// let err = AppError::devops_with_source("DevOps API error", source);
    /// if let AppError::DevOps { message, source, .. } = err {
    ///     assert_eq!(message, "DevOps API error");
    ///     assert!(source.is_some());
    /// }
    /// ```
    pub fn devops_with_source(msg: impl Into<String>, err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::DevOps { 
            message: msg.into(), 
            status_code: None,
            source: Some(Box::new(err)),
        }
    }

    // IO 错误构建器
    /// Creates a file I/O error with the given message.
    ///
    /// The error will not include a file path or source error.
    pub fn io(msg: impl Into<String>) -> Self {
        Self::IO { 
            message: msg.into(), 
            path: None,
            source: None,
        }
    }

    /// Creates an `IO` error variant with a message and associated file path.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::io_with_path("Failed to read file", "/tmp/data.txt");
    /// assert!(matches!(err, AppError::IO { .. }));
    /// ```
    pub fn io_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self {
        Self::IO { 
            message: msg.into(), 
            path: Some(path.into()),
            source: None,
        }
    }

    /// Creates an `IO` error variant with a message, optional file path, and a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::AppError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    /// let app_err = AppError::io_with_source("Failed to read file", Some("data.txt".to_string()), io_err);
    /// ```
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
    /// Creates a network error with the specified message and no associated URL or source error.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::network("Failed to connect to server");
    /// assert!(matches!(err, AppError::Network { .. }));
    /// ```
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network { 
            message: msg.into(), 
            url: None,
            source: None,
        }
    }

    /// Creates a network error with a message and associated URL.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::network_with_url("Failed to connect", "https://example.com");
    /// assert!(matches!(err, AppError::Network { .. }));
    /// ```
    pub fn network_with_url(msg: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Network { 
            message: msg.into(), 
            url: Some(url.into()),
            source: None,
        }
    }

    /// Creates a network error with a message, optional URL, and a source error.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = std::io::Error::new(std::io::ErrorKind::Other, "connection failed");
    /// let app_err = AppError::network_with_source("Failed to reach server", Some("https://api.example.com".to_string()), err);
    /// ```
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
    /// Creates a CLI argument error with the given message.
    ///
    /// Use this for errors related to command-line arguments where no specific argument needs to be referenced.
    pub fn cli(msg: impl Into<String>) -> Self {
        Self::CLI { 
            message: msg.into(), 
            argument: None,
        }
    }

    /// Creates a CLI error with a message and an associated argument.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::cli_with_argument("Invalid value", "--foo");
    /// assert!(matches!(err, AppError::CLI { .. }));
    /// ```
    pub fn cli_with_argument(msg: impl Into<String>, arg: impl Into<String>) -> Self {
        Self::CLI { 
            message: msg.into(), 
            argument: Some(arg.into()),
        }
    }

    // 通用错误构建器
    /// Creates a generic application error with the provided message.
    ///
    /// Use this for errors that do not fit into a specific category.
    #[inline]
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic { 
            message: msg.into(), 
            category: None,
        }
    }

    /// Creates a generic error with a specified message and category.
    ///
    /// # Parameters
    ///
    /// - `msg`: The error message.
    /// - `category`: The category describing the type of generic error.
    ///
    /// # Returns
    ///
    /// An `AppError::Generic` variant containing the provided message and category.
    pub fn generic_with_category(msg: impl Into<String>, category: impl Into<String>) -> Self {
        Self::Generic { 
            message: msg.into(), 
            category: Some(category.into()),
        }
    }

    /// Returns the appropriate process exit code for the error.
    ///
    /// The exit code is determined based on the error variant and its context. Git and DevOps errors use their respective codes if available; configuration and CLI errors return 2; all other errors return 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::Config { message: "Invalid config".to_string(), source: None };
    /// assert_eq!(err.exit_code(), 2);
    /// ```
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Git { exit_code: Some(code), .. } => *code,
            AppError::Config { .. } => 2,
            AppError::CLI { .. } => 2,
            AppError::DevOps { status_code: Some(code), .. } => (*code as i32).max(1),
            _ => 1,
        }
    }

    /// Returns `true` if the error is a configuration error (`AppError::Config`).
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::config("Missing config file");
    /// assert!(err.is_config_error());
    /// ```
    pub fn is_config_error(&self) -> bool {
        matches!(self, AppError::Config { .. })
    }

    /// Returns `true` if the error is a Git-related error variant.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::git("Failed to fetch repository");
    /// assert!(err.is_git_error());
    /// ```
    pub fn is_git_error(&self) -> bool {
        matches!(self, AppError::Git { .. })
    }

    /// Returns `true` if the error is an AI service error variant.
    pub fn is_ai_error(&self) -> bool {
        matches!(self, AppError::AI { .. })
    }

    /// Returns `true` if the error is a network-related error variant.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::network("Failed to connect", Some("https://example.com".to_string()), None);
    /// assert!(err.is_network_error());
    /// ```
    pub fn is_network_error(&self) -> bool {
        matches!(self, AppError::Network { .. })
    }

    /// Returns a detailed error message including contextual information for the error variant.
    ///
    /// The message includes additional details such as command, path, URL, analysis type, language, status code, argument, or category, depending on the error type.
    ///
    /// # Examples
    ///
    /// ```
    /// let err = AppError::IO {
    ///     message: "Failed to read file".to_string(),
    ///     path: Some("config.toml".into()),
    ///     source: None,
    /// };
    /// assert!(err.detailed_message().contains("config.toml"));
    /// ```
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
    /// Converts a `std::io::Error` into an `AppError::IO` variant with a default message and the original error as the source.
    fn from(err: std::io::Error) -> Self {
        Self::io_with_source("文件操作失败".to_string(), None, err)
    }
}

impl From<reqwest::Error> for AppError {
    /// Converts a `reqwest::Error` into an `AppError::Network` variant, preserving the URL and source error if available.
    fn from(err: reqwest::Error) -> Self {
        let url = err.url().map(|u| u.to_string());
        Self::network_with_source("网络请求失败".to_string(), url, err)
    }
}

impl From<serde_json::Error> for AppError {
    /// Converts a `serde_json::Error` into a generic application error with the category "serialization".
    ///
    /// The error message includes details from the original JSON parsing or serialization error.
    fn from(err: serde_json::Error) -> Self {
        Self::generic_with_category(
            format!("JSON 解析错误: {}", err),
            "serialization"
        )
    }
}

impl From<toml::de::Error> for AppError {
    /// Converts a TOML deserialization error into a configuration error variant of `AppError`, preserving the original error as the source.
    fn from(err: toml::de::Error) -> Self {
        Self::config_with_source("TOML 配置解析错误".to_string(), err)
    }
}

impl From<toml::ser::Error> for AppError {
    /// Converts a TOML serialization error into a configuration error variant of `AppError`, preserving the original error as the source.
    fn from(err: toml::ser::Error) -> Self {
        Self::config_with_source("TOML 配置序列化错误".to_string(), err)
    }
}

// 从旧错误系统的转换支持
impl From<crate::errors::AppError> for AppError {
    /// Converts a legacy `crate::errors::AppError` into the unified `AppError` type, preserving relevant context and details from the original error.
    ///
    /// This function maps each variant of the old error type to the corresponding variant in the new error system, including nested error information such as commands, exit codes, messages, and source errors. This enables seamless migration and interoperability between the old and new error handling mechanisms.
    ///
    /// # Examples
    ///
    /// ```
    /// let legacy_error = crate::errors::AppError::Generic("Some error".to_string());
    /// let unified_error = AppError::from(legacy_error);
    /// assert!(matches!(unified_error, AppError::Generic(_)));
    /// ```
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