// 新的模块结构
pub mod common;
pub mod cli;
pub mod git;
pub mod ai;
pub mod app;

// 旧模块（逐步迁移）
pub mod ast_grep_analyzer;
pub mod clients;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod types;
pub mod utils;

use common::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 使用新的应用程序结构
    app::run_app().await
}
