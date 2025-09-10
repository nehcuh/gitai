//! GitAI CLI Module
//!
//! This crate provides the command-line interface for GitAI.

#![warn(missing_docs)]

pub mod app;
pub mod args;
pub mod handlers;

/// Re-export commonly used types
pub use app::*;
pub use args::*;
