use crate::config::Config;
use crate::analysis::{Analyzer, OperationContext, OperationOptions};
use crate::tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};
use crate::project_insights::InsightsGenerator;
use crate::architectural_impact::{GitStateAnalyzer, ArchitecturalImpact};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 评审结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// 发现的问题
    pub findings: Vec<Finding>,
    /// 评分 (可选)
    pub score: Option<u8>,
    /// 建议列表
    pub recommendations: Vec<String>,
}

/// 发现的问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题描述
    pub title: String,
    /// 文件路径
    pub file_path: Option<String>,
    /// 行号
    pub line: Option<u32>,
    /// 严重程度
    pub severity: Severity,
    /// 详细描述
    pub description: String,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 评审配置
#[derive(Debug, Clone)]
pub struct ReviewConfig {
    pub language: Option<String>,
    pub format: String,
    pub output: Option<std::path::PathBuf>,
    pub tree_sitter: bool,
    pub security_scan: bool,
    pub scan_tool: Option<String>,
    pub block_on_critical: bool,
    pub issue_ids: Vec<String>,
    pub deviation_analysis: bool,
}

impl ReviewConfig {
    pub fn from_args(
        language: Option<String>,
        format: String,
        output: Option<std::path::PathBuf>,
        tree_sitter: bool,
        security_scan: bool,
        scan_tool: Option<String>,
        block_on_critical: bool,
        issue_id: Option<String>,
        deviation_analysis: bool,
    ) -> Self {
        let issue_ids = issue_id
            .map(|ids| ids.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        
        // 当指定了 scan_tool 时自动启用 security_scan
        let security_scan = security_scan || scan_tool.is_some();
        
        Self {
            language,
            format,
            output,
            tree_sitter,
            security_scan,
            scan_tool,
            block_on_critical,
            issue_ids,
            deviation_analysis,
        }
    }
    
    pub fn needs_issue_context(&self) -> bool {
        !self.issue_ids.is_empty() || self.deviation_analysis
    }
    
    pub fn deviation_analysis(&self) -> bool {
        self.deviation_analysis
    }
}

/// 评审执行器 - 已被弃用，使用静态函数代替
#[deprecated(note = "Use static functions execute_review and execute_review_with_result instead")]
pub struct ReviewExecutor {
    config: Config,
}

impl ReviewExecutor {
    #[deprecated(note = "Use static functions instead")]
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 执行评审流程
    #[deprecated(note = "Use execute_review static function instead")]
    pub async fn execute(&self, review_config: ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        execute_review(&self.config, review_config).await
    }
    
    /// 执行评审流程并返回结构化结果
    #[deprecated(note = "Use execute_review_with_result static function instead")]
    pub async fn execute_with_result(&self, review_config: ReviewConfig) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
        execute_review_with_result(&self.config, review_config).await
    }
}

/// 执行评审流程
pub async fn execute_review(config: &Config, review_config: ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = execute_review_with_result(config, review_config).await?;
    if !result.success {
        return Err("评审失败".into());
    }
    Ok(())
}

/// 执行评审流程并返回结构化结果
pub async fn execute_review_with_result(config: &Config, review_config: ReviewConfig) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    // Linus principle: Eliminate special cases, make them normal cases
    
    // 1. Get diff and handle early returns with a consistent pattern
    let diff = get_changes()?;
    let cache_key = build_cache_key(&diff, &review_config);
    
    // 2. Try cache first, then handle empty diff as normal case
    if let Some(result) = try_cached_or_empty_diff(&diff, &cache_key, &review_config)? {
        return Ok(result);
    }
    
    // 3. Normal case: do the analysis
    check_staging_status()?;
    let context = build_analysis_context(config, &review_config, diff).await?;
    let result = Analyzer::analyze(&context).await?;
    
    // 4. Save cache and convert result (unified flow)
    save_cache(&cache_key, &result.review_result, &review_config.language)?;
    Ok(convert_analysis_result_with_critical_check(&result, &review_config))
}

/// 获取代码变更
fn get_changes() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    crate::git::get_all_diff()
}

