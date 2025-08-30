// GitAI Library
// 提供AI驱动的Git工作流功能

// 核心模块（始终可用）
pub mod config;
pub mod config_init;
pub mod resource_manager;
pub mod context;
pub mod args;
pub mod git;
pub mod error;
pub mod prompts;
pub mod analysis;
pub mod commit;
pub mod review;
pub mod tree_sitter;
pub mod project_insights;
pub mod architectural_impact;
pub mod features;

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

pub use config::Config;
pub use context::{OperationContext, OperationOptions, Issue};
pub use error::{GitAIError, Result};
pub use tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};
pub use prompts::{PromptManager, PromptContext};
pub use project_insights::{ProjectInsights, InsightsGenerator};
pub use architectural_impact::{ArchitecturalImpactAnalysis, BreakingChange, RiskLevel};
