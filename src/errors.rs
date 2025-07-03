use crate::types::general::CommandOutput;
use thiserror::Error; // Added for DevOpsError

#[allow(unused)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Git command error: {0}")]
    Git(#[from] GitError),
    #[error("AI interaction error: {0}")]
    AI(#[from] AIError),
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),
    #[error("DevOps API error: {0}")]
    DevOps(#[from] DevOpsError), // Added DevOpsError variant
    #[error("I/O error while {0}: {1}")]
    IO(String, #[source] std::io::Error), // For generic I/O errors not covered by specific types
    #[error("Application error: {0}")]
    Generic(String), // For simple string-based errors
}

// Copied and renamed ApiError from src/errors/devops.rs
#[derive(Debug, Error)]
pub enum DevOpsError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Authentication failed: Invalid token")]
    AuthenticationError,

    #[error("Work item {item_id} not found")]
    WorkItemNotFound { item_id: u32 },

    #[error("API rate limit exceeded, please try again later")]
    RateLimitExceeded,

    #[error("Server error: {status_code}")]
    ServerError { status_code: u16 },

    #[error("Response data parsing failed: {0}")]
    ParseError(reqwest::Error), // Changed from serde_json::Error, removed #[from]

    #[error("Request timed out")]
    TimeoutError,

    #[error("API returned an error: Code {code}, Message: {message}")]
    ApiLogicalError { code: i32, message: String },

    #[error("Unexpected response structure from API: {0}")]
    UnexpectedResponseStructure(String),
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read file '{0}': {1}")]
    FileRead(String, #[source] std::io::Error),
    #[error("Failed to write to path '{0}': {1}")]
    FileWrite(String, #[source] std::io::Error),
    #[error("Failed to parse TOML from file '{0}': {1}")]
    TomlParse(String, #[source] toml::de::Error),
    #[error("Critical prompt file '{0}' is missing.")]
    PromptFileMissing(String),
    #[error("Required configuration field '{0}' is missing or invalid")]
    FieldMissing(String), // Added for missing required fields
    #[error("Failed to read Git configuration for {0}: {1}")]
    GitConfigRead(String, #[source] std::io::Error),
    #[error("Failed to read DevOps configuration: {0}")]
    DevOpsConfigMissing(String), // DevOps platform configuration missing
    #[error("Unsupported DevOps platform: {0}")]
    UnsupportedPlatform(String), // Unsupported platform
    #[error("Wrong url format: {0}")]
    InvalidUrl(String),
    #[error("Empty token")]
    EmptyToken,
    #[error("Other Config Error: {0}")]
    Other(String), // Other errors
}

#[allow(unused)]
#[derive(Debug)]
pub enum GitError {
    CommandFailed {
        command: String,
        status_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    PassthroughFailed {
        // For commands where output is not captured (used .status())
        command: String,
        status_code: Option<i32>,
    },
    DiffError(std::io::Error), // Changed to std::io::Error as it's more idiomatic
    NotARepository,
    NoStagedChanges,
    Other(String), // Generic Git errors
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum AIError {
    #[error("AI API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse AI API JSON response: {0}")]
    ResponseParseFailed(#[source] reqwest::Error),
    #[error("AI API responded with error {0}: {1}")]
    ApiResponseError(reqwest::StatusCode, String),
    #[error("AI API response contained no choices.")]
    NoChoiceInResponse,
    #[error("AI returned an empty message.")]
    EmptyMessage,
    #[error("AI explanation generation failed: {0}")]
    ExplanationGenerationFailed(String), // For errors from ai_explaniner
    #[error("AI explainer configuration error: {0}")]
    ExplainerConfigurationError(String), // For config errors specific to explainer
    #[error("AI explainer network error: {0}")]
    ExplainerNetworkError(String), // For network errors from explainer not covered by reqwest::Error
}

#[allow(unused)]
#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Language error: {0}")]
    LanguageError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Initialization error: {0}")]
    InitializationError(String),
    #[error("Analysis timeout: {0}")]
    AnalysisTimeout(String),
    #[error("I/O error: {0}")]
    IOError(#[source] std::io::Error),
    #[error("Analysis error: {0}")]
    Generic(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::CommandFailed {
                command,
                status_code,
                stdout,
                stderr,
            } => {
                write!(f, "Git command '{}' failed", command)?;
                if let Some(c) = status_code {
                    write!(f, " with exit code {}", c)?;
                }
                if !stdout.is_empty() {
                    write!(f, "\nStdout:\n{}", stdout)?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nStderr:\n{}", stderr)?;
                }
                Ok(())
            }
            GitError::PassthroughFailed {
                command,
                status_code,
            } => {
                write!(f, "Git passthrough command '{}' failed", command)?;
                if let Some(c) = status_code {
                    write!(f, " with exit code {}", c)?;
                }
                Ok(())
            }
            GitError::DiffError(e) => write!(f, "Failed to get git diff: {}", e),
            GitError::NotARepository => write!(
                f,
                "Not a git repository (or any of the parent directories)."
            ),
            GitError::NoStagedChanges => write!(f, "No changes staged for commit."),
            GitError::Other(s) => write!(f, "Git error: {}", s),
        }
    }
}

impl std::error::Error for GitError {
    /// Returns the underlying source error if available.
    ///
    /// For `GitError::DiffError`, returns the wrapped `std::io::Error`. For other variants, returns `None`.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitError::DiffError(e) => Some(e),
            _ => None,
        }
    }
}

// --- From implementations for AppError ---

impl From<std::io::Error> for AppError {
    /// Converts a `std::io::Error` into an `AppError::IO` with a default context message.
    fn from(err: std::io::Error) -> Self {
        AppError::IO("I/O operation failed".to_string(), err)
    }
}

/// Converts process output and exit status into a `GitError::CommandFailed`.
///
/// This function extracts the standard output and error streams from a completed process,
/// along with its exit status, and constructs a `GitError::CommandFailed` variant for error reporting.
/// The `status_code` field will be `None` if the process was terminated by a signal rather than an exit code.
///
/// # Examples
///
/// ```
/// use crate::errors::{map_command_error, GitError};
/// use std::process::{Command, Stdio};
///
/// let output = Command::new("false")
///     .stdout(Stdio::piped())
///     .stderr(Stdio::piped())
///     .output()
///     .expect("failed to execute process");
/// let status = output.status;
/// let err = map_command_error("false", output, status);
/// if let GitError::CommandFailed { command, status_code, .. } = err {
///     assert_eq!(command, "false");
///     assert_eq!(status_code, Some(1));
/// }
/// ```
pub fn map_command_error(
    cmd_str: &str,
    output: std::process::Output,     // Takes ownership
    status: std::process::ExitStatus, // Provided seperately as output is consumed for stdout/stderr
) -> GitError {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    GitError::CommandFailed {
        command: cmd_str.to_string(),
        status_code: status.code(),
        stdout,
        stderr,
    }
}

/// Converts a captured `CommandOutput` into a `GitError::CommandFailed`.
///
/// The resulting error includes the command string, optional exit code, and captured stdout and stderr. If the process was terminated by a signal, `status_code` will be `None`.
///
/// # Examples
///
/// ```
/// let output = CommandOutput {
///     status: std::process::ExitStatus::from_raw(1),
///     stdout: String::from("output"),
///     stderr: String::from("error"),
/// };
/// let err = map_command_output_error("git status", output);
/// if let GitError::CommandFailed { command, status_code, stdout, stderr } = err {
///     assert_eq!(command, "git status");
///     assert_eq!(status_code, Some(1));
///     assert_eq!(stdout, "output");
///     assert_eq!(stderr, "error");
/// }
/// ```
pub fn map_command_output_error(cmd_str: &str, output: CommandOutput) -> GitError {
    GitError::CommandFailed {
        command: cmd_str.to_string(),
        status_code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn mock_reqwest_error() -> reqwest::Error {
        // This is reliable way to get a reqwest::Error:
        // try to connect to a non-routable address.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            reqwest::Client::new()
                .get("http://0.0.0.0.0.0.1")
                .send()
                .await
                .unwrap_err()
        })
    }

    fn mock_toml_error() -> toml::de::Error {
        toml::from_str::<toml::Value>("invalid_toml").err().unwrap()
    }

    #[test]
    fn test_config_error_display() {
        let file_name = "test_config.json".to_string();
        let toml_file_name = "test_config.toml".to_string();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let toml_err = mock_toml_error();

        let err_file_read = ConfigError::FileRead(file_name.clone(), io_err);
        assert_eq!(
            format!("{}", err_file_read),
            "Failed to read file 'test_config.json': file not found"
        );

        let err_toml_parse = ConfigError::TomlParse(toml_file_name.clone(), toml_err);
        assert!(
            format!("{}", err_toml_parse)
                .starts_with("Failed to parse TOML from file 'test_config.toml': ")
        );

        let err_prompt_missing = ConfigError::PromptFileMissing("assets/my_prompt".to_string());
        assert_eq!(
            format!("{}", err_prompt_missing),
            "Critical prompt file 'assets/my_prompt' is missing."
        );

        let git_config_io_err =
            io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err_git_config_read =
            ConfigError::GitConfigRead("user name".to_string(), git_config_io_err);
        assert_eq!(
            format!("{}", err_git_config_read),
            "Failed to read Git configuration for user name: permission denied"
        );

        let err_field_missing = ConfigError::FieldMissing("model_name".to_string());
        assert_eq!(
            format!("{}", err_field_missing),
            "Required configuration field 'model_name' is missing or invalid"
        );
    }

    #[test]
    fn test_git_error_display() {
        let io_err_for_diff =
            std::io::Error::new(std::io::ErrorKind::Other, "diff generation failed");
        let err_diff = GitError::DiffError(io_err_for_diff);
        assert_eq!(
            format!("{}", err_diff),
            "Failed to get git diff: diff generation failed"
        );

        let err_not_repo = GitError::NotARepository;
        assert_eq!(
            format!("{}", err_not_repo),
            "Not a git repository (or any of the parent directories)."
        );

        let err_no_staged = GitError::NoStagedChanges;
        assert_eq!(
            format!("{}", err_no_staged),
            "No changes staged for commit."
        );

        let err_cmd_failed_simple = GitError::CommandFailed {
            command: "git version".to_string(),
            status_code: Some(128),
            stdout: "".to_string(),
            stderr: "fatal error".to_string(),
        };
        assert_eq!(
            format!("{}", err_cmd_failed_simple),
            "Git command 'git version' failed with exit code 128\nStderr:\nfatal error"
        );

        let err_cmd_failed_full = GitError::CommandFailed {
            command: "git status".to_string(),
            status_code: Some(0), // Even if code is 0, if it's an error path, it's an error.
            stdout: "on branch master".to_string(),
            stderr: "warning".to_string(),
        };
        assert_eq!(
            format!("{}", err_cmd_failed_full),
            "Git command 'git status' failed with exit code 0\nStdout:\non branch master\nStderr:\nwarning"
        );

        let err_passthrough_failed = GitError::PassthroughFailed {
            command: "git push".to_string(),
            status_code: Some(1),
        };
        assert_eq!(
            format!("{}", err_passthrough_failed),
            "Git passthrough command 'git push' failed with exit code 1"
        );

        let err_other_git = GitError::Other("Some other issue".to_string());
        assert_eq!(format!("{}", err_other_git), "Git error: Some other issue");

        // Test CommandFailed with status_code: None (process terminated by signal)
        let err_cmd_failed_no_status = GitError::CommandFailed {
            command: "git clone".to_string(),
            status_code: None, // Process was terminated by signal
            stdout: "".to_string(),
            stderr: "Terminated".to_string(),
        };
        assert_eq!(
            format!("{}", err_cmd_failed_no_status),
            "Git command 'git clone' failed\nStderr:\nTerminated"
        );

        // Test PassthroughFailed with status_code: None
        let err_passthrough_no_status = GitError::PassthroughFailed {
            command: "git fetch".to_string(),
            status_code: None, // Process was terminated by signal
        };
        assert_eq!(
            format!("{}", err_passthrough_no_status),
            "Git passthrough command 'git fetch' failed"
        );
    }

    #[test]
    fn test_ai_error_display() {
        let req_err = mock_reqwest_error();
        let err_request_failed = AIError::RequestFailed(req_err);
        assert!(format!("{}", err_request_failed).starts_with("AI API request failed: "));

        let parse_err = mock_reqwest_error();
        let err_response_parse_failed = AIError::ResponseParseFailed(parse_err);
        assert!(
            format!("{}", err_response_parse_failed)
                .starts_with("Failed to parse AI API JSON response: ")
        );

        let err_api_response = AIError::ApiResponseError(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "Server meltdown".to_string(),
        );
        assert_eq!(
            format!("{}", err_api_response),
            "AI API responded with error 500 Internal Server Error: Server meltdown"
        );

        let err_no_choice = AIError::NoChoiceInResponse;
        assert_eq!(
            format!("{}", err_no_choice),
            "AI API response contained no choices."
        );

        let err_empty_message = AIError::EmptyMessage;
        assert_eq!(
            format!("{}", err_empty_message),
            "AI returned an empty message."
        );

        let err_expl_gen = AIError::ExplanationGenerationFailed("model unavailable".to_string());
        assert_eq!(
            format!("{}", err_expl_gen),
            "AI explanation generation failed: model unavailable"
        );

        let err_expl_conf = AIError::ExplainerConfigurationError("missing prompt".to_string());
        assert_eq!(
            format!("{}", err_expl_conf),
            "AI explainer configuration error: missing prompt"
        );

        let err_expl_net = AIError::ExplainerNetworkError("connection refused".to_string());
        assert_eq!(
            format!("{}", err_expl_net),
            "AI explainer network error: connection refused"
        );
    }

    #[test]
    fn test_app_error_display() {
        let config_err = ConfigError::PromptFileMissing("prompts/sys".to_string());
        let app_config_err = AppError::from(config_err);
        assert_eq!(
            format!("{}", app_config_err),
            "Configuration error: Critical prompt file 'prompts/sys' is missing."
        );

        let git_err = GitError::NotARepository;
        let app_git_err = AppError::from(git_err);
        assert_eq!(
            format!("{}", app_git_err),
            "Git command error: Not a git repository (or any of the parent directories)."
        );

        let ai_err = AIError::EmptyMessage;
        let app_ai_err = AppError::from(ai_err);
        assert_eq!(
            format!("{}", app_ai_err),
            "AI interaction error: AI returned an empty message."
        );

        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        // Test the generic From<io::Error>
        let app_io_err: AppError = io_err.into();
        assert_eq!(
            format!("{}", app_io_err),
            "I/O error while I/O operation failed: pipe broke" // Default context
        );

        let app_generic_err = AppError::Generic("Something went wrong".to_string());
        assert_eq!(
            format!("{}", app_generic_err),
            "Application error: Something went wrong"
        );
    }
}
