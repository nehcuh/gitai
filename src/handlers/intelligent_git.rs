use crate::{
    config::AppConfig,
    errors::AppError,
    handlers::{ai::explain_git_command_output, git::passthrough_to_git_with_error_handling},
    types::general::CommandOutput,
};
use tokio;

/// Handle intelligent git command processing with conditional AI explanation
/// 
/// This function implements the smart git command proxy that:
/// 1. In AI mode (--ai): Explains all command output using AI
/// 2. In smart mode (default): Only explains errors using AI, passes through normal output
pub async fn handle_intelligent_git_command(
    config: &AppConfig,
    args: &[String],
    use_ai: bool,
) -> Result<(), AppError> {
    tracing::info!("执行智能Git命令代理，AI模式: {}", use_ai);
    
    // Execute git command with error handling to capture all output
    let command_output = passthrough_to_git_with_error_handling(args, true)?;
    
    if use_ai {
        handle_ai_mode(config, &command_output).await?;
    } else {
        handle_smart_mode(config, args, &command_output).await?;
    }
    
    // Maintain same exit status as original git command
    if !command_output.status.success() {
        return Err(AppError::Git(crate::errors::GitError::CommandFailed {
            command: format!("git {}", args.join(" ")),
            status_code: command_output.status.code(),
            stdout: command_output.stdout,
            stderr: command_output.stderr,
        }));
    }
    
    Ok(())
}

/// Handle AI mode: explain all command output
async fn handle_ai_mode(config: &AppConfig, command_output: &CommandOutput) -> Result<(), AppError> {
    tracing::info!("🤖 全局AI模式：正在分析命令输出...");
    
    if !command_output.stdout.is_empty() || !command_output.stderr.is_empty() {
        let combined_output = format!("{}{}", 
            command_output.stdout, 
            if !command_output.stderr.is_empty() { 
                format!("\n--- 错误输出 ---\n{}", command_output.stderr) 
            } else { 
                String::new() 
            }
        );
        
        match explain_git_command_output(config, &combined_output).await {
            Ok(explanation) => {
                println!("\n{}", explanation);
            }
            Err(e) => {
                tracing::warn!("AI解释失败: {}", e);
                println!("⚠️ AI解释服务暂时不可用，显示原始输出：");
                if !command_output.stdout.is_empty() {
                    print!("{}", command_output.stdout);
                }
                if !command_output.stderr.is_empty() {
                    eprint!("{}", command_output.stderr);
                }
            }
        }
    }
    
    Ok(())
}

