
#[allow(unused)]
#[derive(Debug)]
pub enum AppError {
    Config(String),
    Git(String),
    AI(String),
    TreeSitter(String),
    DevOps(String),
    IO(std::io::Error),
    Network(String),
    File(String),
    Generic(String),
}

// Linus-style simple error handling
// No complex error hierarchies - just simple string-based errors

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Config(s) => write!(f, "配置错误: {}", s),
            AppError::Git(s) => write!(f, "Git 错误: {}", s),
            AppError::AI(s) => write!(f, "AI 错误: {}", s),
            AppError::TreeSitter(s) => write!(f, "Tree-sitter 错误: {}", s),
            AppError::DevOps(s) => write!(f, "DevOps 错误: {}", s),
            AppError::IO(e) => write!(f, "IO 错误: {}", e),
            AppError::Network(s) => write!(f, "网络错误: {}", s),
            AppError::File(s) => write!(f, "文件错误: {}", s),
            AppError::Generic(s) => write!(f, "应用错误: {}", s),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::IO(e) => Some(e),
            _ => None,
        }
    }
}

// --- Simple From implementations for AppError ---

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err)
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Config(format!("TOML 解析错误: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(format!("网络请求错误: {}", err))
    }
}

// Helper functions for common error conversions
pub fn config_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Config(msg.into())
}

pub fn git_error<S: Into<String>>(msg: S) -> AppError {
    AppError::Git(msg.into())
}

// Helper for converting Command output to error
#[allow(unused)]
pub fn git_command_error(
    cmd_str: &str,
    output: std::process::Output,
    status: std::process::ExitStatus,
) -> AppError {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    AppError::Git(format!(
        "命令 '{}' 失败 (退出码: {:?})\nStdout:\n{}\nStderr:\n{}",
        cmd_str,
        status.code(),
        stdout,
        stderr
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_error_display() {
        let config_err = AppError::Config("配置文件不存在".to_string());
        assert_eq!(format!("{}", config_err), "配置错误: 配置文件不存在");

        let git_err = AppError::Git("Git 命令失败".to_string());
        assert_eq!(format!("{}", git_err), "Git 错误: Git 命令失败");

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let app_io_err: AppError = io_err.into();
        assert!(format!("{}", app_io_err).contains("IO 错误"));
    }

    #[test]
    fn test_helper_functions() {
        let err = config_error("缺少必需字段");
        assert_eq!(format!("{}", err), "配置错误: 缺少必需字段");

        let err = git_error("仓库不存在");
        assert_eq!(format!("{}", err), "Git 错误: 仓库不存在");

        let err = git_error("文件读取失败");
        assert_eq!(format!("{}", err), "Git 错误: 文件读取失败");
    }
}
