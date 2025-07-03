use crate::{config::AppConfig, errors::AppError};

// Placeholder implementation for generate_gitai_help
/// Returns a static help message describing GitAI usage and available commands.
///
/// The help text includes basic usage instructions and lists primary commands such as `commit` and `review`.
///
/// # Examples
///
/// ```
/// let help = generate_gitai_help(&config);
/// assert!(help.contains("Usage: gitai"));
/// ```
fn generate_gitai_help(_config: &AppConfig) -> String {
    "GitAI Help (placeholder)\n\
    Usage: gitai <command> [options]\n\
    \n\
    Commands:\n\
    commit      Generate a commit message with AI.\n\
    review      Review code changes with AI.\n\
    ... and more."
        .to_string()
}

use super::{ai::explain_git_command_output, git::passthrough_to_git_with_error_handling};

// Filter out gitai-specific args before querying git help
/// Handles help commands for the CLI, displaying either custom GitAI help or forwarding to git help output.
///
/// Depending on the provided arguments, prints GitAI-specific help for commands like `review` or `commit` with special flags, or passes the help request to the underlying git command. Optionally, uses AI to explain the git help output if enabled.
///
/// # Arguments
///
/// * `args` - Command-line arguments to determine which help text to display.
/// * `use_ai` - If true, uses AI to explain the git help output; otherwise, prints the raw help.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an `AppError` if invoking git help fails.
///
/// # Examples
///
/// ```
/// // Display help for the 'review' command
/// handle_help(&config, vec!["review".to_string()], false).await?;
///
/// // Forward help to git and use AI explanation
/// handle_help(&config, vec!["commit".to_string()], true).await?;
/// ```
pub async fn handle_help(
    config: &AppConfig,
    mut args: Vec<String>,
    use_ai: bool,
) -> Result<(), AppError> {
    // 获取 gitie 自定义帮助
    let gitai_help = generate_gitai_help(config);
    if args.iter().any(|arg| arg == "review" || arg == "rv") {
        println!("{}", gitai_help);
        return Ok(());
    }

    // 继续过滤当入参包含 commit 时，可能的 gitai 特有指令
    if args.iter().any(|arg| arg == "commit" || arg == "cm") {
        if args.iter().any(|arg| {
            arg == "-t"
                || arg == "--ast-grep"
                || arg == "-l"
                || arg == "--level"
                || arg == "-r"
                || arg == "--review"
        }) {
            println!("{}", gitai_help);
            return Ok(());
        }
    }

    // 获取完整的git帮助文本
    args.retain(|arg| arg != "-h" && arg != "--help");
    if args.len() <= 1 {
        println!("{}", generate_gitai_help(config));
        return Ok(());
    }
    args.push("--help".to_string());
    let git_help = match passthrough_to_git_with_error_handling(&args[1..], true) {
        Ok(output) => output.stdout,
        Err(e) => {
            eprintln!("获取git帮助信息失败: {}", e);
            return Err(e);
        }
    };

    if use_ai {
        // 使用AI解释帮助内容
        match explain_git_command_output(&config, &git_help).await {
            Ok(explanation) => {
                // 输出AI解释和原始帮助
                println!("{}", explanation);
            }
            Err(e) => {
                tracing::warn!("无法获取AI帮助解释: {}", e);
                // 如果AI解释失败，仍然显示原始帮助
                println!("{}", generate_gitai_help(config));
            }
        }
    } else {
        // 不使用AI，直接显示组合帮助
        println!("{}", &git_help);
    }
    return Ok(());
}
