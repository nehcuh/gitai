// review 分析器模块
// 负责结构分析和架构影响分析

use crate::architectural_impact::{ArchitecturalImpact, GitStateAnalyzer};
use crate::project_insights::InsightsGenerator;
use crate::tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};

/// 执行结构分析
pub async fn perform_structural_analysis(
    diff: &str,
    language: &Option<String>,
) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🌳 正在进行Tree-sitter结构分析...");

    // 检测支持的语言
    let detected_languages = if let Some(lang) = language {
        // 用户指定语言
        if let Some(supported_lang) = detect_supported_language(lang) {
            vec![supported_lang]
        } else {
            println!("  ⚠️  指定的语言 '{lang}' 不支持Tree-sitter分析");
            return Ok(None);
        }
    } else {
        // 自动检测所有语言
        infer_all_languages_from_diff(diff)
    };

    if detected_languages.is_empty() {
        println!("  💡 提示：当前变更中没有支持Tree-sitter分析的语言");
        println!("     支持的语言：Rust, Java, JavaScript, TypeScript, Python, Go, C, C++");
        return Ok(None);
    }

    // 按语言分离代码
    let language_code_map = extract_code_by_language(diff);

    if language_code_map.is_empty() {
        println!("  💡 提示：当前变更中没有可分析的代码内容");
        println!("     这可能是文档、配置文件或二进制文件的变更");
        return Ok(None);
    }

    // 检查是否为多语言项目
    if detected_languages.len() > 1 {
        println!("  🌐 检测到多语言项目：{:?}", detected_languages.iter().map(|l| l.name()).collect::<Vec<_>>());
        return perform_multi_language_analysis(language_code_map, detected_languages).await;
    } else {
        println!("  📏 检测到语言: {:?}", detected_languages[0]);
        return perform_single_language_analysis(language_code_map, detected_languages[0]).await;
    }
}

/// 执行多语言结构分析
async fn perform_multi_language_analysis(
    language_code_map: std::collections::HashMap<String, String>,
    detected_languages: Vec<SupportedLanguage>,
) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::tree_sitter::{LanguageSummary, StructuralSummary};
    
    // 创建 Tree-sitter 管理器
    let mut manager = match TreeSitterManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            println!("  ⚠️  Tree-sitter初始化失败，将使用传统文本分析模式");
            log::debug!("Tree-sitter初始化详情: {e}");
            return Ok(None);
        }
    };

    let mut language_summaries = std::collections::HashMap::new();
    let mut total_functions = 0;
    let mut total_classes = 0;
    let mut total_files = 0;

    for lang in detected_languages {
        let lang_name = lang.name();
        
        if let Some(code) = language_code_map.get(lang_name) {
            if code.trim().is_empty() {
                continue;
            }

            println!("  🔍 分析 {lang_name} 代码...");
            
            match manager.analyze_structure(code, lang) {
                Ok(single_summary) => {
                    let lang_summary = LanguageSummary {
                        language: lang_name.to_string(),
                        functions: single_summary.functions.clone(),
                        classes: single_summary.classes.clone(),
                        imports: single_summary.imports.clone(),
                        exports: single_summary.exports.clone(),
                        comments: single_summary.comments.clone(),
                        complexity_hints: single_summary.complexity_hints.clone(),
                        calls: single_summary.calls.clone(),
                        file_count: 1, // 简化处理，实际需要统计文件数量
                    };
                    
                    total_functions += lang_summary.functions.len();
                    total_classes += lang_summary.classes.len();
                    total_files += 1;
                    
                    println!(
                        "    ✅ {lang_name}: {} 函数, {} 类, {} 注释",
                        lang_summary.functions.len(),
                        lang_summary.classes.len(),
                        lang_summary.comments.len()
                    );
                    
                    language_summaries.insert(lang_name.to_string(), lang_summary);
                }
                Err(e) => {
                    println!("    ⚠️  {lang_name} 分析失败: {e}");
                    log::debug!("{lang_name} Tree-sitter分析详情: {e}");
                }
            }
        }
    }

    if language_summaries.is_empty() {
        println!("  ⚠️  所有语言分析均失败，将使用传统文本分析模式");
        return Ok(None);
    }

    println!("  ✅ 多语言结构分析完成");
    println!("     📊 总计: {} 种语言, {} 函数, {} 类, {} 文件", 
             language_summaries.len(), total_functions, total_classes, total_files);

    Ok(Some(StructuralSummary::multi_language(language_summaries)))
}

