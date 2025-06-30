use crate::{config::AppConfig, errors::AppError, utils::generate_gitai_help};

use super::{ai::explain_git_command_output, git::passthrough_to_git_with_error_handling};

// Filter out gitai-specific args before querying git help
pub async fn handle_help(
    config: &AppConfig,
    mut args: Vec<String>,
    use_ai: bool,
) -> Result<(), AppError> {
    // 获取 gitie 自定义帮助
    let gitai_help = generate_gitai_help();
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
        println!("{}", generate_gitai_help());
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
                println!("{}", generate_gitai_help());
            }
        }
    } else {
        // 不使用AI，直接显示组合帮助
        println!("{}", &git_help);
    }
    return Ok(());
}
