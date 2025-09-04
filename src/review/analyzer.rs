// review 分析器模块
// 负责结构分析和架构影响分析

use crate::architectural_impact::{ArchitecturalImpact, GitStateAnalyzer};
use crate::project_insights::InsightsGenerator;
use crate::tree_sitter::{StructuralSummary, SupportedLanguage, TreeSitterManager};

/// Perform structural analysis on a git diff using Tree-sitter.
///
/// This function orchestrates language detection, code extraction, and delegation
/// to either single-language or multi-language analysis backends:
/// - If `language` is Some, attempts to map it to a supported language; returns
///   `Ok(None)` if the name is unsupported.
/// - If `language` is None, infers all languages present in `diff` by file
///   extensions.
/// - If no supported languages are detected or no analyzable code is extracted,
///   returns `Ok(None)`.
/// - For a single detected language it invokes `perform_single_language_analysis`.
/// - For multiple detected languages it invokes `perform_multi_language_analysis`.
///
/// Parameters:
/// - `diff`: a git-style unified diff string containing the changes to analyze.
/// - `language`: optional user-specified language name (case-insensitive, e.g. "rust", "python").
///   When provided, analysis is restricted to that language; when omitted, all
///   detectable supported languages in the diff are analyzed.
///
/// Returns:
/// - `Ok(Some(StructuralSummary))` on successful analysis with a populated summary.
/// - `Ok(None)` when analysis cannot be performed (unsupported language,
///   no supported languages present, or no analyzable code in the diff).
/// - `Err(...)` for unexpected internal errors.
///
/// # Examples
///
/// ```
/// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
/// let diff = r#"
/// diff --git a/src/main.rs b/src/main.rs
/// index 0000000..1111111 100644
/// --- a/src/main.rs
/// +++ b/src/main.rs
/// @@ -0,0 +1,4 @@
/// +fn main() {
/// +    println!("hello");
/// +}
/// "#;
/// let summary = perform_structural_analysis(diff, &None).await?;
/// assert!(summary.is_some());
/// # Ok(())
/// # }
/// ```
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
        println!(
            "  🌐 检测到多语言项目：{:?}",
            detected_languages
                .iter()
                .map(|l| l.name())
                .collect::<Vec<_>>()
        );
        perform_multi_language_analysis(language_code_map, detected_languages).await
    } else {
        println!("  📏 检测到语言: {:?}", detected_languages[0]);
        perform_single_language_analysis(language_code_map, detected_languages[0]).await
    }
}

/// Performs structural analysis across multiple programming languages using Tree-sitter.
///
/// Given a map from language name -> concatenated source text and a list of detected
/// languages, attempts to analyze each language's code with a Tree-sitter manager
/// and aggregates per-language summaries into a multi-language `StructuralSummary`.
///
/// - language_code_map: keys are language names (as returned by `SupportedLanguage::name()`),
///   values are the source content to analyze for that language. Empty or whitespace-only
///   values are skipped.
/// - detected_languages: the set of languages to attempt; languages not present in
///   `language_code_map` are ignored.
///
/// Returns `Ok(Some(StructuralSummary))` when at least one language was successfully
/// analyzed; returns `Ok(None)` if Tree-sitter initialization fails or if all analyses
/// failed/produced no summaries. Errors are returned only for unexpected failures.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// use std::collections::HashMap;
/// // prepare minimal inputs
/// let mut map = HashMap::new();
/// map.insert("rust".to_string(), "fn hello() {}".to_string());
/// let langs = vec![crate::SupportedLanguage::Rust];
///
/// let result = crate::review::perform_multi_language_analysis(map, langs).await;
/// assert!(result.is_ok());
/// if let Ok(Some(summary)) = result {
///     // expect at least one language summary
///     assert!(summary.language_summaries().len() >= 1);
/// }
/// # });
/// ```
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
    println!(
        "     📊 总计: {} 种语言, {} 函数, {} 类, {} 文件",
        language_summaries.len(),
        total_functions,
        total_classes,
        total_files
    );

    Ok(Some(StructuralSummary::multi_language(language_summaries)))
}

