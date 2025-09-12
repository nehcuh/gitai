//! GitAI Security Module
//!
//! This crate provides security scanning functionality using OpenGrep and other tools.

#![warn(missing_docs)]

/// Security scanning engine abstraction and implementations
pub mod scanner;

/// Re-export commonly used types
pub use scanner::*;
