//! GitAI Core Library
//! 核心功能模块，提供基础服务和通用功能

#![warn(missing_docs)]

pub mod config;
pub mod context;
pub mod git;
pub mod error;
pub mod domain_errors;
pub mod interfaces;
pub mod services;

/// Adapter integration registry and traits exposure
pub mod adapters;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "cache")]
pub mod cache;

// Re-export commonly used types
pub use gitai_types::{
    Severity, RiskLevel, Finding, BreakingChange,
    ImpactLevel, GitAIError, Result,
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
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