/// Perform structural analysis for a single language (backwards compatible).
///
/// This function looks up source code for `language` in `language_code_map` (keyed by the language's
/// canonical name). If no entry exists for the requested language and `language_code_map` contains
/// exactly one entry, that single entry is used as a fallback. If no code is found or the found
/// code is empty, the function returns `Ok(None)`.
///
/// When code is available, the function attempts to initialize the Tree-sitter manager and run a
/// structural analysis for the specified language. On successful analysis it generates architectural
/// insights and returns `Ok(Some(StructuralSummary::single_language(...)))`. If Tree-sitter cannot
/// be initialized or the analysis fails, the function returns `Ok(None)`.
///
/// Notes:
/// - Side effects: prints brief progress/insight summaries to stdout and emits debug logs for
///   Tree-sitter failures.
/// - The function never panics; errors are propagated inside the `Result` wrapper only for
///   unexpected failures in the control flow (the common failure modes return `Ok(None)`).
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// // Prepare a map from language name to source code.
/// let mut map = HashMap::new();
/// map.insert("rust".to_string(), "fn main() { println!(\"hi\"); }".to_string());
///
/// // `SupportedLanguage::Rust` is assumed to be available in scope.
/// let summary = tokio::runtime::Runtime::new()
///     .unwrap()
///     .block_on(async { perform_single_language_analysis(map, SupportedLanguage::Rust).await })
///     .unwrap();
///
/// // `summary` is `Ok(Some(...))` on success, `Ok(None)` when analysis wasn't possible.
/// ```
async fn perform_single_language_analysis(
    language_code_map: std::collections::HashMap<String, String>,
    language: SupportedLanguage,
) -> Result<Option<StructuralSummary>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::tree_sitter::{LanguageSummary, StructuralSummary};

    let lang_name = language.name();
    let code = language_code_map
        .get(lang_name)
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
                    Ok(Some(StructuralSummary::single_language(
                        lang_name.to_string(),
                        lang_summary,
                    )))
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

/// Analyze a git diff and produce an architectural impact report.
///
/// This function runs an asynchronous analysis of the provided unified git diff text
/// and returns an ArchitecturalImpact describing detected changes (functions, structs,
/// interfaces) and affected modules.
///
/// Parameters:
/// - `diff`: the unified git diff content to analyze (as produced by `git diff`).
///
/// Returns:
/// - `Ok(Some(ArchitecturalImpact))` when analysis completes successfully.
/// - `Ok(None)` when analysis fails or cannot be performed (analysis errors are logged).
/// - `Err(...)` only if an unexpected internal error occurs.
///
/// # Examples
///
/// ```
/// # use review_analyzer::analysis::perform_architectural_impact_analysis;
/// # use review_analyzer::impact::ArchitecturalImpact;
/// #[tokio::test]
/// async fn example_architectural_impact() {
///     let diff = "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,3 +1,4 @@\n+pub fn new_api() {}\n";
///     let result = perform_architectural_impact_analysis(diff).await;
///     // result is Ok(Some(...)) on successful analysis, Ok(None) on analysis failure
///     assert!(matches!(result, Ok(_) ));
/// }
/// ```
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

