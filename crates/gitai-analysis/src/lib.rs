//! GitAI Analysis Engine
//!
//! This crate provides code analysis functionality using Tree-sitter and other tools.
//! Includes multi-language structural analysis, change detection, and pattern matching.

#![warn(missing_docs)]

pub mod analysis;
pub mod tree_sitter;
pub mod architectural_impact;
pub mod utils;

// Re-export commonly used types from analysis module
pub use analysis::{
    AnalysisResult, OperationContext, OperationOptions, 
    SecurityFinding, DeviationAnalysis, Deviation,
};

// Re-export Tree-sitter analyzer types
pub use tree_sitter::{
    analyzer::StructureAnalyzer,
    unified_analyzer::UnifiedAnalyzer,
};
