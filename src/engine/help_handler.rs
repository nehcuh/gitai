use crate::errors::AppError;
use crate::types::general::CommandOutput;

/// Execute Git command and optionally handle errors
///
/// Executes Git command, captures output, and based on execution status decides
/// whether to output the result directly or handle errors
///
/// # Arguments
///
/// * `args` - Arguments to pass to Git
/// * `handle_error` - Whether to handle errors (if false, behavior is the same as the original function)
///
/// # Returns
///
/// * `Result<CommandOutput, AppError>` - Command output or error
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
    tracing::debug!("指定系统 git 命令: {}", cmd_str_log);

    // Execute directly and get output instead of just status
    let output = std::process::Command::new("git")
        .args(&command_to_run)
        .output()
        .map_err(|e| AppError::IO(format!("执行系统 git 命令失败：git {}", cmd_str_log), e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // If error handling is disabled or the command succeeds, print output directly
    if !handle_error || output.status.success() {
        // Print stdout and stderr to mimic the original command behavior
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
    }

    if !output.status.success() {
        tracing::warn!(
            "Git 命令 {} 执行失败，错误码 {}",
            cmd_str_log,
            output.status
        );

        // Return an error only when error handling is disabled
        if !handle_error {
            return Err(AppError::Git(GitError::PassthroughFailed {
                command: format!("git {}", cmd_str_log),
                status_code: output.status.code(),
            }));
        }
    }

    Ok(CommandOutput {
        stdout,
        stderr,
        status: output.status,
    })
}
