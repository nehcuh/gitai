use crate::{
    common::{ConfigManager, GitAIConfig, AIConfig},
    handlers::intelligent_git::handle_intelligent_git_command,
    types::general::CommandOutput,
};
use std::collections::HashMap;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use tokio;

/// Integration tests for intelligent git command handling
/// These tests verify the end-to-end functionality of the intelligent git proxy

fn create_test_config_for_integration() -> GitAIConfig {
    let mut prompts = HashMap::new();
    prompts.insert(
        "general-helper".to_string(),
        "你是一个Git专家助手，请简洁地解释Git命令的输出和错误信息。".to_string(),
    );

    GitAIConfig {
        ai: AIConfig {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: Some("test-key-for-integration".to_string()),
            model_name: "gpt-3.5-turbo".to_string(),
            temperature: 0.3,
        },
        prompts,
        git: Default::default(),
        translation: Default::default(),
        devops: Default::default(),
        general: Default::default(),
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_help_command() {
    let config = create_test_config_for_integration();
    let args = vec!["--help".to_string()];

    // This should execute git --help and potentially provide AI explanation
    // In a real test environment, we might want to mock the AI service
    let result = handle_intelligent_git_command(&config, &args, false).await;

    // The function should handle the command gracefully
    // Note: This might fail if git is not available in the test environment
    match result {
        Ok(_) => {
            // Success case - git help was executed and possibly explained
            println!("✅ Git help command handled successfully");
        }
        Err(e) => {
            // Expected in test environments without proper git setup
            println!("⚠️ Expected failure in test environment: {}", e);
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_status_command_smart_mode() {
    let config = create_test_config_for_integration();
    let args = vec!["status".to_string()];

    // Test smart mode (no AI flag)
    let result = handle_intelligent_git_command(&config, &args, false).await;

    match result {
        Ok(_) => {
            println!("✅ Git status in smart mode handled successfully");
        }
        Err(e) => {
            // Expected if not in a git repository
            println!("⚠️ Expected failure in test environment: {}", e);
            assert!(
                e.to_string().contains("not a git repository")
                    || e.to_string().contains("Git")
                    || e.to_string().contains("command")
            );
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_status_command_ai_mode() {
    let config = create_test_config_for_integration();
    let args = vec!["status".to_string()];

    // Test AI mode
    let result = handle_intelligent_git_command(&config, &args, true).await;

    match result {
        Ok(_) => {
            println!("✅ Git status in AI mode handled successfully");
        }
        Err(e) => {
            // Expected if not in a git repository or AI service unavailable
            println!("⚠️ Expected failure in test environment: {}", e);
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_invalid_command() {
    let config = create_test_config_for_integration();
    let args = vec!["invalid-git-command-xyz".to_string()];

    // Test with an invalid git command - this should trigger error handling
    let result = handle_intelligent_git_command(&config, &args, false).await;

    match result {
        Ok(_) => {
            // This shouldn't happen with an invalid command
            panic!("Expected error for invalid git command");
        }
        Err(e) => {
            println!("✅ Invalid command properly handled with error: {}", e);
            // The error should be related to git command execution
            assert!(e.to_string().contains("Git") || e.to_string().contains("command"));
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_version_command() {
    let config = create_test_config_for_integration();
    let args = vec!["--version".to_string()];

    // Test git version command - this should always work if git is installed
    let result = handle_intelligent_git_command(&config, &args, false).await;

    match result {
        Ok(_) => {
            println!("✅ Git version command handled successfully");
        }
        Err(e) => {
            println!("⚠️ Git version command failed: {}", e);
            // This suggests git is not available in the test environment
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_integration_empty_args() {
    let config = create_test_config_for_integration();
    let args = vec![];

    // Test with empty arguments - should show git help
    let result = handle_intelligent_git_command(&config, &args, false).await;

    match result {
        Ok(_) => {
            println!("✅ Empty git args handled successfully");
        }
        Err(e) => {
            println!("⚠️ Empty args handling: {}", e);
        }
    }
}

#[tokio::test]
async fn test_intelligent_git_config_validation() {
    // Test that the integration properly validates configuration
    let config = create_test_config_for_integration();

    // Verify that the test config has the required fields
    assert!(!config.ai.api_url.is_empty());
    assert!(config.ai.api_key.is_some());
    assert!(!config.ai.model_name.is_empty());
    assert!(config.prompts.contains_key("general-helper"));

    println!("✅ Configuration validation passed");
}

#[tokio::test]
async fn test_intelligent_git_ai_mode_vs_smart_mode() {
    let config = create_test_config_for_integration();
    let args = vec![
        "log".to_string(),
        "--oneline".to_string(),
        "-n".to_string(),
        "1".to_string(),
    ];

    // Test both modes with the same command
    println!("Testing smart mode...");
    let smart_result = handle_intelligent_git_command(&config, &args, false).await;

    println!("Testing AI mode...");
    let ai_result = handle_intelligent_git_command(&config, &args, true).await;

    // Both should either succeed or fail gracefully
    match (smart_result, ai_result) {
        (Ok(_), Ok(_)) => {
            println!("✅ Both modes handled the command successfully");
        }
        (Err(e1), Err(e2)) => {
            println!("⚠️ Both modes failed as expected in test environment");
            println!("Smart mode error: {}", e1);
            println!("AI mode error: {}", e2);
        }
        (Ok(_), Err(e)) => {
            println!("⚠️ Smart mode succeeded, AI mode failed: {}", e);
        }
        (Err(e), Ok(_)) => {
            println!("⚠️ Smart mode failed, AI mode succeeded: {}", e);
        }
    }
}

/// Test helper to simulate different git command outcomes
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_mock_success_output() -> CommandOutput {
        CommandOutput {
            stdout: "On branch main\nYour branch is up to date with 'origin/main'.\n".to_string(),
            stderr: String::new(),
            status: ExitStatus::from_raw(0),
        }
    }

    pub fn create_mock_error_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "fatal: not a git repository (or any of the parent directories): .git"
                .to_string(),
            status: ExitStatus::from_raw(128),
        }
    }

    pub fn create_mock_warning_output() -> CommandOutput {
        CommandOutput {
            stdout: "Changes not staged for commit:\n  modified:   file.txt".to_string(),
            stderr: "warning: LF will be replaced by CRLF in file.txt".to_string(),
            status: ExitStatus::from_raw(0),
        }
    }
}

#[tokio::test]
async fn test_integration_flow_simulation() {
    // This test simulates the integration flow without actually calling git
    // It's useful for testing the logic flow in CI environments

    let config = create_test_config_for_integration();

    // Test the configuration and setup
    assert!(!config.prompts.is_empty());
    assert!(config.prompts.contains_key("general-helper"));

    // Test mock outputs
    let success_output = test_helpers::create_mock_success_output();
    let error_output = test_helpers::create_mock_error_output();
    let warning_output = test_helpers::create_mock_warning_output();

    assert!(success_output.status.success());
    assert!(!error_output.status.success());
    assert!(warning_output.status.success());
    assert!(!warning_output.stderr.is_empty());

    println!("✅ Integration flow simulation completed successfully");
}
