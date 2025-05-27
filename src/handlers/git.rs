use std::process::Command;

use crate::{
    errors::{AppError, GitError},
    types::{general::CommandOutput, git::ReviewArgs},
};

pub fn passthrough_to_git(args: &[String]) -> Result<(), AppError> {
    let result = passthrough_to_git_with_error_handling(args, false)?;

    // 检查状态以保持原始行为一致
    if !result.status.success() {
        return Err(AppError::Git(GitError::PassthroughFailed {
            command: format!("git: {}", args.join(" ")),
            status_code: result.status.code(),
        }));
    }

    Ok(())
}

pub fn passthrough_to_git_with_error_handling(
    args: &[String],
    handle_error: bool,
) -> Result<CommandOutput, AppError> {
    let command_to_run = if args.is_empty() {
        vec!["--help".to_string()]
    } else {
        args.to_vec()
    };
    let cmd_str_log = command_to_run.join(" ");
    tracing::debug!("执行系统 git 命令: git {}", cmd_str_log);

    // 直接执行并获取输出，而不是只获取状态
    let output = Command::new("git")
        .args(&command_to_run)
        .output()
        .map_err(|e| AppError::IO(format!("执行系统 git 命令失败: git {}", cmd_str_log), e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // 如果不需要错误处理或命令成功执行，则直接打印输出
    if !handle_error || output.status.success() {
        // 打印标准输出和错误输出，模拟原始命令的行为
        if !stdout.is_empty() {
            print!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }
    }

    if !output.status.success() {
        tracing::warn!("Git 命令 'git {}' 执行失败: {}", cmd_str_log, output.status);

        // 只有当不需要错误处理时才返回错误
        if !handle_error {
            return Err(AppError::Git(GitError::PassthroughFailed {
                command: format!("git {}", cmd_str_log),
                status_code: output.status.code(),
            }));
        }
        // 当handle_error为true时，我们将继续执行并返回Ok结果
    }

    Ok(CommandOutput {
        stdout,
        stderr,
        status: output.status,
    })
}

/// Extract diff information for review
///
/// This function gets the diff between specified commits or the current staged changes
pub(crate) async fn extract_diff_for_review(args: &ReviewArgs) -> Result<String, AppError> {
    match (&args.commit1, &args.commit2) {
        (Some(commit1), Some(commit2)) => {
            // Compare two specific commits
            tracing::info!("比较两个指定的提交: {} 和 {}", commit1, commit2);
            let diff_args = vec![
                "diff".to_string(),
                format!("{}..{}", commit1, commit2),
                "--".to_string(),
            ];
            let result = passthrough_to_git_with_error_handling(&diff_args, false)?;
            Ok(result.stdout)
        }
        (Some(commit), None) => {
            // Compare one commit with HEAD
            tracing::info!("比较指定的提交与HEAD: {}", commit);
            let diff_args = vec![
                "diff".to_string(),
                format!("{}..HEAD", commit),
                "--".to_string(),
            ];
            let result = passthrough_to_git_with_error_handling(&diff_args, false)?;
            Ok(result.stdout)
        }
        (None, None) => {
            // Check if there are staged changes
            let status_result = passthrough_to_git_with_error_handling(
                &["status".to_string(), "--porcelain".to_string()],
                true,
            )?;

            if status_result.stdout.trim().is_empty() {
                return Err(AppError::Generic(
                    "没有检测到变更，无法执行代码评审。请先暂存(git add)或提交一些变更。"
                        .to_string(),
                ));
            }

            // If no commit specified, use staged changes or unstaged changes
            let has_staged = status_result
                .stdout
                .lines()
                .any(|line| line.starts_with(|c| c == 'M' || c == 'A' || c == 'D' || c == 'R'));

            let diff_args = if has_staged {
                tracing::info!("评审已暂存的变更");
                vec!["diff".to_string(), "--staged".to_string()]
            } else {
                tracing::info!("评审工作区的变更");
                vec!["diff".to_string()]
            };

            let result = passthrough_to_git_with_error_handling(&diff_args, false)?;
            Ok(result.stdout)
        }
        (None, Some(_)) => {
            // This should not happen with the CLI parser, but handle it just in case
            Err(AppError::Generic(
                "如果指定了第二个提交，则必须同时指定第一个提交。".to_string(),
            ))
        }
    }
}
