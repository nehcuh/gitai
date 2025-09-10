//! GitAI Analysis Engine
//!
//! This crate provides code analysis functionality using Tree-sitter and other tools.

#![warn(missing_docs)]

pub mod analyzer;
pub mod tree_sitter;

/// Re-export commonly used types
pub use analyzer::*;
