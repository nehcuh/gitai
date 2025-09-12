use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    Network(reqwest::Error),
    Io(std::io::Error),
    Config(String),
    Download(String),
    Parse(String),
    Version(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Network(e) => write!(f, "网络错误: {}", e),
            UpdateError::Io(e) => write!(f, "IO错误: {}", e),
            UpdateError::Config(msg) => write!(f, "配置错误: {}", msg),
            UpdateError::Download(msg) => write!(f, "下载错误: {}", msg),
            UpdateError::Parse(msg) => write!(f, "解析错误: {}", msg),
            UpdateError::Version(msg) => write!(f, "版本错误: {}", msg),
        }
    }
}

impl std::error::Error for UpdateError {}

impl From<reqwest::Error> for UpdateError {
    fn from(e: reqwest::Error) -> Self {
        UpdateError::Network(e)
    }
}

impl From<std::io::Error> for UpdateError {
    fn from(e: std::io::Error) -> Self {
        UpdateError::Io(e)
    }
}
