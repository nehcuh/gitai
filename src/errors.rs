use thiserror::Error; // Added for DevOpsError

#[allow(unused)]
#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Git(GitError),
    AI(AIError),
    Analysis(AnalysisError),
    DevOps(DevOpsError),        // Added DevOpsError variant
    IO(String, std::io::Error), // For generic I/O errors not covered by specific types
    Generic(String),            // For simple string-based errors
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
#[derive(Debug)]
pub enum ConfigError {
    FileRead(String, std::io::Error),
    FileWrite(String, std::io::Error),
    TomlParse(String, toml::de::Error),
    PromptFileMissing(String),
    FieldMissing(String), // Added for missing required fields
    GitConfigRead(String, std::io::Error),
    DevOpsConfigMissing(String), // DevOps platform configuration missing
    UnsupportedPlatform(String), // Unsupported platform
    InvalidUrl(String),
    EmptyToken,
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
#[derive(Debug)]
pub enum AIError {
    RequestFailed(reqwest::Error),
    ResponseParseFailed(reqwest::Error),
    ApiResponseError(reqwest::StatusCode, String),
    NoChoiceInResponse,
    EmptyMessage,
    ExplanationGenerationFailed(String), // For errors from ai_explaniner
    ExplainerConfigurationError(String), // For config errors specific to explainer
    ExplainerNetworkError(String), // For network errors from explainer not covered by reqwest::Error
}

#[allow(unused)]
#[derive(Debug)]
pub enum AnalysisError {
    UnsupportedLanguage(String),
    LanguageError(String),
    ParseError(String),
    QueryError(String),
    CacheError(String),
    InitializationError(String),
    AnalysisTimeout(String),
    IOError(std::io::Error),
    Generic(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Config(e) => write!(f, "Configuration error: {}", e),
            AppError::Git(e) => write!(f, "Git command error: {}", e),
            AppError::AI(e) => write!(f, "AI interaction error: {}", e),
            AppError::Analysis(e) => write!(f, "Analysis error: {}", e),
            AppError::DevOps(e) => write!(f, "DevOps API error: {}", e), // Added for DevOps
            AppError::IO(context, e) => write!(f, "I/O error while {}: {}", context, e),
            AppError::Generic(s) => write!(f, "Application error: {}", s),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Config(e) => Some(e),
            AppError::Git(e) => Some(e),
            AppError::AI(e) => Some(e),
            AppError::Analysis(e) => Some(e),
            AppError::DevOps(e) => Some(e), // Added for DevOps
            AppError::IO(_, e) => Some(e),
            AppError::Generic(_) => None,
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(file, e) => write!(f, "Failed to read file '{}': {}", file, e),
            ConfigError::FileWrite(path, e) => {
                write!(f, "Failed to write to path '{}': {}", path, e)
            }
            ConfigError::TomlParse(file, e) => {
                write!(f, "Failed to parse TOML from file '{}': {}", file, e)
            }
            ConfigError::PromptFileMissing(file) => {
                write!(f, "Critical prompt file '{}' is missing.", file)
            }
            ConfigError::FieldMissing(field) => write!(
                f,
                "Required configuration field '{}' is missing or invalid",
                field
            ),
            ConfigError::GitConfigRead(context, e) => {
                write!(f, "Failed to read Git configuration for {}: {}", context, e)
            }
            ConfigError::DevOpsConfigMissing(context) => {
                write!(f, "Failed to read DevOps configuration: {}", context)
            }
            ConfigError::UnsupportedPlatform(context) => {
                write!(f, "Unsupported DevOps platform: {}", context)
            }
            ConfigError::InvalidUrl(context) => {
                write!(f, "Wrong url format: {}", context)
            }
            ConfigError::EmptyToken => {
                write!(f, "Empty token")
            }
            ConfigError::Other(content) => write!(f, "Other Config Error: {}", content),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::FileRead(_, e) => Some(e),
            ConfigError::FileWrite(_, e) => Some(e),
            ConfigError::TomlParse(_, e) => Some(e),
            ConfigError::PromptFileMissing(_) => None,
            ConfigError::FieldMissing(_) => None,
            ConfigError::GitConfigRead(_, e) => Some(e),
            ConfigError::DevOpsConfigMissing(_) => None,
            ConfigError::UnsupportedPlatform(_) => None,
            ConfigError::InvalidUrl(_) => None,
            ConfigError::EmptyToken => None,
            ConfigError::Other(_) => None,
        }
    }
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
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitError::DiffError(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::RequestFailed(e) => write!(f, "AI API request failed: {}", e),
            AIError::ResponseParseFailed(e) => {
                write!(f, "Failed to parse AI API JSON response: {}", e)
            }
            AIError::ApiResponseError(status, body) => {
                write!(f, "AI API responded with error {}: {}", status, body)
            }
            AIError::NoChoiceInResponse => write!(f, "AI API response contained no choices."),
            AIError::EmptyMessage => write!(f, "AI returned an empty message."),
            AIError::ExplanationGenerationFailed(s) => {
                write!(f, "AI explanation generation failed: {}", s)
            }
            AIError::ExplainerConfigurationError(s) => {
                write!(f, "AI explainer configuration error: {}", s)
            }
            AIError::ExplainerNetworkError(s) => write!(f, "AI explainer network error: {}", s),
        }
    }
}

impl std::error::Error for AIError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AIError::RequestFailed(e) => Some(e),
            AIError::ResponseParseFailed(e) => Some(e),
            _ => None, // Other values are self-contained or wrap String
        }
    }
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported language: {}", lang)
            }
            AnalysisError::LanguageError(msg) => write!(f, "Language error: {}", msg),
            AnalysisError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AnalysisError::QueryError(msg) => write!(f, "Query error: {}", msg),
            AnalysisError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            AnalysisError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            AnalysisError::AnalysisTimeout(msg) => write!(f, "Analysis timeout: {}", msg),
            AnalysisError::IOError(e) => write!(f, "I/O error: {}", e),
            AnalysisError::Generic(msg) => write!(f, "Analysis error: {}", msg),
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnalysisError::IOError(e) => Some(e),
            _ => None, // Other values are self-contained
        }
    }
}

// --- From implementations for AppError ---

impl From<ConfigError> for AppError {
    fn from(err: ConfigError) -> Self {
        AppError::Config(err)
    }
}

impl From<GitError> for AppError {
    fn from(err: GitError) -> Self {
        AppError::Git(err)
    }
}

impl From<AIError> for AppError {
    fn from(err: AIError) -> Self {
        AppError::AI(err)
    }
}

impl From<AnalysisError> for AppError {
    fn from(err: AnalysisError) -> Self {
        AppError::Analysis(err)
    }
}

// Added From<DevOpsError> for AppError
impl From<DevOpsError> for AppError {
    fn from(err: DevOpsError) -> Self {
        AppError::DevOps(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO("I/O operation failed".to_string(), err)
    }
}

// Helper for converting Command output to GitError when output is captured
#[allow(unused)]
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
