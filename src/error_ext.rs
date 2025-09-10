//! Enhanced error handling traits and extensions for GitAI
//!
//! This module provides a unified error handling approach with:
//! - Consistent error conversion patterns
//! - Rich context information
//! - User-friendly error messages
//! - Structured logging support

use log::{debug, error, warn};
use std::fmt;

use crate::error::{GitAIError, Result};

/// Extension trait for enhanced error handling
pub trait GitAIErrorExt {
    /// Convert any error to GitAIError with context
    fn to_gitai_error(self) -> GitAIError;

    /// Log the error with appropriate level
    fn log_error(&self);

    /// Get error chain for detailed diagnostics
    fn error_chain(&self) -> Vec<String>;

    /// Check if error is recoverable
    fn is_recoverable(&self) -> bool;

    /// Get suggested action for the error
    fn suggested_action(&self) -> Option<String>;
}

/// Result extension for better error handling
pub trait ResultExt<T> {
    /// Add context to error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;

    /// Add lazy context (only evaluated on error)
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Log error and continue
    fn log_error_and_continue(self) -> Option<T>;

    /// Convert to GitAI result
    fn to_gitai_result(self) -> Result<T>;

    /// Add user-friendly message
    fn user_message(self, msg: &str) -> Result<T>;

    /// Add technical details for debugging
    fn technical_details(self, details: &str) -> Result<T>;
}

/// Implementation for Box<dyn Error>
impl GitAIErrorExt for Box<dyn std::error::Error + Send + Sync> {
    fn to_gitai_error(self) -> GitAIError {
        // Try to downcast to known error types
        if let Some(io_err) = self.downcast_ref::<std::io::Error>() {
            return GitAIError::from(io_err.to_string());
        }

        // Default to unknown error
        GitAIError::Unknown(self.to_string())
    }

    fn log_error(&self) {
        error!("Error: {}", self);

        // Log the error chain
        let mut current = self.source();
        while let Some(err) = current {
            error!("  Caused by: {}", err);
            current = err.source();
        }
    }

    fn error_chain(&self) -> Vec<String> {
        let mut chain = vec![self.to_string()];
        let mut current = self.source();

        while let Some(err) = current {
            chain.push(err.to_string());
            current = err.source();
        }

        chain
    }

    fn is_recoverable(&self) -> bool {
        // Check for known recoverable error patterns
        let error_str = self.to_string().to_lowercase();

        // Network errors are often recoverable with retry
        if error_str.contains("timeout") || error_str.contains("connection") {
            return true;
        }

        // File not found might be recoverable by creating the file
        if error_str.contains("not found") {
            return true;
        }

        // Default to non-recoverable
        false
    }

    fn suggested_action(&self) -> Option<String> {
        let error_str = self.to_string().to_lowercase();

        if error_str.contains("permission denied") {
            return Some("Check file permissions or run with appropriate privileges".to_string());
        }

        if error_str.contains("not found") {
            return Some("Ensure the file or resource exists".to_string());
        }

        if error_str.contains("timeout") {
            return Some("Check network connection and try again".to_string());
        }

        if error_str.contains("config") {
            return Some("Run 'gitai init' to initialize configuration".to_string());
        }

        None
    }
}

/// Implementation for GitAIError itself
impl GitAIErrorExt for GitAIError {
    fn to_gitai_error(self) -> GitAIError {
        self // Already a GitAIError
    }

    fn log_error(&self) {
        error!("GitAI Error: {}", self);
    }

    fn error_chain(&self) -> Vec<String> {
        vec![self.to_string()]
    }

    fn is_recoverable(&self) -> bool {
        let error_str = self.to_string().to_lowercase();
        error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("not found")
    }

    fn suggested_action(&self) -> Option<String> {
        let error_str = self.to_string().to_lowercase();

        if error_str.contains("permission denied") {
            return Some("Check file permissions or run with appropriate privileges".to_string());
        }

        if error_str.contains("not found") {
            return Some("Ensure the file or resource exists".to_string());
        }

        if error_str.contains("timeout") {
            return Some("Check network connection and try again".to_string());
        }

        if error_str.contains("config") {
            return Some("Run 'gitai init' to initialize configuration".to_string());
        }

        None
    }
}

/// Implementation for anyhow::Error
impl GitAIErrorExt for anyhow::Error {
    fn to_gitai_error(self) -> GitAIError {
        GitAIError::Unknown(self.to_string())
    }

    fn log_error(&self) {
        error!("Error: {}", self);

        // Log the error chain
        for cause in self.chain().skip(1) {
            error!("  Caused by: {}", cause);
        }
    }

    fn error_chain(&self) -> Vec<String> {
        self.chain().map(|e| e.to_string()).collect()
    }

    fn is_recoverable(&self) -> bool {
        // Check the error chain for recoverable patterns
        for cause in self.chain() {
            let error_str = cause.to_string().to_lowercase();
            if error_str.contains("timeout") || error_str.contains("connection") {
                return true;
            }
        }
        false
    }

