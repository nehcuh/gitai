mod config;
mod args;
mod devops;
mod git;
mod ai;
mod analysis;
mod commit;
mod update;
mod tree_sitter;
mod scan;
mod prompts;
mod review;

use std::path::PathBuf;
use std::fs;
use args::{Args, Command, PromptAction};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

/// 获取缓存目录
fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai");
    
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    
    let args = Args::parse();
    let config = config::Config::load()?;
    
    match args.command {
        Command::Review {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_id,
            deviation_analysis,
        } => {
            let review_config = review::ReviewConfig::from_args(
                language, format, output, tree_sitter, security_scan,
                scan_tool, block_on_critical, issue_id, deviation_analysis,
            );
            let executor = review::ReviewExecutor::new(config);
            executor.execute(review_config).await?;
        }
        Command::Scan {
            path,
            tool,
            full,
            remote,
            update_rules,
            format,
            output,
            translate,
            auto_install,
            lang,
            no_history,
            timeout,
            benchmark,
        } => {
            handle_scan(&config, &path, &tool, full, remote, update_rules, &format, output, translate, auto_install, lang.as_deref(), no_history, timeout, benchmark).await?;
        }
        Command::ScanHistory { limit, format: _ } => {
            handle_scan_history(limit)?;
        }
        Command::Prompts { action } => {
            handle_prompts_action(&config, &action).await?;
        }
        Command::Commit {
            message,
            issue_id,
            all,
            review,
            tree_sitter,
            dry_run,
        } => {
            let commit_config = commit::CommitConfig::from_args(message, issue_id, all, review, tree_sitter, dry_run);
            let executor = commit::CommitExecutor::new(config);
            executor.execute(commit_config).await?;
        }
        Command::Update { check, format } => {
            if check {
                handle_update_check(&config, &format).await?;
            } else {
                handle_update(&config).await?;
            }
        }
        Command::Git(git_args) => {
            // 默认不启用AI解释；--ai 显式开启；--noai 可显式关闭（当外部别名强制开启时）
            let use_ai = if args.ai { true } else if args.noai { false } else { false };
            if use_ai {
                handle_git_with_ai(&config, &git_args).await?;
            } else {
                let output = git::run_git(&git_args)?;
                print!("{output}");
            }
        }
    }
    
    Ok(())
}

// 扫描相关处理函数
async fn handle_scan(
    config: &config::Config,
    path: &std::path::Path,
    tool: &str,
    _full: bool,
    _remote: bool,
    update_rules: bool,
    _format: &str,
    output: Option<PathBuf>,
    _translate: bool,
    _auto_install: bool,
    lang: Option<&str>,
    no_history: bool,
    timeout: Option<u64>,
    benchmark: bool,
) -> Result<()> {
    let show_progress = _format != "json";
    
    if show_progress {
        println!("🔍 正在扫描: {}", path.display());
    }

    // 确保扫描工具已安装
    if (tool == "opengrep" || tool == "auto") && !scan::is_opengrep_installed() {
        if _auto_install {
            if show_progress {
                println!("🔧 未检测到 OpenGrep，正在自动安装...");
            }
            if let Err(e) = scan::install_opengrep() {
                return Err(format!("OpenGrep 安装失败: {}", e).into());
            }
        } else {
            return Err("未检测到 OpenGrep，请先安装或使用 --auto-install 进行自动安装".into());
        }
    }
    
    // 更新规则（如果需要）
    if update_rules {
        if show_progress {
            println!("🔄 正在更新扫描规则...");
        }
        let updater = update::AutoUpdater::new(config.clone());
        if let Err(e) = updater.update_scan_rules().await {
            eprintln!("⚠️ 规则更新失败: {}", e);
        }
    }
    
    // 执行扫描
    let result = if tool == "opengrep" || tool == "auto" {
        let include_version = show_progress && !benchmark;
        scan::run_opengrep_scan(config, path, lang, timeout, include_version)?
    } else {
        return Err(format!("不支持的扫描工具: {}", tool).into());
    };

    // 保存扫描历史（无论输出格式）
    if !(no_history || benchmark) {
        let cache_dir = get_cache_dir()?;
        let history_dir = cache_dir.join("scan_history");
        if let Err(e) = fs::create_dir_all(&history_dir) { eprintln!("⚠️ 无法创建扫描历史目录: {}", e); }
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let history_file = history_dir.join(format!("scan_{}_{}.json", result.tool, ts));
        if let Ok(json) = serde_json::to_string(&result) {
            if let Err(e) = fs::write(&history_file, json) { eprintln!("⚠️ 写入扫描历史失败: {}", e); }
        }
    }
    
    // 输出结果
    if _format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        if let Some(output_path) = output {
            fs::write(output_path, json)?;
        } else {
            println!("{}", json);
        }
    } else {
        if show_progress {
            println!("📊 扫描结果:");
            println!("  工具: {}", result.tool);
            println!("  版本: {}", result.version);
            println!("  执行时间: {:.2}s", result.execution_time);
            
            if !result.findings.is_empty() {
                println!("  发现问题: {}", result.findings.len());
                for finding in result.findings.iter().take(5) {
                    println!("    - {} ({}:{})", finding.title, finding.file_path.display(), finding.line);
                }
                if result.findings.len() > 5 {
                    println!("    ... 还有 {} 个问题", result.findings.len() - 5);
                }
            } else {
                println!("  ✅ 未发现问题");
            }
        }
    }
    
    Ok(())
}

