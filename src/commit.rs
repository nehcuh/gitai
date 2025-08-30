use crate::config::Config;
use crate::context::Issue;
use crate::tree_sitter::{TreeSitterManager, SupportedLanguage, StructuralSummary};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 提交结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 提交哈希 (如果成功)
    pub commit_hash: Option<String>,
    /// 变更数量
    pub changes_count: usize,
    /// 评审结果 (如果执行了评审)
    pub review_results: Option<ReviewResults>,
    /// 详细信息
    pub details: HashMap<String, String>,
}

/// 评审结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResults {
    /// 发现的问题数量
    pub issues_found: usize,
    /// 严重问题数量
    pub critical_issues: usize,
    /// 评审报告
    pub report: Option<String>,
}

/// 解析issue ID字符串为列表
fn parse_issue_ids(issue_id: Option<String>) -> Vec<String> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    if let Some(ids) = issue_id {
        for raw in ids.split(',') {
            let mut s = raw.trim().to_string();
            if s.is_empty() { continue; }
            if !s.starts_with('#') { s = format!("#{s}"); }
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    out
}

/// 提交配置
#[derive(Debug, Clone)]
pub struct CommitConfig {
    pub message: Option<String>,
    pub issue_ids: Vec<String>,
    pub add_all: bool,
    pub review: bool,
    pub tree_sitter: bool,
    pub dry_run: bool,
}

impl CommitConfig {
    pub fn from_args(
        message: Option<String>,
        issue_id: Option<String>,
        add_all: bool,
        review: bool,
        tree_sitter: bool,
        dry_run: bool,
    ) -> Self {
        Self {
            message,
            issue_ids: parse_issue_ids(issue_id),
            add_all,
            review,
            tree_sitter,
            dry_run,
        }
    }
    
    pub fn needs_issue_context(&self) -> bool {
        !self.issue_ids.is_empty()
    }
}

// === 触发结构化函数识别的示例改动 ===
// 该函数仅用于验证 Tree-sitter 与架构影响分析能正确识别函数级变更
pub fn sample_util_add(x: i32, y: i32) -> i32 {
    x + y
}

/// 提交操作 - Linus式静态函数设计
/// 
/// 干掉无意义的Executor包装器，直接用函数处理事情！

/// 执行提交流程
pub async fn execute_commit(config: &Config, commit_config: CommitConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = execute_commit_with_result(config, commit_config).await?;
    if !result.success {
        return Err("提交失败".into());
    }
    Ok(())
}

/// 执行提交流程并返回结构化结果  
pub async fn execute_commit_with_result(config: &Config, commit_config: CommitConfig) -> Result<CommitResult, Box<dyn std::error::Error + Send + Sync>> {
    let diff = get_changes()?;
    if diff.is_empty() {
        return Ok(CommitResult {
            success: true,
            message: "没有代码变更需要提交".to_string(),
            commit_hash: None,
            changes_count: 0,
            review_results: None,
            details: HashMap::new(),
        });
    }
    
    let issues = get_issue_context(config, &commit_config.issue_ids).await?;
    let commit_message = generate_commit_message(config, &diff, &issues, &commit_config).await?;
    
    let mut review_results = None;
    if commit_config.review {
        review_results = perform_review_with_result(config, &diff, &issues).await?;
    }
    
    let commit_hash = execute_git_operations_with_result(config, &commit_message, &commit_config).await?;
    
    // 计算变更数量
    let changes_count = count_changes(&diff)?;
    
    let mut details = HashMap::new();
    details.insert("review".to_string(), commit_config.review.to_string());
    details.insert("tree_sitter".to_string(), commit_config.tree_sitter.to_string());
    details.insert("add_all".to_string(), commit_config.add_all.to_string());
    details.insert("dry_run".to_string(), commit_config.dry_run.to_string());
    
    if !commit_config.issue_ids.is_empty() {
        details.insert("issue_ids".to_string(), commit_config.issue_ids.join(", "));
    }
    
    if let Some(ref message) = commit_config.message {
        details.insert("message".to_string(), message.clone());
    }
    
    Ok(CommitResult {
        success: true,
        message: "提交完成".to_string(),
        commit_hash,
        changes_count,
        review_results,
        details,
    })
}
    
/// 获取代码变更
fn get_changes() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    crate::git::get_all_diff()
}

/// 获取Issue上下文
async fn get_issue_context(config: &Config, issue_ids: &[String]) -> Result<Vec<Issue>, Box<dyn std::error::Error + Send + Sync>> {
    if issue_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    #[cfg(feature = "devops")]
    {
        if let Some(ref devops_config) = config.devops {
            let client = crate::devops::DevOpsClient::new(devops_config.clone());
            return client.get_issues(issue_ids).await.map_err(|e| e);
        }
    }
    
    eprintln!("⚠️ DevOps功能未启用或未配置，无法获取Issue信息");
    Ok(Vec::new())
}
    