/// 检查暂存状态
fn check_staging_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let has_unstaged = crate::git::has_unstaged_changes().unwrap_or(false);
    let has_staged = crate::git::has_staged_changes().unwrap_or(false);
    
    if has_unstaged {
        println!("💡 提示：检测到未暂存的代码变更");
        println!("   使用 `git add .` 暂存所有变更，或使用 `git add <file>` 暂存特定文件");
        if has_staged {
            println!("   当前已暂存的变更也会被评审");
        }
        println!("   📝 GitAI将分析所有变更（已暂存 + 未暂存）");
        println!();
    } else if has_staged {
        println!("✅ 已暂存的代码准备就绪");
        println!("   📝 GitAI将分析已暂存的变更");
    } else {
        println!("🔍 检查未推送的提交...");
        println!("   📝 GitAI将分析最近的提交变更");
    }
    
    Ok(())
}

/// 生成缓存键：包含 diff、language、security_scan、deviation_analysis、issue_ids
fn build_cache_key(diff: &str, cfg: &ReviewConfig) -> String {
    let diff_hash = format!("{:x}", md5::compute(diff.as_bytes()));
    let mut ids = cfg.issue_ids.clone();
    ids.sort();
    let payload = serde_json::json!({
        "diff": diff_hash,
        "language": cfg.language,
        "security_scan": cfg.security_scan,
        "deviation_analysis": cfg.deviation_analysis,
        "issue_ids": ids,
    });
    format!("{:x}", md5::compute(payload.to_string().as_bytes()))
}

/// 检查缓存
fn check_cache(cache_key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("review_cache");
    
    let cache_file = cache_dir.join(format!("review_{}.json", cache_key));
    
    if !cache_file.exists() {
        return Ok(None);
    }
    
    let content = std::fs::read_to_string(&cache_file)?;
    let cache: ReviewCache = serde_json::from_str(&content)?;
    
    if cache.is_expired(3600) {
        return Ok(None);
    }
    
    Ok(Some(cache.review_result))
}

/// 保存缓存
fn save_cache(cache_key: &str, result: &str, language: &Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("review_cache");
    
    std::fs::create_dir_all(&cache_dir)?;
    
    let cache = ReviewCache::new(cache_key, result.to_string(), language.clone());
    let cache_file = cache_dir.join(format!("review_{}.json", cache_key));
    
    let content = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&cache_file, content)?;
    
    Ok(())
}

/// 获取Issue上下文
async fn get_issue_context(config: &Config, issue_ids: &[String]) -> Result<Vec<crate::devops::Issue>, Box<dyn std::error::Error + Send + Sync>> {
    if issue_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    if let Some(ref devops_config) = config.devops {
        let client = crate::devops::DevOpsClient::new(devops_config.clone());
        client.get_issues(issue_ids).await
    } else {
        eprintln!("⚠️ 未配置DevOps平台，无法获取Issue信息");
        Ok(Vec::new())
    }
}

/// 执行架构影响分析
async fn perform_architectural_impact_analysis(diff: &str) -> Result<Option<ArchitecturalImpact>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🏗️ 正在进行架构影响分析...");
    
    // 创建GitStateAnalyzer并分析
    let analyzer = GitStateAnalyzer::new();
    match analyzer.analyze_git_diff(diff).await {
        Ok(impact) => {
            println!("  ✅ 架构影响分析完成");
            
            // 输出关键指标
            let total_changes = impact.function_changes.len() + 
                                impact.struct_changes.len() + 
                                impact.interface_changes.len();
            println!("     📊 总变更数: {}", total_changes);
            println!("     🔧 函数变更: {}", impact.function_changes.len());
            println!("     🏗️ 结构体变更: {}", impact.struct_changes.len());
            println!("     🔌 接口变更: {}", impact.interface_changes.len());
            
            // 输出影响范围
            if !impact.impact_summary.affected_modules.is_empty() {
                println!("     📦 影响模块: {}", impact.impact_summary.affected_modules.len());
            }
            if !impact.impact_summary.breaking_changes.is_empty() {
                println!("     ⚠️  破坏性变更: {}", impact.impact_summary.breaking_changes.len());
            }
            
            Ok(Some(impact))
        }
        Err(e) => {
            println!("  ⚠️  架构影响分析失败: {}", e);
            log::debug!("架构影响分析详情: {}", e);
            Ok(None)
        }
    }
}

