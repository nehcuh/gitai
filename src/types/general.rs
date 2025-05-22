use colored::*;
use std::process::ExitStatus; // Keep only ExitStatus as HashMap is not used

/// Represents the output of a command execution
///
/// This structure captures the stdout, stderr, and exit status
#[derive(Debug)]
pub struct CommandOutput {
    /// Standard output from the command
    pub stdout: String,

    /// Standard error output from the command
    pub stderr: String,

    /// Exit status of the command
    pub status: ExitStatus,
}

impl CommandOutput {
    /// Returns true if the command executed successfully
    #[allow(unused)]
    pub fn is_success(&self) -> bool {
        self.status.success()
    }

    /// Returns the exit code of the command, if available
    #[allow(unused)]
    pub fn exit_code(&self) -> Option<i32> {
        self.status.code()
    }

    /// Returns the combined output (stdout + stderr) with stderr
    #[allow(unused)]
    pub fn combined_output(&self) -> String {
        let mut output = self.stdout.clone();

        if !self.stderr.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(
                &self
                    .stderr
                    .lines()
                    .map(|line| format!("{}: {}", "Error\n".red(), line))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }

        output
    }

    /// Return true if both stdout and stderr are empty
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }

    /// Returns stdout as a vector of lines
    #[allow(unused)]
    pub fn stdout_lines(&self) -> Vec<String> {
        self.stdout.lines().map(String::from).collect()
    }

    /// Returns a formatted display of the command output for user interaction
    #[allow(unused)]
    pub fn formatted_display(&self) -> String {
        let mut display = String::new();

        if !self.stdout.is_empty() {
            display.push_str(&"Output:".green().bold().to_string());
            display.push_str("\n");
            display.push_str(&self.stdout);
            display.push('\n');
        }

        if !self.stderr.is_empty() {
            display.push_str(&"Errors:".red().bold().to_string());
            display.push('\n');
            display.push_str(&self.stderr);
            display.push('\n');
        }

        if !self.is_success() {
            let exit_code = self.exit_code().unwrap_or(-1);
            display.push_str(&"Exit code: {}".yellow().bold().to_string());
            display.push_str(&format!("{}", exit_code).red());
            display.push('\n');
        }

        display
    }
}