/// 生成提交信息
async fn generate_commit_message(
    config: &Config,
    diff: &str,
    issues: &[Issue],
    commit_config: &CommitConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ref message) = commit_config.message {
        let final_message = format_commit_message(message, &commit_config.issue_ids);
        println!("📝 提交信息: {}", final_message);
        return Ok(final_message);
    }
    
    // 如果启用了 tree_sitter，先进行结构分析
    let tree_sitter_summary = if commit_config.tree_sitter {
        perform_structural_analysis(diff).await?
            .map(|s| serde_json::to_string(&s).unwrap_or_default())
    } else {
        None
    };
    
    // 尝试直接使用模板生成提交信息（传递 tree_sitter 信息）
    #[cfg(feature = "ai")]
    let ai_message = match crate::ai::generate_commit_message_with_template(
        config, 
        diff,
        tree_sitter_summary.as_deref()
    ).await {
        Ok(message) => message,
        Err(template_error) => {
            log::warn!("使用模板生成提交信息失败，降级为传统方式: {}", template_error);
            
            // 降级为传统方式：构建prompt然后调用AI
            let prompt = build_commit_prompt_fallback(config, diff, issues, commit_config).await?;
            crate::ai::call_ai(config, &prompt).await?
        }
    };

    #[cfg(not(feature = "ai"))]
    let ai_message = {
        // 未启用 AI 时的简易提交信息
        let changes = count_changes(diff).unwrap_or(0);
        let mut msg = if changes > 0 {
            format!("chore: update code ({} lines changed)", changes)
        } else {
            "chore: update code".to_string()
        };
        if tree_sitter_summary.is_some() {
            msg.push_str(" [structure]");
        }
        msg
    };
    
    let final_message = format_commit_message(ai_message.trim(), &commit_config.issue_ids);
    println!("📝 提交信息: {}", final_message);
    Ok(final_message)
}

/// 格式化提交信息（添加issue前缀）
fn format_commit_message(message: &str, issue_ids: &[String]) -> String {
    if issue_ids.is_empty() {
        message.to_string()
    } else {
        format!("{} {}", issue_ids.join(","), message)
    }
}

// 需要的支撑函数

/// 传统方式构建AI提示词（作为模板失败的降级方案）
async fn build_commit_prompt_fallback(config: &Config, diff: &str, issues: &[Issue], commit_config: &CommitConfig) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut prompt = format!("请为以下代码变更生成一个简洁的提交信息：\n\n{}", diff);
    
    // 添加Tree-sitter结构分析（如果启用）
    if commit_config.tree_sitter {
        if let Some(structural_summary) = perform_structural_analysis(diff).await? {
            let structure_info = format_structure_info(&structural_summary);
            prompt.push_str(&format!("\n\n{}", structure_info));
        }
    }
    
    if !issues.is_empty() {
        let context = build_issue_context(issues);
        prompt.push_str(&format!("\n\n相关Issue信息：\n{}", context));
    }
    
    Ok(prompt)
}

