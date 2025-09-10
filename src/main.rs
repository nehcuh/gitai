#![allow(
    clippy::uninlined_format_args,
    clippy::collapsible_else_if,
    clippy::wildcard_in_or_patterns,
    clippy::too_many_arguments,
    clippy::unnecessary_map_or
)]

// Use modules from the library crate
use gitai::{
    args::{ConfigAction, PromptAction},
    config::{self},
    git,
};

#[cfg(feature = "metrics")]
use gitai::args::MetricsAction;

// Conditionally import feature-gated modules
#[cfg(feature = "ai")]
use gitai::ai;

#[cfg(feature = "security")]
use gitai::scan;

#[cfg(feature = "update-notifier")]
use gitai::update;

#[cfg(feature = "metrics")]
use gitai::metrics;

// Always available modules (used in legacy code)
#[allow(unused_imports)]
use gitai::{commit, features, review};

use std::fs;
use std::path::PathBuf;
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
#[allow(dead_code)]
fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai");

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    let args = gitai::args::Args::parse();
    let mut app = cli::CliApp::new(args);

    // Initialize configuration if needed
    app.initialize().await?;

    // Run the application
    app.run().await
}

// All command handling is now managed by cli::CliApp
// Legacy command handling code has been fully migrated to cli::handlers module
/*
        Command::Review {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_id,
            space_id,
            full,
            ..
        } => {
            let review_config = review::ReviewConfig::from_args(
                language,
                format,
                output,
                tree_sitter,
                security_scan,
                scan_tool,
                block_on_critical,
                issue_id,
                space_id,
                full,
            );
            review::execute_review(&config, review_config).await?;
        }
        #[cfg(feature = "security")]
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
            handle_scan(
                &config,
                &path,
                &tool,
                full,
                remote,
                update_rules,
                &format,
                output,
                translate,
                auto_install,
                lang.as_deref(),
                no_history,
                timeout,
                benchmark,
            )
            .await?;
        }
        #[cfg(not(feature = "security"))]
        Command::Scan { .. } => {
            eprintln!("❌ 安全扫描功能未启用");
            eprintln!("💡 请使用包含 'security' 功能的构建版本");
            return Err("功能未启用".into());
        }
        #[cfg(feature = "security")]
        Command::ScanHistory { limit, format: _ } => {
            handle_scan_history(limit)?;
        }
        #[cfg(not(feature = "security"))]
        Command::ScanHistory { .. } => {
            eprintln!("❌ 安全扫描历史功能未启用");
            eprintln!("💡 请使用包含 'security' 功能的构建版本");
            return Err("功能未启用".into());
        }
        Command::Prompts { action } => {
            handle_prompts_action(&config, &action).await?;
        }
        Command::Commit {
            message,
            issue_id,
            space_id,
            all,
            review,
            tree_sitter,
            dry_run,
        } => {
            let commit_config = commit::CommitConfig::from_args(
                message,
                issue_id,
                space_id,
                all,
                review,
                tree_sitter,
                dry_run,
            );
            commit::execute_commit(&config, commit_config).await?;
        }
        #[cfg(feature = "update-notifier")]
        Command::Update { check, format } => {
            if check {
                handle_update_check(&config, &format).await?;
            } else {
                handle_update(&config).await?;
            }
        }
        #[cfg(not(feature = "update-notifier"))]
        Command::Update { .. } => {
            eprintln!("❌ 更新功能未启用");
            eprintln!("💡 请使用包含 'update-notifier' 功能的构建版本");
            return Err("功能未启用".into());
        }
        Command::Git(git_args) => {
            // 默认不启用AI解释；--ai 显式开启；--noai 可显式关闭（当外部别名强制开启时）
            let use_ai = args.ai && !args.noai;

            #[cfg(feature = "ai")]
            {
                if use_ai {
                    handle_git_with_ai(&config, &git_args).await?;
                } else {
                    let output = git::run_git(&git_args)?;
                    print!("{output}");
                }
            }

            #[cfg(not(feature = "ai"))]
            {
                // 未启用 AI 时，总是直接执行 git
                let output = git::run_git(&git_args)?;
                print!("{output}");
            }
        }
        #[cfg(feature = "mcp")]
        Command::Mcp { transport, addr } => {
            handle_mcp(&config, &transport, &addr).await?;
        }
        #[cfg(not(feature = "mcp"))]
        Command::Mcp { .. } => {
            eprintln!("❌ MCP 服务器功能未启用");
            eprintln!("💡 请使用包含 'mcp' 功能的构建版本");
            return Err("功能未启用".into());
        }
        Command::Init { .. } => {
            // 已在上面处理
            unreachable!()
        }
        Command::Config { action } => {
            handle_config(&config, &action, args.offline).await?;
        }
        #[cfg(feature = "metrics")]
        Command::Metrics { action } => {
            handle_metrics(&config, &action).await?;
        }
        #[cfg(not(feature = "metrics"))]
        Command::Metrics { .. } => {
            eprintln!("❌ 度量功能未启用");
            eprintln!("💡 请使用包含 'metrics' 功能的构建版本");
            return Err("功能未启用".into());
        }
        Command::Graph {
            path,
            output,
            threshold,
            summary,
            radius,
            top_k,
            seeds_from_diff,
            summary_format,
            budget_tokens,
            community,
            comm_alg,
            max_communities,
            max_nodes_per_community,
            with_paths,
            path_samples,
            path_max_hops,
        } => {
            if summary {
                handle_graph_summary(
                    &path,
                    radius,
                    top_k,
                    budget_tokens,
                    seeds_from_diff,
                    &summary_format,
                    community,
                    &comm_alg,
                    max_communities,
                    max_nodes_per_community,
                    with_paths,
                    path_samples,
                    path_max_hops,
                    output.as_ref(),
                )
                .await?;
            } else {
                handle_graph_export(&path, output.as_ref(), threshold).await?;
            }
        }
        Command::Features { format } => {
            features::display_features(&format);
        }
    }

    Ok(())
}
*/

