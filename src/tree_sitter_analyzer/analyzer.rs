use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::config::TreeSitterConfig;
use crate::errors::TreeSitterError;
use crate::types::git::{ChangeType, DiffHunk};

use super::core::LanguageRegistry;
use super::core::{
    AffectedNode, ChangeAnalysis, ChangePattern, ChangeScope, ChangeStats, DiffAnalysis,
    FileAnalysis, FileAst, create_language_registry, get_node_analysis_config, is_node_public,
    parse_git_diff, load_query_file,
};
use super::utils::calculate_hash;
use super::query_manager::{QueryManager, QueryManagerConfig};



pub struct TreeSitterAnalyzer {
    pub config: TreeSitterConfig,
    pub project_root: PathBuf,
    language_registry: LanguageRegistry,
    file_asts: HashMap<PathBuf, FileAst>,
    structure_queries: HashMap<String, Query>,
    query_manager: QueryManager,
}

// 节点分析配置
#[derive(Debug, Clone)]
pub struct NodeAnalysisConfig {
    pub language: &'static str,
    pub capture_names: &'static [&'static str],
    pub important_nodes: &'static [&'static str],
    pub visibility_indicators: &'static [&'static str],
    pub scope_indicators: &'static [&'static str],
}

// 统一的节点增强器
struct UnifiedNodeEnhancer {
    analysis_config: NodeAnalysisConfig,
}

impl UnifiedNodeEnhancer {
    fn new(language: &str) -> Option<Self> {
        get_node_analysis_config(language).map(|config| Self {
            analysis_config: config,
        })
    }

