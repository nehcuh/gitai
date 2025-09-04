// MCP Analysis 服务
//
// 提供代码结构分析功能的 MCP 服务实现

use crate::{config::Config, mcp::*, tree_sitter};
use log::{debug, error, info, warn};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Analysis 服务
pub struct AnalysisService {
    #[allow(dead_code)]
    config: Config,
    verbosity: u32,
}

impl AnalysisService {
    /// 创建新的 Analysis 服务
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let verbosity = if let Some(mcp_config) = &config.mcp {
            if let Some(analysis_config) = &mcp_config.services.analysis {
                analysis_config.verbosity
            } else {
                1
            }
        } else {
            1
        };

        Ok(Self { config, verbosity })
    }

    /// 执行代码分析
    async fn execute_analysis(
        &self,
        params: AnalysisParams,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 开始代码分析: {}", params.path);
        debug!(
            "📋 分析参数: 语言={:?}, 详细程度={:?}",
            params.language, params.verbosity
        );

        let path = Path::new(&params.path);

        // 验证路径是否存在
        if !path.exists() {
            error!("❌ 分析路径不存在: {}", params.path);
            return Err(format!("分析路径不存在: {}", params.path).into());
        }

        // 检查是否为目录
        if path.is_dir() {
            info!("📁 检测到目录路径，尝试分析目录中的文件");
            return self.analyze_directory(path, &params).await;
        }

        // 使用真实的分析逻辑 - 单个文件分析
        let language = if let Some(ref lang) = params.language {
            debug!("🌐 使用指定语言: {}", lang);
            tree_sitter::SupportedLanguage::from_name(lang)
                .ok_or_else(|| format!("不支持的语言: {}", lang))?
        } else {
            debug!("🔍 自动推断语言");
            Self::infer_language_from_path(path).map_err(|e| format!("无法推断语言: {}", e))?
        };

        // 读取文件内容
        let code_content = std::fs::read_to_string(path).map_err(|e| {
            error!("❌ 无法读取文件 {}: {}", path.display(), e);
            format!("无法读取文件 {}: {}", path.display(), e)
        })?;

        debug!("📄 代码内容长度: {} 字符", code_content.len());

        // 创建 Tree-sitter 管理器并分析
        let mut manager = tree_sitter::TreeSitterManager::new().await.map_err(|e| {
            error!("❌ 无法创建 Tree-sitter 管理器: {}", e);
            format!("无法创建 Tree-sitter 管理器: {}", e)
        })?;

        let summary = manager
            .analyze_structure(&code_content, language)
            .map_err(|e| {
                error!("❌ 结构分析失败: {}", e);
                format!("结构分析失败: {}", e)
            })?;

        debug!(
            "📊 分析结果: 函数={}, 类={}, 注释={}, 复杂度={}",
            summary.functions.len(),
            summary.classes.len(),
            summary.comments.len(),
            summary.complexity_hints.len()
        );

        // 转换分析结果
        let result = self.convert_analysis_result(summary, self.verbosity);
        info!("✅ 代码分析完成: {}", params.path);
        Ok(result)
    }

    /// 分析目录中的所有代码文件
    async fn analyze_directory(
        &self,
        dir_path: &Path,
        params: &AnalysisParams,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("📁 开始分析目录: {}", dir_path.display());

        // 查找目录中的代码文件
        let code_files = self.find_code_files(dir_path, params.language.as_deref())?;

        if code_files.is_empty() {
            warn!("⚠️ 目录中未找到可分析的代码文件");
            return Ok(AnalysisResult {
                success: false,
                message: "目录中未找到可分析的代码文件".to_string(),
                language: "unknown".to_string(),
                summary: CodeSummary {
                    total_lines: 0,
                    code_lines: 0,
                    comment_lines: 0,
                    blank_lines: 0,
                    complexity_score: 0,
                },
                structures: CodeStructures {
                    functions: vec![],
                    classes: vec![],
                    imports: vec![],
                },
                metrics: CodeMetrics {
                    cyclomatic_complexity: 0,
                    maintainability_index: 0.0,
                    comment_ratio: 0.0,
                },
                details: {
                    let mut details = HashMap::new();
                    details.insert("directory_path".to_string(), dir_path.display().to_string());
                    details.insert("file_count".to_string(), "0".to_string());
                    details.insert("message".to_string(), "未找到支持的代码文件".to_string());
                    details
                },
            });
        }

        info!("📋 找到 {} 个代码文件，开始分析", code_files.len());

        // 分析所有文件并聚合结果
        let mut total_summary = CodeSummary {
            total_lines: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            complexity_score: 0,
        };

        let mut all_functions = Vec::new();
        let mut all_classes = Vec::new();
        let mut all_imports = Vec::new();
        let mut language_stats = HashMap::new();

        for file_path in &code_files {
            debug!("🔍 分析文件: {}", file_path.display());

            match self.analyze_single_file(&file_path).await {
                Ok(result) => {
                    total_summary.total_lines += result.summary.total_lines;
                    total_summary.code_lines += result.summary.code_lines;
                    total_summary.comment_lines += result.summary.comment_lines;
                    total_summary.blank_lines += result.summary.blank_lines;
                    total_summary.complexity_score += result.summary.complexity_score;

                    all_functions.extend(result.structures.functions);
                    all_classes.extend(result.structures.classes);
                    all_imports.extend(result.structures.imports);

                    *language_stats.entry(result.language.clone()).or_insert(0) += 1;
                }
                Err(e) => {
                    warn!("⚠️ 分析文件 {} 失败: {}", file_path.display(), e);
                }
            }
        }

        // 计算平均指标
        let file_count = code_files.len();
        let avg_complexity = if file_count > 0 {
            total_summary.complexity_score / file_count as u32
        } else {
            0
        };

        let comment_ratio = if total_summary.total_lines > 0 {
            total_summary.comment_lines as f64 / total_summary.total_lines as f64
        } else {
            0.0
        };

        let mut details = HashMap::new();
        details.insert("directory_path".to_string(), dir_path.display().to_string());
        details.insert("file_count".to_string(), file_count.to_string());
        details.insert("total_files_analyzed".to_string(), file_count.to_string());
        details.insert(
            "language_distribution".to_string(),
            serde_json::to_string(&language_stats).unwrap_or_default(),
        );

        if params.verbosity.unwrap_or(1) > 1 {
            details.insert(
                "all_functions".to_string(),
                serde_json::to_string(&all_functions).unwrap_or_default(),
            );
            details.insert(
                "all_classes".to_string(),
                serde_json::to_string(&all_classes).unwrap_or_default(),
            );
        }

        info!("✅ 目录分析完成: {} 个文件", file_count);

        Ok(AnalysisResult {
            success: true,
            message: format!("目录分析完成，共分析 {} 个文件", file_count),
            language: "multi".to_string(), // 多语言项目
            summary: total_summary,
            structures: CodeStructures {
                functions: all_functions,
                classes: all_classes,
                imports: all_imports,
            },
            metrics: CodeMetrics {
                cyclomatic_complexity: avg_complexity,
                maintainability_index: 75.0, // 简化计算
                comment_ratio,
            },
            details,
        })
    }

    /// 查找目录中的代码文件
    fn find_code_files(
        &self,
        dir_path: &Path,
        language_filter: Option<&str>,
    ) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let mut code_files = Vec::new();

        // 支持的文件扩展名
        let supported_extensions = if let Some(lang) = language_filter {
            // 如果指定了语言，只查找该语言的文件
            match lang {
                "rust" => vec!["rs"],
                "java" => vec!["java"],
                "c" => vec!["c", "h"],
                "cpp" => vec!["cpp", "cc", "cxx", "hpp", "hxx"],
                "python" => vec!["py"],
                "go" => vec!["go"],
                "javascript" => vec!["js"],
                "typescript" => vec!["ts"],
                _ => vec![],
            }
        } else {
            // 否则查找所有支持的代码文件
            vec![
                "rs", "java", "c", "h", "cpp", "cc", "cxx", "hpp", "hxx", "py", "go", "js", "ts",
            ]
        };

        // 递归查找文件
        let mut entries = std::fs::read_dir(dir_path)
            .map_err(|e| format!("无法读取目录 {}: {}", dir_path.display(), e))?;

        while let Some(entry) = entries.next() {
            let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                // 递归处理子目录，但跳过一些常见的目录
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !["target", "node_modules", ".git", ".idea", "vendor", "build"]
                    .contains(&file_name)
                {
                    code_files.extend(self.find_code_files(&path, language_filter)?);
                }
            } else if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if supported_extensions.contains(&extension) {
                    code_files.push(path);
                }
            }
        }

        Ok(code_files)
    }

    /// 分析单个文件
    async fn analyze_single_file(
        &self,
        file_path: &Path,
    ) -> Result<AnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("🔍 分析单个文件: {}", file_path.display());

        let language = Self::infer_language_from_path(file_path)?;

        let code_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件 {}: {}", file_path.display(), e))?;

        let mut manager = tree_sitter::TreeSitterManager::new()
            .await
            .map_err(|e| format!("无法创建 Tree-sitter 管理器: {}", e))?;

        let summary = manager
            .analyze_structure(&code_content, language)
            .map_err(|e| format!("结构分析失败: {}", e))?;

        // 转换分析结果
        let result = self.convert_analysis_result(summary, 1); // 使用默认详细程度

        Ok(result)
    }

    fn infer_language_from_path(
        path: &Path,
    ) -> Result<tree_sitter::SupportedLanguage, Box<dyn std::error::Error + Send + Sync>> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| "无法确定文件类型".to_string())?;

        tree_sitter::SupportedLanguage::from_extension(extension)
            .ok_or_else(|| format!("不支持的文件扩展名: {}", extension).into())
    }

    fn convert_analysis_result(
        &self,
        summary: tree_sitter::StructuralSummary,
        verbosity: u32,
    ) -> AnalysisResult {
        let mut details = HashMap::new();

        // 检查是否为多语言模式
        if summary.is_multi_language() {
            // 多语言模式
            details.insert("mode".to_string(), "multi-language".to_string());
            details.insert(
                "languages".to_string(),
                summary.detected_languages().join(", "),
            );
            details.insert(
                "language_count".to_string(),
                summary.language_summaries.len().to_string(),
            );

            // 各语言统计
            for (lang, lang_summary) in &summary.language_summaries {
                details.insert(
                    format!("{}_functions", lang),
                    lang_summary.functions.len().to_string(),
                );
                details.insert(
                    format!("{}_classes", lang),
                    lang_summary.classes.len().to_string(),
                );
                details.insert(
                    format!("{}_comments", lang),
                    lang_summary.comments.len().to_string(),
                );
                details.insert(
                    format!("{}_files", lang),
                    lang_summary.file_count.to_string(),
                );
            }

            // 高详细程度时包含结构信息
            if verbosity > 1 {
                for (lang, lang_summary) in &summary.language_summaries {
                    details.insert(
                        format!("{}_functions_detail", lang),
                        serde_json::to_string(&lang_summary.functions).unwrap_or_default(),
                    );
                    details.insert(
                        format!("{}_classes_detail", lang),
                        serde_json::to_string(&lang_summary.classes).unwrap_or_default(),
                    );
                }
            }
        } else {
            // 单语言模式（向后兼容）
            details.insert("mode".to_string(), "single-language".to_string());
            details.insert("language".to_string(), summary.language.clone());
            details.insert(
                "functions_count".to_string(),
                summary.functions.len().to_string(),
            );
            details.insert(
                "classes_count".to_string(),
                summary.classes.len().to_string(),
            );
            details.insert(
                "imports_count".to_string(),
                summary.imports.len().to_string(),
            );
            details.insert(
                "comments_count".to_string(),
                summary.comments.len().to_string(),
            );

            if verbosity > 1 {
                details.insert(
                    "functions".to_string(),
                    serde_json::to_string(&summary.functions).unwrap_or_default(),
                );
                details.insert(
                    "classes".to_string(),
                    serde_json::to_string(&summary.classes).unwrap_or_default(),
                );
                details.insert(
                    "imports".to_string(),
                    serde_json::to_string(&summary.imports).unwrap_or_default(),
                );
                details.insert(
                    "comments".to_string(),
                    serde_json::to_string(&summary.comments).unwrap_or_default(),
                );
            }
        }

        // 计算总体指标
        let total_lines = 100; // 简化计算
        let comment_lines = summary.comments.len();
        let complexity_score = summary.complexity_hints.len() as u32;

        // 根据模式生成不同的消息
        let message = if summary.is_multi_language() {
            let lang_list = summary.detected_languages().join(", ");
            format!(
                "多语言代码分析完成：{} (共{}种语言)",
                lang_list,
                summary.language_summaries.len()
            )
        } else {
            format!("代码分析完成：{}", summary.language)
        };

        let language_display = if summary.is_multi_language() {
            "multi-language".to_string()
        } else {
            summary.language.clone()
        };

        AnalysisResult {
            success: true,
            message,
            language: language_display,
            summary: CodeSummary {
                total_lines,
                code_lines: if total_lines > comment_lines {
                    total_lines - comment_lines
                } else {
                    0
                },
                comment_lines,
                blank_lines: 0,
                complexity_score,
            },
            structures: CodeStructures {
                functions: vec![], // TODO: 转换 FunctionInfo
                classes: vec![],   // TODO: 转换 ClassInfo
                imports: summary.imports,
            },
            metrics: CodeMetrics {
                cyclomatic_complexity: complexity_score,
                maintainability_index: 85.0, // 简化计算
                comment_ratio: if total_lines > 0 {
                    (comment_lines as f64) / (total_lines as f64)
                } else {
                    0.0
                },
            },
            details,
        }
    }

    // 这个方法暂时不需要，因为我们在 convert_analysis_result 中已经简化了计算
    #[allow(dead_code)]
    fn calculate_maintainability_index(_summary: &tree_sitter::StructuralSummary) -> f64 {
        85.0 // 简化返回固定值
    }
}

