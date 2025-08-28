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
mod mcp;

use std::path::PathBuf;
use std::fs;
use args::{Args, Command, PromptAction, ConfigAction};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn init_logger() {
    use std::io::Write;
    
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "\x1b[31m", // 红色
                log::Level::Warn => "\x1b[33m",  // 黄色
                log::Level::Info => "\x1b[32m",  // 绿色
                log::Level::Debug => "\x1b[36m", // 青色
                log::Level::Trace => "\x1b[90m", // 灰色
            };
            
            writeln!(
                buf,
                "{}{} [{}] {}",
                level_style,
                chrono::Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
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
    
    // 处理 Init 命令（不需要配置）
    if let Command::Init { config_url, offline, resources_dir, dev } = &args.command {
        return handle_init(
            config_url.clone(),
            *offline || args.offline,
            resources_dir.clone(),
            *dev
        ).await;
    }
    
    // 加载配置文件，提供友好错误信息
    let config = match config::Config::load() {
        Ok(config) => {
            log::debug!("配置文件加载成功");
            config
        }
        Err(e) => {
            eprintln!("❌ 配置加载失败: {}", e);
            eprintln!("💡 提示: 请检查 ~/.config/gitai/config.toml 文件");
            eprintln!("💡 可以使用 'gitai init' 初始化配置");
            return Err(format!("配置加载失败: {}", e).into());
        }
    };
    
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
        Command::Mcp { transport, addr } => {
            handle_mcp(&config, &transport, &addr).await?;
        }
        Command::Init { .. } => {
            // 已在上面处理
            unreachable!()
        }
        Command::Config { action } => {
            handle_config(&config, &action, args.offline).await?;
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

async fn handle_init(
    config_url: Option<String>,
    offline: bool,
    _resources_dir: Option<PathBuf>,
    _dev: bool,
) -> Result<()> {
    use gitai::config_init::ConfigInitializer;
    
    println!("🚀 初始化 GitAI 配置...");
    
    let mut initializer = ConfigInitializer::new();
    
    if let Some(url) = config_url {
        println!("📥 使用配置URL: {}", url);
        initializer = initializer.with_config_url(Some(url));
    }
    
    if offline {
        println!("🔌 离线模式初始化");
        initializer = initializer.with_offline_mode(true);
    }
    
    match initializer.initialize().await {
        Ok(config_path) => {
            println!("✅ 配置初始化成功!");
            println!("📁 配置文件: {}", config_path.display());
            println!();
            println!("🎉 您现在可以使用 GitAI 了:");
            println!("  gitai review     - 代码评审");
            println!("  gitai commit     - 智能提交");
            println!("  gitai scan       - 安全扫描");
            println!("  gitai --help     - 查看更多命令");
        }
        Err(e) => {
            eprintln!("❌ 初始化失败: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn handle_config(config: &config::Config, action: &ConfigAction, offline: bool) -> Result<()> {
    use gitai::resource_manager::{ResourceManager, load_resource_config};
    
    match action {
        ConfigAction::Check => {
            println!("🔍 检查配置状态...");
            
            // 检查配置文件
            let config_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai");
            let config_path = config_dir.join("config.toml");
            
            if config_path.exists() {
                println!("✅ 配置文件: {}", config_path.display());
            } else {
                println!("❌ 配置文件不存在");
            }
            
            // 检查缓存目录
            let cache_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache/gitai");
            
            if cache_dir.exists() {
                println!("✅ 缓存目录: {}", cache_dir.display());
                
                // 检查规则
                let rules_dir = cache_dir.join("rules");
                if rules_dir.exists() {
                    println!("  ✅ 规则缓存: 已就绪");
                } else {
                    println!("  ⚠️  规则缓存: 未找到");
                }
                
                // 检查 Tree-sitter
                let ts_dir = cache_dir.join("tree-sitter");
                if ts_dir.exists() {
                    println!("  ✅ Tree-sitter缓存: 已就绪");
                } else {
                    println!("  ⚠️  Tree-sitter缓存: 未找到");
                }
            } else {
                println!("❌ 缓存目录不存在");
            }
        }
        ConfigAction::Show { format } => {
            match format.as_str() {
                "json" => {
                    // Config 可能没有实现 Serialize，暂时用简单格式
                    println!("{{");
                    println!("  \"ai\": {{");
                    println!("    \"api_url\": \"{}\",", config.ai.api_url);
                    println!("    \"model\": \"{}\"", config.ai.model);
                    println!("  }},");
                    println!("  \"scan\": {{");
                    println!("    \"default_path\": \"{}\"", config.scan.default_path.as_deref().unwrap_or("."));
                    println!("  }}");
                    println!("}}");
                }
                "toml" => {
                    // Config 类型可能没有实现 Serialize，暂时显示简单信息
                    println!("📋 TOML 格式输出暂不可用");
                }
                _ => {
                    println!("📋 当前配置:");
                    println!("  AI服务: {}", config.ai.api_url);
                    println!("  AI模型: {}", config.ai.model);
                    // config.scan 是 ScanConfig 类型，不是 Option
                    println!("  扫描路径: {}", config.scan.default_path.as_deref().unwrap_or("."));
                }
            }
        }
        ConfigAction::Update { force } => {
            println!("🔄 更新资源...");
            
            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");
            
            if let Ok(resource_config) = load_resource_config(&config_path) {
                let manager = ResourceManager::new(resource_config)?;
                
                if offline {
                    eprintln!("⚠️  离线模式下无法更新资源");
                    return Ok(());
                }
                
                if *force {
                    println!("🚀 强制更新所有资源...");
                }
                
                manager.update_all().await?;
                println!("✅ 资源更新完成");
            } else {
                eprintln!("❌ 无法加载资源配置");
            }
        }
        ConfigAction::Reset { no_backup } => {
            println!("🔄 重置配置...");
            
            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");
            
            if !no_backup && config_path.exists() {
                let backup_path = config_path.with_extension("toml.backup");
                fs::copy(&config_path, &backup_path)?;
                println!("💾 已备份到: {}", backup_path.display());
            }
            
            // 写入默认配置
            let default_config = include_str!("../assets/config.enhanced.toml");
            fs::write(&config_path, default_config)?;
            println!("✅ 配置已重置到默认值");
        }
        ConfigAction::Clean => {
            println!("🧹 清理缓存...");
            
            let config_path = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config/gitai/config.toml");
            
            if let Ok(resource_config) = load_resource_config(&config_path) {
                let manager = ResourceManager::new(resource_config)?;
                manager.clean_cache().await?;
                println!("✅ 缓存清理完成");
            } else {
                eprintln!("❌ 无法加载资源配置");
            }
        }
    }
    
    Ok(())
}

async fn handle_mcp(config: &config::Config, transport: &str, addr: &str) -> Result<()> {
    // 检查 MCP 是否启用
    if !config.mcp.as_ref().map_or(false, |mcp| mcp.enabled) {
        eprintln!("❌ MCP 服务未启用，请在配置文件中启用 MCP");
        std::process::exit(1);
    }
    
    println!("🚀 启动 GitAI MCP 服务器");
    println!("📡 传输协议: {}", transport);
    
    match transport {
        "stdio" => {
            println!("🔌 使用 stdio 传输");
            mcp::bridge::start_mcp_server(config.clone()).await?;
        }
        "tcp" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  TCP 传输暂未实现");
        }
        "sse" => {
            println!("🌐 监听地址: {}", addr);
            eprintln!("⚠️  SSE 传输暂未实现");
        }
        _ => {
            eprintln!("❌ 不支持的传输协议: {}", transport);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
