//! GitAI Core Library
//! 核心功能模块，提供基础服务和通用功能

#![warn(missing_docs)]

/// Configuration management (loading, validation)
pub mod config;
// TODO: commit module needs refactoring
// pub mod commit;
/// Shared operation context and domain structures
pub mod context;
/// Domain-specific error types and results
pub mod domain_errors;
/// Core error types and conversions
pub mod error;
/// Git helpers and wrappers around VCS operations
pub mod git;
/// Public interfaces and data contracts for subsystems
pub mod interfaces;
/// Service layer adapters and orchestration helpers
pub mod services;
/// Utility modules for common functionality
pub mod utils;

/// Adapter integration registry and traits exposure
// TODO: Fix circular dependency issue with gitai-adapters
// pub mod adapters;

#[cfg(feature = "ai")]
/// AI integration façade (enabled via `ai` feature)
pub mod ai;

#[cfg(feature = "cache")]
/// Caching facilities (enabled via `cache` feature)
pub mod cache;

// Re-export commonly used types
pub use gitai_types::{
    BreakingChange, Finding, GitAIError, ImpactLevel, Result, RiskLevel, Severity,
};

pub use config::Config;
pub use context::{OperationContext, OperationOptions};

/// GitAI Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core library
pub fn init() -> Result<()> {
    // Initialize logging
    tracing::info!("Initializing GitAI Core v{}", VERSION);

    // Perform any necessary initialization
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        debug_assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
