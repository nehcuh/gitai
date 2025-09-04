// MCP Dependency 服务
//
// 提供依赖图生成和分析功能的 MCP 服务实现

use crate::{architectural_impact::dependency_graph::*, config::Config, mcp::*, tree_sitter};
use log::{debug, error, info, warn};
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Dependency 服务
pub struct DependencyService {
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    verbosity: u32,
}

/// 依赖图生成参数
#[derive(Debug, Deserialize)]
pub struct DependencyParams {
    /// 分析路径
    pub path: String,
    /// 生成格式 (json, dot, svg, mermaid)
    pub format: Option<String>,
    /// 输出文件路径（可选）
    pub output: Option<String>,
    /// 分析深度
    pub depth: Option<u32>,
    /// 是否包含函数调用关系
    pub include_calls: Option<bool>,
    /// 是否包含导入关系
    pub include_imports: Option<bool>,
    /// 详细程度 (0-3)
    pub verbosity: Option<u32>,
    /// 确认生成完整依赖图（大项目会非常大）。
    /// 未确认时将建议优先使用 summarize_graph。
    pub confirm: Option<bool>,
}

/// 图格式转换参数
#[derive(Debug, Deserialize)]
pub struct ConvertGraphParams {
    /// 输入格式（dot 或 mermaid）
    pub input_format: String,
    /// 输入内容
    pub input_content: String,
    /// 输出格式（png、svg、pdf）
    pub output_format: String,
    /// 输出文件路径
    pub output_path: String,
    /// Graphviz 布局引擎（可选）
    pub engine: Option<String>,
}

/// 图格式转换结果
#[derive(Debug, Serialize)]
pub struct ConvertGraphResult {
    /// 操作是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 输出文件路径
    pub output_path: String,
    /// 额外信息
    pub details: HashMap<String, String>,
}

/// 依赖图分析结果
#[derive(Debug, Serialize)]
pub struct DependencyResult {
    /// 操作是否成功
    pub success: bool,
    /// 结果消息
    pub message: String,
    /// 生成的格式
    pub format: String,
    /// 输出文件路径
    pub output_path: Option<String>,
    /// 依赖图统计信息
    pub statistics: GraphStatistics,
    /// 依赖图内容（JSON 格式时）
    pub graph_data: Option<DependencyGraph>,
    /// DOT 格式内容
    pub dot_content: Option<String>,
    /// Mermaid 格式内容
    pub mermaid_content: Option<String>,
    /// ASCII 文本内容
    pub ascii_content: Option<String>,
    /// 额外信息
    pub details: HashMap<String, String>,
}

impl DependencyService {
    /// 创建新的 Dependency 服务
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let verbosity = if let Some(mcp_config) = &config.mcp {
            if let Some(services_config) = mcp_config.services.dependency.as_ref() {
                services_config.verbosity
            } else {
                1
            }
        } else {
            1
        };

