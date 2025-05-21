use colored::*;
// use regex::Regex;
use std::process::ExitStatus; // Keep only ExitStatus as HashMap is not used
use serde::{Deserialize, Serialize};

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

/// Represents a chat message with a role and content
///
/// This structure is used for both requests to and responses from AI chat models
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64, // Typically a UNIX Timestamp
    pub model: String,
    pub system_fingerprint: Option<String>, // This field exists based on the exampale provided
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)] // Add allow(dead_code) to suppress warnings for unused fields
pub struct OpenAIChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
    // pub logprobs: Option<serde_json::Value> // If logprobs parsing is needed
}
