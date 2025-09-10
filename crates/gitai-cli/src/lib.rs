//! GitAI CLI Module
//!
//! This crate provides the command-line interface for GitAI.

#![warn(missing_docs)]

pub mod app;
pub mod handlers;

/// Re-export commonly used types
pub use app::*;