/// 执行结构分析
async fn perform_structural_analysis(diff: &str, language: &Option<String>) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🌳 正在进行Tree-sitter结构分析...");
    
    // 从diff中提取代码内容
    let code_content = extract_code_from_diff(diff);
    if code_content.is_empty() {
        println!("  💡 提示：当前变更中没有可分析的代码内容");
        println!("     这可能是文档、配置文件或二进制文件的变更");
        return Ok(None);
    }
    
    // 推断语言
    let language = if let Some(lang) = language {
        detect_supported_language(lang)
    } else {
        infer_language_from_diff(diff)
    };
    
    let Some(supported_lang) = language else {
        println!("  💡 提示：当前变更的语言类型不支持Tree-sitter分析");
        println!("     支持的语言：Rust, Java, JavaScript, Python, Go, C, C++");
        return Ok(None);
    };
    
    println!("  📝 检测到语言: {:?}", supported_lang);
    
    // 创建Tree-sitter管理器并分析
    match TreeSitterManager::new().await {
        Ok(mut manager) => {
            match manager.analyze_structure(&code_content, supported_lang) {
                Ok(summary) => {
                    println!("  ✅ 结构分析完成");
                    
                    // 生成架构洞察
                    let insights = InsightsGenerator::generate(&summary, None);
                    
                    // 输出架构洞察而不是简单统计
                    println!("     🏗️ 架构模式违规: {}", insights.architecture.pattern_violations.len());
                    println!("     🔄 循环依赖: {}", insights.architecture.module_dependencies.circular_dependencies.len());
                    println!("     ⚡ 复杂度热点: {}", insights.quality_hotspots.complexity_hotspots.len());
                    println!("     📊 API 接口: {}", insights.api_surface.public_apis.len());
                    
                    Ok(Some(summary))
                }
                Err(e) => {
                    println!("  ⚠️  结构分析失败，将使用传统文本分析模式");
                    log::debug!("Tree-sitter分析详情: {}", e);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            println!("  ⚠️  Tree-sitter初始化失败，将使用传统文本分析模式");
            log::debug!("Tree-sitter初始化详情: {}", e);
            Ok(None)
        }
    }
}

/// 从diff中提取代码内容
fn extract_code_from_diff(diff: &str) -> String {
    let mut code_lines = Vec::new();
    let mut in_file_section = false;
    
    for line in diff.lines() {
        // 检测文件变更开始
        if line.starts_with("diff --git") {
            in_file_section = true;
            continue;
        }
        
        // 跳过diff元数据行
        if line.starts_with("index")
            || line.starts_with("+++")
            || line.starts_with("---")
            || line.starts_with("@@") {
            continue;
        }
        
        // 空行表示文件变更结束
        if line.is_empty() && in_file_section {
            in_file_section = false;
            // 添加文件分隔符，保持代码结构
            code_lines.push("\n// === 文件分隔符 ===\n");
            continue;
        }
        
        // 提取添加的行（+开头）和上下文行（没有+/-前缀）
        if line.starts_with('+') {
            code_lines.push(&line[1..]);
        } else if !line.starts_with('-') && !line.trim().is_empty() {
            code_lines.push(line);
        }
    }
    
    let result = code_lines.join("\n");
    
    // 清理多余的分隔符
    result.trim_matches('\n').to_string()
}

/// 检测支持的语言
fn detect_supported_language(language: &str) -> Option<SupportedLanguage> {
    match language.to_lowercase().as_str() {
        "java" => Some(SupportedLanguage::Java),
        "rust" => Some(SupportedLanguage::Rust),
        "c" => Some(SupportedLanguage::C),
        "cpp" | "c++" => Some(SupportedLanguage::Cpp),
        "python" => Some(SupportedLanguage::Python),
        "go" => Some(SupportedLanguage::Go),
        "javascript" | "js" => Some(SupportedLanguage::JavaScript),
        "typescript" | "ts" => Some(SupportedLanguage::TypeScript),
        _ => None,
    }
}

/// 从diff中推断语言
fn infer_language_from_diff(diff: &str) -> Option<SupportedLanguage> {
    let mut detected_files = Vec::new();
    
    // 查找文件路径以推断语言
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("+++") {
            if let Some(path) = line.split_whitespace().last() {
                if let Some(extension) = std::path::Path::new(path)
                    .extension()
                    .and_then(|ext| ext.to_str()) {
                    detected_files.push((path.to_string(), extension.to_string()));
                }
            }
        }
    }
    
    // 如果没有检测到文件，返回None
    if detected_files.is_empty() {
        return None;
    }
    
    // 优先返回第一个支持的语言
    for (file_path, extension) in &detected_files {
        if let Some(lang) = SupportedLanguage::from_extension(extension) {
            log::debug!("从文件 {} 检测到语言: {:?}", file_path, lang);
            return Some(lang);
        }
    }
    
    // 如果没有支持的语言，记录日志
    let unsupported_files: Vec<String> = detected_files
        .into_iter()
        .map(|(path, ext)| format!("{} ({})", path, ext))
        .collect();
    
    log::debug!("检测到不支持的文件类型: {:?}", unsupported_files);
    None
}

