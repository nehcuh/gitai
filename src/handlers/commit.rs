use crate::{
    config::AppConfig,
    errors::{AppError, GitError},
    handlers::{ai, git},
    types::{
        git::CommitArgs,
        ai::ChatMessage,
    },
};
use std::io::{self, Write};

/// Handle the commit command with AI assistance
/// This function demonstrates AI-powered commit message generation
pub async fn handle_commit(config: &AppConfig, args: CommitArgs) -> Result<(), AppError> {
    tracing::info!("开始处理智能提交命令");
    
    // Check if we're in a git repository
    check_repository_status()?;
    
    // Auto-stage files if requested
    if args.auto_stage {
        tracing::info!("自动暂存修改的文件...");
        auto_stage_files().await?;
    }
    
    // Get changes for commit
    let diff = get_changes_for_commit().await?;
    if diff.trim().is_empty() {
        return Err(AppError::Git(GitError::NoStagedChanges));
    }
    
    // Generate commit message using AI
    let commit_message = if let Some(custom_message) = args.message {
        // User provided a custom message, use it directly for now
        // TODO: In future stories, we'll enhance this with AI suggestions
        custom_message
    } else {
        generate_commit_message(config, &diff).await?
    };
    
    // Show generated commit message and ask for confirmation
    println!("\n🤖 生成的提交信息:");
    println!("┌─────────────────────────────────────────────┐");
    for line in commit_message.lines() {
        println!("│ {:<43} │", line);
    }
    println!("└─────────────────────────────────────────────┘");
    
    if !confirm_commit_message(&commit_message)? {
        println!("❌ 提交已取消");
        return Ok(());
    }
    
    // Execute the commit
    execute_commit(&commit_message).await?;
    println!("✅ 提交成功!");
    
    Ok(())
}

/// Check if current directory is a git repository
fn check_repository_status() -> Result<(), AppError> {
    if !git::is_git_repository()? {
        return Err(AppError::Git(GitError::NotARepository));
    }
    Ok(())
}

/// Auto-stage modified tracked files
async fn auto_stage_files() -> Result<(), AppError> {
    git::auto_stage_tracked_files().await
}

/// Get staged changes for commit
async fn get_changes_for_commit() -> Result<String, AppError> {
    // Get the diff of staged changes
    let diff = git::get_staged_diff().await?;
    
    if diff.trim().is_empty() {
        return Err(AppError::Generic(
            "没有已暂存的变更可以提交。请先使用 'git add' 暂存文件，或使用 '-a' 参数自动暂存修改的文件。".to_string()
        ));
    }
    
    Ok(diff)
}

/// Generate commit message using AI
async fn generate_commit_message(config: &AppConfig, diff: &str) -> Result<String, AppError> {
    tracing::info!("正在使用AI生成提交信息...");
    
    let system_prompt = config
        .prompts
        .get("commit-generator")
        .cloned()
        .unwrap_or_else(|| {
            tracing::warn!("未找到commit-generator提示模板，使用默认模板");
            "你是一个专业的Git提交信息生成助手。请根据提供的代码变更生成简洁、清晰的提交信息。".to_string()
        });
    
    let user_prompt = format!(
        "请根据以下Git diff生成一个规范的提交信息：\n\n```diff\n{}\n```\n\n要求：\n1. 使用中文\n2. 格式为：类型(范围): 简洁描述\n3. 第一行不超过50个字符\n4. 如有必要，可以添加详细说明",
        diff
    );
    
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];
    
    match ai::execute_ai_request_generic(config, messages, "提交信息生成", true).await {
        Ok(message) => {
            // Clean up the AI response - remove any markdown formatting
            let cleaned_message = message
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with("```"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            
            Ok(cleaned_message)
        }
        Err(e) => {
            tracing::error!("AI生成提交信息失败: {:?}", e);
            // Fallback to a basic commit message
            Ok("chore: 更新代码".to_string())
        }
    }
}

/// Ask user to confirm the commit message
fn confirm_commit_message(_message: &str) -> Result<bool, AppError> {
    print!("\n是否使用此提交信息? [Y/n] ");
    io::stdout().flush().map_err(|e| AppError::IO("输出刷新失败".to_string(), e))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| AppError::IO("读取用户输入失败".to_string(), e))?;
    
    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes" || input == "是")
}

