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

/// Check if current directory is a git repository
pub fn is_git_repository() -> Result<bool, AppError> {
    let result = Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|e| AppError::IO("检查Git仓库状态失败".to_string(), e))?;
    
    Ok(result.status.success())
}

/// Get the status of staged files (names only)
pub async fn get_staged_files_status() -> Result<String, AppError> {
    let args = vec!["diff".to_string(), "--cached".to_string(), "--name-only".to_string()];
    let result = passthrough_to_git_with_error_handling(&args, true)?;
    Ok(result.stdout)
}

/// Get comprehensive Git repository status
pub async fn get_repository_status() -> Result<String, AppError> {
    get_repository_status_in_dir(None).await
}

/// Get comprehensive Git repository status in specified directory
pub async fn get_repository_status_in_dir(dir: Option<&str>) -> Result<String, AppError> {
    let mut args = vec![];
    if let Some(directory) = dir {
        args.extend(vec!["-C".to_string(), directory.to_string()]);
    }
    args.extend(vec!["status".to_string(), "--porcelain".to_string()]);
    let result = passthrough_to_git_with_error_handling(&args, true)?;
    Ok(result.stdout)
}

/// Format Git status into human-readable format
pub async fn get_formatted_repository_status() -> Result<String, AppError> {
    get_formatted_repository_status_in_dir(None).await
}

/// Format Git status into human-readable format for specified directory
pub async fn get_formatted_repository_status_in_dir(dir: Option<&str>) -> Result<String, AppError> {
    let status_output = get_repository_status_in_dir(dir).await?;
    
    if status_output.trim().is_empty() {
        return Ok("🌟 工作目录干净，没有未跟踪的文件".to_string());
    }
    
    let mut staged_files = Vec::new();
    let mut unstaged_files = Vec::new();
    let mut untracked_files = Vec::new();
    
    for line in status_output.lines() {
        if line.len() < 3 {
            continue;
        }
        
        let staged_status = line.chars().nth(0).unwrap_or(' ');
        let unstaged_status = line.chars().nth(1).unwrap_or(' ');
        let file_path = &line[3..];
        
        // Check staged changes
        if staged_status != ' ' && staged_status != '?' {
            let status_desc = match staged_status {
                'A' => "新增",
                'M' => "修改",
                'D' => "删除",
                'R' => "重命名",
                'C' => "复制",
                _ => "变更",
            };
            staged_files.push(format!("  {} {}", status_desc, file_path));
        }
        
        // Check unstaged changes
        if unstaged_status != ' ' && unstaged_status != '?' {
            let status_desc = match unstaged_status {
                'M' => "修改",
                'D' => "删除",
                _ => "变更",
            };
            unstaged_files.push(format!("  {} {}", status_desc, file_path));
        }
        
        // Check untracked files
        if staged_status == '?' && unstaged_status == '?' {
            untracked_files.push(format!("  {}", file_path));
        }
    }
    
    let mut result = String::new();
    
    if !staged_files.is_empty() {
        result.push_str("📋 暂存的更改:\n");
        result.push_str(&staged_files.join("\n"));
        result.push('\n');
    }
    
    if !unstaged_files.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("📝 未暂存的更改:\n");
        result.push_str(&unstaged_files.join("\n"));
        result.push('\n');
    }
    
    if !untracked_files.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("❓ 未跟踪的文件:\n");
        result.push_str(&untracked_files.join("\n"));
    }
    
    Ok(result)
}

/// Get diff of staged changes
pub async fn get_staged_diff() -> Result<String, AppError> {
    get_staged_diff_in_dir(None).await
}

/// Get diff of staged changes in specified directory
pub async fn get_staged_diff_in_dir(dir: Option<&str>) -> Result<String, AppError> {
    let mut args = vec![];
    if let Some(directory) = dir {
        args.extend(vec!["-C".to_string(), directory.to_string()]);
    }
    args.extend(vec!["diff".to_string(), "--cached".to_string()]);
    let result = passthrough_to_git_with_error_handling(&args, true)?;
    Ok(result.stdout)
}

/// Get diff of unstaged changes only
pub async fn get_unstaged_diff() -> Result<String, AppError> {
    get_unstaged_diff_in_dir(None).await
}

/// Get diff of unstaged changes only in specified directory
pub async fn get_unstaged_diff_in_dir(dir: Option<&str>) -> Result<String, AppError> {
    let mut args = vec![];
    if let Some(directory) = dir {
        args.extend(vec!["-C".to_string(), directory.to_string()]);
    }
    args.extend(vec!["diff".to_string()]);
    let result = passthrough_to_git_with_error_handling(&args, true)?;
    Ok(result.stdout)
}

