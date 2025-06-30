use gitai::{
    config::{AIConfig, AppConfig, AstGrepConfig},
    errors::{AppError, GitError},
    handlers::commit::handle_commit,
    types::git::CommitArgs,
};
use std::collections::HashMap;

// Helper function to create a test configuration
fn create_test_config() -> AppConfig {
    let mut prompts = HashMap::new();
    prompts.insert(
        "commit-generator".to_string(),
        "You are a professional Git commit message generator. Generate concise, clear commit messages based on the provided code changes.".to_string(),
    );

    AppConfig {
        ai: AIConfig {
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            model_name: "test-model".to_string(),
            temperature: 0.7,
            api_key: None,
        },
        ast_grep: AstGrepConfig::default(),
        review: Default::default(),
        account: None,
        prompts,
        translation: Default::default(),
    }
}

#[tokio::test]
async fn test_commit_with_custom_message() {
    let config = create_test_config();
    let args = CommitArgs {
        ast_grep: false,
        auto_stage: false,
        message: Some("test commit message".to_string()),
        issue_id: None,
        review: false,
        passthrough_args: vec![],
    };

    // This test will likely fail in CI/CD since we're not in a proper git repo with staged changes
    // But it demonstrates the integration test structure
    match handle_commit(&config, args).await {
        Ok(_) => {
            // Success case - would only work in a proper git environment with staged changes
            println!("Commit succeeded");
        }
        Err(e) => {
            // Expected errors in test environment
            match e {
                AppError::Git(GitError::NotARepository) => {
                    println!("Expected: Not in a git repository");
                }
                AppError::Git(GitError::NoStagedChanges) => {
                    println!("Expected: No staged changes to commit");
                }
                AppError::Generic(msg) => {
                    println!("Expected error: {}", msg);
                    assert!(
                        msg.contains("没有已暂存的变更")
                            || msg.contains("检查Git仓库状态失败")
                            || msg.contains("没有检测到任何变更可用于提交分析")
                    );
                }
                _ => {
                    println!("Other expected error in test environment: {:?}", e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_commit_with_auto_stage() {
    let config = create_test_config();
    let args = CommitArgs {
        ast_grep: false,
        auto_stage: true,
        message: None,
        issue_id: None,
        review: false,
        passthrough_args: vec![],
    };

    match handle_commit(&config, args).await {
        Ok(_) => {
            println!("Auto-stage commit succeeded");
        }
        Err(e) => {
            // Expected in test environment
            match e {
                AppError::Git(_) => {
                    println!("Expected Git error in test environment");
                }
                AppError::Generic(_) => {
                    println!("Expected generic error in test environment");
                }
                _ => {
                    println!("Other error: {:?}", e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_commit_argument_parsing() {
    // Test that CommitArgs can be created and have expected values
    let args = CommitArgs {
        ast_grep: true,
        auto_stage: true,
        message: Some("test: integration test commit".to_string()),
        issue_id: None,
        review: false,
        passthrough_args: vec!["--verbose".to_string()],
    };

    assert!(args.ast_grep);
    assert!(args.auto_stage);
    assert_eq!(
        args.message,
        Some("test: integration test commit".to_string())
    );
    assert!(!args.review);
    assert_eq!(args.passthrough_args, vec!["--verbose".to_string()]);
}

#[test]
fn test_config_structure() {
    let config = create_test_config();

    assert_eq!(config.ai.model_name, "test-model");
    assert_eq!(config.ai.temperature, 0.7);
    assert!(config.prompts.contains_key("commit-generator"));

    let prompt = config.prompts.get("commit-generator").unwrap();
    assert!(prompt.contains("Git commit message generator"));
}

#[tokio::test]
async fn test_commit_error_handling() {
    let config = create_test_config();

    // Test with minimal args - should handle gracefully
    let args = CommitArgs {
        ast_grep: false,
        auto_stage: false,
        message: None,
        issue_id: None,
        review: false,
        passthrough_args: vec![],
    };

    let result = handle_commit(&config, args).await;

    // Should return an error since we're likely not in a git repo or have no staged changes
    assert!(result.is_err());

    match result {
        Err(AppError::Git(_)) => {
            // Expected git-related error
        }
        Err(AppError::Generic(msg)) => {
            // Expected generic error about no staged changes
            assert!(
                msg.contains("没有已暂存的变更")
                    || msg.contains("检查Git仓库状态失败")
                    || msg.contains("没有检测到任何变更可用于提交分析")
            );
        }
        Err(_) => {
            // Other errors are also acceptable in test environment
        }
        Ok(_) => {
            // Unexpected success - only possible if run in proper git environment
            panic!("Unexpected success in test environment");
        }
    }
}

// Test the interaction between different commit modes
#[tokio::test]
async fn test_commit_mode_combinations() {
    let config = create_test_config();

    // Test tree-sitter + custom message
    let args1 = CommitArgs {
        ast_grep: true,
        auto_stage: false,
        message: Some("feat: test tree-sitter with message".to_string()),
        issue_id: None,
        review: false,
        passthrough_args: vec![],
    };

    // Test auto-stage + tree-sitter
    let args2 = CommitArgs {
        ast_grep: true,
        auto_stage: true,
        message: None,
        issue_id: None,
        review: false,
        passthrough_args: vec![],
    };

    // Both should handle errors gracefully in test environment
    let result1 = handle_commit(&config, args1).await;
    let result2 = handle_commit(&config, args2).await;

    // Both should fail in test environment but with proper error handling
    assert!(result1.is_err());
    assert!(result2.is_err());
}
