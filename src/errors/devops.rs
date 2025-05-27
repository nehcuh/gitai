use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
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
    ParseError(#[from] serde_json::Error),
    
    #[error("Request timed out")]
    TimeoutError,

    #[error("API returned an error: Code {code}, Message: {message}")]
    ApiLogicalError { code: i32, message: String },

    #[error("Unexpected response structure from API: {0}")]
    UnexpectedResponseStructure(String),
}
