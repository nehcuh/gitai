//! GitAI Security Module
//!
//! This crate provides security scanning functionality using OpenGrep and other tools.

#![warn(missing_docs)]

pub mod scanner;
pub mod rules;

/// Re-export commonly used types
pub use scanner::*;
