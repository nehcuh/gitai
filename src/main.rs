pub mod clients;
mod config;
mod errors;
mod handlers;
mod tree_sitter_analyzer;
mod types;
mod utils;

use handlers::git::passthrough_to_git;
use handlers::review::handle_review;
use handlers::commit::handle_commit;
use utils::{construct_review_args, construct_commit_args};

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::handlers::help::handle_help;
use crate::utils::generate_gitai_help;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    // Default configuration can be overwritten by command-line commands
    let mut config = match AppConfig::load() {
        // Prefix with underscore to mark as unused
        Ok(config) => config,
        Err(e) => return Err(AppError::Config(e)),
    };

    // Handling cmd args
    let mut args: Vec<String> = std::env::args().collect();

    // 检测是否需要弃用 AI
    if args.iter().any(|arg| arg == "--noai") {
        passthrough_to_git(&args)?;
        return Ok(());
    }

    // 检测是否需要启用全局 AI
    let mut use_ai = false;
    if args.iter().any(|arg| arg == "--ai") {
        tracing::info!("检测到 AI 标识，全局启用 AI 能力");
        use_ai = true;
    } else {
        tracing::info!("智能启用 AI 能力");
    }

    // Filter ai flags
    args.retain(|arg| arg != "--ai" && arg != "--noai");

    if args.len() <= 1 {
        println!("{}", generate_gitai_help());
        return Ok(());
    }

    // 帮助信息的全局处理
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        tracing::info!("检测到 help 标识");
        handle_help(&config, args, use_ai).await?;
        return Ok(());
    }

    // gitai 特殊指令处理
    // review 处理
    if args.iter().any(|arg| arg == "review" || arg == "rv") {
        tracing::info!("检测到review命令");
        let review_args = construct_review_args(&args);
        // review_args can overwritten config tree-sitter config
        handle_review(&mut config, review_args).await?;
        return Ok(());
    }

    // commit 处理
    if args.iter().any(|arg| arg == "commit" || arg == "cm") {
        tracing::info!("检测到commit命令");
        let commit_args = construct_commit_args(&args);
        handle_commit(&config, commit_args).await?;
        return Ok(());
    }

    // 标准 git 指令处理
    // 1. 当全局 ai 标识启用时，同时捕捉标准输出和标准错误，利用 AI 解释
    // 2. 当没有指定全局 ai 标识时，标准输出直接转发，捕获标准错误并提供 AI 解释

    Ok(())
}
