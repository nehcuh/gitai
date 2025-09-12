//! Error types for GitAI

use thiserror::Error;

/// GitAI specific error types with detailed variants
#[derive(Debug, Error)]
pub enum GitAIError {
    /// Configuration related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),

    /// Git related errors
    #[error("Git error: {0}")]
    Git(#[from] GitError),

    /// Analysis errors
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Security scan errors
    #[error("Security scan error: {0}")]
    Security(#[from] ScanError),

    /// Metrics errors
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// MCP related errors
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    /// AI service errors
    #[error("AI service error: {0}")]
    Ai(#[from] AiError),

    /// DevOps platform errors
    #[error("DevOps platform error: {0}")]
    DevOps(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// Parse errors
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Container errors
    #[error("Container error: {0}")]
    Container(#[from] ContainerError),

    /// Update errors
    #[error("Update error: {0}")]
    Update(#[from] UpdateError),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// User cancelled operation
    #[error("Operation cancelled by user")]
    UserCancelled,

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

/// Configuration error details
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    #[error("Failed to load configuration: {0}")]
    LoadFailed(String),
    #[error("Missing configuration key: {0}")]
    Missing(String),
}

/// Git operation error details
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    #[error("Commit not found: {0}")]
    CommitNotFound(String),
    #[error("Working directory is dirty: {0}")]
    WorkingDirectoryDirty(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// File system error details
#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Directory traversal failed: {0}")]
    DirectoryTraversal(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO operation failed: {0}")]
    Io(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Network error details
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Request timeout: {0}")]
    Timeout(String),
    #[error("DNS resolution failed: {0}")]
    DnsFailed(String),
    #[error("SSL error: {0}")]
    Ssl(String),
    #[error("HTTP error: {0}")]
    Http(String),
}

/// Parse error details
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("JSON parse failed: {0}")]
    Json(String),
    #[error("TOML parse failed: {0}")]
    Toml(String),
    #[error("{0} parse failed: {1}")]
    Other(String, String),
}

/// Scan tool error details
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("Scan tool not found: {0}")]
    ToolNotFound(String),
    #[error("Scan execution failed: {0}")]
    ScanExecutionFailed(String),
    #[error("Rule load failed: {0}")]
    RuleLoadFailed(String),
    #[error("Result parse failed: {0}")]
    ResultParseFailed(String),
    #[error("File access failed: {0}")]
    FileAccessFailed(String),
    #[error("Scan timeout: {0} seconds")]
    Timeout(u64),
}

/// AI service error details
#[derive(Debug, Error)]
pub enum AiError {
    #[error("API call failed: {0}")]
    ApiCallFailed(String),
    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),
    #[error("Request timeout: {0} seconds")]
    RequestTimeout(u64),
    #[error("Response parse failed: {0}")]
    ResponseParseFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
}

/// Container error details
#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Service not registered: {type_name}. Available: {available_services:?}. {suggestion:?}")]
    ServiceNotRegistered {
        type_name: String,
        available_services: Vec<String>,
        suggestion: Option<String>,
    },
    #[error("Circular dependency detected: chain {service_chain:?}, cycle at {cycle_point}")]
    CircularDependency {
        service_chain: Vec<String>,
        cycle_point: String,
    },
    #[error("Service creation failed for {service_type}: {reason}")]
    ServiceCreationFailed {
        service_type: String,
        reason: String,
    },
    #[error("Type cast failed: expected {expected}, got {actual}")]
    TypeCastFailed { expected: String, actual: String },
    #[error("Scope error in {operation}: {reason}")]
    ScopeError { operation: String, reason: String },
}

/// MCP error details
#[derive(Debug, Error)]
pub enum McpError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("File operation error: {0}")]
    FileOperationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("External tool error: {0}")]
    ExternalToolError(String),
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Update error details
#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Download error: {0}")]
    Download(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Version error: {0}")]
    Version(String),
}

/// Result type for GitAI operations
pub type Result<T> = std::result::Result<T, GitAIError>;