    fn enhance_node(&self, node: &mut AffectedNode) {
        let content = node
            .content
            .as_ref()
            .map(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // 检查可见性
        match self.analysis_config.language {
            "python" => {
                // Python：双下划线开头视为私有，其余视为公开
                node.is_public = !node.name.starts_with("__");
            }
            "go" => {
                // Go：首字母大写视为公开，首字母小写视为私有
                node.is_public = node
                    .name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
            }
            _ => {
                for &indicator in self.analysis_config.visibility_indicators {
                    if content.contains(indicator) {
                        node.is_public = match self.analysis_config.language {
                            "rust" => indicator.starts_with("pub"),
                            "java" | "cpp" => indicator == "public",
                            "js" => indicator == "export",
                            "c" => indicator == "extern",
                            _ => false,
                        };
                        break;
                    }
                }
            }
        }

        // 语言特定的节点类型优化
        // 它根据节点的原始类型（如 "struct_item"、"function_item"、"class_declaration" 等）和节点的内容（content），结合特定语言的语法和特征，对节点类型进行更细粒度的标记。
        // 以 Rust 为例: 如果节点类型是 "function_item"，但内容里包含了 #[test]，那么就会把这个节点优化为 "test_function"；
        self.optimize_node_type(node, &content);
    }

    fn optimize_node_type(&self, node: &mut AffectedNode, content: &str) {
        match self.analysis_config.language {
            "rust" => self.optimize_rust_node(node, content),
            "java" => self.optimize_java_node(node, content),
            "python" => self.optimize_python_node(node, content),
            "go" => self.optimize_go_node(node, content),
            "js" => self.optimize_js_node(node, content),
            "c" => self.optimize_c_node(node, content),
            "cpp" => self.optimize_cpp_node(node, content),
            _ => {}
        }
    }

    fn optimize_rust_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "struct_item" => {
                if content.contains("#[derive(Debug") {
                    "debuggable_struct".to_string()
                } else {
                    "struct_definition".to_string()
                }
            }
            "enum_item" => {
                if content.contains("#[derive(Debug") {
                    "debuggable_enum".to_string()
                } else {
                    "enum_definition".to_string()
                }
            }
            "function_item" => {
                if content.contains("#[test]") {
                    "test_function".to_string()
                } else if content.contains("async fn") {
                    "async_function".to_string()
                } else if content.contains("unsafe fn") {
                    "unsafe_function".to_string()
                } else if content.contains("pub fn") {
                    "public_function".to_string()
                } else {
                    "function_definition".to_string()
                }
            }
            "impl_item" => {
                if content.contains(" for ") {
                    "trait_impl".to_string()
                } else {
                    "inherent_impl".to_string()
                }
            }
            "trait_item" => "trait_definition".to_string(),
            _ => node.node_type.clone(),
        };
    }

    fn optimize_java_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "class_declaration" => {
                if content.contains("@Service")
                    || content.contains("@Component")
                    || content.contains("@Controller")
                    || content.contains("@Repository")
                {
                    "spring_component".to_string()
                } else if content.contains("@Entity") || content.contains("@Table") {
                    "jpa_entity".to_string()
                } else {
                    "class_structure".to_string()
                }
            }
            "method_declaration" => {
                if content.contains("@Override") {
                    "overridden_method".to_string()
                } else if content.contains("@GetMapping") || content.contains("@PostMapping") {
                    "api_endpoint".to_string()
                } else if content.contains("@Test") {
                    "test_method".to_string()
                } else {
                    "method".to_string()
                }
            }
            "field_declaration" => {
                if content.contains("@Autowired") || content.contains("@Inject") {
                    "injected_field".to_string()
                } else {
                    "field".to_string()
                }
            }
            _ => node.node_type.clone(),
        };
    }

    fn optimize_python_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "function_definition" => {
                if content.contains("async def") {
                    "async_function".to_string()
                } else if content.contains("def test_") {
                    "test_function".to_string()
                } else {
                    "function_definition".to_string()
                }
            }
            "class_definition" => {
                if content.contains("(models.Model)") {
                    "django_model".to_string()
                } else {
                    "class_definition".to_string()
                }
            }
            _ => node.node_type.clone(),
        };
    }

    fn optimize_go_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "function_declaration" => {
                if content.contains("func (") {
                    "method_definition".to_string()
                } else {
                    "function_definition".to_string()
                }
            }
            _ => node.node_type.clone(),
        };
    }

    fn optimize_js_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "function_declaration" => {
                if content.contains("async ") {
                    "async_function".to_string()
                } else {
                    "function_definition".to_string()
                }
            }
            "arrow_function" => "arrow_function".to_string(),
            _ => node.node_type.clone(),
        };
    }

    fn optimize_c_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "struct_specifier" => {
                if content.contains("typedef struct") {
                    "typedef_struct".to_string()
                } else {
                    "struct_definition".to_string()
                }
            }
            "function_definition" => {
                let mut tags = vec!["function".to_string()];
                if content.contains("static") {
                    tags.push("static".to_string());
                }
                if content.contains("inline") {
                    tags.push("inline".to_string());
                }
                if content.contains("main(") {
                    tags.push("main_function".to_string());
                }
                tags.join("|")
            }
            _ => node.node_type.clone(),
        };
    }

    fn optimize_cpp_node(&self, node: &mut AffectedNode, content: &str) {
        node.node_type = match node.node_type.as_str() {
            "class_specifier" => {
                if content.contains("template") {
                    "template_class".to_string()
                } else {
                    "class_definition".to_string()
                }
            }
            "function_definition" => {
                if content.contains("virtual") {
                    "virtual_function".to_string()
                } else if content.contains("template") {
                    "template_function".to_string()
                } else {
                    "function_definition".to_string()
                }
            }
            _ => node.node_type.clone(),
        };
    }
}

// 通用的摘要生成器
struct SummaryGenerator {
    language_name: String,
    node_counts: HashMap<String, usize>,
    additions: usize,
    deletions: usize,
    modifications: usize,
    public_items: usize,
}

impl SummaryGenerator {
    fn new(language_name: String) -> Self {
        Self {
            language_name,
            node_counts: HashMap::new(),
            additions: 0,
            deletions: 0,
            modifications: 0,
            public_items: 0,
        }
    }

