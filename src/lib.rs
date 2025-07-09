pub mod config;
pub mod errors;
pub mod handlers;
pub mod rule_manager;
pub mod scanner;
pub mod ast_grep_integration;
pub mod ast_grep_installer;
pub mod types;
pub mod utils;
pub mod tree_sitter_analyzer;
pub mod clients;
pub mod mcp; // MCP 模块 - 使用兼容性适配层
pub mod mcp_bridge; // MCP 桥接模块，提供 CLI/MCP 双兼容性

// Re-export commonly used items for convenience
pub use config::AppConfig;
pub use errors::AppError;