/// 从缓存的文本结果中解析结构化信息
fn parse_cached_result(cached_result: &str, _config: &ReviewConfig) -> Result<ReviewResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut details = HashMap::new();
    details.insert("cached".to_string(), "true".to_string());
    
    // 简单的文本解析，提取关键信息
    let score = if cached_result.contains("优秀") || cached_result.contains("Excellent") {
        Some(90)
    } else if cached_result.contains("良好") || cached_result.contains("Good") {
        Some(75)
    } else if cached_result.contains("一般") || cached_result.contains("Average") {
        Some(60)
    } else {
        None
    };
    
    Ok(ReviewResult {
        success: true,
        message: "使用缓存的评审结果".to_string(),
        details,
        findings: Vec::new(), // 缓存结果不包含详细的问题信息
        score,
        recommendations: vec!["建议定期更新缓存以获得最新的分析结果".to_string()],
    })
}

/// 将分析结果转换为结构化的ReviewResult
fn convert_analysis_result(result: &crate::analysis::AnalysisResult, config: &ReviewConfig) -> ReviewResult {
    let mut details = HashMap::new();
    let mut findings = Vec::new();
    let mut recommendations = Vec::new();
    
    // 转换安全发现
    for finding in &result.security_findings {
        findings.push(Finding {
            title: finding.title.clone(),
            file_path: Some(finding.file_path.clone()),
            line: Some(finding.line as u32),
            severity: match parse_severity(&finding.severity) {
                crate::scan::Severity::Error => Severity::Error,
                crate::scan::Severity::Warning => Severity::Warning,
                crate::scan::Severity::Info => Severity::Info,
            },
            description: finding.code_snippet.clone().unwrap_or_else(|| "发现安全问题的代码段".to_string()),
        });
    }
    
    // 添加配置信息
    details.insert("tree_sitter".to_string(), config.tree_sitter.to_string());
    details.insert("security_scan".to_string(), config.security_scan.to_string());
    details.insert("deviation_analysis".to_string(), config.deviation_analysis.to_string());
    details.insert("issue_ids_count".to_string(), config.issue_ids.len().to_string());
    
    if !config.issue_ids.is_empty() {
        details.insert("issue_ids".to_string(), config.issue_ids.join(", "));
    }
    
    // 添加偏离分析结果
    if let Some(deviation) = &result.deviation_analysis {
        details.insert("requirement_coverage".to_string(), format!("{:.1}%", deviation.requirement_coverage * 100.0));
        details.insert("quality_score".to_string(), format!("{:.1}%", deviation.quality_score * 100.0));
        
        // 根据质量评分给出建议
        if deviation.quality_score < 0.6 {
            recommendations.push("代码质量评分较低，建议进行重构".to_string());
        } else if deviation.quality_score < 0.8 {
            recommendations.push("代码质量有待提升，建议优化关键部分".to_string());
        }
    }
    
    // 根据安全问题给出建议
    let critical_count = findings.iter()
        .filter(|f| matches!(f.severity, Severity::Error))
        .count();
    let warning_count = findings.iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .count();
    
    if critical_count > 0 {
        recommendations.push(format!("发现 {} 个严重安全问题，必须立即修复", critical_count));
    }
    if warning_count > 0 {
        recommendations.push(format!("发现 {} 个警告问题，建议修复", warning_count));
    }
    
    // 计算总体评分
    let score = if critical_count > 0 {
        Some(30)
    } else if warning_count > 0 {
        Some(60)
    } else if let Some(deviation) = &result.deviation_analysis {
        Some((deviation.quality_score * 100.0) as u8)
    } else {
        Some(80)
    };
    
    ReviewResult {
        success: true,
        message: "代码评审完成".to_string(),
        details,
        findings,
        score,
        recommendations,
    }
}

