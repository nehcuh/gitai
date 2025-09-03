// GitAI Library
// 提供AI驱动的Git工作流功能

// 核心模块（始终可用）
pub mod analysis;
pub mod architectural_impact;
pub mod args;
pub mod commit;
pub mod config;
pub mod config_init;
pub mod context;
pub mod error;
pub mod features;
pub mod git;
pub mod project_insights;
pub mod prompts;
pub mod resource_manager;
pub mod review;
pub mod tree_sitter;
pub mod utils;

// 条件编译模块
#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "security")]
pub mod scan;
#[cfg(feature = "update-notifier")]
pub mod update;

#[cfg(feature = "devops")]
pub mod devops;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "mcp")]
pub mod mcp;

pub use architectural_impact::{ArchitecturalImpactAnalysis, BreakingChange, RiskLevel};
pub use config::Config;
pub use context::{Issue, OperationContext, OperationOptions};
pub use error::{GitAIError, Result};
pub use project_insights::{InsightsGenerator, ProjectInsights};
pub use prompts::{PromptContext, PromptManager};
pub use tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};