        Ok(Self { config, verbosity })
    }

    /// 执行依赖图生成
    async fn execute_dependency_graph(
        &self,
        params: DependencyParams,
    ) -> Result<DependencyResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔗 开始生成依赖图: {path}", path = params.path);
        debug!(
            "📋 分析参数: 格式={format:?}, 深度={depth:?}",
            format = params.format,
            depth = params.depth
        );

        // 如果用户指定了输出路径但未指定导出格式，提示用户选择格式
        if params.output.is_some() && params.format.is_none() {
            return Err("未指定导出格式。请在参数中设置 format: json|dot|svg|mermaid|ascii。建议：大项目先使用 summarize_graph 获取摘要。".into());
        }

        let path = Path::new(&params.path);

        // 验证路径是否存在
        if !path.exists() {
            error!("❌ 依赖图分析路径不存在: {path}", path = params.path);
            return Err(format!("分析路径不存在: {path}", path = params.path).into());
        }

        // 检查是否为目录
        if path.is_dir() {
            info!("📁 检测到目录路径，分析目录中的所有代码文件");
            return self.analyze_directory_dependencies(path, &params).await;
        }

        // 分析单个文件
        self.analyze_file_dependencies(path, &params).await
    }

    /// 分析单个文件的依赖关系
    async fn analyze_file_dependencies(
        &self,
        file_path: &Path,
        params: &DependencyParams,
    ) -> Result<DependencyResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!("📄 分析单个文件依赖: {path}", path = file_path.display());

        // 推断语言
        let language =
            Self::infer_language_from_path(file_path).map_err(|e| format!("无法推断语言: {e}"))?;

        // 读取文件内容
        let code_content = std::fs::read_to_string(file_path).map_err(|e| {
            error!(
                "❌ 无法读取文件 {path}: {e}",
                path = file_path.display(),
                e = e
            );
            format!(
                "无法读取文件 {path}: {e}",
                path = file_path.display(),
                e = e
            )
        })?;

        // 创建 Tree-sitter 管理器并分析
        let mut manager = tree_sitter::TreeSitterManager::new().await.map_err(|e| {
            error!("❌ 无法创建 Tree-sitter 管理器: {e}");
            format!("无法创建 Tree-sitter 管理器: {e}")
        })?;

        let summary = manager
            .analyze_structure(&code_content, language)
            .map_err(|e| {
                error!("❌ 结构分析失败: {e}");
                format!("结构分析失败: {e}")
            })?;

        // 从结构化摘要构建依赖图
        let file_path_str = file_path.to_string_lossy();
        let dependency_graph = DependencyGraph::from_structural_summary(&summary, &file_path_str);

        debug!(
            "📊 依赖图构建完成: 节点={}, 边={}",
            dependency_graph.nodes.len(),
            dependency_graph.edges.len()
        );

        // 生成输出
        self.generate_dependency_output(dependency_graph, params)
            .await
    }

    /// 分析目录中所有文件的依赖关系
    async fn analyze_directory_dependencies(
        &self,
        dir_path: &Path,
        params: &DependencyParams,
    ) -> Result<DependencyResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("📁 开始分析目录依赖关系: {}", dir_path.display());

        // 查找所有代码文件
        let code_files = self.find_code_files(dir_path)?;

        // 对超大项目做保护，提示优先使用 summarize_graph
        const LARGE_FILE_THRESHOLD: usize = 1500;
        let is_large = code_files.len() > LARGE_FILE_THRESHOLD;
        let confirmed = params.confirm.unwrap_or(false);
        if is_large && !confirmed {
            warn!(
                "⚠️ 检测到大型项目 (files={})，建议优先使用 summarize_graph",
                code_files.len()
            );
            return Err("大型项目依赖图可能非常庞大。建议先使用 summarize_graph 获取摘要；若确需导出完整图，请在调用时传入 confirm=true".into());
        }

        if code_files.is_empty() {
            warn!("⚠️ 目录中未找到可分析的代码文件");
            return Ok(DependencyResult {
                success: false,
                message: "目录中未找到可分析的代码文件".to_string(),
                format: params.format.clone().unwrap_or_else(|| "json".to_string()),
                output_path: None,
                statistics: GraphStatistics {
                    node_count: 0,
                    edge_count: 0,
                    avg_degree: 0.0,
                    cycles_count: 0,
                    critical_nodes_count: 0,
                },
                graph_data: None,
                dot_content: None,
                mermaid_content: None,
                ascii_content: None,
                details: {
                    let mut details = HashMap::new();
                    details.insert("directory_path".to_string(), dir_path.display().to_string());
                    details.insert("message".to_string(), "未找到支持的代码文件".to_string());
                    details
                },
            });
        }

        info!("📋 找到 {} 个代码文件，开始分析", code_files.len());

        // 创建合并的依赖图
        let mut merged_graph = DependencyGraph::new();

        // 分析每个文件并合并依赖图
        for file_path in &code_files {
            debug!("🔍 分析文件依赖: {}", file_path.display());

            match self.analyze_single_file_for_merge(file_path).await {
                Ok(file_graph) => {
                    self.merge_dependency_graph(&mut merged_graph, file_graph);
                }
                Err(e) => {
                    warn!(
                        "⚠️ 分析文件 {path} 失败: {e}",
                        path = file_path.display(),
                        e = e
                    );
                }
            }
        }

        debug!(
            "📊 合并依赖图完成: 节点={}, 边={}",
            merged_graph.nodes.len(),
            merged_graph.edges.len()
        );

        // 生成输出
        self.generate_dependency_output(merged_graph, params).await
    }

    /// 分析单个文件用于合并（内部方法）
    async fn analyze_single_file_for_merge(
        &self,
        file_path: &Path,
    ) -> Result<DependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
        let language = Self::infer_language_from_path(file_path)
            .map_err(|e| format!("无法推断语言: {}", e))?;

        let code_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件 {}: {}", file_path.display(), e))?;

        let mut manager = tree_sitter::TreeSitterManager::new()
            .await
            .map_err(|e| format!("无法创建 Tree-sitter 管理器: {}", e))?;

        let summary = manager
            .analyze_structure(&code_content, language)
            .map_err(|e| format!("结构分析失败: {}", e))?;

        let file_path_str = file_path.to_string_lossy();
        Ok(DependencyGraph::from_structural_summary(
            &summary,
            &file_path_str,
        ))
    }

    /// 合并依赖图
    fn merge_dependency_graph(&self, target: &mut DependencyGraph, source: DependencyGraph) {
        // 合并节点
        for (id, node) in source.nodes {
            target.nodes.insert(id, node);
        }

        // 合并边
        target.edges.extend(source.edges);

        // 重建邻接列表
        target.rebuild_adjacency_lists();
    }

    /// 生成依赖图输出
    async fn generate_dependency_output(
        &self,
        graph: DependencyGraph,
        params: &DependencyParams,
    ) -> Result<DependencyResult, Box<dyn std::error::Error + Send + Sync>> {
        // 从配置中获取默认格式
        let default_format = if let Some(mcp_config) = &self.config.mcp {
            if let Some(dependency_config) = mcp_config.services.dependency.as_ref() {
                dependency_config.default_format.clone()
            } else {
                "json".to_string()
            }
        } else {
            "json".to_string()
        };

        let format = params.format.clone().unwrap_or(default_format);
        let statistics = graph.get_statistics();

        match format.to_lowercase().as_str() {
            "json" => {
                info!("📄 生成 JSON 格式依赖图");
                Ok(DependencyResult {
                    success: true,
                    message: "依赖图生成成功".to_string(),
                    format: "json".to_string(),
                    output_path: params.output.clone(),
                    statistics,
                    graph_data: Some(graph),
                    dot_content: None,
                    mermaid_content: None,
                    ascii_content: None,
                    details: HashMap::new(),
                })
            }
            "dot" => {
                info!("📄 生成 DOT 格式依赖图");
                let dot_options = DotOptions::default();
                let dot_content = graph.to_dot(Some(&dot_options));

                // 如果指定了输出文件，写入文件
                if let Some(output_path) = &params.output {
                    std::fs::write(output_path, &dot_content)
                        .map_err(|e| format!("无法写入 DOT 文件: {e}"))?;
                    info!("📁 DOT 文件已保存到: {output_path}");
                }

                Ok(DependencyResult {
                    success: true,
                    message: "DOT 格式依赖图生成成功".to_string(),
                    format: "dot".to_string(),
                    output_path: params.output.clone(),
                    statistics,
                    graph_data: None,
                    dot_content: Some(dot_content),
                    mermaid_content: None,
                    ascii_content: None,
                    details: HashMap::new(),
                })
            }
            "svg" => {
                info!("📄 生成 SVG 格式依赖图");
                // 先生成 DOT 内容
                let dot_options = DotOptions::default();
                let dot_content = graph.to_dot(Some(&dot_options));

                if let Some(out_path) = &params.output {
                    // 如果提供了输出路径，尝试直接转换为 SVG（通过 stdin 传给 Graphviz）
                    match self
                        .convert_graph_to_image(ConvertGraphParams {
                            input_format: "dot".to_string(),
                            input_content: dot_content.clone(),
                            output_format: "svg".to_string(),
                            output_path: out_path.clone(),
                            engine: None,
                        })
                        .await
                    {
                        Ok(conv) => {
                            return Ok(DependencyResult {
                                success: true,
                                message: format!(
                                    "依赖图已导出为 SVG: {}",
                                    conv.output_path
                                ),
                                format: "svg".to_string(),
                                output_path: Some(conv.output_path),
                                statistics,
                                graph_data: None,
                                dot_content: None,
                                mermaid_content: None,
                                ascii_content: None,
                                details: conv.details,
                            });
                        }
                        Err(e) => {
                            warn!("⚠️ SVG 转换失败，将返回 DOT 内容: {}", e);
                        }
                    }
                }

                // 未提供输出路径或转换失败：返回 DOT 内容与转换建议
                warn!("⚠️ SVG 转换需要 Graphviz。已返回 DOT 内容，可使用 convert_graph_to_image 工具或本地 dot 命令进行转换");
                Ok(DependencyResult {
                    success: true,
                    message: "已生成 DOT 内容。请使用 Graphviz 将其转换为 SVG（例如：dot -Tsvg -o out.svg）".to_string(),
                    format: "dot".to_string(),
                    output_path: None,
                    statistics,
                    graph_data: None,
                    dot_content: Some(dot_content),
                    mermaid_content: None,
                    ascii_content: None,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("note".to_string(), "需要 Graphviz 将 DOT 转换为 SVG".to_string());
                        details.insert("hint".to_string(), "可改用 summarize_graph 获取摘要，或调用 convert_graph_to_image 进行转换".to_string());
                        details
                    },
                })
            }
            "mermaid" => {
                info!("📄 生成 Mermaid 格式依赖图");
                let mermaid_content = Self::convert_to_mermaid(&graph);

                // 如果指定了输出文件，写入文件
                if let Some(output_path) = &params.output {
                    std::fs::write(output_path, &mermaid_content)
                        .map_err(|e| format!("无法写入 Mermaid 文件: {e}"))?;
                    info!("📁 Mermaid 文件已保存到: {output_path}");
                }

                Ok(DependencyResult {
                    success: true,
                    message: "Mermaid 格式依赖图生成成功".to_string(),
                    format: "mermaid".to_string(),
                    output_path: params.output.clone(),
                    statistics,
                    graph_data: None,
                    dot_content: None,
                    mermaid_content: Some(mermaid_content),
                    ascii_content: None,
                    details: HashMap::new(),
                })
            }
            "ascii" => {
                info!("📄 生成 ASCII 文本依赖图");
                let verbosity = params.verbosity.unwrap_or(self.verbosity);
                let ascii_content = Self::convert_to_ascii(&graph, verbosity);

                // 如果指定了输出文件，写入文件
                if let Some(output_path) = &params.output {
                    std::fs::write(output_path, &ascii_content)
                        .map_err(|e| format!("无法写入 ASCII 文件: {e}"))?;
                    info!("📁 ASCII 文件已保存到: {output_path}");
                }

                Ok(DependencyResult {
                    success: true,
                    message: "ASCII 文本依赖图生成成功".to_string(),
                    format: "ascii".to_string(),
                    output_path: params.output.clone(),
                    statistics,
                    graph_data: None,
                    dot_content: None,
                    mermaid_content: None,
                    ascii_content: Some(ascii_content),
                    details: HashMap::new(),
                })
            }
            _ => {
                error!("❌ 不支持的格式: {format}");
                Err(format!("不支持的格式: {format}").into())
            }
        }
    }

    /// 查找代码文件
    #[allow(clippy::only_used_in_recursion)]
    fn find_code_files(
        &self,
        dir_path: &Path,
    ) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let mut code_files = Vec::new();

        // 获取是否排除测试代码的配置
        let exclude_test_code = if let Some(mcp_config) = &self.config.mcp {
            if let Some(dependency_config) = mcp_config.services.dependency.as_ref() {
                dependency_config.exclude_test_code
            } else {
                true // 默认排除测试代码
            }
        } else {
            true // 默认排除测试代码
        };

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // 检查是否为测试文件
                if exclude_test_code && Self::is_test_file(&path) {
                    debug!("🚫 跳过测试文件: {}", path.display());
                    continue;
                }

                if let Some(extension) = path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if Self::is_supported_code_file(ext_str) {
                            code_files.push(path);
                        }
                    }
                }
            } else if path.is_dir() {
                // 检查是否为测试目录
                if exclude_test_code && Self::is_test_directory(&path) {
                    debug!("🚫 跳过测试目录: {}", path.display());
                    continue;
                }

                // 递归搜索子目录
                let sub_files = self.find_code_files(&path)?;
                code_files.extend(sub_files);
            }
        }

        Ok(code_files)
    }

    /// 检查是否为支持的代码文件
    fn is_supported_code_file(extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "rs" | "java" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp"
        )
    }

    /// 检查是否为测试文件
    fn is_test_file(path: &Path) -> bool {
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                let name_lower = name_str.to_lowercase();
                // 检查常见的测试文件命名模式
                return name_lower.ends_with("_test.rs")
                    || name_lower.ends_with("_tests.rs")
                    || name_lower.starts_with("test_")
                    || name_lower.ends_with("_test.go")
                    || name_lower.ends_with("_test.py")
                    || name_lower.ends_with("_test.js")
                    || name_lower.ends_with("_test.ts")
                    || name_lower.ends_with(".test.js")
                    || name_lower.ends_with(".test.ts")
                    || name_lower.ends_with(".spec.js")
                    || name_lower.ends_with(".spec.ts")
                    || name_lower.ends_with("_test.java")
                    || name_lower == "test.rs"
                    || name_lower == "tests.rs";
            }
        }

        // 检查路径中是否包含 tests 目录
        if let Some(path_str) = path.to_str() {
            return path_str.contains("/tests/")
                || path_str.contains("/test/")
                || path_str.contains("/__tests__/")
                || path_str.contains("/test_")
                || path_str.contains("/tests_");
        }

        false
    }

    /// 检查是否为测试目录
    fn is_test_directory(path: &Path) -> bool {
        if let Some(dir_name) = path.file_name() {
            if let Some(name_str) = dir_name.to_str() {
                let name_lower = name_str.to_lowercase();
                // 检查常见的测试目录名称
                return name_lower == "tests"
                    || name_lower == "test"
                    || name_lower == "__tests__"
                    || name_lower == "test_data"
                    || name_lower == "testdata"
                    || name_lower == "testing"
                    || name_lower.starts_with("test_")
                    || name_lower.starts_with("tests_")
                    || name_lower.ends_with("_test")
                    || name_lower.ends_with("_tests");
            }
        }
        false
    }

    /// 从文件路径推断语言
    fn infer_language_from_path(path: &Path) -> Result<tree_sitter::SupportedLanguage, String> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| "无法获取文件扩展名".to_string())?;

        match extension.to_lowercase().as_str() {
            "rs" => Ok(tree_sitter::SupportedLanguage::Rust),
            "java" => Ok(tree_sitter::SupportedLanguage::Java),
            "py" => Ok(tree_sitter::SupportedLanguage::Python),
            "js" => Ok(tree_sitter::SupportedLanguage::JavaScript),
            "ts" => Ok(tree_sitter::SupportedLanguage::TypeScript),
            "go" => Ok(tree_sitter::SupportedLanguage::Go),
            "c" | "h" => Ok(tree_sitter::SupportedLanguage::C),
            "cpp" | "hpp" => Ok(tree_sitter::SupportedLanguage::Cpp),
            _ => Err(format!("不支持的文件扩展名: {extension}")),
        }
    }

    /// 将依赖图转换为 Mermaid 格式
    fn convert_to_mermaid(graph: &DependencyGraph) -> String {
        let mut mermaid = String::new();

        // Mermaid 文档头，使用 flowchart 语法
        mermaid.push_str("flowchart TD\n");
        mermaid.push_str("    %% Generated by GitAI Dependency Service\n");
        mermaid.push('\n');

        // 为不同类型的节点定义样式
        let mut node_id_map = HashMap::new();

        // 首先生成所有节点的定义
        for (node_counter, (node_id, node)) in graph.nodes.iter().enumerate() {
            let safe_id = format!("node{node_counter}");
            node_id_map.insert(node_id.clone(), safe_id.clone());

            let label = Self::get_node_display_name(&node.id);
            let shape_and_style = Self::get_mermaid_node_style(&node.node_type);
            let replaced = shape_and_style.replace("{label}", &label);

            mermaid.push_str(&format!("    {safe_id}{replaced}\n"));
        }

        mermaid.push('\n');

        // 然后生成所有边的定义
        for edge in &graph.edges {
            if let (Some(from_id), Some(to_id)) =
                (node_id_map.get(&edge.from), node_id_map.get(&edge.to))
            {
                let arrow_style = Self::get_mermaid_edge_style(&edge.edge_type);
                let edge_label = if let Some(metadata) = &edge.metadata {
                    if let Some(notes) = &metadata.notes {
                        if !notes.is_empty() {
                            format!("|{notes}|")
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                mermaid.push_str(&format!("    {from_id}{arrow_style}{edge_label} {to_id}\n"));
            }
        }

        // 添加样式定义
        mermaid.push('\n');
        mermaid.push_str("    %% Styles\n");
        mermaid.push_str("    classDef fileNode fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        mermaid
            .push_str("    classDef functionNode fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
        mermaid.push_str("    classDef classNode fill:#fff3e0,stroke:#e65100,stroke-width:2px\n");
        mermaid.push_str("    classDef moduleNode fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px\n");

        // 应用样式到节点
        for (node_id, node) in &graph.nodes {
            if let Some(safe_id) = node_id_map.get(node_id) {
                let class_name = match node.node_type {
                    NodeType::File(_) => "fileNode",
                    NodeType::Function(_) => "functionNode",
                    NodeType::Class(_) => "classNode",
                    NodeType::Module(_) => "moduleNode",
                };
                mermaid.push_str(&format!("    class {safe_id} {class_name}\n"));
            }
        }

        mermaid
    }

    /// 将依赖图转换为 ASCII 文本
    fn convert_to_ascii(graph: &DependencyGraph, verbosity: u32) -> String {
        // 为节点分配短 ID，保证稳定性（按原 ID 排序）
        let mut node_ids: Vec<&String> = graph.nodes.keys().collect();
        node_ids.sort();
        let mut id_map: HashMap<String, String> = HashMap::new();
        for (i, id) in node_ids.iter().enumerate() {
            id_map.insert((**id).clone(), format!("N{}", i + 1));
        }

        let mut out = String::new();
        out.push_str("# Dependency Graph (ASCII)\n");
        out.push_str(&format!(
            "nodes: {}, edges: {}\n",
            graph.nodes.len(),
            graph.edges.len()
        ));
        let stats = graph.get_statistics();
        out.push_str(&format!(
            "avg_degree: {:.2}, cycles: {}, critical: {}\n\n",
            stats.avg_degree, stats.cycles_count, stats.critical_nodes_count
        ));

        // 打印节点映射
        out.push_str("[Nodes]\n");
        let mut nodes_sorted: Vec<(
            &String,
            &crate::architectural_impact::dependency_graph::Node,
        )> = graph.nodes.iter().collect();
        nodes_sorted.sort_by(|a, b| a.0.cmp(b.0));
        for (id, node) in nodes_sorted {
            let short = id_map.get(id).cloned().unwrap_or_else(|| id.clone());
            let label = match &node.node_type {
                NodeType::Function(f) => format!("fn {}()", f.name),
                NodeType::Class(c) => format!("class {}", c.name),
                NodeType::Module(m) => format!("mod {}", m.name),
                NodeType::File(f) => format!("file {}", f.path),
            };
            if verbosity >= 2 {
                out.push_str(&format!(
                    "  {short}: {label}  [loc={}:{}..{}, score={:.2}]\n",
                    node.metadata.file_path,
                    node.metadata.start_line,
                    node.metadata.end_line,
                    node.importance_score
                ));
            } else {
                out.push_str(&format!("  {short}: {label}\n"));
            }
        }
        out.push_str("\n[Edges]\n");
        // 排序边，保证稳定性
        let mut edges_sorted = graph.edges.clone();
        edges_sorted.sort_by(|a, b| {
            let c = a.from.cmp(&b.from);
            if c == std::cmp::Ordering::Equal {
                a.to.cmp(&b.to)
            } else {
                c
            }
        });
        for e in edges_sorted {
            let from_s = id_map
                .get(&e.from)
                .cloned()
                .unwrap_or_else(|| e.from.clone());
            let to_s = id_map.get(&e.to).cloned().unwrap_or_else(|| e.to.clone());
            let etype = match e.edge_type {
                EdgeType::Calls => "CALLS",
                EdgeType::Imports => "IMPORTS",
                EdgeType::Exports => "EXPORTS",
                EdgeType::Inherits => "INHERITS",
                EdgeType::Implements => "IMPLEMENTS",
                EdgeType::Uses => "USES",
                EdgeType::References => "REFS",
                EdgeType::Contains => "CONTAINS",
                EdgeType::DependsOn => "DEPENDS",
            };
            if verbosity >= 2 {
                let mut meta = String::new();
                if let Some(m) = &e.metadata {
                    if let Some(ref notes) = m.notes {
                        if !notes.is_empty() {
                            meta.push_str(&format!(" notes={}", notes));
                        }
                    }
                    if let Some(cc) = m.call_count {
                        meta.push_str(&format!(" calls={}", cc));
                    }
                    if m.is_strong_dependency {
                        meta.push_str(" strong");
                    }
                }
                out.push_str(&format!(
                    "  {from} -[{etype} w={:.2}]{meta}-> {to}\n",
                    e.weight,
                    from = from_s,
                    to = to_s
                ));
            } else {
                out.push_str(&format!(
                    "  {from} -[{etype}]-> {to}\n",
                    from = from_s,
                    to = to_s
                ));
            }
        }

        out
    }

    /// 获取节点的显示名称
    fn get_node_display_name(node_id: &str) -> String {
        // 从节点 ID 中提取有意义的名称
        if let Some(last_part) = node_id.split("::").last() {
            if let Some(name_part) = last_part.split('/').next_back() {
                return name_part.to_string();
            }
        }

        // 如果无法解析，就返回简化的版本
        if node_id.len() > 20 {
            format!("{}...", &node_id[..17])
        } else {
            node_id.to_string()
        }
    }

    /// 获取 Mermaid 节点样式
    fn get_mermaid_node_style(node_type: &NodeType) -> String {
        match node_type {
            NodeType::File(_) => "[{label}]".to_string(), // 矩形表示文件
            NodeType::Function(_) => "({label})".to_string(), // 圆形表示函数
            NodeType::Class(_) => "{{{label}}}".to_string(), // 菱形表示类
            NodeType::Module(_) => "[/{label}/]".to_string(), // 平行四边形表示模块
        }
    }

    /// 获取 Mermaid 边样式
    fn get_mermaid_edge_style(edge_type: &EdgeType) -> String {
        match edge_type {
            EdgeType::Calls => "-.->".to_string(),      // 虚线箭头表示调用
            EdgeType::Imports => "-->".to_string(),     // 实线箭头表示导入
            EdgeType::Exports => "-->".to_string(),     // 实线箭头表示导出
            EdgeType::Inherits => "==>".to_string(),    // 粗实线箭头表示继承
            EdgeType::Implements => "==>".to_string(),  // 粗实线箭头表示实现
            EdgeType::Uses => "-.->".to_string(),       // 虚线箭头表示使用
            EdgeType::References => "-.->".to_string(), // 虚线箭头表示引用
            EdgeType::Contains => "-->".to_string(),    // 实线箭头表示包含
            EdgeType::DependsOn => "-->".to_string(),   // 实线箭头表示依赖
        }
    }

    /// 将图格式转换为图像
    async fn convert_graph_to_image(
        &self,
        params: ConvertGraphParams,
    ) -> Result<ConvertGraphResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "🎨 开始转换图格式: {} -> {}",
            params.input_format, params.output_format
        );

        // 验证输入格式
        let input_format = params.input_format.to_lowercase();
        if input_format != "dot" && input_format != "mermaid" {
            return Err(format!("不支持的输入格式: {}", params.input_format).into());
        }

        // 验证输出格式
        let output_format = params.output_format.to_lowercase();
        if !matches!(output_format.as_str(), "png" | "svg" | "pdf") {
            return Err(format!("不支持的输出格式: {}", params.output_format).into());
        }

        // 如果输入是 Mermaid，先转换为 DOT
        let dot_content = if input_format == "mermaid" {
            info!("🔄 将 Mermaid 转换为 DOT 格式");
            // Mermaid 转 DOT 需要特殊处理，目前不支持
            return Err("目前不支持 Mermaid 转换，请使用 DOT 格式输入".into());
        } else {
            params.input_content.clone()
        };

        // 使用 Graphviz 转换（通过 stdin，避免在只读环境创建临时文件）
        let engine = params.engine.unwrap_or_else(|| "dot".to_string());

        use std::io::Write as _;
        let mut child = std::process::Command::new(&engine)
            .arg("-T")
            .arg(&output_format)
            .arg("-o")
            .arg(&params.output_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("无法执行 Graphviz 命令 '{}': {}\n请确保 Graphviz 已安装并在 PATH 中", engine, e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(dot_content.as_bytes())
                .map_err(|e| format!("向 Graphviz 写入 DOT 内容失败: {}", e))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("等待 Graphviz 进程结束失败: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Graphviz 转换失败: {}\n命令: {} -T{} -o {} (stdin)",
                stderr,
                engine,
                output_format,
                params.output_path
            )
            .into());
        }

        // 检查输出文件是否存在
        if !std::path::Path::new(&params.output_path).exists() {
            return Err(format!("输出文件未生成: {}", params.output_path).into());
        }

        let file_size = std::fs::metadata(&params.output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        info!(
            "✅ 图像生成成功: {} ({} bytes)",
            params.output_path, file_size
        );

        Ok(ConvertGraphResult {
            success: true,
            message: format!(
                "成功将 {} 转换为 {} 格式",
                params.input_format, params.output_format
            ),
            output_path: params.output_path,
            details: {
                let mut details = HashMap::new();
                details.insert("engine".to_string(), engine);
                details.insert("file_size".to_string(), file_size.to_string());
                details.insert("format".to_string(), output_format);
                details
            },
        })
    }
}

#[async_trait::async_trait]
impl GitAiMcpService for DependencyService {
    fn name(&self) -> &str {
        "dependency"
    }

    fn description(&self) -> &str {
        "生成和分析代码依赖图，支持多种编程语言和输出格式"
    }

    fn tools(&self) -> Vec<Tool> {
        let schema = std::sync::Arc::new(
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "要分析的文件或目录路径"},
                    "format": {"type": "string", "enum": ["json", "dot", "svg", "mermaid", "ascii"], "description": "输出格式（默认 ascii）"},
                    "output": {"type": "string", "description": "输出文件路径（可选）"},
                    "depth": {"type": "integer", "description": "分析深度（可选，默认为无限制）"},
                    "include_calls": {"type": "boolean", "description": "是否包含函数调用关系（默认为 true）"},
                    "include_imports": {"type": "boolean", "description": "是否包含导入关系（默认为 true）"},
                    "verbosity": {"type": "integer", "minimum": 0, "maximum": 3, "description": "详细程度，0-3（可选，默认为 1）"},
                    "confirm": {"type": "boolean", "description": "大型项目下确认生成完整依赖图（建议先使用 summarize_graph）"}
                },
                "required": ["path"]
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        let convert_schema = Arc::new(
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input_format": {
                        "type": "string",
                        "enum": ["dot", "mermaid"],
                        "description": "输入格式（dot 或 mermaid）"
                    },
                    "input_content": {
                        "type": "string",
                        "description": "输入的图内容（DOT 或 Mermaid 格式）"
                    },
                    "output_format": {
                        "type": "string",
                        "enum": ["png", "svg", "pdf"],
                        "description": "输出格式（png、svg 或 pdf）"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "输出文件路径"
                    },
                    "engine": {
                        "type": "string",
                        "enum": ["dot", "neato", "circo", "fdp", "sfdp", "twopi"],
                        "description": "Graphviz 布局引擎（默认 dot）"
                    }
                },
                "required": ["input_format", "input_content", "output_format", "output_path"]
            })
            .as_object()
            .unwrap()
            .clone(),
        );

        vec![
            Tool {
                name: "execute_dependency_graph".to_string().into(),
                description: "生成代码依赖图（默认 ASCII）。注意：在大型项目上输出可能非常庞大，建议优先使用 summarize_graph，仅在必要时并经确认后再导出完整图。"
                    .to_string()
                    .into(),
                input_schema: schema.clone(),
            },
            Tool {
                name: "convert_graph_to_image".to_string().into(),
                description: "将 DOT 或 Mermaid 格式的图转换为图像文件（PNG、SVG、PDF）"
                    .to_string()
                    .into(),
                input_schema: convert_schema,
            },
        ]
    }

    async fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value> {
        debug!("🔧 Dependency 服务处理工具调用: {}", tool_name);

        match tool_name {
            "execute_dependency_graph" => {
                let params: DependencyParams =
                    serde_json::from_value(arguments).map_err(|e| parse_error("dependency", e))?;

                let result = self
                    .execute_dependency_graph(params)
                    .await
                    .map_err(|e| execution_error("dependency", e))?;

                serde_json::to_value(&result).map_err(|e| serialize_error("dependency", e))
            }
            "convert_graph_to_image" => {
                let params: ConvertGraphParams = serde_json::from_value(arguments)
                    .map_err(|e| parse_error("convert_graph", e))?;

                let result = self
                    .convert_graph_to_image(params)
                    .await
                    .map_err(|e| execution_error("convert_graph", e))?;

                serde_json::to_value(&result).map_err(|e| serialize_error("convert_graph", e))
            }
            _ => Err(invalid_parameters_error(format!(
                "Unknown tool: {}",
                tool_name
            ))),
        }
    }
}
