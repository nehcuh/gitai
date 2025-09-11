//! CLI handlers module

pub mod commit;
pub mod config;
pub mod features;
pub mod git;
pub mod graph;
pub mod init;
pub mod mcp;
pub mod metrics;
pub mod prompts;
pub mod review;
pub mod scan;
pub mod update;

// 处理器模块由 app.rs 中的路由逻辑直接使用

