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

    println!("  📝 检测到语言: {supported_lang:?}");

    // 创建Tree-sitter管理器并分析
    match TreeSitterManager::new().await {
        Ok(mut manager) => {
            match manager.analyze_structure(&code_content, supported_lang) {
                Ok(summary) => {
                    println!("  ✅ 结构分析完成");

                    // 生成架构洞察
                    let insights = InsightsGenerator::generate(&summary, None);

                    // 输出架构洞察而不是简单统计
                    println!(
                        "     🏗️ 架构模式违规: {}",
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

                    Ok(Some(summary))
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
            || line.starts_with("@@")
        {
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
        if let Some(stripped) = line.strip_prefix('+') {
            code_lines.push(stripped);
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

/// 从diff推断语言
fn infer_language_from_diff(diff: &str) -> Option<SupportedLanguage> {
    // 从文件扩展名推断
    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("+++") || line.starts_with("---") {
            if line.contains(".rs") {
                return Some(SupportedLanguage::Rust);
            } else if line.contains(".java") {
                return Some(SupportedLanguage::Java);
            } else if line.contains(".py") {
                return Some(SupportedLanguage::Python);
            } else if line.contains(".go") {
                return Some(SupportedLanguage::Go);
            } else if line.contains(".js") || line.contains(".mjs") || line.contains(".cjs") {
                return Some(SupportedLanguage::JavaScript);
            } else if line.contains(".ts") || line.contains(".tsx") {
                return Some(SupportedLanguage::TypeScript);
            } else if line.contains(".c") && !line.contains(".cpp") && !line.contains(".cc") {
                return Some(SupportedLanguage::C);
            } else if line.contains(".cpp")
                || line.contains(".cc")
                || line.contains(".cxx")
                || line.contains(".hpp")
            {
                return Some(SupportedLanguage::Cpp);
            }
        }
    }

    None
}
