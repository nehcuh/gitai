// GitAI Library
// 提供AI驱动的Git工作流功能

pub mod config;
pub mod config_init;
pub mod resource_manager;
pub mod context;
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
pub mod project_insights;
pub mod metrics;

pub use config::Config;
pub use context::{OperationContext, OperationOptions};
pub use error::{GitAIError, Result};
pub use tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};
pub use prompts::{PromptManager, PromptContext};
pub use project_insights::{ProjectInsights, InsightsGenerator};
