mod config;
mod args;
mod git;
mod ai;
mod scan;

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Review结果缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewCache {
    /// 评审时间
    timestamp: u64,
    /// 评审的代码差异
    diff_hash: String,
    /// 评审结果
    review_result: String,
    /// 语言
    language: Option<String>,
    /// 关注点
    focus_areas: Option<Vec<String>>,
}

impl ReviewCache {
    fn new(diff: &str, review_result: String, language: Option<String>, focus_areas: Option<Vec<String>>) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            diff_hash: Self::hash_diff(diff),
            review_result,
            language,
            focus_areas,
        }
    }
    
    fn hash_diff(diff: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        diff.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now.saturating_sub(self.timestamp) > max_age_seconds
    }
}

/// 获取缓存目录
fn get_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    get_cache_subdir("review")
}

/// 获取缓存子目录 - 减少重复代码
fn get_cache_subdir(subdir: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join(subdir);
    
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// 保存review结果到缓存
fn save_review_cache(cache: &ReviewCache) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("review_{}.json", cache.diff_hash));
    
    let json = serde_json::to_string_pretty(cache)?;
    fs::write(&cache_file, json)?;
    
    println!("💾 Review结果已缓存到: {}", cache_file.display());
    Ok(())
}

/// 从缓存加载review结果
fn load_review_cache(diff_hash: &str, max_age_seconds: u64) -> Result<Option<ReviewCache>, Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("review_{}.json", diff_hash));
    
    if !cache_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&cache_file)?;
    let cache: ReviewCache = serde_json::from_str(&content)?;
    
    if cache.is_expired(max_age_seconds) {
        println!("🕐 缓存已过期，重新评审");
        fs::remove_file(&cache_file)?;
        return Ok(None);
    }
    
    println!("🎯 使用缓存的review结果");
    Ok(Some(cache))
}