/// 执行单语言结构分析（保持向后兼容）
async fn perform_single_language_analysis(
    language_code_map: std::collections::HashMap<String, String>,
    language: SupportedLanguage,
) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::tree_sitter::{LanguageSummary, StructuralSummary};
    
    let lang_name = language.name();
    let code = language_code_map.get(lang_name)
        .or_else(|| {
            // 如果没有找到对应语言的代码，尝试合并所有代码
            if language_code_map.len() == 1 {
                language_code_map.values().next()
            } else {
                None
            }
        })
        .ok_or("No code found for the specified language")?;

    if code.trim().is_empty() {
        println!("  💡 提示：{lang_name} 代码为空");
        return Ok(None);
    }

    // 创建 Tree-sitter 管理器并分析
    match TreeSitterManager::new().await {
        Ok(mut manager) => {
            match manager.analyze_structure(code, language) {
                Ok(summary) => {
                    println!("  ✅ 结构分析完成");

                    // 生成架构洞察
                    let insights = InsightsGenerator::generate(&summary, None);

                    // 输出架构洞察
                    println!(
                        "     🏢️ 架构模式违规: {}",
                        insights.architecture.pattern_violations.len()
                    );
                    println!(
                        "     🔄 循环依赖: {}",
                        insights
                            .architecture
                            .module_dependencies
                            .circular_dependencies
                            .len()
                    );
                    println!(
                        "     ⚡ 复杂度热点: {}",
                        insights.quality_hotspots.complexity_hotspots.len()
                    );
                    println!(
                        "     📊 API 接口: {}",
                        insights.api_surface.public_apis.len()
                    );

                    // 返回单语言模式的结果（保持向后兼容）
                    let lang_summary = LanguageSummary::from_structural_summary(&summary);
                    Ok(Some(StructuralSummary::single_language(lang_name.to_string(), lang_summary)))
                }
                Err(e) => {
                    println!("  ⚠️  结构分析失败，将使用传统文本分析模式");
                    log::debug!("Tree-sitter分析详情: {e}");
                    Ok(None)
                }
            }
        }
        Err(e) => {
            println!("  ⚠️  Tree-sitter初始化失败，将使用传统文本分析模式");
            log::debug!("Tree-sitter初始化详情: {e}");
            Ok(None)
        }
    }
}

/// 执行架构影响分析  
pub async fn perform_architectural_impact_analysis(
    diff: &str,
) -> Result<Option<ArchitecturalImpact>, Box<dyn std::error::Error + Send + Sync>> {
    println!("🏗️ 正在进行架构影响分析...");

    // 创建GitStateAnalyzer并分析
    let analyzer = GitStateAnalyzer::new();
    match analyzer.analyze_git_diff(diff).await {
        Ok(impact) => {
            println!("  ✅ 架构影响分析完成");

            // 输出关键指标
            let total_changes = impact.function_changes.len()
                + impact.struct_changes.len()
                + impact.interface_changes.len();
            println!("     📊 总变更数: {total_changes}");
            println!("     🔧 函数变更: {}", impact.function_changes.len());
            println!("     🏗️ 结构体变更: {}", impact.struct_changes.len());
            println!("     🔌 接口变更: {}", impact.interface_changes.len());

            // 输出影响范围
            if !impact.impact_summary.affected_modules.is_empty() {
                println!(
                    "     📦 影响模块: {}",
                    impact.impact_summary.affected_modules.len()
                );
            }
            if !impact.impact_summary.breaking_changes.is_empty() {
                println!(
                    "     ⚠️  破坏性变更: {}",
                    impact.impact_summary.breaking_changes.len()
                );
            }

            Ok(Some(impact))
        }
        Err(e) => {
            println!("  ⚠️  架构影响分析失败: {e}");
            log::debug!("架构影响分析详情: {e}");
            Ok(None)
        }
    }
}

/// 从 diff 中提取代码内容（单语言模式，保持向后兼容）
fn extract_code_from_diff(diff: &str) -> String {
    let language_code_map = extract_code_by_language(diff);
    
    // 合并所有语言的代码
    let mut all_code = Vec::new();
    for (language, code) in language_code_map {
        if !code.trim().is_empty() {
            all_code.push(format!("// === {} ===\n{}", language, code));
        }
    }
    
    all_code.join("\n\n")
}