// The following helper functions have been migrated to cli::handlers module
// These are kept temporarily for reference and can be removed after full verification

#[allow(dead_code)]
async fn handle_graph_export(
    path: &std::path::Path,
    output: Option<&std::path::PathBuf>,
    threshold: f32,
) -> Result<()> {
    use gitai::architectural_impact::graph_export::export_dot_string;
    let dot = export_dot_string(path, threshold).await?;
    if let Some(out) = output {
        std::fs::write(out, dot)?;
        println!("📁 依赖图已导出: {}", out.display());
    } else {
        println!("{dot}");
    }
    Ok(())
}

#[allow(dead_code, clippy::too_many_arguments)]
async fn handle_graph_summary(
    path: &std::path::Path,
    radius: usize,
    top_k: usize,
    budget_tokens: usize,
    seeds_from_diff: bool,
    format: &str,
    with_communities: bool,
    comm_alg: &str,
    max_communities: usize,
    max_nodes_per_community: usize,
    with_paths: bool,
    path_samples: usize,
    path_max_hops: usize,
    output: Option<&std::path::PathBuf>,
) -> Result<()> {
    use gitai::architectural_impact::graph_export::export_summary_string;
    let summary = export_summary_string(
        path,
        radius,
        top_k,
        seeds_from_diff,
        format,
        budget_tokens,
        with_communities,
        comm_alg,
        max_communities,
        max_nodes_per_community,
        with_paths,
        path_samples,
        path_max_hops,
    )
    .await?;
    if let Some(out) = output {
        std::fs::write(out, &summary)?;
        println!("📁 图摘要已导出: {}", out.display());
    } else {
        println!("{summary}");
    }
    Ok(())
}