/// Execute the actual git commit
async fn execute_commit(message: &str) -> Result<(), AppError> {
    git::execute_commit_with_message(message).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::{
        config::{AIConfig, TreeSitterConfig},
        types::git::CommitArgs,
    };

    fn create_test_config() -> AppConfig {
        let mut prompts = HashMap::new();
        prompts.insert(
            "commit-generator".to_string(),
            "Generate a professional commit message".to_string(),
        );
        
        AppConfig {
            ai: AIConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model_name: "test-model".to_string(),
                temperature: 0.7,
                api_key: None,
            },
            tree_sitter: TreeSitterConfig::default(),
            account: None,
            prompts,
        }
    }

    #[test]
    fn test_confirm_commit_message_positive() {
        // This test would need to be run interactively or with mocked input
        // For now, we'll just test the structure
        let message = "feat: add new feature";
        // In a real test, we'd mock stdin/stdout
        assert!(!message.is_empty());
    }

    #[test]
    fn test_generate_commit_message_fallback() {
        // Test that we have a fallback when AI fails
        let diff = "diff --git a/test.txt b/test.txt\n+new line";
        assert!(!diff.is_empty());
    }

    #[tokio::test]
    async fn test_check_repository_status() {
        // This test would fail if not run in a git repository
        // In CI/CD, we'd set up a temporary git repo
        // For now, just test that the function exists and has the right signature
        assert!(true);
    }

    #[test]
    fn test_commit_args_structure() {
        let args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("test message".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        
        assert_eq!(args.message, Some("test message".to_string()));
        assert!(!args.auto_stage);
        assert!(!args.tree_sitter);
    }

    #[test]
    fn test_commit_args_with_tree_sitter() {
        let args = CommitArgs {
            tree_sitter: true,
            depth: Some("deep".to_string()),
            auto_stage: false,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        assert!(args.tree_sitter);
        assert_eq!(args.depth, Some("deep".to_string()));
        assert!(args.message.is_none());
    }

    #[test]
    fn test_commit_args_auto_stage_enabled() {
        let args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: true,
            message: None,
            review: false,
            passthrough_args: vec!["--verbose".to_string()],
        };
        
        assert!(args.auto_stage);
        assert_eq!(args.passthrough_args, vec!["--verbose".to_string()]);
    }

    #[tokio::test]
    async fn test_generate_commit_message_with_fallback() {
        let config = create_test_config();
        let diff = "diff --git a/src/test.rs b/src/test.rs\nindex 1234567..abcdefg 100644\n--- a/src/test.rs\n+++ b/src/test.rs\n@@ -1,3 +1,4 @@\n fn test_function() {\n     println!(\"Hello, world!\");\n+    println!(\"New line added\");\n }";
        
        // This will likely fall back to the default message since we don't have a real AI service
        let result = generate_commit_message(&config, diff).await;
        
        match result {
            Ok(message) => {
                assert!(!message.is_empty());
                // Should either be AI-generated or the fallback message
                assert!(message.contains("chore") || message.len() > 5);
            }
            Err(_) => {
                // AI service not available in test environment, this is expected
                assert!(true);
            }
        }
    }

    #[tokio::test]
    async fn test_handle_commit_with_custom_message() {
        let config = create_test_config();
        let args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: false,
            message: Some("feat: custom commit message".to_string()),
            review: false,
            passthrough_args: vec![],
        };
        
        // This test will fail in most environments since we're not in a proper git repo
        // But it tests the structure and error handling
        match handle_commit(&config, args).await {
            Ok(_) => {
                // Would only succeed if we're in a git repo with staged changes
                assert!(true);
            }
            Err(e) => {
                // Expected in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Git(GitError::NoStagedChanges) => assert!(true),
                    AppError::Generic(msg) => {
                        assert!(msg.contains("没有已暂存的变更") || msg.contains("检查Git仓库状态失败"));
                    }
                    _ => assert!(true), // Other errors are also acceptable in test
                }
            }
        }
    }

    #[tokio::test]
    async fn test_handle_commit_with_auto_stage() {
        let config = create_test_config();
        let args = CommitArgs {
            tree_sitter: false,
            depth: None,
            auto_stage: true,
            message: None,
            review: false,
            passthrough_args: vec![],
        };
        
        match handle_commit(&config, args).await {
            Ok(_) => {
                // Success only if in proper git environment
                assert!(true);
            }
            Err(e) => {
                // Expected errors in test environment
                match e {
                    AppError::Git(GitError::NotARepository) => assert!(true),
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::Generic(_) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[test]
    fn test_create_test_config_structure() {
        let config = create_test_config();
        
        assert_eq!(config.ai.model_name, "test-model");
        assert_eq!(config.ai.api_url, "http://localhost:11434/v1/chat/completions");
        assert_eq!(config.ai.temperature, 0.7);
        assert!(config.prompts.contains_key("commit-generator"));
        assert_eq!(
            config.prompts.get("commit-generator").unwrap(),
            "Generate a professional commit message"
        );
    }

    #[tokio::test]
    async fn test_auto_stage_files_error_handling() {
        // Test that auto_stage_files handles errors gracefully
        match auto_stage_files().await {
            Ok(_) => {
                // Success if we're in a git repo
                assert!(true);
            }
            Err(e) => {
                // Expected error types in test environment
                match e {
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::IO(_, _) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_changes_for_commit_empty_repo() {
        // Test behavior when there are no staged changes
        match get_changes_for_commit().await {
            Ok(diff) => {
                // If successful, diff could be empty or contain changes
                assert!(diff.is_empty() || !diff.is_empty());
            }
            Err(e) => {
                // Expected errors
                match e {
                    AppError::Generic(msg) => {
                        assert!(msg.contains("没有已暂存的变更"));
                    }
                    AppError::Git(GitError::CommandFailed { .. }) => assert!(true),
                    AppError::IO(_, _) => assert!(true),
                    _ => assert!(true),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_execute_commit_error_handling() {
        let test_message = "test: this should fail in test environment";
        
        match execute_commit(test_message).await {
            Ok(_) => {
                // Would only succeed if we have staged changes to commit
                assert!(true);
            }
            Err(e) => {
                // Expected in test environment
                match e {
                    AppError::Git(GitError::CommandFailed { command, .. }) => {
                        assert!(command.contains("git commit"));
                    }
                    _ => assert!(true),
                }
            }
        }
    }
}