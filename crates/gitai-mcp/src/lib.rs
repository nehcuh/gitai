//! GitAI MCP Server - 简化版本
//!
//! This crate provides a simplified Model Context Protocol server implementation.

#![warn(missing_docs)]

pub mod server;
pub mod services;
pub mod registry;
pub mod manager;
pub mod bridge;
pub mod error;

/// Re-export commonly used types
pub use server::*;
pub use error::*;
pub use manager::*;
pub use services::*;
