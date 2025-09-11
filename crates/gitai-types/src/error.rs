//! Error types for GitAI

use thiserror::Error;

/// GitAI specific error types
#[derive(Debug, Error)]
pub enum GitAIError {
    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Git related errors
    #[error("Git error: {0}")]
    Git(String),

    /// Analysis errors
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Security scan errors
    #[error("Security scan error: {0}")]
    Security(String),

    /// Metrics errors
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// MCP related errors
    #[error("MCP error: {0}")]
    Mcp(String),

    /// AI service errors
    #[error("AI service error: {0}")]
    Ai(String),

    /// DevOps platform errors
    #[error("DevOps platform error: {0}")]
    DevOps(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Parse errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type for GitAI operations
pub type Result<T> = std::result::Result<T, GitAIError>;
