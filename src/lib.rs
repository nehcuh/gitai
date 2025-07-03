// 新的模块结构
pub mod common;
pub mod cli;
pub mod git;
pub mod ai;

// 旧模块（逐步迁移）
pub mod ast_grep_analyzer;
pub mod clients;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod types;
pub mod utils;

// 重新导出常用项
pub use common::{AppError, AppResult, SupportedLanguage, ChatMessage};
pub use cli::{GitAIArgs, GitAICommand, ParsedArgs, CLIParser};
pub use git::{GitOperations, GitOps, GitCommit, GitStatus};

// 兼容性导出（用于过渡期）
pub use config::AppConfig;
pub use types::git::CommitArgs;