/// Get diff for commit analysis (staged changes with fallback to unstaged)
pub async fn get_diff_for_commit() -> Result<String, AppError> {
    // First try to get staged changes
    let staged_diff = get_staged_diff().await?;
    
    if !staged_diff.trim().is_empty() {
        tracing::debug!("使用已暂存的变更进行提交分析");
        return Ok(staged_diff);
    }
    
    // If no staged changes, check for unstaged changes
    let args = vec!["diff".to_string()];
    let result = passthrough_to_git_with_error_handling(&args, true)?;
    
    if !result.stdout.trim().is_empty() {
        tracing::debug!("使用未暂存的变更进行提交分析");
        return Ok(result.stdout);
    }
    
    Err(AppError::Generic(
        "没有检测到任何变更可用于提交分析".to_string()
    ))
}

/// Auto-stage tracked modified files
pub async fn auto_stage_tracked_files() -> Result<(), AppError> {
    let args = vec!["add".to_string(), "-u".to_string()];
    let result = passthrough_to_git_with_error_handling(&args, false)?;
    
    if !result.status.success() {
        return Err(AppError::Git(GitError::CommandFailed {
            command: "git add -u".to_string(),
            status_code: result.status.code(),
            stdout: result.stdout,
            stderr: result.stderr,
        }));
    }
    
    Ok(())
}

/// Execute git commit with message
pub async fn execute_commit_with_message(message: &str) -> Result<(), AppError> {
    let args = vec!["commit".to_string(), "-m".to_string(), message.to_string()];
    let result = passthrough_to_git_with_error_handling(&args, false)?;
    
    if !result.status.success() {
        return Err(AppError::Git(GitError::CommandFailed {
            command: format!("git commit -m \"{}\"", message),
            status_code: result.status.code(),
            stdout: result.stdout,
            stderr: result.stderr,
        }));
    }
    
    Ok(())
}

/// Extract diff information for review
///
/// This function gets the diff between specified commits or the current staged changes
pub(crate) async fn extract_diff_for_review(args: &ReviewArgs) -> Result<String, AppError> {
    extract_diff_for_review_in_dir(args, None).await
}

/// Extract diff information for review in specified directory
///
/// This function gets the diff between specified commits or the current staged changes
pub(crate) async fn extract_diff_for_review_in_dir(args: &ReviewArgs, dir: Option<&str>) -> Result<String, AppError> {
    match (&args.commit1, &args.commit2) {
        (Some(commit1), Some(commit2)) => {
            // Compare two specific commits
            tracing::info!("比较两个指定的提交: {} 和 {}", commit1, commit2);
            let mut diff_args = vec![];
            if let Some(directory) = dir {
                diff_args.extend(vec!["-C".to_string(), directory.to_string()]);
            }
            diff_args.extend(vec![
                "diff".to_string(),
                format!("{}..{}", commit1, commit2),
                "--".to_string(),
            ]);
            let result = passthrough_to_git_with_error_handling(&diff_args, false)?;
            Ok(result.stdout)
        }
        (Some(commit), None) => {
            // Compare one commit with HEAD
            tracing::info!("比较指定的提交与HEAD: {}", commit);
            let mut diff_args = vec![];
            if let Some(directory) = dir {
                diff_args.extend(vec!["-C".to_string(), directory.to_string()]);
            }
            diff_args.extend(vec![
                "diff".to_string(),
                format!("{}..HEAD", commit),
                "--".to_string(),
            ]);
            let result = passthrough_to_git_with_error_handling(&diff_args, false)?;
            Ok(result.stdout)
        }
        (None, None) => {
            // Check if there are staged changes
            let mut status_args = vec![];
            if let Some(directory) = dir {
                status_args.extend(vec!["-C".to_string(), directory.to_string()]);
            }
            status_args.extend(vec!["status".to_string(), "--porcelain".to_string()]);
            let status_result = passthrough_to_git_with_error_handling(&status_args, true)?;

            if status_result.stdout.trim().is_empty() {
                return Err(AppError::Generic(
                    "没有检测到变更，无法执行代码评审。请先暂存(git add)或提交一些变更。"
                        .to_string(),
                ));
            }

            // If no commit specified, use staged changes or unstaged changes
            // In git status --porcelain output:
            // - First character shows staged status
            // - Second character shows unstaged status
            let has_staged = status_result
                .stdout
                .lines()
                .any(|line| {
                    !line.is_empty() && 
                    line.chars().next().map_or(false, |c| c != ' ' && c != '?')
                });

            let mut diff_args = vec![];
            if let Some(directory) = dir {
                diff_args.extend(vec!["-C".to_string(), directory.to_string()]);
            }
            
            if has_staged {
                tracing::info!("评审已暂存的变更");
                diff_args.extend(vec!["diff".to_string(), "--staged".to_string()]);
            } else {
                tracing::info!("评审工作区的变更");
                diff_args.extend(vec!["diff".to_string()]);
            }

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
