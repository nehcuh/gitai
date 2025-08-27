use crate::config::Config;
use crate::analysis::{AnalysisConfig, AnalysisContext, Analyzer};
use crate::tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};

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

/// 评审执行器
pub struct ReviewExecutor {
    config: Config,
}

impl ReviewExecutor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// 执行评审流程
    pub async fn execute(&self, review_config: ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔍 正在进行代码评审...");
        
        // 1. 获取代码变更
        let diff = self.get_changes()?;
        if diff.is_empty() {
            println!("❌ 没有检测到任何代码变更");
            return Ok(());
        }
        
        // 2. 检查暂存状态
        self.check_staging_status()?;
        
        // 3. 检查缓存（包含配置维度）
        let cache_key = self.build_cache_key(&diff, &review_config);
        if let Some(cached_result) = self.check_cache(&cache_key)? {
            println!("📋 使用缓存的评审结果");
            self.output_result(&cached_result, &review_config)?;
            return Ok(());
        }
        
        // 4. Tree-sitter结构分析（如果启用）
        let structural_summary = if review_config.tree_sitter {
            self.perform_structural_analysis(&diff, &review_config.language).await?
        } else {
            None
        };
        
        // 5. 获取Issue上下文
        let issues = self.get_issue_context(&review_config.issue_ids).await?;
        
        // 6. 执行分析
        let analysis_config = AnalysisConfig {
            issue_ids: review_config.issue_ids.clone(),
            deviation_analysis: review_config.deviation_analysis,
            security_scan: review_config.security_scan,
        };
        
        let mut context = AnalysisContext::new(diff, issues, analysis_config);
        // 将结构分析结果添加到上下文中
        if let Some(summary) = structural_summary {
            context = self.enrich_context_with_structure(context, summary);
        }
        
        let analyzer = Analyzer::new(self.config.clone());
        let result = analyzer.analyze(context).await?;
        
        // 6. 保存缓存
        self.save_cache(&cache_key, &result.review_result, &review_config.language)?;
        
        // 7. 输出结果
        self.output_analysis_result(&result, &review_config)?;
        
        // 8. 检查是否阻止提交
        if review_config.block_on_critical && self.has_critical_issues(&result) {
            eprintln!("🚨 发现严重问题，已阻止提交");
            return Err("发现严重安全问题".into());
        }
        
        Ok(())
    }
    
    /// 获取代码变更
    fn get_changes(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        crate::git::get_all_diff().map_err(|e| e)
    }
    
    /// 检查暂存状态
    fn check_staging_status(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    fn build_cache_key(&self, diff: &str, cfg: &ReviewConfig) -> String {
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
    fn check_cache(&self, cache_key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
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
    fn save_cache(&self, cache_key: &str, result: &str, language: &Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    async fn get_issue_context(&self, issue_ids: &[String]) -> Result<Vec<crate::devops::Issue>, Box<dyn std::error::Error + Send + Sync>> {
        if issue_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        if let Some(ref devops_config) = self.config.devops {
            let client = crate::devops::DevOpsClient::new(devops_config.clone());
            client.get_issues(issue_ids).await
        } else {
            eprintln!("⚠️ 未配置DevOps平台，无法获取Issue信息");
            Ok(Vec::new())
        }
    }
    
    /// 输出分析结果
    fn output_analysis_result(&self, result: &crate::analysis::AnalysisResult, config: &ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("📋 代码评审结果：");
        println!("{}", result.review_result);
        
        // 输出安全扫描结果
        println!("\n🛡️ 安全扫描结果：");
        if !result.security_findings.is_empty() {
            let critical_count = result.security_findings.iter()
                .filter(|f| matches!(self.parse_severity(&f.severity), crate::scan::Severity::Error))
                .count();
            let warning_count = result.security_findings.iter()
                .filter(|f| matches!(self.parse_severity(&f.severity), crate::scan::Severity::Warning))
                .count();
            
            if critical_count > 0 {
                println!("  🚨 严重问题: {} 个", critical_count);
            }
            if warning_count > 0 {
                println!("  ⚠️  警告问题: {} 个", warning_count);
            }
            
            println!("  📝 详细问题：");
            for finding in result.security_findings.iter().take(5) {
                let severity_icon = match self.parse_severity(&finding.severity) {
                    crate::scan::Severity::Error => "🚨",
                    crate::scan::Severity::Warning => "⚠️ ",
                    crate::scan::Severity::Info => "ℹ️ ",
                };
                println!("    {} {} ({})", severity_icon, finding.title, finding.file_path);
            }
            if result.security_findings.len() > 5 {
                println!("    ... 还有 {} 个问题", result.security_findings.len() - 5);
            }
        } else {
            println!("  ✅ 未发现安全问题");
        }
        
        // 输出偏离度分析
        if let Some(deviation) = &result.deviation_analysis {
            println!("\n📊 偏离度分析：");
            let coverage_color = if deviation.requirement_coverage >= 0.8 { "🟢" } else if deviation.requirement_coverage >= 0.6 { "🟡" } else { "🔴" };
            let quality_color = if deviation.quality_score >= 0.8 { "🟢" } else if deviation.quality_score >= 0.6 { "🟡" } else { "🔴" };
            
            println!("  {} 需求覆盖率: {:.1}%", coverage_color, deviation.requirement_coverage * 100.0);
            println!("  {} 质量评分: {:.1}%", quality_color, deviation.quality_score * 100.0);
        }
        
        self.output_result(&result.review_result, config)?;
        
        Ok(())
    }
    
    /// 输出结果到文件或控制台
    fn output_result(&self, result: &str, config: &ReviewConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref output_path) = config.output {
            std::fs::write(output_path, result)?;
            println!("📄 结果已保存到: {}", output_path.display());
        }
        Ok(())
    }
    
    /// 将字符串严重级别映射为严格的枚举
    fn parse_severity(&self, s: &str) -> crate::scan::Severity {
        match s.to_ascii_uppercase().as_str() {
            "ERROR" | "CRITICAL" | "SEVERE" => crate::scan::Severity::Error,
            "WARNING" | "WARN" => crate::scan::Severity::Warning,
            _ => crate::scan::Severity::Info,
        }
    }

    /// 检查是否有严重问题（严格按枚举判断）
    fn has_critical_issues(&self, result: &crate::analysis::AnalysisResult) -> bool {
        result.security_findings.iter()
            .any(|f| matches!(self.parse_severity(&f.severity), crate::scan::Severity::Error))
    }
    
    /// 执行结构分析
    async fn perform_structural_analysis(&self, diff: &str, language: &Option<String>) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
        println!("🌳 正在进行Tree-sitter结构分析...");
        
        // 从diff中提取代码内容
        let code_content = self.extract_code_from_diff(diff);
        if code_content.is_empty() {
            println!("  💡 提示：当前变更中没有可分析的代码内容");
            println!("     这可能是文档、配置文件或二进制文件的变更");
            return Ok(None);
        }
        
        // 推断语言
        let language = if let Some(lang) = language {
            self.detect_supported_language(lang)
        } else {
            self.infer_language_from_diff(diff)
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
                        println!("     🔢 函数数量: {}", summary.functions.len());
                        println!("     🏗️  类数量: {}", summary.classes.len());
                        println!("     💬 注释数量: {}", summary.comments.len());
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
    fn extract_code_from_diff(&self, diff: &str) -> String {
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
    fn detect_supported_language(&self, language: &str) -> Option<SupportedLanguage> {
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
    fn infer_language_from_diff(&self, diff: &str) -> Option<SupportedLanguage> {
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
    
    /// 使用结构分析结果丰富上下文
    fn enrich_context_with_structure(&self, mut context: AnalysisContext, summary: StructuralSummary) -> AnalysisContext {
        // 将结构摘要转换为上下文信息
        let structure_info = self.format_structure_info(&summary);
        
        // 添加到上下文的额外信息中
        // 注意：这里需要AnalysisContext支持添加额外信息
        // 如果AnalysisContext没有这个方法，我们可能需要修改它
        context.add_structural_info(structure_info);
        
        context
    }
    
    /// 格式化结构信息
    fn format_structure_info(&self, summary: &StructuralSummary) -> String {
        let mut info = Vec::new();
        
        info.push(format!("## 代码结构分析 ({})", summary.language));
        
        if !summary.functions.is_empty() {
            info.push("### 函数列表:".to_string());
            for func in &summary.functions {
                info.push(format!("- `{}` (第{}行): 参数{}个", 
                    func.name, func.line_start, func.parameters.len()));
                if let Some(ref return_type) = func.return_type {
                    info.push(format!("  返回类型: {}", return_type));
                }
            }
            info.push("".to_string());
        }
        
        if !summary.classes.is_empty() {
            info.push("### 类/结构体列表:".to_string());
            for class in &summary.classes {
                info.push(format!("- `{}` (第{}行)", class.name, class.line_start));
                if let Some(ref extends) = class.extends {
                    info.push(format!("  继承自: {}", extends));
                }
            }
            info.push("".to_string());
        }
        
        if !summary.complexity_hints.is_empty() {
            info.push("### 复杂度建议:".to_string());
            for hint in &summary.complexity_hints {
                info.push(format!("- {}", hint));
            }
            info.push("".to_string());
        }
        
        info.join("\n")
    }
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