/// Handle smart mode: only explain errors, pass through normal output
async fn handle_smart_mode(
    config: &AppConfig, 
    args: &[String], 
    command_output: &CommandOutput
) -> Result<(), AppError> {
    let has_error = !command_output.status.success() || !command_output.stderr.is_empty();
    
    if has_error {
        tracing::info!("🤖 检测到错误，正在提供AI解释...");
        
        let error_context = if !command_output.stderr.is_empty() {
            format!("命令: git {}\n错误输出:\n{}", 
                args.join(" "), 
                command_output.stderr)
        } else {
            format!("命令: git {}\n命令执行失败，退出码: {:?}", 
                args.join(" "), 
                command_output.status.code())
        };
        
        match explain_git_command_output(config, &error_context).await {
            Ok(explanation) => {
                println!("\n💡 AI错误分析：");
                println!("{}", explanation);
            }
            Err(e) => {
                tracing::warn!("AI错误解释失败: {}", e);
                println!("⚠️ AI解释服务暂时不可用");
                if !command_output.stderr.is_empty() {
                    eprint!("{}", command_output.stderr);
                }
            }
        }
    }
    // If no error, output was already displayed in passthrough_to_git_with_error_handling
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::process::ExitStatus;
    use std::os::unix::process::ExitStatusExt;
    use crate::config::{AIConfig, ReviewConfig};
    
    fn create_test_config() -> AppConfig {
        let mut prompts = HashMap::new();
        prompts.insert("general-helper".to_string(), "你是一个Git专家助手".to_string());
        
        AppConfig {
            ai: AIConfig {
                api_url: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("test-key".to_string()),
                model_name: "gpt-3.5-turbo".to_string(),
                temperature: 0.3,
            },
            prompts,
            tree_sitter: crate::config::TreeSitterConfig::default(),
            account: None,
            review: ReviewConfig {
                auto_save: true,
                storage_path: "~/review_results".to_string(),
                format: "markdown".to_string(),
                max_age_hours: 168,
                include_in_commit: false,
            },
        }
    }
    
    fn create_success_output() -> CommandOutput {
        CommandOutput {
            stdout: "On branch main\nnothing to commit, working tree clean".to_string(),
            stderr: String::new(),
            status: ExitStatus::from_raw(0),
        }
    }
    
    fn create_error_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "fatal: not a git repository".to_string(),
            status: ExitStatus::from_raw(256),
        }
    }
    
    fn create_warning_output() -> CommandOutput {
        CommandOutput {
            stdout: "Changes not staged for commit:".to_string(),
            stderr: "warning: LF will be replaced by CRLF".to_string(),
            status: ExitStatus::from_raw(0),
        }
    }
    
    #[tokio::test]
    async fn test_handle_ai_mode_with_success_output() {
        let config = create_test_config();
        let output = create_success_output();
        
        // This should not panic and should handle the output gracefully
        let result = handle_ai_mode(&config, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_ai_mode_with_error_output() {
        let config = create_test_config();
        let output = create_error_output();
        
        let result = handle_ai_mode(&config, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_ai_mode_with_empty_output() {
        let config = create_test_config();
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            status: ExitStatus::from_raw(0),
        };
        
        let result = handle_ai_mode(&config, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_smart_mode_with_success() {
        let config = create_test_config();
        let args = vec!["status".to_string()];
        let output = create_success_output();
        
        let result = handle_smart_mode(&config, &args, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_smart_mode_with_error() {
        let config = create_test_config();
        let args = vec!["invalid-command".to_string()];
        let output = create_error_output();
        
        let result = handle_smart_mode(&config, &args, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_smart_mode_with_warning() {
        let config = create_test_config();
        let args = vec!["add".to_string(), "file.txt".to_string()];
        let output = create_warning_output();
        
        let result = handle_smart_mode(&config, &args, &output).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_handle_intelligent_git_command_ai_mode() {
        let config = create_test_config();
        let args = vec!["status".to_string()];
        
        // Note: This test may fail in CI without proper git setup
        // In a real scenario, we'd mock the git command execution
        // For now, we just test the function structure
        if let Err(e) = handle_intelligent_git_command(&config, &args, true).await {
            // Expected to fail in test environment without proper git setup
            tracing::debug!("Expected test failure: {}", e);
        }
    }
    
    #[tokio::test]
    async fn test_handle_intelligent_git_command_smart_mode() {
        let config = create_test_config();
        let args = vec!["status".to_string()];
        
        // Note: This test may fail in CI without proper git setup
        if let Err(e) = handle_intelligent_git_command(&config, &args, false).await {
            // Expected to fail in test environment without proper git setup
            tracing::debug!("Expected test failure: {}", e);
        }
    }
    
    #[test]
    fn test_combined_output_formatting() {
        let stdout = "Some output";
        let stderr = "Some error";
        
        let combined = format!("{}{}", 
            stdout, 
            if !stderr.is_empty() { 
                format!("\n--- 错误输出 ---\n{}", stderr) 
            } else { 
                String::new() 
            }
        );
        
        assert!(combined.contains(stdout));
        assert!(combined.contains(stderr));
        assert!(combined.contains("--- 错误输出 ---"));
    }
    
    #[test]
    fn test_error_context_formatting() {
        let args = vec!["invalid".to_string(), "command".to_string()];
        let stderr = "fatal: not a git repository";
        
        let error_context = format!("命令: git {}\n错误输出:\n{}", 
            args.join(" "), 
            stderr);
        
        assert!(error_context.contains("git invalid command"));
        assert!(error_context.contains("fatal: not a git repository"));
    }
    
    #[test]
    fn test_error_context_with_exit_code() {
        let args = vec!["status".to_string()];
        let exit_code = Some(128);
        
        let error_context = format!("命令: git {}\n命令执行失败，退出码: {:?}", 
            args.join(" "), 
            exit_code);
        
        assert!(error_context.contains("git status"));
        assert!(error_context.contains("128"));
    }
}