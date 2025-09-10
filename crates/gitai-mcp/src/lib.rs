//! GitAI MCP Server
//!
//! This crate provides the Model Context Protocol server implementation.

#![warn(missing_docs)]

pub mod server;
pub mod services;
pub mod registry;

/// Re-export commonly used types
pub use server::*;
