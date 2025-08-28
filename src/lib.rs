// GitAI Library
// 提供AI驱动的Git工作流功能

pub mod config;
pub mod args;
pub mod git;
pub mod ai;
pub mod scan;
pub mod update;
pub mod devops;
pub mod analysis;
pub mod commit;
pub mod review;
pub mod tree_sitter;
pub mod prompts;
pub mod mcp;
pub mod error;

pub use config::Config;
pub use tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};
pub use prompts::{PromptManager, PromptContext};
