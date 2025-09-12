//! GitAI MCP Server - 简化版本
//!
//! This crate provides a simplified Model Context Protocol server implementation.

#![warn(missing_docs)]

pub mod bridge;
pub mod error;
pub mod http;
pub mod manager;
pub mod registry;
pub mod server;
pub mod services;

pub use error::*;
pub use http::*;
pub use manager::*;
/// Re-export commonly used types
pub use server::*;
pub use services::*;
