
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Git 错误: {0}")]
    Git(#[from] GitError),
    
    #[error("AI 错误: {0}")]
    AI(#[from] AIError),
    
    #[error("Tree-sitter 错误: {0}")]
    TreeSitter(#[from] TreeSitterError),
    
    #[error("DevOps 错误: {0}")]
    DevOps(#[from] DevOpsError),
    
    #[error("IO 错误: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),
    
    #[error("文件错误: {0}")]
    File(#[from] FileError),
    
    #[error("应用错误: {0}")]
    Generic(String),
    
    // Backwards compatibility constructors
    #[error("配置错误: {0}")]
    ConfigString(String),
    
    #[error("Git 错误: {0}")]
    GitString(String),
    
    #[error("AI 错误: {0}")]
    AIString(String),
    
    #[error("Tree-sitter 错误: {0}")]
    TreeSitterString(String),
    
    #[error("DevOps 错误: {0}")]
    DevOpsString(String),
    
    #[error("网络错误: {0}")]
    NetworkString(String),
    
    #[error("文件错误: {0}")]
    FileString(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("TOML 解析错误: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("配置文件不存在: {path}")]
    FileNotFound { path: String },
    
    #[error("缺少必需配置: {field}")]
    MissingField { field: String },
    
    #[error("配置值无效: {field} = {value}")]
    InvalidValue { field: String, value: String },
    
    #[error("配置验证失败: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git 命令执行失败: {command}")]
    CommandFailed { command: String, exit_code: Option<i32>, stdout: String, stderr: String },
    
    #[error("Git 仓库不存在: {path}")]
    RepositoryNotFound { path: String },
    
    #[error("Git 分支不存在: {branch}")]
    BranchNotFound { branch: String },
    
    #[error("Git 操作失败: {0}")]
    OperationFailed(String),
}

#[derive(Error, Debug)]
pub enum AIError {
    #[error("AI 请求失败: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("AI 响应解析失败: {0}")]
    ResponseParse(String),
    
    #[error("AI 服务不可用: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("AI 配置错误: {0}")]
    Configuration(String),
}

#[derive(Error, Debug)]
pub enum TreeSitterError {
    #[error("语法解析失败: {language}")]
    ParseFailed { language: String },
    
    #[error("查询执行失败: {0}")]
    QueryFailed(String),
    
    #[error("节点类型不支持: {node_type}")]
    UnsupportedNodeType { node_type: String },
    
    #[error("语言不支持: {language}")]
    LanguageNotSupported { language: String },
    
    #[error("查询编译失败: {language}, 错误: {error}")]
    QueryCompilationFailed { language: String, error: String },
    
    #[error("文件解析失败: {file}, 错误: {error}")]
    FileParseFailed { file: String, error: String },
    
    #[error("缓存操作失败: {operation}")]
    CacheFailed { operation: String },
    
    #[error("配置验证失败: {field} = {value}")]
    ConfigValidation { field: String, value: String },
    
    #[error("初始化失败: {reason}")]
    InitializationFailed { reason: String },
}

#[derive(Error, Debug)]
pub enum DevOpsError {
    #[error("构建失败: {0}")]
    BuildFailed(String),
    
    #[error("测试失败: {0}")]
    TestFailed(String),
    
    #[error("部署失败: {0}")]
    DeploymentFailed(String),
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("网络请求失败: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("连接超时: {url}")]
    ConnectionTimeout { url: String },
    
    #[error("DNS 解析失败: {host}")]
    DnsResolveFailed { host: String },
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("文件不存在: {path}")]
    NotFound { path: String },
    
    #[error("文件读取失败: {path}")]
    ReadFailed { path: String, source: std::io::Error },
    
    #[error("文件写入失败: {path}")]
    WriteFailed { path: String, source: std::io::Error },
    
    #[error("权限不足: {path}")]
    PermissionDenied { path: String },
}

// --- Helper functions for common error conversions ---

pub fn config_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Config(ConfigError::Validation(msg.into()))
}

pub fn git_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Git(GitError::OperationFailed(msg.into()))
}

pub fn ai_error<S: Into<String>>(msg: S) -> AppError {
    AppError::AI(AIError::Configuration(msg.into()))
}

pub fn tree_sitter_error<S: Into<String>>(msg: S) -> AppError {
    AppError::TreeSitter(TreeSitterError::QueryFailed(msg.into()))
}

// 专门的tree-sitter错误构造函数
pub fn tree_sitter_parse_error<S: Into<String>>(language: S) -> AppError {
    AppError::TreeSitter(TreeSitterError::ParseFailed { 
        language: language.into() 
    })
}

pub fn tree_sitter_language_error<S: Into<String>>(language: S) -> AppError {
    AppError::TreeSitter(TreeSitterError::LanguageNotSupported { 
        language: language.into() 
    })
}

pub fn tree_sitter_query_compilation_error<S1: Into<String>, S2: Into<String>>(language: S1, error: S2) -> AppError {
    AppError::TreeSitter(TreeSitterError::QueryCompilationFailed { 
        language: language.into(), 
        error: error.into() 
    })
}

pub fn tree_sitter_file_parse_error<S1: Into<String>, S2: Into<String>>(file: S1, error: S2) -> AppError {
    AppError::TreeSitter(TreeSitterError::FileParseFailed { 
        file: file.into(), 
        error: error.into() 
    })
}

pub fn tree_sitter_cache_error<S: Into<String>>(operation: S) -> AppError {
    AppError::TreeSitter(TreeSitterError::CacheFailed { 
        operation: operation.into() 
    })
}

pub fn tree_sitter_config_error<S1: Into<String>, S2: Into<String>>(field: S1, value: S2) -> AppError {
    AppError::TreeSitter(TreeSitterError::ConfigValidation { 
        field: field.into(), 
        value: value.into() 
    })
}

pub fn tree_sitter_init_error<S: Into<String>>(reason: S) -> AppError {
    AppError::TreeSitter(TreeSitterError::InitializationFailed { 
        reason: reason.into() 
    })
}

pub fn devops_error<S: Into<String>>(msg: S) -> AppError {
    AppError::DevOps(DevOpsError::BuildFailed(msg.into()))
}

pub fn network_error<S: Into<String>>(msg: S) -> AppError {
    AppError::NetworkString(msg.into())
}

pub fn file_error<S: Into<String>>(msg: S) -> AppError {
    AppError::File(FileError::NotFound { path: msg.into() })
}

// Macro for converting any error to AppError::TreeSitter
#[macro_export]
macro_rules! tree_sitter_err {
    ($e:expr) => {
        AppError::TreeSitter(TreeSitterError::QueryFailed($e.to_string()))
    };
}

// Macro for converting any error to AppError::Git
#[macro_export]
macro_rules! git_err {
    ($e:expr) => {
        AppError::Git(GitError::OperationFailed($e.to_string()))
    };
}

// Backwards compatibility functions to ease migration
pub fn config_string_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Config(ConfigError::Validation(msg.into()))
}

pub fn git_string_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Git(GitError::OperationFailed(msg.into()))
}

// Additional From implementations for backwards compatibility
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_string())
    }
}