/// Partition a git-style diff into per-language code snippets.
///
/// Scans a unified git diff and groups added and context code lines by the inferred
/// programming language of the affected files. Language detection is performed
/// from diff headers and `+++` file path lines using `detect_language_from_file_path`.
///
/// Behavior notes:
/// - Only files whose language can be mapped to a supported language are included.
/// - Added lines (those starting with `+`) and context lines (non-`-` lines that are not empty)
///   are collected; removed lines (starting with `-`) are ignored.
/// - Language -> code is returned as a HashMap<String, String> where the String value
///   is the collected lines joined with `\n`.
/// - Empty lines that occur while inside a file section are treated as the end of that file's section.
///
/// Returns a map from the language name (as returned by `SupportedLanguage::name()`) to the
/// concatenated code fragment for that language.
///
/// # Examples
///
/// ```
/// let diff = "\
/// diff --git a/src/main.rs b/src/main.rs
/// index 123..456 100644
/// --- a/src/main.rs
/// +++ b/src/main.rs
/// @@ -1,3 +1,4 @@
///  fn used() {}
/// +fn added() {}
/// ";
///
/// let map = extract_code_by_language(diff);
/// assert!(map.contains_key("rust"));
/// let rust_code = &map["rust"];
/// assert!(rust_code.contains("fn used() {}"));
/// assert!(rust_code.contains("fn added() {}"));
/// ```
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
            if let Some(file_path) = line
                .strip_prefix("+++ ")
                .or_else(|| line.strip_prefix("+++ b/"))
            {
                current_file_language = detect_language_from_file_path(file_path);
            }
            continue;
        }

        // 跳过其他diff元数据行
        if line.starts_with("index") || line.starts_with("---") || line.starts_with("@@") {
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
                language_code_map
                    .entry(lang_name)
                    .or_default()
                    .push(stripped.to_string());
            } else if !line.starts_with('-') && !line.trim().is_empty() {
                // 上下文行
                language_code_map
                    .entry(lang_name)
                    .or_default()
                    .push(line.to_string());
            }
        }
    }

    // 将 Vec<String> 转换为 String
    language_code_map
        .into_iter()
        .map(|(lang, lines)| (lang, lines.join("\n")))
        .collect()
}

/// Extracts the file path from a git diff header line (e.g., `diff --git a/path b/path`).
///
/// Returns the second path token (the `b/...` path) with a leading `b/` stripped if present.
/// Returns `None` if the line does not start with `diff --git ` or does not contain a second path token.
///
/// # Examples
///
/// ```
/// let line = "diff --git a/src/main.rs b/src/main.rs";
/// assert_eq!(extract_file_path_from_diff_line(line), Some("src/main.rs".to_string()));
///
/// let no_diff = "index 83db48f..bf3e3aa 100644";
/// assert_eq!(extract_file_path_from_diff_line(no_diff), None);
/// ```
fn extract_file_path_from_diff_line(line: &str) -> Option<String> {
    // diff --git a/src/main.rs b/src/main.rs
    if let Some(rest) = line.strip_prefix("diff --git ") {
        if let Some(b_part) = rest.split_whitespace().nth(1) {
            return b_part
                .strip_prefix("b/")
                .map(|s| s.to_string())
                .or_else(|| Some(b_part.to_string()));
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

/// Map a user-provided language name or common alias to a SupportedLanguage.
///
/// Performs case-insensitive matching for common names and aliases. Recognized inputs include:
/// - "java"
/// - "rust"
/// - "c"
/// - "cpp", "c++"
/// - "python"
/// - "go"
/// - "javascript", "js"
/// - "typescript", "ts"
///
/// Returns `Some(SupportedLanguage)` when a match is found, or `None` for unrecognized names.
///
/// # Examples
///
/// ```
/// assert_eq!(detect_supported_language("Rust"), Some(SupportedLanguage::Rust));
/// assert_eq!(detect_supported_language("c++"), Some(SupportedLanguage::Cpp));
/// assert_eq!(detect_supported_language("TS"), Some(SupportedLanguage::TypeScript));
/// assert_eq!(detect_supported_language("unknown"), None);
/// ```
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

/// Infers all programming languages present in a git-style diff by file extension.
///
/// Scans diff header and file indicator lines (e.g., those starting with `diff --git`, `+++`, or `---`)
/// and collects any supported languages detected from common file extensions. Returns a vector of
/// unique `SupportedLanguage` entries (order is unspecified).
///
/// # Examples
///
/// ```
/// let diff = r#"
/// diff --git a/src/main.rs b/src/main.rs
/// index 83db48f..f735c13 100644
/// --- a/src/main.rs
/// +++ b/src/main.rs
/// diff --git a/app.py b/app.py
/// --- a/app.py
/// +++ b/app.py
/// "#;
///
/// let langs = infer_all_languages_from_diff(diff);
/// assert!(langs.contains(&SupportedLanguage::Rust));
/// assert!(langs.contains(&SupportedLanguage::Python));
/// ```
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