    fn suggested_action(&self) -> Option<String> {
        // Check the entire error chain for actionable patterns
        for cause in self.chain() {
            let error_str = cause.to_string().to_lowercase();

            if error_str.contains("permission denied") {
                return Some("Check file permissions".to_string());
            }

            if error_str.contains("not found") {
                return Some("Ensure the resource exists".to_string());
            }
        }
        None
    }
}

/// Implementation for Result with Box<dyn Error>
impl<T> ResultExt<T> for std::result::Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", context, e);
            GitAIError::Unknown(msg)
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", f(), e);
            GitAIError::Unknown(msg)
        })
    }

    fn log_error_and_continue(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Error (continuing): {}", e);
                None
            }
        }
    }

    fn to_gitai_result(self) -> Result<T> {
        self.map_err(|e| e.to_gitai_error())
    }

    fn user_message(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            debug!("Technical error: {}", e);
            GitAIError::Unknown(msg.to_string())
        })
    }

    fn technical_details(self, details: &str) -> Result<T> {
        self.map_err(|e| {
            let msg = format!("{}\nTechnical details: {}", e, details);
            GitAIError::Unknown(msg)
        })
    }
}

/// Implementation for Result with GitAIError (already our error type)
impl<T> ResultExt<T> for Result<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", context, e);
            GitAIError::Unknown(msg)
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", f(), e);
            GitAIError::Unknown(msg)
        })
    }

    fn log_error_and_continue(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Error (continuing): {}", e);
                None
            }
        }
    }

    fn to_gitai_result(self) -> Result<T> {
        self // Already is a GitAI result
    }

    fn user_message(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            debug!("Technical error: {}", e);
            GitAIError::Unknown(msg.to_string())
        })
    }

    fn technical_details(self, details: &str) -> Result<T> {
        self.map_err(|e| {
            let msg = format!("{}\nTechnical details: {}", e, details);
            GitAIError::Unknown(msg)
        })
    }
}

/// Implementation for Result with anyhow::Error
impl<T> ResultExt<T> for std::result::Result<T, anyhow::Error> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", context, e);
            GitAIError::Unknown(msg)
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", f(), e);
            GitAIError::Unknown(msg)
        })
    }

    fn log_error_and_continue(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(e) => {
                warn!("Error (continuing): {}", e);
                None
            }
        }
    }

    fn to_gitai_result(self) -> Result<T> {
        self.map_err(|e| e.to_gitai_error())
    }

    fn user_message(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            debug!("Technical error: {}", e);
            GitAIError::Unknown(msg.to_string())
        })
    }

    fn technical_details(self, details: &str) -> Result<T> {
        self.map_err(|e| {
            let msg = format!("{}\nTechnical details: {}", e, details);
            GitAIError::Unknown(msg)
        })
    }
}

/// Macro for easy error context
#[macro_export]
macro_rules! gitai_error {
    ($msg:literal) => {
        $crate::error::GitAIError::Unknown($msg.to_string())
    };
    ($fmt:literal, $($arg:tt)*) => {
        $crate::error::GitAIError::Unknown(format!($fmt, $($arg)*))
    };
}

/// Macro for adding context to results
#[macro_export]
macro_rules! with_context {
    ($result:expr, $msg:literal) => {
        $result.context($msg)
    };
    ($result:expr, $fmt:literal, $($arg:tt)*) => {
        $result.context(format!($fmt, $($arg)*))
    };
}

/// Error recovery helper
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Attempt to recover from an error
    pub fn try_recover<E: GitAIErrorExt>(error: &E) -> Option<RecoveryAction> {
        if !error.is_recoverable() {
            return None;
        }

        let error_str = error.error_chain()[0].to_lowercase();

        if error_str.contains("timeout") {
            return Some(RecoveryAction::Retry { delay_ms: 1000 });
        }

        if error_str.contains("permission denied") {
            return Some(RecoveryAction::RequestPermission);
        }

        if error_str.contains("not found") {
            return Some(RecoveryAction::CreateResource);
        }

        None
    }
}

/// Recovery actions
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation
    Retry { delay_ms: u64 },
    /// Request permission
    RequestPermission,
    /// Create missing resource
    CreateResource,
    /// Use fallback
    UseFallback,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_chain() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(io_error);

        let chain = boxed.error_chain();
        assert!(!chain.is_empty());
        assert!(chain[0].contains("not found"));
    }

    #[test]
    fn test_recoverable_errors() {
        let timeout_error: Box<dyn std::error::Error + Send + Sync> = Box::new(
            std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout"),
        );

        assert!(timeout_error.is_recoverable());

        let permission_error: Box<dyn std::error::Error + Send + Sync> = Box::new(
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied"),
        );

        assert!(!permission_error.is_recoverable());
    }

    #[test]
    fn test_suggested_actions() {
        let not_found: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ));

        let suggestion = not_found.suggested_action();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("exists"));
    }
}