// Legacy From implementations for backwards compatibility
impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Config(ConfigError::TomlParse(err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(NetworkError::RequestFailed(err))
    }
}

// Helper for converting Command output to error
pub fn git_command_error(
    cmd_str: &str,
    output: std::process::Output,
    status: std::process::ExitStatus,
) -> AppError {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    AppError::Git(GitError::CommandFailed {
        command: cmd_str.to_string(),
        exit_code: status.code(),
        stdout,
        stderr,
    })
}

// Additional helper functions for specific error types
pub fn config_file_not_found<S: Into<String>>(path: S) -> AppError {
    AppError::Config(ConfigError::FileNotFound { path: path.into() })
}

pub fn config_missing_field<S: Into<String>>(field: S) -> AppError {
    AppError::Config(ConfigError::MissingField { field: field.into() })
}

pub fn config_invalid_value<F: Into<String>, V: Into<String>>(field: F, value: V) -> AppError {
    AppError::Config(ConfigError::InvalidValue { 
        field: field.into(), 
        value: value.into() 
    })
}

pub fn git_repo_not_found<S: Into<String>>(path: S) -> AppError {
    AppError::Git(GitError::RepositoryNotFound { path: path.into() })
}

pub fn file_not_found<S: Into<String>>(path: S) -> AppError {
    AppError::File(FileError::NotFound { path: path.into() })
}

pub fn file_read_failed<S: Into<String>>(path: S, source: std::io::Error) -> AppError {
    AppError::File(FileError::ReadFailed { path: path.into(), source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = config_error("配置验证失败");
        assert!(format!("{}", err).contains("配置错误: 配置验证失败"));

        let err = config_file_not_found("/path/to/config");
        assert!(format!("{}", err).contains("配置文件不存在: /path/to/config"));

        let err = config_missing_field("api_key");
        assert!(format!("{}", err).contains("缺少必需配置: api_key"));

        let err = config_invalid_value("timeout", "invalid");
        assert!(format!("{}", err).contains("配置值无效: timeout = invalid"));
    }

    #[test]
    fn test_git_error_display() {
        let err = git_error("操作失败");
        assert!(format!("{}", err).contains("Git 错误: 操作失败"));

        let err = git_repo_not_found("/path/to/repo");
        assert!(format!("{}", err).contains("Git 仓库不存在: /path/to/repo"));

        let err = git_command_error(
            "git status",
            std::process::Output {
                status: std::process::ExitStatus::from_raw(1),
                stdout: b"".to_vec(),
                stderr: b"fatal: not a git repository".to_vec(),
            },
            std::process::ExitStatus::from_raw(1),
        );
        assert!(format!("{}", err).contains("Git 命令执行失败: git status"));
    }

    #[test]
    fn test_file_error_display() {
        let err = file_not_found("/path/to/file");
        assert!(format!("{}", err).contains("文件不存在: /path/to/file"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let err = file_read_failed("/path/to/file", io_err);
        assert!(format!("{}", err).contains("文件读取失败: /path/to/file"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let file_err = file_read_failed("/path/to/file", io_err);
        let app_err: AppError = file_err;
        
        // Test that we can access the source error
        assert!(format!("{}", app_err).contains("文件读取失败"));
    }
}
