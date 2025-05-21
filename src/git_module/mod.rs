use crate::core::{
    errors::{AppError, GitError},
    types::CommandOutput,
};

/// Execute a git command and captures its output
///
/// This function runs a git command with the provided argument and return
/// the command's output, include stdout, stderr, and exit status.
///
/// # Arguments
///
/// * `args` - A slice of String containing the arguments to pass to git
///
/// # Returns
///
/// * `Result<CommandOutput, AppError>` - The command output or an error
///
/// # Example
///
/// ```
/// use gitai::git_module::execute_git_command_and_capture_output;
///
/// let args = vec!["status".to_string(), "--short".to_string()];
/// match execute_git_command_and_capture_output(&args) {
///     Ok(output) => println!("Git status: {}", output.stdout),
///     Err(err) => eprintln!("Error: {}", err)
/// }
pub fn execute_git_command_and_capture_output(args: &[String]) -> Result<CommandOutput, AppError> {
    let cmd_to_run = if args.is_empty() {
        vec!["--help".to_string()]
    } else {
        args.to_vec()
    };

    tracing::debug!("Capturing output: git {}", cmd_to_run.join(" "));

    let output = Command::new("git")
        .args(&cmd_to_run)
        .output()
        .map_err(|e| {
            AppError::IO(
                format!("Failed to execute: git {}", cmd_to_run.join(" ")),
                e,
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        tracing::warn!(
            "Git cmd 'git {}' non-success {}. Stdout: [{}], Stderr: [{}]",
            cmd_to_run.join(" "),
            output.status,
            stdout,
            stderr
        );
    }

    Ok(CommandOutput {
        stdout,
        stderr,
        status: output.status,
    })
}

/// Parses arguments directly to the system's git command
///
/// This function is used when the gitai needs to delegate to the
/// underlying git command without modification
///
/// # Arguments
///
/// * `args` - A slice of String containing the arguments to pass to git
///
/// # Returns
///
/// * `Result<(), AppError>` - Success or an error
pub fn passthrough_to_git(args: &[String]) -> Result<(), AppError> {
    // Call the new function and specify not to handle errors
    let result = passthrough_to_git_with_error_handling(args, false)?;

    // Check status to preserve original behavior
    if !result.status.success() {
        return Err(AppError::Git(GitError::PassthroughFailed {
            command: format!("git: {}", args.join(" ")),
            status_code: result.status.code(),
        }));
    }

    Ok(())
}

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