    fn add_node(&mut self, node: &AffectedNode) {
        let node_type = node.node_type.split('|').next().unwrap_or("unknown");
        *self.node_counts.entry(node_type.to_string()).or_insert(0) += 1;

        if let Some(change_type) = &node.change_type {
            match change_type.as_str() {
                "added" | "added_content" => self.additions += 1,
                "deleted" => self.deletions += 1,
                "modified" | "modified_with_deletion" => self.modifications += 1,
                _ => {}
            }
        }

        if node.is_public {
            self.public_items += 1;
        }
    }

    fn generate_summary(
        &self,
        file_path: &std::path::Path,
        affected_nodes: &[AffectedNode],
    ) -> String {
        let mut summary = format!(
            "{}文件 {} 变更分析：",
            self.language_name,
            file_path.display()
        );

        if affected_nodes.is_empty() {
            return format!("{}未检测到结构性变更", summary);
        }

        let structure_parts = self.format_structure_changes();
        if !structure_parts.is_empty() {
            summary.push_str(&format!("影响了{}", structure_parts.join("、")));
        }

        if self.public_items > 0 {
            summary.push_str(&format!("。其中{}个为公开项", self.public_items));
        }

        summary.push_str(&format!(
            "。共有{}个新增、{}个删除、{}个修改",
            self.additions, self.deletions, self.modifications
        ));

        summary
    }

    fn format_structure_changes(&self) -> Vec<String> {
        let mut parts = Vec::new();

        for (node_type, count) in &self.node_counts {
            if *count > 0 && node_type != "unknown" {
                let display_name = self.get_display_name(node_type);
                parts.push(format!("{}个{}", count, display_name));
            }
        }

        parts
    }

    fn get_display_name<'a>(&self, node_type: &'a str) -> &'a str {
        match (self.language_name.as_str(), node_type) {
            ("Rust", "struct_definition") => "结构体",
            ("Rust", "enum_definition") => "枚举",
            ("Rust", "function_definition") => "函数",
            ("Rust", "trait_definition") => "Trait",
            ("Java", "class_structure") => "类",
            ("Java", "method") => "方法",
            ("Java", "field") => "字段",
            ("C", "struct_definition") => "结构体",
            ("C", "function") => "函数",
            ("C++", "class_definition") => "类",
            ("C++", "template_class") => "模板类",
            ("C++", "function_definition") => "函数",
            ("C++", "virtual_function") => "虚函数",
            ("C++", "template_function") => "模板函数",
            ("C++", "struct_specifier") => "结构体",
            ("Python", "function_definition") => "函数",
            ("Python", "async_function") => "异步函数",
            ("Python", "test_function") => "测试函数",
            ("Python", "class_definition") => "类",
            ("Python", "django_model") => "Django模型",
            ("Go", "function_definition") => "函数",
            ("Go", "method_definition") => "方法",
            ("JavaScript", "function_definition") => "函数",
            ("JavaScript", "async_function") => "异步函数",
            ("JavaScript", "arrow_function") => "箭头函数",
            ("JavaScript", "class_declaration") => "类",
            ("JavaScript", "method_definition") => "方法",
            _ => node_type,
        }
    }
}

impl TreeSitterAnalyzer {
    pub fn new(config: TreeSitterConfig) -> Result<Self, TreeSitterError> {
        let query_manager = QueryManager::new(config.query_manager_config.clone())?;
        
        let mut analyzer = Self {
            config,
            project_root: PathBuf::new(),
            language_registry: create_language_registry(),
            file_asts: HashMap::new(),
            structure_queries: HashMap::new(),
            query_manager,
        };

        analyzer.initialize_queries()?;
        Ok(analyzer)
    }

    pub fn new_with_query_config(config: TreeSitterConfig, query_config: QueryManagerConfig) -> Result<Self, TreeSitterError> {
        let query_manager = QueryManager::new(query_config)?;
        
        let mut analyzer = Self {
            config,
            project_root: PathBuf::new(),
            language_registry: create_language_registry(),
            file_asts: HashMap::new(),
            structure_queries: HashMap::new(),
            query_manager,
        };

        analyzer.initialize_queries()?;
        Ok(analyzer)
    }