/// 按语言分离 diff 中的代码变更
fn extract_code_by_language(diff: &str) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut language_code_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_file_language: Option<SupportedLanguage> = None;
    let mut in_file_section = false;

    for line in diff.lines() {
        // 检测文件变更开始
        if line.starts_with("diff --git") {
            // 从 diff 行中提取文件路径
            if let Some(file_path) = extract_file_path_from_diff_line(line) {
                current_file_language = detect_language_from_file_path(&file_path);
            }
            in_file_section = true;
            continue;
        }

        // 从 +++ 行中提取文件路径（更准确）
        if line.starts_with("+++") {
            if let Some(file_path) = line.strip_prefix("+++ ").or_else(|| line.strip_prefix("+++ b/")) {
                current_file_language = detect_language_from_file_path(file_path);
            }
            continue;
        }

        // 跳过其他diff元数据行
        if line.starts_with("index")
            || line.starts_with("---")
            || line.starts_with("@@")
        {
            continue;
        }

        // 空行表示文件变更结束
        if line.is_empty() && in_file_section {
            in_file_section = false;
            current_file_language = None;
            continue;
        }

        // 提取代码行（仅当能识别语言时）
        if let Some(lang) = current_file_language {
            let lang_name = lang.name().to_string();
            
            if let Some(stripped) = line.strip_prefix('+') {
                // 添加的行
                language_code_map.entry(lang_name).or_default().push(stripped.to_string());
            } else if !line.starts_with('-') && !line.trim().is_empty() {
                // 上下文行
                language_code_map.entry(lang_name).or_default().push(line.to_string());
            }
        }
    }

    // 将 Vec<String> 转换为 String
    language_code_map
        .into_iter()
        .map(|(lang, lines)| (lang, lines.join("\n")))
        .collect()
}

/// 从 diff 行中提取文件路径
fn extract_file_path_from_diff_line(line: &str) -> Option<String> {
    // diff --git a/src/main.rs b/src/main.rs
    if let Some(rest) = line.strip_prefix("diff --git ") {
        if let Some(b_part) = rest.split_whitespace().nth(1) {
            return b_part.strip_prefix("b/").map(|s| s.to_string()).or_else(|| Some(b_part.to_string()));
        }
    }
    None
}

/// 从文件路径检测语言
fn detect_language_from_file_path(file_path: &str) -> Option<SupportedLanguage> {
    if let Some(extension) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            return SupportedLanguage::from_extension(ext_str);
        }
    }
    None
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

/// 从diff推断语言（单语言模式，保持向后兼容）
fn infer_language_from_diff(diff: &str) -> Option<SupportedLanguage> {
    let languages = infer_all_languages_from_diff(diff);
    languages.into_iter().next()
}

/// 从diff推断所有语言
fn infer_all_languages_from_diff(diff: &str) -> Vec<SupportedLanguage> {
    use std::collections::HashSet;
    let mut detected_languages = HashSet::new();
    
    // 从文件扩展名推断
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("+++") || line.starts_with("---") {
            if line.contains(".rs") {
                detected_languages.insert(SupportedLanguage::Rust);
            }
            if line.contains(".java") {
                detected_languages.insert(SupportedLanguage::Java);
            }
            if line.contains(".py") || line.contains(".pyi") {
                detected_languages.insert(SupportedLanguage::Python);
            }
            if line.contains(".go") {
                detected_languages.insert(SupportedLanguage::Go);
            }
            if line.contains(".js") || line.contains(".mjs") || line.contains(".cjs") {
                detected_languages.insert(SupportedLanguage::JavaScript);
            }
            if line.contains(".ts") || line.contains(".tsx") {
                detected_languages.insert(SupportedLanguage::TypeScript);
            }
            if line.contains(".c") && !line.contains(".cpp") && !line.contains(".cc") {
                detected_languages.insert(SupportedLanguage::C);
            }
            if line.contains(".cpp")
                || line.contains(".cc")
                || line.contains(".cxx")
                || line.contains(".hpp")
            {
                detected_languages.insert(SupportedLanguage::Cpp);
            }
        }
    }

    detected_languages.into_iter().collect()
}
