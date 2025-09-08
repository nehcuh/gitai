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
#[cfg(test)]
pub mod error_tests;
pub mod features;
pub mod git;
pub mod project_insights;
pub mod prompts;
pub mod resource_manager;
pub mod review;
pub mod tree_sitter;
pub mod utils;

// 基础设施层 - 新的架构组件
pub mod infrastructure;

// 领域层 - 业务逻辑抽象
pub mod domain;

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
pub use utils::error_handling::{SafeResult, DomainErrorHandler, convenience};
pub use project_insights::{InsightsGenerator, ProjectInsights};
pub use prompts::{PromptContext, PromptManager};
pub use tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};

// 导出新的基础设施组件
pub use infrastructure::{
    ServiceContainer, ContainerError,
};
