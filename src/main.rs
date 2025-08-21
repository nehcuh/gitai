pub mod config;
pub mod errors;
pub mod handlers;
pub mod tree_sitter_analyzer;
pub mod types;
pub mod utils;

use handlers::commit::handle_commit;
use handlers::git::passthrough_to_git;
use handlers::intelligent_git::handle_intelligent_git_command;
use handlers::query_update::{handle_query_update, handle_query_cleanup, handle_query_status};
use handlers::review::handle_review;
use handlers::scan::entry::handle_scan;
use utils::{construct_commit_args, construct_review_args, construct_scan_args};

use crate::config::AppConfig;
use crate::errors::{AppError, config_error};
use crate::handlers::help::handle_help;
use crate::utils::generate_gitai_help;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();
    
    let config = AppConfig::load().map_err(|e| config_error(e.to_string()))?;
    let mut args: Vec<String> = std::env::args().collect();
    args.remove(0);
    
    if args.is_empty() {
        println!("{}", generate_gitai_help());
        return Ok(());
    }
    
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        handle_help(&config, args, false).await?;
        return Ok(());
    }
    
    if args.iter().any(|arg| arg == "--noai") {
        passthrough_to_git(&args)?;
        return Ok(());
    }
    
    match args[0].as_str() {
        "review" | "rv" => {
            let review_args = construct_review_args(&args);
            handle_review(&config, review_args).await?;
        }
        "commit" | "cm" => {
            let commit_args = construct_commit_args(&args);
            handle_commit(&config, commit_args).await?;
        }
        "scan" => {
            let scan_args = construct_scan_args(&args);
            handle_scan(&config, scan_args).await?;
        }
        "update-queries" => {
            handle_query_update().await?;
        }
        "cleanup-queries" => {
            handle_query_cleanup()?;
        }
        "query-status" => {
            handle_query_status()?;
        }
        _ => {
            handle_intelligent_git_command(&config, &args, false).await?;
        }
    }
    
    Ok(())
}