    pub fn set_project_root(&mut self, root: PathBuf) {
        self.project_root = root;
        self.file_asts.clear();
    }

    pub fn set_analysis_depth(&mut self, depth: String) {
        self.config.analysis_depth = depth;
    }

    /// 强制更新所有查询
    pub async fn update_queries(&mut self) -> Result<(), TreeSitterError> {
        self.query_manager.force_update_all().await?;
        self.structure_queries.clear();
        self.initialize_queries()
    }

    /// 清理查询缓存
    pub fn cleanup_query_cache(&mut self) -> Result<(), TreeSitterError> {
        self.query_manager.cleanup_cache()
    }

    /// 获取查询管理器支持的语言
    pub fn get_query_supported_languages(&self) -> Vec<String> {
        self.query_manager.get_supported_languages()
    }

    fn initialize_queries(&mut self) -> Result<(), TreeSitterError> {
        for language in self.language_registry.get_all_languages() {
            if let Some(config) = self.language_registry.get_config(language) {
                // 尝试加载结构化查询文件，如果存在的话
                let structure_query_content = load_query_file(language, "structure");
                if !structure_query_content.is_empty() {
                    let query = Query::new(&config.get_language(), &structure_query_content)
                        .map_err(|e| TreeSitterError::QueryError(format!("{}: {}", language, e)))?;
                    self.structure_queries.insert(language.to_string(), query);
                }
            }
        }
        Ok(())
    }