/// 清理过期的缓存
fn cleanup_expired_cache(max_age_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cache) = serde_json::from_str::<ReviewCache>(&content) {
                        if cache.is_expired(max_age_seconds) {
                            fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 扫描参数结构体 - 简化版本
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

/// 代码评审参数结构体 - 简化版本
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
            let params = ReviewParams {
                config: &config,
                depth,
                focus,
                language,
                format: &format,
                output,
                tree_sitter,
                security_scan,
                scan_tool,
                block_on_critical,
            };
            handle_review(params).await?;
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
            let scan_params = ScanParams {
                config: &config,
                path: &path,
                tool: &tool,
                full,
                remote,
                update_rules,
                format: &format,
                output,
                translate,
                auto_install,
            };
            handle_scan(scan_params).await?;
        }
        Command::ScanHistory { limit, format } => {
            handle_scan_history(limit, &format)?;
        }
        Command::Prompts { action } => {
            handle_prompts_action(&config, &action).await?;
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
    
    // 获取所有代码变更（包括工作区和暂存区）
    let diff = match git::get_all_diff() {
        Ok(diff) => diff,
        Err(_) => {
            println!("❌ 没有检测到任何代码变更");
            return Ok(());
        }
    };
    
    // 检查暂存状态并给出智能提示
    let has_unstaged = git::has_unstaged_changes().unwrap_or(false);
    let has_staged = git::has_staged_changes().unwrap_or(false);
    
    if has_unstaged {
        println!("💡 提示：检测到未暂存的代码变更");
        println!("   使用 `git add .` 暂存所有变更，或使用 `git add <file>` 暂存特定文件");
        if has_staged {
            println!("   当前已暂存的变更也会被评审");
        }
        println!();
    } else if has_staged {
        println!("✅ 已暂存的代码准备就绪");
    }
    
    // 计算diff的hash用于缓存
    let diff_hash = ReviewCache::hash_diff(&diff);
    let cache_max_age = 3600; // 1小时缓存
    
    // 尝试从缓存加载
    let review_result = if let Ok(Some(cache)) = load_review_cache(&diff_hash, cache_max_age) {
        cache.review_result
    } else {
        // 执行AI评审
        println!("🤖 正在进行AI代码评审...");
        let result = ai::review_code(params.config, &diff).await?;
        
        // 保存到缓存
        let cache = ReviewCache::new(
            &diff,
            result.clone(),
            params.language.clone(),
            params.focus.as_ref().map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
        );
        
        if let Err(e) = save_review_cache(&cache) {
            eprintln!("⚠️ 无法保存缓存: {}", e);
        }
        
        // 清理过期缓存
        if let Err(e) = cleanup_expired_cache(cache_max_age) {
            eprintln!("⚠️ 清理缓存失败: {}", e);
        }
        
        result
    };
    
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
                        println!("  - {} ({}) ({})", finding.title, finding.file_path.display(), finding.rule_id);
                    }
                    if result.findings.len() > 5 {
                        println!("  - ... 还有 {} 个问题", result.findings.len() - 5);
                    }
                } else {
                    println!("✅ 安全扫描未发现问题");
                }
            }
            Err(e) => {
                println!("⚠️  安全扫描失败: {}", e);
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
    let cache_dir = get_cache_subdir("scan-results")?;
    
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
    
    let cache_dir = get_cache_subdir("scan-results")?;
    
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


/// 处理带AI解释的Git命令
async fn handle_git_with_ai(config: &Config, git_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // 首先尝试执行git命令
    match git::run_git(git_args) {
        Ok(output) => {
            // 命令执行成功，直接输出结果
            print!("{output}");
            
            // 如果是commit命令，尝试显示相关的review结果
            if git_args.first().map(|s| s == "commit").unwrap_or(false) {
                if let Err(e) = show_related_review_results(config) {
                    eprintln!("⚠️ 无法显示review结果: {}", e);
                }
            }
        }
        Err(e) => {
            // 命令执行失败，提供AI解释和建议
            println!("❌ Git命令执行失败: {}", e);
            
            // 构建AI提示词，询问解决方案
            let prompt = format!(
                "用户执行Git命令时遇到错误，请提供帮助：\n\n命令: git {}\n\n错误信息: {}\n\n请提供：\n1. 错误原因分析\n2. 正确的命令格式\n3. 解决方案建议\n4. 相关的最佳实践",
                git_args.join(" "),
                e
            );
            
            match ai::call_ai(config, &prompt).await {
                Ok(explanation) => {
                    println!("\n🤖 AI建议:");
                    println!("{}", explanation);
                }
                Err(ai_error) => {
                    println!("\n⚠️ 无法获取AI建议: {}", ai_error);
                    println!("请检查Git命令是否正确，或使用 --noai 参数直接执行Git命令。");
                }
            }
        }
    }
    
    Ok(())
}

/// 显示相关的review结果（在commit成功后调用）
fn show_related_review_results(_config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    
    // 策略1：检查当前是否有未提交的变更
    if let Ok(current_diff) = git::get_all_diff() {
        let diff_hash = ReviewCache::hash_diff(&current_diff);
        let cache_file = cache_dir.join(format!("review_{}.json", diff_hash));
        
        if cache_file.exists() {
            println!("\n📋 当前代码的评审结果:");
            println!("   (来自最近的review缓存)");
            
            let content = fs::read_to_string(&cache_file)?;
            let cache: ReviewCache = serde_json::from_str(&content)?;
            
            if !cache.is_expired(3600) {
                println!("{}", cache.review_result);
            } else {
                println!("   (缓存已过期，建议重新运行 gitai review)");
            }
            return Ok(());
        }
    }
    
    // 策略2：查找最新的review缓存
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        let mut most_recent_cache: Option<ReviewCache> = None;
        let mut most_recent_time = std::time::SystemTime::UNIX_EPOCH;
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") 
                && path.file_name().and_then(|s| s.to_str()).map(|s| s.starts_with("review_")).unwrap_or(false) {
                
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified_time) = metadata.modified() {
                        if modified_time > most_recent_time {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(cache) = serde_json::from_str::<ReviewCache>(&content) {
                                    if !cache.is_expired(3600) {
                                        most_recent_cache = Some(cache);
                                        most_recent_time = modified_time;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if let Some(cache) = most_recent_cache {
            println!("\n📋 相关的代码评审结果:");
            println!("   (来自最近的review缓存)");
            println!("{}", cache.review_result);
            return Ok(());
        }
    }
    
    Ok(())
}

/// 处理提示词相关操作
async fn handle_prompts_action(_config: &Config, action: &args::PromptAction) -> Result<(), Box<dyn std::error::Error>> {
    use args::PromptAction;
    
    match action {
        PromptAction::List => {
            println!("📋 提示词管理功能暂未实现");
        }
        PromptAction::Show { name, language } => {
            println!("📝 显示提示词 '{}' (语言: {:?}) - 功能暂未实现", name, language);
        }
        PromptAction::Update => {
            println!("🔄 更新提示词 - 功能暂未实现");
        }
        PromptAction::Init => {
            println!("✅ 初始化提示词目录 - 功能暂未实现");
        }
    }
    
    Ok(())
}