/// 构建问题上下文
fn build_issue_context(issues: &[Issue]) -> String {
    issues.iter()
        .map(|issue| format!(
            "Issue #{}: {}\n描述: {}\n状态: {}\n",
            issue.id, issue.title, issue.description, issue.status
        ))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 执行结构分析
async fn perform_structural_analysis(diff: &str) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🌳 正在进行Tree-sitter结构分析...");
    
    // 从diff中提取代码内容
    let code_content = extract_code_from_diff(diff);
    if code_content.is_empty() {
        println!("⚠️ 未能从diff中提取到代码内容");
        return Ok(None);
    }
    
    // 推断语言
    let language = infer_language_from_diff(diff);
    let Some(supported_lang) = language else {
        println!("⚠️ 不支持的语言或无法推断语言类型");
        return Ok(None);
    };
    
    println!("  检测到语言: {:?}", supported_lang);
    
    // 创建Tree-sitter管理器并分析
    match TreeSitterManager::new().await {
        Ok(mut manager) => {
            match manager.analyze_structure(&code_content, supported_lang) {
                Ok(summary) => {
                    println!("  ✅ 结构分析完成");
                    println!("     函数数量: {}", summary.functions.len());
                    println!("     类数量: {}", summary.classes.len());
                    Ok(Some(summary))
                }
                Err(e) => {
                    println!("  ⚠️ 结构分析失败: {}", e);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            println!("  ⚠️ Tree-sitter管理器初始化失败: {}", e);
            Ok(None)
        }
    }
}

/// 从diff中提取代码内容
fn extract_code_from_diff(diff: &str) -> String {
    let mut code_lines = Vec::new();
    
    for line in diff.lines() {
        // 跳过diff元数据行
        if line.starts_with("diff --git") 
            || line.starts_with("index")
            || line.starts_with("+++")
            || line.starts_with("---")
            || line.starts_with("@@") {
            continue;
        }
        
        // 提取添加的行（+开头）和上下文行（没有+/-前缀）
        if line.starts_with('+') {
            code_lines.push(&line[1..]);
        } else if !line.starts_with('-') && !line.is_empty() {
            code_lines.push(line);
        }
    }
    
    code_lines.join("\n")
}

/// 从diff中推断语言
fn infer_language_from_diff(diff: &str) -> Option<SupportedLanguage> {
    // 查找文件路径以推断语言
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("+++") {
            if let Some(path) = line.split_whitespace().last() {
                if let Some(extension) = std::path::Path::new(path)
                    .extension()
                    .and_then(|ext| ext.to_str()) {
                    return SupportedLanguage::from_extension(extension);
                }
            }
        }
    }
    
    None
}

/// 格式化结构信息用于提交信息生成
fn format_structure_info(summary: &StructuralSummary) -> String {
    let mut info = Vec::new();
    
    info.push(format!("代码结构分析 ({})语言:", summary.language));
    
    if !summary.functions.is_empty() {
        info.push(format!("新增/修改了 {} 个函数:", summary.functions.len()));
        for func in summary.functions.iter().take(3) {
            info.push(format!("- {}", func.name));
        }
        if summary.functions.len() > 3 {
            info.push(format!("- ... 还有 {} 个函数", summary.functions.len() - 3));
        }
    }
    
    if !summary.classes.is_empty() {
        info.push(format!("新增/修改了 {} 个类/结构体:", summary.classes.len()));
        for class in summary.classes.iter().take(3) {
            info.push(format!("- {}", class.name));
        }
        if summary.classes.len() > 3 {
            info.push(format!("- ... 还有 {} 个类", summary.classes.len() - 3));
        }
    }
    
    if !summary.complexity_hints.is_empty() {
        info.push("复杂度提示:".to_string());
        for hint in summary.complexity_hints.iter().take(2) {
            info.push(format!("- {}", hint));
        }
    }
    
    info.join("\n")
}

/// 执行评审并返回结构化结果
async fn perform_review_with_result(config: &Config, _diff: &str, issues: &[Issue]) -> Result<Option<ReviewResults>, Box<dyn std::error::Error + Send + Sync>> {
    // 创建评审配置
    let review_config = crate::review::ReviewConfig {
        language: None,
        format: "text".to_string(),
        output: None,
        tree_sitter: true,
        security_scan: true,
        scan_tool: None,
        block_on_critical: false,
        issue_ids: issues.iter().map(|i| i.id.clone()).collect(),
        deviation_analysis: true,
    };
    
    // 执行评审 - 现在使用静态函数！
    match crate::review::execute_review_with_result(config, review_config).await {
        Ok(result) => {
            let critical_count = result.findings.iter()
                .filter(|f| matches!(f.severity, crate::review::Severity::Error))
                .count();
            
            Ok(Some(ReviewResults {
                issues_found: result.findings.len(),
                critical_issues: critical_count,
                report: Some(result.message),
            }))
        }
        Err(_) => {
            // 评审失败不影响提交，只是不包含评审结果
            Ok(None)
        }
    }
}

/// 执行Git操作并返回提交哈希
async fn execute_git_operations_with_result(_config: &Config, commit_message: &str, commit_config: &CommitConfig) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    if commit_config.dry_run {
        println!("🔍 干运行模式 - 不会实际提交");
        return Ok(None);
    }
    
    // 添加文件到暂存区
    if commit_config.add_all {
        println!("📝 添加所有变更到暂存区...");
        crate::git::git_add_all()?;
    }
    
    // 执行提交
    println!("📝 执行提交: {}", commit_message);
    match crate::git::git_commit(commit_message) {
        Ok(hash) => {
            println!("✅ 提交成功: {}", hash);
            Ok(Some(hash))
        }
        Err(e) => {
            eprintln!("❌ 提交失败: {}", e);
            Err(e)
        }
    }
}

/// 计算变更数量
fn count_changes(diff: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut added_lines = 0;
    let mut removed_lines = 0;
    
    for line in diff.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            added_lines += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed_lines += 1;
        }
    }
    
    Ok(added_lines + removed_lines)
}