    pub fn detect_language(
        &self,
        path: &std::path::Path,
    ) -> Result<Option<String>, TreeSitterError> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if let Some(config) = self
            .language_registry
            .detect_language_by_extension(extension)
        {
            Ok(Some(config.name.to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn parse_file(&mut self, file_path: &std::path::Path) -> Result<FileAst, TreeSitterError> {
        let lang_id = self.detect_language(file_path)?.ok_or_else(|| {
            TreeSitterError::UnsupportedLanguage(format!("Unknown file type: {:?}", file_path))
        })?;

        let config = self
            .language_registry
            .get_config(&lang_id)
            .ok_or_else(|| TreeSitterError::UnsupportedLanguage(lang_id.clone()))?;

        let source_code = fs::read_to_string(file_path).map_err(TreeSitterError::IOError)?;

        let current_hash = calculate_hash(&source_code);

        // 检查缓存
        if let Some(cached_ast) = self.file_asts.get(file_path) {
            if cached_ast.content_hash == current_hash {
                return Ok(cached_ast.clone());
            }
        }

        let mut parser = Parser::new();
        parser
            .set_language(&config.get_language())
            .map_err(|e| TreeSitterError::ParseError(format!("Failed to set language: {}", e)))?;

        let tree = parser
            .parse(&source_code, None)
            .ok_or_else(|| TreeSitterError::ParseError("Failed to parse file".to_string()))?;

        let file_ast = FileAst {
            path: file_path.to_path_buf(),
            tree,
            source: source_code,
            content_hash: current_hash,
            last_parsed: SystemTime::now(),
            language_id: lang_id,
        };

        if self.config.cache_enabled {
            self.file_asts
                .insert(file_path.to_path_buf(), file_ast.clone());
        }

        Ok(file_ast)
    }

    pub fn analyze_file_changes(
        &self,
        file_ast: &FileAst,
        hunks: &[DiffHunk],
    ) -> Result<Vec<AffectedNode>, TreeSitterError> {
        let mut affected_nodes = self.analyze_generic_file_changes(file_ast, hunks)?;

        // 应用语言特定的增强
        if let Some(enhancer) = UnifiedNodeEnhancer::new(&file_ast.language_id) {
            for node in &mut affected_nodes {
                enhancer.enhance_node(node);
            }
        }

        Ok(affected_nodes)
    }

    fn analyze_generic_file_changes(
        &self,
        file_ast: &FileAst,
        hunks: &[DiffHunk],
    ) -> Result<Vec<AffectedNode>, TreeSitterError> {
        let mut affected_nodes = Vec::new();

        let query = self
            .structure_queries
            .get(&file_ast.language_id)
            .ok_or_else(|| TreeSitterError::UnsupportedLanguage(file_ast.language_id.clone()))?;

        let source_bytes = file_ast.source.as_bytes();
        let tree_root = file_ast.tree.root_node();
        let mut cursor = QueryCursor::new();

        for hunk in hunks {
            let hunk_start_line = hunk.new_range.start.saturating_sub(1);
            let hunk_end_line = hunk_start_line + hunk.new_range.count;

            let mut matches = cursor.matches(query, tree_root, source_bytes);
            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let node = capture.node;
                    let node_start_line = node.start_position().row;
                    let node_end_line = node.end_position().row;

                    // 检查节点是否与hunk重叠
                    if node_start_line <= hunk_end_line && node_end_line >= hunk_start_line {
                        let content = node.utf8_text(source_bytes).unwrap_or("").to_string();
                        let node_name = self.extract_node_name(&m, query, source_bytes);

                        let change_type = self.determine_change_type(hunk);

                        affected_nodes.push(AffectedNode {
                            node_type: node.kind().to_string(),
                            name: node_name,
                            range: (node.byte_range().start, node.byte_range().end),
                            is_public: is_node_public(&node, file_ast),
                            content: Some(content),
                            line_range: (node_start_line, node_end_line),
                            change_type: Some(change_type),
                            additions: None,
                            deletions: None,
                        });
                    }
                }
            }
        }

        // 去重
        affected_nodes.sort_by_key(|n| (n.range.0, n.range.1, n.node_type.clone()));
        affected_nodes.dedup_by_key(|n| (n.range.0, n.range.1, n.node_type.clone()));

        Ok(affected_nodes)
    }

    fn extract_node_name(
        &self,
        m: &tree_sitter::QueryMatch,
        query: &Query,
        source: &[u8],
    ) -> String {
        // First try to find captures ending with .name
        if let Some(capture) = m.captures
            .iter()
            .find(|c| query.capture_names()[c.index as usize].ends_with(".name"))
        {
            if let Ok(name) = capture.node.utf8_text(source) {
                return name.to_string();
            }
        }

        // Then try to find other meaningful captures
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            
            // Skip generic captures like "identifier", "type_identifier"
            if capture_name == &"identifier" || capture_name == &"type_identifier" || capture_name == &"field_identifier" {
                continue;
            }
            
            // Try to extract text from more specific captures
            if let Ok(text) = capture.node.utf8_text(source) {
                // Filter out very long text (likely code blocks)
                if text.len() < 100 && !text.contains('\n') {
                    return text.to_string();
                }
            }
        }

        // Fallback to first capture if no specific name found
        if let Some(capture) = m.captures.first() {
            if let Ok(text) = capture.node.utf8_text(source) {
                // Only return short, single-line text
                if text.len() < 50 && !text.contains('\n') {
                    return text.to_string();
                }
            }
        }

        "unknown".to_string()
    }

    fn determine_change_type(&self, hunk: &DiffHunk) -> String {
        if hunk.old_range.count == 0 {
            "added".to_string()
        } else if hunk.new_range.count == 0 {
            "deleted".to_string()
        } else {
            "modified".to_string()
        }
    }

    pub fn generate_file_summary(
        &self,
        file_ast: &FileAst,
        affected_nodes: &[AffectedNode],
    ) -> String {
        let config = self.language_registry.get_config(&file_ast.language_id);
        let language_display_name = config.map(|c| c.display_name).unwrap_or("Unknown");

        let mut generator = SummaryGenerator::new(language_display_name.to_string());

        for node in affected_nodes {
            generator.add_node(node);
        }

        generator.generate_summary(&file_ast.path, affected_nodes)
    }

