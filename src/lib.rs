#![allow(clippy::uninlined_format_args)]
#![allow(clippy::multiple_bound_locations)]

// GitAI Library
// 提供AI驱动的Git工作流功能

// 核心模块（始终可用）
pub mod analysis;
pub mod architectural_impact;
// args模块已迁移到crates/gitai-cli
// commit模块已迁移到crates/gitai-core
// config模块已迁移到crates/gitai-core
pub mod config_init;
pub mod context;
// error模块已迁移到crates/gitai-types
pub mod error_ext;
#[cfg(test)]
pub mod error_tests;
// features模块已迁移到crates/gitai-cli
// git模块已迁移到crates/gitai-core
pub mod project_insights;
pub mod prompts;
pub mod resource_manager;
pub mod review;
pub mod tree_sitter;
// utils模块已迁移到crates/gitai-core

// 基础设施层 - 新的架构组件
pub mod infrastructure;

// 领域层 - 业务逻辑抽象
pub mod domain;

// 条件编译模块
// ai模块已迁移到crates/gitai-core

// scan模块已迁移到crates/gitai-security
#[cfg(feature = "update-notifier")]
pub mod update;

// devops模块已迁移到crates/gitai-adapters

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "mcp")]
pub mod mcp;

pub use architectural_impact::{ArchitecturalImpactAnalysis, BreakingChange, RiskLevel};
// Config已从crates/gitai-core导出
pub use gitai_core::config::Config;
pub use context::{Issue, OperationContext, OperationOptions};
// Error类型已从crates/gitai-types导出
pub use gitai_types::error::{GitAIError, Result};
pub use project_insights::{InsightsGenerator, ProjectInsights};
pub use prompts::{PromptContext, PromptManager};
pub use tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};
// utils已从gitai-core导出
pub use gitai_core::utils::error_handling::{convenience, DomainErrorHandler, SafeResult};

// 导出新的基础设施组件
pub use infrastructure::{ContainerError, ServiceContainer};
