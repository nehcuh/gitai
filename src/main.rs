mod config;
mod args;
mod git;
mod ai;
mod scan;

/// 扫描参数结构体
struct ScanParams<'a> {
    config: &'a Config,
    path: &'a std::path::Path,
    tool: &'a str,
    full: bool,
    remote: bool,
    update_rules: bool,
    format: &'a str,
    output: Option<std::path::PathBuf>,
    translate: bool,
    auto_install: bool,
}

#[allow(dead_code)]
impl<'a> ScanParams<'a> {
    fn new(
        config: &'a Config,
        path: &'a std::path::Path,
        tool: &'a str,
        full: bool,
        remote: bool,
        update_rules: bool,
        format: &'a str,
        output: Option<std::path::PathBuf>,
        translate: bool,
        auto_install: bool,
    ) -> Self {
        Self {
            config,
            path,
            tool,
            full,
            remote,
            update_rules,
            format,
            output,
            translate,
            auto_install,
        }
    }
}

/// 代码评审参数结构体
struct ReviewParams<'a> {
    config: &'a Config,
    depth: Option<String>,
    focus: Option<String>,
    language: Option<String>,
    format: &'a str,
    output: Option<std::path::PathBuf>,
    tree_sitter: bool,
    security_scan: bool,
    scan_tool: Option<String>,
    block_on_critical: bool,
}

#[allow(dead_code)]
impl<'a> ReviewParams<'a> {
    fn new(
        config: &'a Config,
        depth: Option<String>,
        focus: Option<String>,
        language: Option<String>,
        format: &'a str,
        output: Option<std::path::PathBuf>,
        tree_sitter: bool,
        security_scan: bool,
        scan_tool: Option<String>,
        block_on_critical: bool,
    ) -> Self {
        Self {
            config,
            depth,
            focus,
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
        }
    }
}

use args::{Args, Command};
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析参数
    let args = Args::parse();
    
    // 加载配置
    let config = Config::load()?;
    
    // 处理命令
    match args.command {
        Command::Review {
            depth,
            focus,
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
        } => {
            let params = ReviewParams::new(
                &config,
                depth,
                focus,
                language,
                &format,
                output,
                tree_sitter,
                security_scan,
                scan_tool,
                block_on_critical,
            );
            handle_review(params).await?;
        }
        Command::Commit {
            message,
            tree_sitter,
            auto_stage,
            issue_id,
            review,
        } => {
            handle_commit(&config, message, tree_sitter, auto_stage, issue_id, review).await?;
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
        } => {
            let scan_params = ScanParams::new(
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
            );
            handle_scan(scan_params).await?;
        }
        Command::ScanHistory { limit, format } => {
            handle_scan_history(limit, &format)?;
        }
        Command::Git(git_args) => {
            if args.noai {
                // 直接执行Git命令
                let output = git::run_git(&git_args)?;
                print!("{output}");
            } else {
                // 带AI解释的Git命令
                handle_git_with_ai(&config, &git_args).await?;
            }
        }
    }
    
    Ok(())
}

/// 处理代码评审
async fn handle_review(params: ReviewParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 正在进行代码评审...");
    
    // 获取代码变更
    let diff = git::get_diff()?;
    if diff.trim().is_empty() {
        println!("❌ 没有检测到代码变更");
        return Ok(());
    }
    
    // AI评审
    let review_result = ai::review_code(params.config, &diff).await?;
    
    // 安全扫描
    if params.security_scan {
        println!("🛡️  正在进行安全扫描...");
        // 使用当前目录作为扫描路径
        let current_dir = std::env::current_dir()?;
        
        // 尝试使用OpenGrep进行安全扫描
        match scan::run_opengrep_scan(params.config, &current_dir) {
            Ok(result) => {
                if !result.findings.is_empty() {
                    println!("⚠️  发现安全问题:");
                    for finding in result.findings.iter().take(5) { // 只显示前5个
                        println!("  - {title} ({}) ({rule_id})", finding.file_path.display(), title = finding.title, rule_id = finding.rule_id);
                    }
                    if result.findings.len() > 5 {
                        println!("  - ... 还有 {} 个问题", result.findings.len() - 5);
                    }
                } else {
                    println!("✅ 安全扫描未发现问题");
                }
            }
            Err(e) => {
                println!("⚠️  安全扫描失败: {e}");
            }
        }
    }
    
    // 输出结果
    let output_content = format!("📋 代码评审结果:\n\n{review_result}");
    
    match params.format {
        "json" => {
            let json = serde_json::json!({"review": review_result});
            let output_str = serde_json::to_string_pretty(&json)?;
            if let Some(path) = params.output {
                std::fs::write(&path, output_str)?;
            } else {
                println!("{output_str}");
            }
        }
        _ => {
            if let Some(path) = params.output {
                std::fs::write(&path, &output_content)?;
            } else {
                println!("{output_content}");
            }
        }
    }
    
    Ok(())
}