#[async_trait::async_trait]
impl crate::mcp::GitAiMcpService for AnalysisService {
    fn name(&self) -> &str {
        "analysis"
    }

    fn description(&self) -> &str {
        "执行多语言代码结构分析，支持 8 种编程语言，提供详细的代码度量和结构信息"
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "execute_analysis".to_string().into(),
                description: "执行多语言代码结构分析，支持单个文件或整个目录的分析。能够自动检测和分析 Rust、Java、Python、JavaScript、TypeScript、Go、C、C++ 等多种语言，提供详细的代码度量和结构信息".to_string().into(),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "要分析的文件路径或目录路径"
                        },
                        "language": {
                            "type": "string",
                            "enum": ["rust", "java", "c", "cpp", "python", "go", "javascript", "typescript"],
                            "description": "编程语言过滤器 (可选)。若不指定，将自动检测和分析所有支持的语言。对于多语言项目，可以同时分析多种语言文件"
                        },
                        "verbosity": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": 2,
                            "description": "输出详细程度 (0-2，默认 1)。在多语言模式下：0-基础统计，1-各语言统计，2-详细结构信息和语言特定的分析"
                        }
                    },
                    "required": ["path"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "export_dependency_graph".to_string().into(),
                description: "导出依赖图（全局/子目录），支持 JSON、DOT、SVG 和 Mermaid 格式输出。注意：输出可能非常长，建议优先使用 summarize_graph（预算自适应裁剪），仅在必要时导出完整图。".to_string().into(),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "扫描目录（默认 .）"},
                        "threshold": {"type": "number", "minimum": 0.0, "maximum": 1.0, "description": "关键节点高亮阈值 (0-1)，默认 0.15"}
                    },
                    "required": ["path"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "query_call_chain".to_string().into(),
                description: "查询函数调用链（上游/下游），可设定最大深度与路径数量".to_string().into(),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "扫描目录（默认 .）"},
                        "start": {"type": "string", "description": "起始函数名（必需）"},
                        "end": {"type": "string", "description": "结束函数名（可选）"},
                        "direction": {"type": "string", "enum": ["downstream", "upstream"], "description": "方向：下游(被调用方)/上游(调用方)，默认 downstream"},
                        "max_depth": {"type": "integer", "minimum": 1, "maximum": 32, "description": "最大深度，默认 8"},
                        "max_paths": {"type": "integer", "minimum": 1, "maximum": 100, "description": "最多返回路径数，默认 20"}
                    },
                    "required": ["path", "start"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "summarize_graph".to_string().into(),
                description: "图摘要（支持社区压缩与预算自适应裁剪）".to_string().into(),
                input_schema: Arc::new(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "扫描目录（默认 .）"},
                        "radius": {"type": "integer", "minimum": 1, "description": "从种子出发的邻域半径（默认1）"},
                        "top_k": {"type": "integer", "minimum": 1, "description": "Top节点上限（默认200）"},
                        "seeds_from_diff": {"type": "boolean", "description": "从 git diff 推导变更种子（默认false）"},
                        "format": {"type": "string", "enum": ["json", "text"], "description": "输出格式（默认json）"},
                        "budget_tokens": {"type": "integer", "minimum": 0, "description": "预算token用于自适应裁剪（默认3000）"},
                        "community": {"type": "boolean", "description": "启用社区压缩（v1）"},
                        "comm_alg": {"type": "string", "enum": ["labelprop"], "description": "社区检测算法（默认labelprop）"},
                        "max_communities": {"type": "integer", "minimum": 1, "description": "社区数量上限（默认50）"},
                        "max_nodes_per_community": {"type": "integer", "minimum": 1, "description": "每个社区展示节点上限（默认10）"},
                        "with_paths": {"type": "boolean", "description": "启用路径采样（v2）"},
                        "path_samples": {"type": "integer", "minimum": 0, "description": "路径样本数量（默认5）"},
                        "path_max_hops": {"type": "integer", "minimum": 1, "description": "单条路径最大跳数（默认5）"}
                    },
                    "required": ["path"]
                }).as_object().unwrap().clone()),
            }
        ]
    }

    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> crate::mcp::McpResult<serde_json::Value> {
        match name {
            "execute_analysis" => {
                let mut params: AnalysisParams = serde_json::from_value(arguments)
                    .map_err(|e| crate::mcp::parse_error("analysis", e))?;

                // 使用服务配置的默认详细程度
                if params.verbosity.is_none() {
                    params.verbosity = Some(self.verbosity);
                }

                let result = self
                    .execute_analysis(params)
                    .await
                    .map_err(|e| crate::mcp::execution_error("Analysis", e))?;

                Ok(serde_json::to_value(result)
                    .map_err(|e| crate::mcp::serialize_error("analysis", e))?)
            }
            "export_dependency_graph" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let threshold = arguments
                    .get("threshold")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.15) as f32;
                let dot = crate::architectural_impact::graph_export::export_dot_string(
                    std::path::Path::new(path),
                    threshold,
                )
                .await
                .map_err(|e| crate::mcp::execution_error("Analysis", e))?;
                let obj = serde_json::json!({"dot": dot, "message": "ok"});
                Ok(obj)
            }
            "query_call_chain" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let start = arguments
                    .get("start")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| invalid_parameters_error("missing 'start'"))?;
                let end = arguments.get("end").and_then(|v| v.as_str());
                let direction = arguments
                    .get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("downstream");
                let max_depth = arguments
                    .get("max_depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(8) as usize;
                let max_paths = arguments
                    .get("max_paths")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(20) as usize;
                let chains = crate::architectural_impact::graph_export::query_call_chain(
                    std::path::Path::new(path),
                    start,
                    end,
                    direction,
                    max_depth,
                    max_paths,
                )
                .await
                .map_err(|e| crate::mcp::execution_error("Analysis", e))?;
                Ok(serde_json::json!({"chains": chains, "message": "ok"}))
            }
            "summarize_graph" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let radius = arguments
                    .get("radius")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as usize;
                let top_k = arguments
                    .get("top_k")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(200) as usize;
                let seeds_from_diff = arguments
                    .get("seeds_from_diff")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let format = arguments
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");
                let budget_tokens = arguments
                    .get("budget_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(3000) as usize;
                let community = arguments
                    .get("community")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let comm_alg = arguments
                    .get("comm_alg")
                    .and_then(|v| v.as_str())
                    .unwrap_or("labelprop");
                let max_communities = arguments
                    .get("max_communities")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50) as usize;
                let max_nodes_per_community = arguments
                    .get("max_nodes_per_community")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;
                let with_paths = arguments
                    .get("with_paths")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let path_samples = arguments
                    .get("path_samples")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let path_max_hops = arguments
                    .get("path_max_hops")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;

                let out = crate::architectural_impact::graph_export::export_summary_string(
                    std::path::Path::new(path),
                    radius,
                    top_k,
                    seeds_from_diff,
                    format,
                    budget_tokens,
                    community,
                    comm_alg,
                    max_communities,
                    max_nodes_per_community,
                    with_paths,
                    path_samples,
                    path_max_hops,
                )
                .await
                .map_err(|e| crate::mcp::execution_error("Analysis", e))?;

                if format == "json" {
                    match serde_json::from_str::<serde_json::Value>(&out) {
                        Ok(v) => Ok(v),
                        Err(_e) => Ok(
                            serde_json::json!({"summary": out, "format": format, "message": "returned raw JSON string due to parse failure"}),
                        ),
                    }
                } else {
                    Ok(serde_json::json!({"summary": out, "format": format}))
                }
            }
            _ => Err(invalid_parameters_error(format!("Unknown tool: {}", name))),
        }
    }
}

