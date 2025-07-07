pub mod clients;
mod config;
mod errors;
mod handlers;
mod rule_manager;
mod scanner;
mod ast_grep_integration;
mod tree_sitter_analyzer;
mod types;
mod utils;

use handlers::commit::handle_commit;
use handlers::git::passthrough_to_git;
use handlers::intelligent_git::handle_intelligent_git_command;
use handlers::query_update::{handle_query_update, handle_query_cleanup, handle_query_status};
use handlers::review::handle_review;
use handlers::scan::handle_scan;
use handlers::translate::handle_translate;
use utils::{construct_commit_args, construct_review_args, construct_scan_args, construct_translate_args};

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

    // ========================================
    // Step 1: Parse AI mode flags
    // ========================================
    let mut use_ai = false;
    let mut disable_ai = false;
    
    // Check for AI mode flags
    if args.iter().any(|arg| arg == "--ai") {
        use_ai = true;
        tracing::info!("🤖 强制启用 AI 模式：所有输出都会被 AI 解释");
    }
    
    if args.iter().any(|arg| arg == "--noai") {
        disable_ai = true;
        tracing::info!("🚫 禁用 AI 模式：使用纯 Git 行为");
    }
    
    // Handle conflicting flags
    if use_ai && disable_ai {
        eprintln!("❌ 错误：--ai 和 --noai 标志不能同时使用");
        std::process::exit(1);
    }
    
    // If --noai is specified, pass through to pure git immediately
    if disable_ai {
        tracing::info!("💤 AI 已禁用，直接传递给标准 Git");
        passthrough_to_git(&args)?;
        return Ok(());
    }
    
    // Remove AI flags from arguments before further processing
    args.retain(|arg| arg != "--ai" && arg != "--noai");
    
    if !use_ai {
        tracing::info!("🧠 智能 AI 模式：仅在出错时提供解释");
    }

    // ========================================
    // Step 2: Handle special cases
    // ========================================
    
    // Show help if no arguments provided
    if args.is_empty() {
        println!("{}", generate_gitai_help());
        return Ok(());
    }

    // Handle help requests
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        tracing::info!("📚 显示帮助信息");
        handle_help(&config, args, use_ai).await?;
        return Ok(());
    }

    // ========================================
    // Step 3: Handle gitai-specific AI-enhanced commands
    // ========================================
    
    // 🚀 AI-Enhanced Commands
    if args.iter().any(|arg| arg == "review" || arg == "rv") {
        tracing::info!("🔍 执行 AI 代码评审");
        let review_args = construct_review_args(&args);
        handle_review(&mut config, review_args).await?;
        return Ok(());
    }

    if args.iter().any(|arg| arg == "scan") {
        tracing::info!("🛡️ 执行代码安全扫描");
        let scan_args = construct_scan_args(&args);
        handle_scan(&config, scan_args).await?;
        return Ok(());
    }

    if args.iter().any(|arg| arg == "translate") {
        tracing::info!("🌐 执行 AI 翻译");
        let translate_args = construct_translate_args(&args);
        handle_translate(&config, translate_args).await?;
        return Ok(());
    }

    if args.iter().any(|arg| arg == "commit" || arg == "cm") {
        tracing::info!("💬 执行 AI 增强提交");
        let commit_args = construct_commit_args(&args);
        handle_commit(&config, commit_args).await?;
        return Ok(());
    }

    // ========================================
    // Step 4: Handle management commands  
    // ========================================
    
    // 🔧 Management Commands
    if !args.is_empty() {
        match args[0].as_str() {
            "update-queries" => {
                tracing::info!("🔄 更新 Tree-sitter 查询文件");
                handle_query_update()?;
                return Ok(());
            }
            "cleanup-queries" => {
                tracing::info!("🧹 清理查询文件");
                handle_query_cleanup()?;
                return Ok(());
            }
            "query-status" => {
                tracing::info!("📊 显示查询文件状态");
                handle_query_status()?;
                return Ok(());
            }
            _ => {
                // Continue to git proxy handling
            }
        }
    }

    // ========================================
    // Step 5: Handle standard Git commands with intelligent AI proxy
    // ========================================
    
    // 📦 Standard Git Commands (with smart AI assistance)
    // Behavior:
    // - Default mode: Only provide AI explanation on errors
    // - --ai mode: AI explains all output (success + errors)  
    // - All standard Git functionality is preserved
    
    tracing::info!("⚡ 执行标准 Git 命令: {}", args.join(" "));
    
    match handle_intelligent_git_command(&config, &args, use_ai).await {
        Ok(_) => {
            tracing::debug!("✅ Git 命令执行成功");
        },
        Err(AppError::Git(crate::errors::GitError::CommandFailed { status_code, .. })) => {
            tracing::debug!("❌ Git 命令执行失败，退出码: {:?}", status_code);
            // Maintain same exit status as original git command
            std::process::exit(status_code.unwrap_or(1));
        }
        Err(e) => {
            tracing::error!("💥 gitai 内部错误: {}", e);
            return Err(e);
        },
    }

    Ok(())
}
