//! GitAI Analysis Engine
//!
//! This crate provides code analysis functionality using Tree-sitter and other tools.
//! Includes multi-language structural analysis, change detection, and pattern matching.

#![warn(missing_docs)]

/// High-level orchestration for multi-dimensional analysis
pub mod analysis;
/// Architectural impact analysis and reporting
pub mod architectural_impact;
/// Tree-sitter based structural analysis (multi-language)
pub mod tree_sitter;
/// Internal utilities (error handling, helpers)
pub mod utils;

// Re-export commonly used types from analysis module
pub use analysis::{
    AnalysisResult, Deviation, DeviationAnalysis, OperationContext, OperationOptions,
    SecurityFinding,
};

// Re-export Tree-sitter analyzer types
pub use tree_sitter::{analyzer::StructureAnalyzer, unified_analyzer::UnifiedAnalyzer};