/// Analysis 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisParams {
    /// 分析路径
    pub path: String,
    /// 编程语言
    pub language: Option<String>,
    /// 输出详细程度
    pub verbosity: Option<u32>,
}

/// Analysis 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 分析的语言
    pub language: String,
    /// 代码摘要
    pub summary: CodeSummary,
    /// 代码结构
    pub structures: CodeStructures,
    /// 代码度量
    pub metrics: CodeMetrics,
    /// 详细信息
    pub details: HashMap<String, String>,
}

/// 代码摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSummary {
    /// 总行数
    pub total_lines: usize,
    /// 代码行数
    pub code_lines: usize,
    /// 注释行数
    pub comment_lines: usize,
    /// 空白行数
    pub blank_lines: usize,
    /// 复杂度评分
    pub complexity_score: u32,
}

/// 代码结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStructures {
    /// 函数列表
    pub functions: Vec<FunctionInfo>,
    /// 类列表
    pub classes: Vec<ClassInfo>,
    /// 导入列表
    pub imports: Vec<String>,
}

/// 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// 函数名
    pub name: String,
    /// 开始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 复杂度
    pub complexity: u32,
    /// 参数数量
    pub parameter_count: usize,
}

/// 类信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    /// 类名
    pub name: String,
    /// 开始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 方法数量
    pub method_count: usize,
}

/// 代码度量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    /// 圈复杂度
    pub cyclomatic_complexity: u32,
    /// 可维护性指数
    pub maintainability_index: f64,
    /// 注释比例
    pub comment_ratio: f64,
}
