mod ast_grep_analyzer;
pub mod clients;
mod config;
mod errors;
mod handlers;
mod types;
mod utils;

use ast_grep_analyzer::translation::SupportedLanguage;

use handlers::commit::handle_commit;
use handlers::git::passthrough_to_git;
use handlers::intelligent_git::handle_intelligent_git_command;
use handlers::review::handle_review;
use handlers::scan::handle_scan;
use handlers::update_rules::handle_update_rules;
use utils::{
    construct_commit_args, construct_review_args, construct_scan_args, construct_update_rules_args,
};

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

    // Remove program name from arguments
    if !args.is_empty() {
        args.remove(0);
    }

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

    // Extract and parse --lang parameter
    let mut language_override: Option<SupportedLanguage> = None;

    // Find --lang parameter
    let mut i = 0;
    while i < args.len() {
        if let Some(arg) = args.get(i) {
            if arg.starts_with("--lang=") {
                // Handle --lang=value format
                let lang_value = arg.strip_prefix("--lang=").unwrap();
                language_override = parse_language_parameter(lang_value);
                args.remove(i);
                continue;
            } else if arg == "--lang" && i + 1 < args.len() {
                // Handle --lang value format
                if let Some(lang_value) = args.get(i + 1) {
                    language_override = parse_language_parameter(lang_value);
                    args.remove(i); // Remove --lang
                    args.remove(i); // Remove value (index shifts after first removal)
                    continue;
                }
            }
        }
        i += 1;
    }

    // Apply language override to config if provided
    if let Some(lang) = language_override {
        config.translation.default_language = lang.clone();
        tracing::info!("Language override applied: {:?}", lang);
    }

    // Filter ai flags
    args.retain(|arg| arg != "--ai" && arg != "--noai");

    if args.is_empty() {
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

    // scan 处理
    if args.iter().any(|arg| arg == "scan" || arg == "sc") {
        tracing::info!("检测到scan命令");
        let scan_args = construct_scan_args(&args);
        handle_scan(&config, &scan_args).await?;
        return Ok(());
    }

    // update-rules 处理
    if args.iter().any(|arg| arg == "update-rules" || arg == "ur") {
        tracing::info!("检测到update-rules命令");
        let update_rules_args = construct_update_rules_args(&args);
        handle_update_rules(&config, &update_rules_args).await?;
        return Ok(());
    }

    // 标准 git 指令处理
    // 1. 当全局 ai 标识启用时，同时捕捉标准输出和标准错误，利用 AI 解释
    // 2. 当没有指定全局 ai 标识时，如果没有报错，则标准输出直接输出，如果有错误，则捕获标准错误并提供 AI 解释

    match handle_intelligent_git_command(&config, &args, use_ai).await {
        Ok(_) => {}
        Err(AppError::Git(crate::errors::GitError::CommandFailed { status_code, .. })) => {
            // Maintain same exit status as original git command
            std::process::exit(status_code.unwrap_or(1));
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

/// Parse language parameter from command line argument
fn parse_language_parameter(lang_str: &str) -> Option<SupportedLanguage> {
    match lang_str.to_lowercase().as_str() {
        "zh" | "chinese" | "中文" => Some(SupportedLanguage::Chinese),
        "en" | "english" | "英文" => Some(SupportedLanguage::English),
        "auto" | "自动" => Some(SupportedLanguage::Auto),
        _ => {
            tracing::warn!(
                "Unsupported language parameter: {}. Supported values: zh, en, auto",
                lang_str
            );
            None
        }
    }
}