// 扫描相关处理函数
#[cfg(feature = "security")]
#[allow(dead_code)]
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
    // 将 'security' 映射为 'opengrep' 以保持向后兼容性
    let normalized_tool = match tool {
        "security" => "opengrep",
        other => other,
    };

    if (normalized_tool == "opengrep" || normalized_tool == "auto")
        && !scan::is_opengrep_installed()
    {
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
        #[cfg(feature = "update-notifier")]
        {
            let updater = update::AutoUpdater::new(config.clone());
            if let Err(e) = updater.update_scan_rules().await {
                eprintln!("⚠️ 规则更新失败: {}", e);
            }
        }
        #[cfg(not(feature = "update-notifier"))]
        {
            eprintln!("ℹ️  update-notifier 功能未启用，跳过规则更新。");
        }
    }

    // 执行扫描
    let result = if normalized_tool == "opengrep" || normalized_tool == "auto" {
        let include_version = show_progress && !benchmark;
        scan::run_opengrep_scan(config, path, lang, timeout, include_version)?
    } else {
        return Err(format!(
            "不支持的扫描工具: {} (支持的工具: opengrep, security, auto)",
            tool
        )
        .into());
    };

    // 保存扫描历史（无论输出格式）
    if !(no_history || benchmark) {
        let cache_dir = get_cache_dir()?;
        let history_dir = cache_dir.join("scan_history");
        if let Err(e) = fs::create_dir_all(&history_dir) {
            eprintln!("⚠️ 无法创建扫描历史目录: {}", e);
        }
        let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
        let history_file = history_dir.join(format!("scan_{}_{}.json", result.tool, ts));
        if let Ok(json) = serde_json::to_string(&result) {
            if let Err(e) = fs::write(&history_file, json) {
                eprintln!("⚠️ 写入扫描历史失败: {}", e);
            }
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
                    println!(
                        "    - {} ({}:{})",
                        finding.title,
                        finding.file_path.display(),
                        finding.line
                    );
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

#[cfg(feature = "security")]
#[allow(dead_code)]
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
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    // 按修改时间排序（最新的在前）
    entries.sort_by(|a, b| {
        let a_time = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });

    println!("📋 扫描历史 (最近{}次):", limit);
    println!();

    for (i, entry) in entries.iter().take(limit).enumerate() {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(result) = serde_json::from_str::<scan::ScanResult>(&content) {
                let modified = entry
                    .metadata()
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

#[cfg(feature = "ai")]
#[allow(dead_code)]
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
            println!("{explanation}");
        }
        Err(e) => {
            log::warn!("AI解释失败: {e}");
        }
    }

    Ok(())
}

#[allow(dead_code)]
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
                ("commit.md", include_str!("../assets/prompts/commit.md")),
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
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        println!("  - {name}");
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

            let file_path = prompts_dir.join(format!("{name}.md"));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                println!("📝 提示词模板: {name}");
                println!("{content}");
            } else {
                println!("❌ 未找到提示词模板: {name}");
            }
        }
        PromptAction::Update => {
            println!("🔄 更新提示词模板功能暂未实现");
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn handle_init(
    config_url: Option<String>,
    offline: bool,
    _resources_dir: Option<PathBuf>,
    _dev: bool,
    download_resources: bool,
) -> Result<()> {
    use gitai::config_init::ConfigInitializer;

    println!("🚀 初始化 GitAI 配置...");

    let mut initializer = ConfigInitializer::new();

    if let Some(url) = config_url {
        println!("📥 使用配置URL: {url}");
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

            // 如果需要下载资源
            if download_resources && !offline {
                println!();
                println!("📦 正在下载资源...");

                // 下载 Tree-sitter queries
                println!("🌳 下载 Tree-sitter queries...");
                match download_tree_sitter_resources().await {
                    Ok(()) => println!("✅ Tree-sitter queries 下载完成"),
                    Err(e) => eprintln!("⚠️  Tree-sitter queries 下载失败: {e}"),
                }

                // 下载 OpenGrep 规则（如果可能的话）
                println!("🔒 下载 OpenGrep 规则...");
                match download_opengrep_resources(&config_path).await {
                    Ok(()) => println!("✅ OpenGrep 规则下载完成"),
                    Err(e) => eprintln!("⚠️  OpenGrep 规则下载失败: {e}"),
                }

                println!("✅ 资源下载完成！");
            } else if download_resources && offline {
                println!();
                println!("⚠️  离线模式下无法下载资源");
            }

            println!();
            println!("🎉 您现在可以使用 GitAI 了:");
            println!("  gitai review     - 代码评审");
            println!("  gitai commit     - 智能提交");
            println!("  gitai scan       - 安全扫描");
            println!("  gitai --help     - 查看更多命令");
        }
        Err(e) => {
            eprintln!("❌ 初始化失败: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn handle_config(
    config: &config::Config,
    action: &ConfigAction,
    offline: bool,
) -> Result<()> {
    use gitai::resource_manager::{load_resource_config, ResourceManager};

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
                    println!(
                        "    \"default_path\": \"{}\"",
                        config.scan.default_path.as_deref().unwrap_or(".")
                    );
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
                    println!(
                        "  扫描路径: {}",
                        config.scan.default_path.as_deref().unwrap_or(".")
                    );
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

#[cfg(feature = "metrics")]
#[allow(dead_code)]
async fn handle_metrics(_config: &config::Config, action: &MetricsAction) -> Result<()> {
    use gitai::metrics::QualityTracker;
    use gitai::project_insights::InsightsGenerator;
    use gitai::tree_sitter::TreeSitterManager;

    match action {
        MetricsAction::Record { tags, force } => {
            println!("📊 记录代码质量快照...");

            // 检查是否有代码变化（除非强制记录）
            if !force {
                let status = git::run_git(&["status".to_string(), "--porcelain".to_string()])?;
                if status.trim().is_empty() {
                    println!("ℹ️  没有检测到代码变化");
                    println!("💡 使用 --force 强制记录快照");
                    return Ok(());
                }
            }

            // 创建质量追踪器
            let mut tracker = QualityTracker::new()?;

            // 分析当前代码
            println!("🔍 分析代码结构...");
            let mut manager = TreeSitterManager::new().await?;

            // 获取当前目录的代码文件并分析
            let mut summary = gitai::tree_sitter::StructuralSummary::default();
            let code_files = find_code_files(".")?;

            for file_path in &code_files {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                        if let Some(lang) =
                            gitai::tree_sitter::SupportedLanguage::from_extension(ext)
                        {
                            if let Ok(file_summary) = manager.analyze_structure(&content, lang) {
                                // 合并结果
                                summary.functions.extend(file_summary.functions);
                                summary.classes.extend(file_summary.classes);
                                summary.comments.extend(file_summary.comments);
                            }
                        }
                    }
                }
            }

            // 生成项目洞察
            println!("💡 生成项目洞察...");
            let insights = InsightsGenerator::generate(&summary, None);

            // 记录快照
            let mut snapshot = tracker.record_snapshot(&summary, &insights)?;

            // 添加标签
            if !tags.is_empty() {
                snapshot.tags = tags.clone();
            }

            println!("✅ 质量快照已记录");
            println!("   Commit: {}", &snapshot.commit_hash[..7]);
            println!("   分支: {}", snapshot.branch);
            println!("   代码行数: {}", snapshot.lines_of_code);
            println!("   技术债务: {:.1}", snapshot.technical_debt.debt_score);
            println!(
                "   复杂度: {:.1}",
                snapshot.complexity_metrics.avg_cyclomatic_complexity
            );
        }
        MetricsAction::Analyze {
            days,
            format,
            output,
        } => {
            println!("📈 分析质量趋势...");

            let tracker = QualityTracker::new()?;
            let analysis = tracker.analyze_trends(*days)?;

            let result = match format.as_str() {
                "json" => serde_json::to_string_pretty(&analysis)?,
                "markdown" | "html" => {
                    let visualizer = metrics::visualizer::TrendVisualizer::new();
                    if format == "html" {
                        visualizer.generate_html_report(&analysis, tracker.get_snapshots())?
                    } else {
                        visualizer.generate_report(&analysis, tracker.get_snapshots())?
                    }
                }
                _ => {
                    // 文本格式
                    format!(
                        "质量趋势分析\n\n\
                        整体趋势: {:?}\n\
                        时间范围: {} 到 {}\n\
                        快照数量: {}\n\
                        关键发现: {}\n\
                        改进建议: {}\n",
                        analysis.overall_trend,
                        analysis.time_range.start.format("%Y-%m-%d"),
                        analysis.time_range.end.format("%Y-%m-%d"),
                        analysis.time_range.snapshots_count,
                        analysis.key_findings.len(),
                        analysis.recommendations.len()
                    )
                }
            };

            if let Some(output_path) = output {
                std::fs::write(output_path, result)?;
                println!("📁 分析结果已保存到: {}", output_path.display());
            } else {
                println!("{}", result);
            }
        }
        MetricsAction::Report {
            report_type: _,
            output,
            html,
        } => {
            println!("📄 生成质量报告...");

            let tracker = QualityTracker::new()?;

            let report = if *html {
                let analysis = tracker.analyze_trends(None)?;
                let visualizer = metrics::visualizer::TrendVisualizer::new();
                visualizer.generate_html_report(&analysis, tracker.get_snapshots())?
            } else {
                tracker.generate_report(output.as_deref())?
            };

            if let Some(output_path) = output {
                std::fs::write(output_path, report)?;
                println!("✅ 报告已生成: {}", output_path.display());
            } else {
                println!("{}", report);
            }
        }
        MetricsAction::List {
            limit,
            branch,
            format,
        } => {
            let tracker = QualityTracker::new()?;
            let snapshots = tracker.get_snapshots();

            // 过滤分支
            let filtered: Vec<_> = if let Some(branch_name) = branch {
                snapshots
                    .iter()
                    .filter(|s| s.branch == *branch_name)
                    .collect()
            } else {
                snapshots.iter().collect()
            };

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(
                        &filtered.into_iter().take(*limit).collect::<Vec<_>>(),
                    )?;
                    println!("{}", json);
                }
                "table" | _ => {
                    println!("📋 历史快照 (最近{}个):", limit);
                    println!("┌────┬──────────────┬─────────┬──────┬─────────┬────────┬────────┐");
                    println!("│ #  │ 时间         │ Commit  │ LOC  │ 债务    │ 复杂度 │ API稳定│");
                    println!("├────┼──────────────┼─────────┼──────┼─────────┼────────┼────────┤");

                    for (i, snapshot) in filtered.iter().rev().take(*limit).enumerate() {
                        println!(
                            "│{:3} │ {} │ {:7} │{:5} │{:8.1} │{:7.1} │{:7.0}%│",
                            i + 1,
                            snapshot.timestamp.format("%m-%d %H:%M"),
                            &snapshot.commit_hash[..7],
                            snapshot.lines_of_code,
                            snapshot.technical_debt.debt_score,
                            snapshot.complexity_metrics.avg_cyclomatic_complexity,
                            snapshot.api_metrics.stability_score,
                        );
                    }
                    println!("└────┴──────────────┴─────────┴──────┴─────────┴────────┴────────┘");
                }
            }
        }
        MetricsAction::Compare { from, to, format } => {
            let tracker = QualityTracker::new()?;
            let snapshots = tracker.get_snapshots();

            // 查找快照
            let from_snapshot = if from == "latest" {
                snapshots.last()
            } else if let Ok(index) = from.parse::<usize>() {
                snapshots.get(index.saturating_sub(1))
            } else {
                snapshots.iter().find(|s| s.commit_hash.starts_with(from))
            };

            let to_snapshot = if let Some(to_ref) = to {
                if to_ref == "latest" {
                    snapshots.last()
                } else if let Ok(index) = to_ref.parse::<usize>() {
                    snapshots.get(index.saturating_sub(1))
                } else {
                    snapshots.iter().find(|s| s.commit_hash.starts_with(to_ref))
                }
            } else {
                snapshots.last()
            };

            match (from_snapshot, to_snapshot) {
                (Some(from_s), Some(to_s)) => {
                    let changes = tracker.compare_snapshots(from_s, to_s);

                    if format == "json" {
                        println!("{}", serde_json::to_string_pretty(&changes)?);
                    } else {
                        println!("📊 快照比较:");
                        println!(
                            "   从: {} ({})",
                            &from_s.commit_hash[..7],
                            from_s.timestamp.format("%Y-%m-%d")
                        );
                        println!(
                            "   到: {} ({})",
                            &to_s.commit_hash[..7],
                            to_s.timestamp.format("%Y-%m-%d")
                        );
                        println!();
                        println!("   变化:");
                        for (key, value) in &changes {
                            let emoji = if *value > 0.0 {
                                "📈"
                            } else if *value < 0.0 {
                                "📉"
                            } else {
                                "➡️"
                            };
                            println!("     {} {}: {:+.2}", emoji, key, value);
                        }
                    }
                }
                _ => {
                    eprintln!("❌ 未找到指定的快照");
                }
            }
        }
        MetricsAction::Clean { keep_days, yes } => {
            if !yes {
                println!("⚠️  确认清理超过{}天的历史数据？使用 --yes 确认", keep_days);
                return Ok(());
            }

            let mut tracker = QualityTracker::new()?;
            let removed = tracker.cleanup_old_snapshots(*keep_days)?;
            println!("🧹 已清理 {} 个旧快照", removed);
        }
        MetricsAction::Export {
            format,
            output,
            branches,
        } => {
            println!("📤 导出质量数据...");

            let tracker = QualityTracker::new()?;
            let snapshots = if branches.is_empty() {
                tracker.get_snapshots().to_vec()
            } else {
                tracker
                    .get_snapshots()
                    .iter()
                    .filter(|s| branches.contains(&s.branch))
                    .cloned()
                    .collect()
            };

            match format.as_str() {
                "csv" => {
                    metrics::storage::export_to_csv(&snapshots, output)?;
                    println!("✅ 已导出到: {}", output.display());
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&snapshots)?;
                    std::fs::write(output, json)?;
                    println!("✅ 已导出到: {}", output.display());
                }
                _ => {
                    eprintln!("❌ 不支持的导出格式: {}", format);
                }
            }
        }
    }

    Ok(())
}

