//! Demonstration of enhanced error handling features
//!
//! This example shows how the new error handling system provides better
//! error messages, recovery suggestions, and logging.

use gitai::error::{GitAIError, Result};
use gitai::error_ext::{ErrorRecovery, GitAIErrorExt, RecoveryAction, ResultExt};
use std::fs;
use std::path::Path;

/// Simulated file operation that might fail
fn read_config_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        .context(format!("Failed to read configuration from {}", path.display()))
        .user_message("Unable to load configuration. Please run 'gitai init' to create a default configuration.")
}

/// Simulated network operation
fn fetch_remote_data(url: &str) -> Result<String> {
    // Simulate a timeout error
    if url.contains("slow") {
        return Err(GitAIError::Unknown("Connection timeout".to_string()));
    }

    // Simulate success
    Ok("Remote data".to_string())
}

/// Operation with recovery
fn operation_with_recovery() -> Result<String> {
    let result = fetch_remote_data("https://slow.example.com");

    if let Err(ref e) = result {
        // Check if error is recoverable
        if e.is_recoverable() {
            println!("Error is recoverable!");

            // Get recovery suggestion
            if let Some(suggestion) = e.suggested_action() {
                println!("Suggestion: {suggestion}");
            }

            // Try recovery action
            if let Some(action) = ErrorRecovery::try_recover(e) {
                match action {
                    RecoveryAction::Retry { delay_ms } => {
                        println!("Retrying after {delay_ms}ms delay...");
                        // In real code, you would sleep and retry
                        return fetch_remote_data("https://fast.example.com");
                    }
                    _ => println!("Other recovery action: {action:?}"),
                }
            }
        }
    }

    result
}

/// Demonstrate error chaining
fn demonstrate_error_chain() {
    println!("\n=== Error Chain Demonstration ===");

    let result = read_config_file(Path::new("/nonexistent/config.toml"));

    if let Err(e) = result {
        println!("Error occurred: {e}");

        // Show full error chain
        let chain = e.error_chain();
        println!("\nError chain ({} levels):", chain.len());
        for (i, cause) in chain.iter().enumerate() {
            println!("  {i}: {cause}");
        }

        // Log the error (would go to log file in production)
        e.log_error();
    }
}

/// Demonstrate user-friendly messages
fn demonstrate_user_messages() {
    println!("\n=== User-Friendly Messages ===");

    // Technical error
    let technical_result: Result<()> = Err(GitAIError::Unknown(
        "Failed to parse AST node at position 42 in tree-sitter analysis".to_string(),
    ));

    // Convert to user-friendly message
    let user_result = technical_result
        .user_message("Code analysis failed. Please ensure your code syntax is correct.");

    if let Err(e) = user_result {
        println!("User sees: {e}");
    }
}

/// Demonstrate recovery suggestions
fn demonstrate_recovery() {
    println!("\n=== Recovery Demonstration ===");

    let result = operation_with_recovery();

    match result {
        Ok(data) => println!("Success: {data}"),
        Err(e) => {
            println!("Final error: {e}");
            if !e.is_recoverable() {
                println!("Error is not recoverable");
            }
        }
    }
}

/// Demonstrate technical details
fn demonstrate_technical_details() {
    println!("\n=== Technical Details ===");

    let result: Result<()> = fs::create_dir("/root/test")
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        .context("Failed to create directory")
        .technical_details("Ensure you have write permissions to /root or run with sudo");

    if let Err(e) = result {
        println!("Error with details: {e}");
    }
}

fn main() {
    println!("GitAI Enhanced Error Handling Demo");
    println!("===================================");

    demonstrate_error_chain();
    demonstrate_user_messages();
    demonstrate_recovery();
    demonstrate_technical_details();

    println!("\n=== Demo Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_has_context() {
        let result = read_config_file(Path::new("/nonexistent"));
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_str = err.to_string();
        // user_message takes precedence in display; ensure it is present
        assert!(err_str.contains("Unable to load configuration"));
    }

    #[test]
    fn test_recovery_detection() {
        let timeout_err = GitAIError::Unknown("Connection timeout".to_string());
        assert!(timeout_err.is_recoverable());

        let permission_err = GitAIError::Unknown("Permission denied".to_string());
        assert!(!permission_err.is_recoverable()); // Not marked as recoverable in our implementation
    }
}