async fn handle_update_check(config: &config::Config, format: &str) -> Result<()> {
    let updater = update::AutoUpdater::new(config.clone());
    let status = updater.check_update_status();
    
    if format == "json" {
        let json = serde_json::to_string_pretty(&status)?;
        println!("{}", json);
    } else {
        println!("🔎 更新检查:");
        println!();
        
        for item in &status {
            println!("📦 {}: {}", item.name, item.message);
        }
        
        println!();
        if status.is_empty() {
            println!("就绪状态: ✅ 已就绪");
        } else {
            println!("就绪状态: ❌ 需要更新");
        }
    }
    
    Ok(())
}

async fn handle_update(config: &config::Config) -> Result<()> {
    println!("🔄 正在更新规则...");
    let updater = update::AutoUpdater::new(config.clone());
    let result = updater.update_scan_rules().await?;
    
    println!("✅ 更新完成");
    println!("   更新状态: {}", result.message);
    
    Ok(())
}

fn handle_scan_history(limit: usize) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let history_dir = cache_dir.join("scan_history");
    
    if !history_dir.exists() {
        println!("📁 扫描历史目录不存在");
        return Ok(());
    }
    
    // 获取历史文件
    let mut entries: Vec<_> = fs::read_dir(&history_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|s| s.to_str()) == Some("json")
        })
        .collect();
    
    // 按修改时间排序（最新的在前）
    entries.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });
    
    println!("📋 扫描历史 (最近{}次):", limit);
    println!();
    
    for (i, entry) in entries.iter().take(limit).enumerate() {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(result) = serde_json::from_str::<scan::ScanResult>(&content) {
                let modified = entry.metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                    .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知时间".to_string());
                
                println!("{}. {} - {}", i + 1, modified, result.tool);
                println!("   执行时间: {:.2}s", result.execution_time);
                println!("   发现问题: {}", result.findings.len());
                if !result.findings.is_empty() {
                    println!("   前3个问题:");
                    for finding in result.findings.iter().take(3) {
                        println!("     - {}", finding.title);
                    }
                }
                println!();
            }
        }
    }
    
    Ok(())
}

async fn handle_git_with_ai(config: &config::Config, git_args: &[String]) -> Result<()> {
    // 执行Git命令
    let output = git::run_git(git_args)?;
    print!("{output}");
    
    // 添加AI解释
    let command_str = format!("git {}", git_args.join(" "));
    let prompt = format!(
        "用户刚刚执行了以下Git命令：\n\n{}\n\n命令输出：\n{}\n\n请简洁地解释这个命令的作用和输出结果。",
        command_str,
        output.trim()
    );
    
    match ai::call_ai(config, &prompt).await {
        Ok(explanation) => {
            println!("\n🤖 AI解释:");
            println!("{}", explanation);
        }
        Err(e) => {
            log::warn!("AI解释失败: {}", e);
        }
    }
    
    Ok(())
}

async fn handle_prompts_action(_config: &config::Config, action: &PromptAction) -> Result<()> {
    match action {
        PromptAction::Init => {
            println!("🔄 正在初始化提示词目录...");
            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");
            
            fs::create_dir_all(&prompts_dir)?;
            
            // 创建默认模板
            let templates = [
                ("commit-generator.md", include_str!("../assets/prompts/commit-generator.md")),
                ("review.md", include_str!("../assets/prompts/review.md")),
            ];
            
            for (filename, content) in &templates {
                let file_path = prompts_dir.join(filename);
                if !file_path.exists() {
                    fs::write(&file_path, content)?;
                }
            }
            
            println!("✅ 提示词目录已就绪: {}", prompts_dir.display());
        }
        PromptAction::List => {
            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");
            
            if !prompts_dir.exists() {
                println!("📁 提示词目录不存在，请先运行: gitai prompts init");
                return Ok(());
            }
            
            println!("📝 可用的提示词模板:");
            let entries = fs::read_dir(&prompts_dir)?;
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            println!("  - {}", name);
                        }
                    }
                }
            }
        }
        PromptAction::Show { name, language: _ } => {
            let prompts_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("gitai")
                .join("prompts");
            
            let file_path = prompts_dir.join(format!("{}.md", name));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                println!("📝 提示词模板: {}", name);
                println!("{}", content);
            } else {
                println!("❌ 未找到提示词模板: {}", name);
            }
        }
        PromptAction::Update => {
            println!("🔄 更新提示词模板功能暂未实现");
        }
    }
    
    Ok(())
}
