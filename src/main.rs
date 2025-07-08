pub mod clients;
mod config;
mod errors;
mod handlers;
mod rule_manager;
mod scanner;
mod ast_grep_integration;
mod ast_grep_installer;
mod tree_sitter_analyzer;
mod types;
mod utils;

use handlers::commit::handle_commit;
use handlers::git::passthrough_to_git;
use handlers::intelligent_git::handle_intelligent_git_command;
use handlers::query_update::{handle_query_update, handle_query_cleanup, handle_query_status};
use handlers::review::handle_review;
use handlers::scan::{handle_scan, handle_update_scan_rules};
use handlers::translate::handle_translate;
use utils::{construct_commit_args, construct_review_args, construct_scan_args, construct_translate_args};
use ast_grep_installer::AstGrepInstaller;
use colored::Colorize;

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
    let mut language_param: Option<String> = None;
    
    // Check for language parameter
    for i in 0..args.len() {
        if args[i] == "--lang" && i + 1 < args.len() {
            language_param = Some(args[i + 1].clone());
            tracing::info!("🌐 指定输出语言: {}", args[i + 1]);
            break;
        } else if args[i].starts_with("--lang=") {
            let lang = args[i].strip_prefix("--lang=").unwrap();
            language_param = Some(lang.to_string());
            tracing::info!("🌐 指定输出语言: {}", lang);
            break;
        }
    }
    
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
    
    // Remove AI and language flags from arguments before further processing
    let mut cleaned_args = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--ai" || arg == "--noai" {
            // Skip AI flags
            i += 1;
        } else if arg == "--lang" {
            // Skip --lang and its value
            i += 2;
        } else if arg.starts_with("--lang=") {
            // Skip --lang=value
            i += 1;
        } else {
            cleaned_args.push(arg.clone());
            i += 1;
        }
    }
    args = cleaned_args;
    
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
        handle_review(&mut config, review_args, language_param.as_deref()).await?;
        return Ok(());
    }

    if args.iter().any(|arg| arg == "scan") {
        tracing::info!("🛡️ 执行代码安全扫描");
        let scan_args = construct_scan_args(&args);
        handle_scan(&config, scan_args, language_param.as_deref()).await?;
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
            "update-scan-rules" => {
                tracing::info!("🔄 更新代码扫描规则");
                handle_update_scan_rules(&config).await?;
                return Ok(());
            }
            "install-ast-grep" => {
                tracing::info!("📦 安装 ast-grep 可执行文件");
                handle_install_ast_grep().await?;
                return Ok(());
            }
            "check-ast-grep" => {
                tracing::info!("🔍 检查 ast-grep 安装状态");
                handle_check_ast_grep().await?;
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

/// Handle ast-grep installation command
async fn handle_install_ast_grep() -> Result<(), AppError> {
    println!("{}", "🔧 ast-grep 安装工具".bold().blue());
    
    let mut installer = AstGrepInstaller::new();
    
    // Show system information
    let system_info = installer.get_system_info();
    system_info.print();
    
    // Check if already installed
    if let Some(path) = installer.detect_ast_grep() {
        println!("{}", format!("✅ ast-grep 已经安装在: {}", path.display()).green());
        return Ok(());
    }
    
    // Attempt installation
    println!("{}", "🚀 开始安装 ast-grep...".cyan());
    match installer.ensure_ast_grep_available().await {
        Ok(path) => {
            println!("{}", format!("🎉 ast-grep 安装成功！路径: {}", path.display()).green());
            println!("{}", "现在您可以使用 gitai scan 命令进行代码扫描了。".green());
        }
        Err(e) => {
            println!("{}", format!("❌ 安装失败: {}", e).red());
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle ast-grep status check command
async fn handle_check_ast_grep() -> Result<(), AppError> {
    println!("{}", "🔍 ast-grep 状态检查".bold().blue());
    
    let installer = AstGrepInstaller::new();
    
    // Show system information
    let system_info = installer.get_system_info();
    system_info.print();
    
    // Check installation status
    if let Some(path) = installer.detect_ast_grep() {
        println!("{}", format!("✅ ast-grep 已安装: {}", path.display()).green());
        
        // Try to get version information
        match std::process::Command::new("sg").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("{}", format!("📦 版本信息: {}", version.trim()).blue());
                }
            }
            Err(_) => {
                println!("{}", "⚠️ 无法获取版本信息".yellow());
            }
        }
        
        // Check if it can run basic commands
        match std::process::Command::new("sg").arg("--help").output() {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", "✅ ast-grep 可以正常运行".green());
                } else {
                    println!("{}", "❌ ast-grep 运行异常".red());
                }
            }
            Err(e) => {
                println!("{}", format!("❌ 无法运行 ast-grep: {}", e).red());
            }
        }
    } else {
        println!("{}", "❌ ast-grep 未安装".red());
        println!("{}", "💡 使用 'gitai install-ast-grep' 命令进行安装".yellow());
    }
    
    Ok(())
}