/// 处理提交
async fn handle_commit(
    config: &Config,
    message: Option<String>,
    _tree_sitter: bool,
    auto_stage: bool,
    issue_id: Option<String>,
    review: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 正在处理提交...");
    
    // 自动暂存
    if auto_stage {
        println!("📦 自动暂存变更...");
        git::git_add_all()?;
    }
    
    // 获取代码变更
    let diff = git::get_diff()?;
    if diff.trim().is_empty() {
        println!("❌ 没有检测到代码变更");
        return Ok(());
    }
    
    // 生成提交信息
    let commit_message = match message {
        Some(msg) => msg,
        None => {
            println!("🤖 AI正在生成提交信息...");
            ai::generate_commit_message(config, &diff).await?
        }
    };
    
    // 添加Issue ID前缀
    let final_message = match issue_id {
        Some(id) => format!("{id} {commit_message}"),
        None => commit_message,
    };
    
    // 代码评审
    if review {
        println!("🔍 正在评审代码...");
        let review_result = ai::review_code(config, &diff).await?;
        println!("📋 评审结果:\n{review_result}");
    }
    
    // 执行提交
    println!("✅ 执行提交...");
    git::git_commit(&final_message)?;
    
    println!("🎉 提交成功: {final_message}");
    Ok(())
}

/// 处理扫描
async fn handle_scan(params: ScanParams<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let show_progress = params.format != "json";
    
    if show_progress {
        println!("🔍 正在进行安全扫描...");
    }
    
    let start_time = std::time::Instant::now();
    let scan_result = scan::run_smart_scan(params.config, params.path, params.tool, params.auto_install)?;
    let scan_duration = start_time.elapsed();
    
    // 自动保存到缓存目录
    let cache_start = std::time::Instant::now();
    save_scan_to_cache(&scan_result, params.path, &params.tool)?;
    let cache_duration = cache_start.elapsed();
    
    // 添加详细的性能分析
    if show_progress {
        println!("📊 扫描执行时间: {:?}", scan_duration);
        println!("📊 缓存保存时间: {:?}", cache_duration);
        println!("📊 OpenGrep内部执行时间: {:.2}s", scan_result.execution_time);
        println!("📊 GitAI包装开销: {:?}", scan_duration - std::time::Duration::from_secs_f64(scan_result.execution_time));
    }
    
    // 输出结果
    match params.format {
        "json" => {
            let json = serde_json::to_string_pretty(&scan_result)?;
            if let Some(path) = params.output {
                std::fs::write(&path, json)?;
                println!("✅ 扫描结果已保存到: {}", path.display());
            } else {
                println!("{json}");
            }
        }
        _ => {
            let output_content = format_scan_results(&scan_result);
            if let Some(path) = params.output {
                std::fs::write(&path, &output_content)?;
                println!("✅ 扫描结果已保存到: {}", path.display());
            } else {
                print!("{output_content}");
            }
        }
    }
    
    Ok(())
}

