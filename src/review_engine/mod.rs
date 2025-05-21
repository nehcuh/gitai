//! Code review module
//!
//! This module provides basic code review functionality, including a simplified review result data structure.

use serde::{Deserialize, Serialize};

use crate::config_management::settings::TreeSitterConfig;

/// Rule category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Code style
    Style,
    /// Security
    Security,
    /// Performance
    Performance,
    /// Code complexity
    Complexity,
    /// Best practices
    BestPractices,
    /// Potential bugs
    Bugs,
}

/// Severity of issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Error: Critical issues that need immediate fixing
    Error,
    /// Warning: Issues that should be fixed
    Warning,
    /// Info: Areas for improvement
    Info,
    /// Hint: Minor improvement suggestions
    Hint,
}

/// Analysis depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisDepth {
    /// Basic analysis
    Basic,
    /// Normal analysis
    Normal,
    /// Deep analysis
    Deep,
}

/// Simplified review result
pub struct SimpleReviewResult {
    /// Review title
    pub title: String,
    /// Review content
    pub content: String,
    /// Severity
    pub severity: Severity,
}

impl SimpleReviewer {
    /// Creates a new code reviewer
    pub fn new(config: TreeSitterConfig) -> Self {
        Self { config }
    }

    /// Executes a simple review
    pub fn review(&self, diff: &GitDiff) -> Vec<SimpleReviewResult> {
        let mut results = Vec::new();

        // Simple check for hardcoded credentials
        for file in &diff.changed_files {
            for hunk in &file.hunks {
                for line in &hunk.lines() {
                    if line.startswith('+')
                        && (line.contains("password")
                            || line.contains("secret")
                            || line.contains("token")
                            || line.contains("api_key"))
                    {
                        results.push(SimpleReviewResult {
                            title: "检测到硬编码凭证".to_string(),
                            content:
                                "代码中可能包含硬编码的敏感信息，建议使用环境变量或配置文件存储"
                                    .to_string(),
                            severity: Severity::Error,
                        });
                    }
                    // Check for long lines
                    if line.starts_with('+') && line.len() > 100 {
                        results.push(SimpleReviewResult {
                            title: "行长度过长".to_string(),
                            content: "检测到长度超过 100 字符的行，建议拆分以提高可读性"
                                .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        // If no issues are found, add a positive feedback
        if results.is_empty() {
            results.push(SimpleReviewResult {
                title: "代码质量良好".to_string(),
                content: "未发现明显问题，代码质量良好".to_string(),
                severity: Severity::Info,
            })
        }

        results
    }
}