/// 将字符串严重级别映射为严格的枚举
fn parse_severity(s: &str) -> crate::scan::Severity {
    match s.to_ascii_uppercase().as_str() {
        "ERROR" | "CRITICAL" | "SEVERE" => crate::scan::Severity::Error,
        "WARNING" | "WARN" => crate::scan::Severity::Warning,
        _ => crate::scan::Severity::Info,
    }
}

/// 检查是否有严重问题（严格按枚举判断）
fn has_critical_issues(result: &crate::analysis::AnalysisResult) -> bool {
    result.security_findings.iter()
        .any(|f| matches!(parse_severity(&f.severity), crate::scan::Severity::Error))
}

/// 简化的Review缓存
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ReviewCache {
    timestamp: u64,
    diff_hash: String,
    review_result: String,
    language: Option<String>,
}

impl ReviewCache {
    fn new(diff_hash: &str, review_result: String, language: Option<String>) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            diff_hash: diff_hash.to_string(),
            review_result,
            language,
        }
    }
    
    fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.timestamp) > max_age_seconds
    }
}

// =============================================================================
// Helper functions to eliminate special cases following Linus's good taste
// =============================================================================

/// Try cached result or handle empty diff - eliminates 2 special cases
fn try_cached_or_empty_diff(
    diff: &str, 
    cache_key: &str, 
    review_config: &ReviewConfig
) -> Result<Option<ReviewResult>, Box<dyn std::error::Error + Send + Sync>> {
    // Handle empty diff as normal case (was special case 1)
    if diff.is_empty() {
        return Ok(Some(ReviewResult {
            success: true,
            message: "没有检测到任何代码变更".to_string(),
            details: HashMap::new(),
            findings: Vec::new(),
            score: None,
            recommendations: Vec::new(),
        }));
    }
    
    // Try cache (was special case 2)
    if let Some(cached_result) = check_cache(cache_key)? {
        return Ok(Some(parse_cached_result(&cached_result, review_config)?));
    }
    
    Ok(None)
}

/// Build analysis context - eliminates manual OperationOptions construction
async fn build_analysis_context(
    config: &Config, 
    review_config: &ReviewConfig, 
    diff: String
) -> Result<OperationContext, Box<dyn std::error::Error + Send + Sync>> {
    // Get structural analysis if needed
    let structural_summary = if review_config.tree_sitter {
        perform_structural_analysis(&diff, &review_config.language).await?
    } else {
        None
    };
    
    // Perform architectural impact analysis if Tree-sitter is enabled
    let architectural_impact = if review_config.tree_sitter {
        perform_architectural_impact_analysis(&diff).await?
    } else {
        None
    };
    
    // Get issue context
    let issues = get_issue_context(config, &review_config.issue_ids).await?;
    
    // Build options from config - unified pattern
    let options = OperationOptions {
        issue_ids: review_config.issue_ids.clone(),
        deviation_analysis: review_config.deviation_analysis,
        security_scan: review_config.security_scan,
        tree_sitter: review_config.tree_sitter,
        language: review_config.language.clone(),
        format: Some(review_config.format.clone()),
        output: review_config.output.clone(),
        block_on_critical: review_config.block_on_critical,
        ..Default::default()
    };
    
    let mut context = OperationContext::new(config.clone())
        .with_diff(diff)
        .with_issues(issues)
        .with_options(options);
        
    // Add structural info if available
    if let Some(summary) = structural_summary {
        context = context.with_structural_info(summary);
    }
    
    // Add architectural impact if available
    if let Some(impact) = architectural_impact {
        context = context.with_architectural_impact(impact);
    }
    
    Ok(context)
}

/// Convert analysis result with critical check - eliminates special case 3
fn convert_analysis_result_with_critical_check(
    result: &crate::analysis::AnalysisResult, 
    review_config: &ReviewConfig
) -> ReviewResult {
    let review_result = convert_analysis_result(result, review_config);
    
    // Handle critical check as normal flow (was special case 3)
    if review_config.block_on_critical && has_critical_issues(result) {
        ReviewResult {
            success: false,
            message: "发现严重安全问题，已阻止提交".to_string(),
            details: review_result.details,
            findings: review_result.findings,
            score: review_result.score,
            recommendations: review_result.recommendations,
        }
    } else {
        review_result
    }
}