/// 处理扫描历史查看
fn handle_scan_history(limit: usize, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("scan-results");
    
    if !cache_dir.exists() {
        println!("📁 扫描历史目录不存在: {}", cache_dir.display());
        return Ok(());
    }
    
    // 获取所有扫描结果文件并按时间排序
    let mut entries: Vec<_> = std::fs::read_dir(&cache_dir)?
        .filter_map(|entry| entry.ok())
        .collect();
    
    // 按文件名排序（时间戳在前）
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    
    let count = entries.len().min(limit);
    
    if format == "json" {
        let mut history = Vec::new();
        for entry in entries.iter().take(count) {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(scan_result) = serde_json::from_str::<scan::ScanResult>(&content) {
                    history.push(scan_result);
                }
            }
        }
        let json = serde_json::to_string_pretty(&history)?;
        println!("{}", json);
    } else {
        println!("📋 扫描历史 (最近{}次):", count);
        println!();
        
        for (index, entry) in entries.iter().take(count).enumerate() {
            let path = entry.path();
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            // 解析文件名获取信息
            let parts: Vec<&str> = filename.split('_').collect();
            if parts.len() >= 3 {
                let timestamp = parts[0];
                let tool = parts[1];
                let datetime = chrono::DateTime::from_timestamp(timestamp.parse::<i64>().unwrap_or(0), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "未知时间".to_string());
                
                // 读取扫描结果摘要
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(scan_result) = serde_json::from_str::<scan::ScanResult>(&content) {
                        println!("{}. {} - {}", index + 1, datetime, tool);
                        println!("   路径: {}", scan_result.findings.len());
                        println!("   工具: {} ({})", scan_result.tool, scan_result.version);
                        println!("   执行时间: {:.2}秒", scan_result.execution_time);
                        println!("   发现问题: {}个", scan_result.findings.len());
                        if !scan_result.findings.is_empty() {
                            let error_count = scan_result.findings.iter()
                                .filter(|f| f.severity == scan::Severity::Error)
                                .count();
                            let warning_count = scan_result.findings.iter()
                                .filter(|f| f.severity == scan::Severity::Warning)
                                .count();
                            println!("   严重程度: Error({}), Warning({})", error_count, warning_count);
                        }
                        println!("   文件: {}", path.display());
                        println!();
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 格式化扫描结果
fn format_scan_results(result: &scan::ScanResult) -> String {
    let mut output = String::new();
    
    output.push_str("📋 扫描结果:\n\n");
    output.push_str(&format!("工具: {} (版本: {})\n", result.tool, result.version));
    output.push_str(&format!("执行时间: {:.2}秒\n", result.execution_time));
    
    if let Some(error) = &result.error {
        output.push_str(&format!("❌ 错误: {}\n", error));
        return output;
    }
    
    if result.findings.is_empty() {
        output.push_str("✅ 未发现安全问题\n");
        return output;
    }
    
    output.push_str(&format!("🔍 发现 {} 个问题:\n", result.findings.len()));
    
    for (index, finding) in result.findings.iter().enumerate() {
        output.push_str(&format!("\n{}. {}\n", index + 1, finding.title));
        output.push_str(&format!("   文件: {}\n", finding.file_path.display()));
        output.push_str(&format!("   位置: 第{}行\n", finding.line));
        output.push_str(&format!("   严重程度: {:?}\n", finding.severity));
        output.push_str(&format!("   规则ID: {}\n", finding.rule_id));
        
        if let Some(snippet) = &finding.code_snippet {
            output.push_str("   代码片段:\n");
            for line in snippet.lines().take(3) {
                output.push_str(&format!("     {}\n", line));
            }
        }
    }
    
    output
}

/// 保存扫描结果到缓存目录
fn save_scan_to_cache(result: &scan::ScanResult, scan_path: &std::path::Path, tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // 创建缓存目录
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("scan-results");
    
    std::fs::create_dir_all(&cache_dir)?;
    
    // 生成文件名：时间戳_工具_路径hash.json
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let path_hash = format!("{:x}", md5::compute(scan_path.to_string_lossy().as_bytes()));
    let filename = format!("{}_{}_{}.json", timestamp, tool, path_hash);
    let cache_file = cache_dir.join(filename);
    
    // 保存JSON格式
    let json = serde_json::to_string_pretty(result)?;
    std::fs::write(&cache_file, json)?;
    
    println!("📁 扫描结果已自动保存到: {}", cache_file.display());
    
    Ok(())
}

/// 打印扫描结果
fn print_scan_results(result: &scan::ScanResult) {
    println!("📋 扫描结果:");
    println!("工具: {tool} (版本: {version})", tool = result.tool, version = result.version);
    println!("执行时间: {:.2}秒", result.execution_time);
    
    if let Some(error) = &result.error {
        println!("❌ 错误: {error}");
        return;
    }
    
    if result.findings.is_empty() {
        println!("✅ 未发现安全问题");
        return;
    }
    
    println!("🔍 发现 {} 个问题:", result.findings.len());
    
    for (index, finding) in result.findings.iter().enumerate() {
        println!("\n{}. {title}", index + 1, title = finding.title);
        println!("   文件: {}", finding.file_path.display());
        println!("   位置: 第{}行", finding.line);
        println!("   严重程度: {:?}", finding.severity);
        println!("   规则ID: {rule_id}", rule_id = finding.rule_id);
        
        if let Some(snippet) = &finding.code_snippet {
            println!("   代码片段:");
            for line in snippet.lines().take(3) {
                println!("     {line}");
            }
        }
    }
}

/// 处理带AI解释的Git命令
async fn handle_git_with_ai(config: &Config, git_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let output = git::run_git(git_args)?;
    
    // 如果输出为空，直接显示
    if output.trim().is_empty() {
        println!("命令执行完成，无输出");
        return Ok(());
    }
    
    // AI解释输出
    let prompt = format!(
        "解释以下Git命令输出的含义:\n\n命令: git {}\n\n输出:\n{output}",
        git_args.join(" ")
    );
    
    let explanation = ai::call_ai(config, &prompt).await?;
    
    println!("🔧 Git命令输出:");
    println!("{output}");
    println!("\n🤖 AI解释:");
    println!("{explanation}");
    
    Ok(())
}