// 辅助函数：查找代码文件
/// 下载 Tree-sitter 资源
#[allow(dead_code)]
async fn download_tree_sitter_resources() -> Result<()> {
    // 创建 TreeSitterManager 实例，这通常会触发初始化和下载
    // 检查是否启用了任意 Tree-sitter 语言支持
    #[cfg(any(
        feature = "tree-sitter-rust",
        feature = "tree-sitter-java",
        feature = "tree-sitter-python",
        feature = "tree-sitter-javascript",
        feature = "tree-sitter-typescript",
        feature = "tree-sitter-go",
        feature = "tree-sitter-c",
        feature = "tree-sitter-cpp"
    ))]
    {
        match gitai::tree_sitter::TreeSitterManager::new().await {
            Ok(_) => {
                log::info!("Tree-sitter 资源初始化成功");
                Ok(())
            }
            Err(e) => {
                log::warn!("Tree-sitter 资源初始化失败: {e}");
                Err(format!("Tree-sitter 资源下载失败: {e}").into())
            }
        }
    }
    #[cfg(not(any(
        feature = "tree-sitter-rust",
        feature = "tree-sitter-java",
        feature = "tree-sitter-python",
        feature = "tree-sitter-javascript",
        feature = "tree-sitter-typescript",
        feature = "tree-sitter-go",
        feature = "tree-sitter-c",
        feature = "tree-sitter-cpp"
    )))]
    {
        log::info!("Tree-sitter 功能未启用，跳过资源下载");
        Ok(())
    }
}

