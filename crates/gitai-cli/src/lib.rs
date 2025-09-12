//! GitAI CLI Module
//!
//! This crate provides the command-line interface for GitAI.

// #![warn(missing_docs)]  // 暂时关闭文档警告
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::useless_conversion)]

pub mod app;
pub mod args;
pub mod handlers;

/// Re-export commonly used types
pub use app::*;
pub use args::*;
