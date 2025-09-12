//! GitAI Facade Library
//! 
//! This is a facade module that provides a unified API for GitAI functionality.
//! All core implementations have been moved to separate crates.
//! This module only re-exports public interfaces.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::multiple_bound_locations)]

// ============================================================================
// TEMPORARY: Legacy modules still in src/
// These will be migrated in future phases
// ============================================================================

// TODO: Migrate these to appropriate crates
pub mod analysis;
pub mod architectural_impact;
pub mod config_init;
pub mod context;
pub mod error_ext;
#[cfg(test)]
pub mod error_tests;
pub mod project_insights;
pub mod prompts;
pub mod resource_manager;
pub mod review;
pub mod tree_sitter;

// Legacy infrastructure
pub mod infrastructure;
pub mod domain;

// Conditional modules
#[cfg(feature = "update-notifier")]
pub mod update;
#[cfg(feature = "metrics")]
pub mod metrics;
#[cfg(feature = "mcp")]
pub mod mcp;

// ============================================================================
// PUBLIC API: Re-exports from crates
// ============================================================================

// Core types and errors
pub use gitai_types::{
    error::{GitAIError, Result},
    // Re-export common types
    ComplexityLevel, DependencyType, Finding, ImpactLevel, 
    NodeType, RiskLevel, Severity,
};

// Core functionality
pub use gitai_core::{
    config::Config,
    utils::error_handling::{convenience, DomainErrorHandler, SafeResult},
};

// Security features
#[cfg(feature = "security")]
pub use gitai_security::{
    scanner::SecurityScanner,
    // Add more security exports as needed
};

// CLI functionality
pub use gitai_cli::{
    args::Args,
    features::FeatureFlags,
    // Add more CLI exports as needed
};

// Adapters
#[cfg(feature = "devops")]
pub use gitai_adapters::{
    devops::DevOpsClient,
    // Add more adapter exports as needed
};

// Analysis features
pub use gitai_analysis::{
    // TODO: Add analysis exports once migrated
};

// Metrics
#[cfg(feature = "metrics")]
pub use gitai_metrics::{
    // TODO: Add metrics exports once migrated
};

// MCP Protocol
#[cfg(feature = "mcp")]
pub use gitai_mcp::{
    // TODO: Add MCP exports once migrated
};

// ============================================================================
// TEMPORARY: Legacy re-exports (will be removed after full migration)
// ============================================================================

pub use architectural_impact::{ArchitecturalImpactAnalysis, BreakingChange};
pub use context::{Issue, OperationContext, OperationOptions};
pub use project_insights::{InsightsGenerator, ProjectInsights};
pub use prompts::{PromptContext, PromptManager};
pub use tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};
pub use infrastructure::{ContainerError, ServiceContainer};