/// 下载 OpenGrep 规则资源
#[allow(dead_code)]
async fn download_opengrep_resources(_config_path: &std::path::Path) -> Result<()> {
    #[cfg(feature = "security")]
    {
        use gitai::resource_manager::{load_resource_config, ResourceManager};

        // 尝试加载资源配置
        match load_resource_config(_config_path) {
            Ok(resource_config) => {
                let manager = ResourceManager::new(resource_config)?;
                match manager.update_all().await {
                    Ok(_) => {
                        log::info!("OpenGrep 规则资源更新成功");
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("OpenGrep 规则资源更新失败: {}", e);
                        Err(format!("OpenGrep 规则下载失败: {}", e).into())
                    }
                }
            }
            Err(e) => {
                log::warn!("无法加载资源配置: {}", e);
                // 不将此视为错误，因为可能配置还未完全设置
                Ok(())
            }
        }
    }
    #[cfg(not(feature = "security"))]
    {
        log::info!("安全扫描功能未启用，跳过 OpenGrep 规则下载");
        Ok(())
    }
}

// 辅助函数：查找代码文件
#[cfg(feature = "metrics")]
#[allow(dead_code)]
fn find_code_files(dir: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let supported_extensions = ["rs", "java", "py", "js", "ts", "go", "c", "cpp"];

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
    {
        let path = entry.path();

        // 跳过隐藏目录和常见的排除目录
        if path.components().any(|c| {
            c.as_os_str().to_str().is_some_and(|s| {
                s.starts_with('.') || s == "target" || s == "node_modules" || s == "build"
            })
        }) {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if supported_extensions.contains(&ext) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}