    // 向后兼容的 analyze_diff 方法
    pub fn analyze_diff(&mut self, diff_text: &str) -> Result<DiffAnalysis, TreeSitterError> {
        let git_diff = parse_git_diff(diff_text)?;

        let mut file_analyses = Vec::new();
        let mut total_affected_nodes = 0;
        let mut language_counts = HashMap::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;
        let mut total_modifications = 0;

        for file_diff_info in &git_diff.changed_files {
            match file_diff_info.change_type {
                ChangeType::Added | ChangeType::Modified => {
                    let file_path = self.project_root.join(&file_diff_info.path);
                    if !file_path.exists() {
                        continue;
                    }

                    if let Ok(Some(lang_id)) = self.detect_language(&file_path) {
                        *language_counts.entry(lang_id.clone()).or_insert(0) += 1;

                        if let Ok(file_ast) = self.parse_file(&file_path) {
                            let affected_nodes =
                                self.analyze_file_changes(&file_ast, &file_diff_info.hunks)?;
                            total_affected_nodes += affected_nodes.len();

                            for node in &affected_nodes {
                                if let Some(change_type) = &node.change_type {
                                    match change_type.as_str() {
                                        "added" | "added_content" => total_additions += 1,
                                        "deleted" => total_deletions += 1,
                                        "modified" | "modified_with_deletion" => {
                                            total_modifications += 1
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            let summary = self.generate_file_summary(&file_ast, &affected_nodes);

                            file_analyses.push(FileAnalysis {
                                path: file_ast.path.clone(),
                                language: file_ast.language_id.clone(),
                                change_type: file_diff_info.change_type.clone(),
                                affected_nodes: affected_nodes.clone(),
                                summary: Some(summary),
                            });
                        }
                    }
                }
                _ => {
                    file_analyses.push(FileAnalysis {
                        path: file_diff_info.path.clone(),
                        language: "unknown".to_string(),
                        change_type: file_diff_info.change_type.clone(),
                        affected_nodes: Vec::new(),
                        summary: Some("文件被删除或重命名".to_string()),
                    });
                }
            }
        }

        let change_pattern = ChangePattern::MixedChange; // 简化的模式检测
        
        // 根据分析深度动态调整阈值
        let (major_threshold, moderate_threshold) = match self.config.analysis_depth.as_str() {
            "shallow" | "basic" => (30, 10),  // 较高阈值，更容易归类为minor
            "deep" => (10, 3),                // 较低阈值，更敏感地检测变更
            _ => (20, 5),                     // 默认medium深度
        };
        
        let change_scope = if total_affected_nodes > major_threshold {
            ChangeScope::Major
        } else if total_affected_nodes > moderate_threshold {
            ChangeScope::Moderate
        } else {
            ChangeScope::Minor
        };

        let overall_summary = format!(
            "分析完成。共影响{}个文件，{}个代码结构。新增{}，删除{}，修改{}。",
            file_analyses.len(),
            total_affected_nodes,
            total_additions,
            total_deletions,
            total_modifications
        );

        let change_analysis = ChangeAnalysis {
            function_changes: 0, // 简化统计
            type_changes: 0,
            method_changes: 0,
            interface_changes: 0,
            other_changes: total_affected_nodes,
            change_pattern,
            change_scope,
        };

        Ok(DiffAnalysis {
            file_analyses,
            overall_summary,
            change_analysis,
        })
    }
}

pub fn collect_change_stats(affected_nodes: &[AffectedNode]) -> ChangeStats {
    let mut node_type_counts = HashMap::new();
    let mut change_type_counts = HashMap::new();

    for node in affected_nodes {
        *node_type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
        if let Some(ref change_type) = node.change_type {
            *change_type_counts.entry(change_type.clone()).or_insert(0) += 1;
        }
    }

    ChangeStats {
        node_type_counts,
        change_type_counts,
    }
}
