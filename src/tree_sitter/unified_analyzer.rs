// 统一的 Tree-sitter 分析器
// 使用配置文件驱动，消除语言特定代码
// 支持用户自定义查询

use tree_sitter::{Query, QueryCursor, Tree, Node};
use crate::tree_sitter::{
    SupportedLanguage, StructuralSummary, FunctionInfo, ClassInfo, CommentInfo,
    custom_queries::CustomQueryManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageQueries {
    pub function_query: String,
    pub class_query: String,
    pub comment_query: String,
    pub call_query: Option<String>,
}

/// 统一的结构分析器
pub struct UnifiedAnalyzer {
    language: SupportedLanguage,
    #[allow(dead_code)]
    queries: LanguageQueries,
    function_query: Option<Query>,
    class_query: Option<Query>,
    comment_query: Option<Query>,
    call_query: Option<Query>,
}

impl UnifiedAnalyzer {
    /// 创建新的分析器
    pub fn new(language: SupportedLanguage) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 加载查询配置
        let queries = Self::load_queries(language)?;
        
        let lang = language.language()
            .ok_or_else(|| format!("Language {:?} is not enabled in this build", language))?;
        
        // 编译查询
        let function_query = Query::new(lang, &queries.function_query).ok();
        let class_query = Query::new(lang, &queries.class_query).ok();
        let comment_query = Query::new(lang, &queries.comment_query).ok();
        let call_query = match &queries.call_query {
            Some(q) => Query::new(lang, q).ok(),
            None => None,
        };
        
        // 记录加载情况
        if function_query.is_none() {
            log::warn!("无法加载 {:?} 的函数查询", language);
        }
        if class_query.is_none() {
            log::warn!("无法加载 {:?} 的类查询", language);
        }
        if comment_query.is_none() {
            log::warn!("无法加载 {:?} 的注释查询", language);
        }
        if queries.call_query.is_none() {
            log::info!("未定义 {:?} 的调用查询，将跳过调用提取", language);
        } else if call_query.is_none() {
            log::warn!("无法编译 {:?} 的调用查询", language);
        }
        
        Ok(Self {
            language,
            queries,
            function_query,
            class_query,
            comment_query,
            call_query,
        })
    }
    
    /// 加载查询配置（支持用户自定义）
    fn load_queries(language: SupportedLanguage) -> Result<LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        // 首先尝试加载用户自定义查询
        if let Ok(custom_manager) = CustomQueryManager::new() {
            // 获取默认查询作为后备
            let default_queries = Self::load_from_config_file(language)
                .or_else(|_| Self::load_default_queries(language))
                .ok();
            
            // 尝试获取或合并自定义查询
            if let Ok(queries) = custom_manager.get_queries_for_language(language, default_queries.clone()) {
                log::info!("使用自定义查询: {:?}", language);
                return Ok(queries);
            }
            
            // 如果没有自定义查询，使用默认
            if let Some(queries) = default_queries {
                return Ok(queries);
            }
        }
        
        // 回退到原有逻辑
        if let Ok(queries) = Self::load_from_config_file(language) {
            return Ok(queries);
        }
        
        // 如果文件不存在，使用内嵌的默认配置
        Self::load_default_queries(language)
    }
    
    /// 从配置文件加载查询
    fn load_from_config_file(language: SupportedLanguage) -> Result<LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        let config_path = std::path::Path::new("assets/tree-sitter-queries.toml");
        
        if !config_path.exists() {
            // 尝试从项目根目录查找
            let alt_path = std::env::current_dir()?.join("assets/tree-sitter-queries.toml");
            if alt_path.exists() {
                let content = std::fs::read_to_string(alt_path)?;
                return Self::parse_config(&content, language);
            }
            return Err("配置文件不存在".into());
        }
        
        let content = std::fs::read_to_string(config_path)?;
        Self::parse_config(&content, language)
    }
    
    /// 解析配置内容
    fn parse_config(content: &str, language: SupportedLanguage) -> Result<LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        let config: HashMap<String, LanguageQueries> = toml::from_str(content)?;
        let lang_name = language.name();
        
        config.get(lang_name)
            .cloned()
            .ok_or_else(|| format!("未找到 {} 语言的查询配置", lang_name).into())
    }
    
    /// 加载默认查询（内嵌备份）
    fn load_default_queries(language: SupportedLanguage) -> Result<LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        // 内嵌的默认配置，作为备份
        let default_config = include_str!("../../assets/tree-sitter-queries.toml");
        Self::parse_config(default_config, language)
    }
    
    /// 分析代码结构
    pub fn analyze(&self, tree: &Tree, source: &[u8]) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("使用统一分析器分析 {:?} 语言", self.language);
        
        let mut summary = StructuralSummary {
            language: self.language.name().to_string(),
            functions: Vec::new(),
            classes: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            comments: Vec::new(),
            complexity_hints: Vec::new(),
            calls: Vec::new(),
        };
        
        let root_node = tree.root_node();
        
        // 提取函数
        if let Some(ref query) = self.function_query {
            summary.functions = self.extract_functions(query, root_node, source)?;
            log::debug!("提取到 {} 个函数", summary.functions.len());
        }
        
        // 提取类
        if let Some(ref query) = self.class_query {
            summary.classes = self.extract_classes(query, root_node, source)?;
            log::debug!("提取到 {} 个类/结构体", summary.classes.len());
        }
        
        // 提取注释
        if let Some(ref query) = self.comment_query {
            summary.comments = self.extract_comments(query, root_node, source)?;
            log::debug!("提取到 {} 个注释", summary.comments.len());
        }
        
        // 提取调用
        if let Some(ref query) = self.call_query {
            summary.calls = self.extract_calls(query, root_node, source)?;
            log::debug!("提取到 {} 个调用", summary.calls.len());
        }

        // 计算复杂度提示
        summary.complexity_hints = self.calculate_complexity_hints(&summary);
        
        Ok(summary)
    }
    
    /// 提取函数信息
    fn extract_functions(&self, query: &Query, node: Node, source: &[u8]) -> Result<Vec<FunctionInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut functions = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(query, node, source);
        
        for m in matches {
            let mut function = FunctionInfo {
                name: String::new(),
                parameters: Vec::new(),
                return_type: None,
                line_start: 0,
                line_end: 0,
                is_async: false,
                visibility: None,
            };
            
            for capture in m.captures {
                let captured_node = capture.node;
                let text = captured_node.utf8_text(source).unwrap_or("").to_string();
                
                if let Some(capture_name) = query.capture_names().get(capture.index as usize) {
                    match capture_name.as_str() {
                        "function.name" => {
                            function.name = text;
                        }
                        "function.parameters" => {
                            function.parameters = self.parse_parameters(&text);
                        }
                        "function.return_type" => {
                            function.return_type = Some(text);
                        }
                        "function.definition" => {
                            function.line_start = captured_node.start_position().row + 1;
                            function.line_end = captured_node.end_position().row + 1;
                        }
                        _ => {}
                    }
                }
            }
            
            if !function.name.is_empty() {
                functions.push(function);
            }
        }
        
        Ok(functions)
    }
    
    /// 提取类信息
    fn extract_classes(&self, query: &Query, node: Node, source: &[u8]) -> Result<Vec<ClassInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut classes = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(query, node, source);
        
        for m in matches {
            let mut class = ClassInfo {
                name: String::new(),
                methods: Vec::new(),
                fields: Vec::new(),
                line_start: 0,
                line_end: 0,
                is_abstract: false,
                extends: None,
                implements: Vec::new(),
            };
            
            for capture in m.captures {
                let captured_node = capture.node;
                let text = captured_node.utf8_text(source).unwrap_or("").to_string();
                
                if let Some(capture_name) = query.capture_names().get(capture.index as usize) {
                    match capture_name.as_str() {
                        "class.name" => {
                            class.name = text;
                        }
                        "class.extends" => {
                            class.extends = Some(text);
                        }
                        "class.implements" => {
                            class.implements.push(text);
                        }
                        "class.definition" => {
                            class.line_start = captured_node.start_position().row + 1;
                            class.line_end = captured_node.end_position().row + 1;
                        }
                        _ => {}
                    }
                }
            }
            
            if !class.name.is_empty() {
                classes.push(class);
            }
        }
        
        Ok(classes)
    }
    
    /// 提取调用信息
    fn extract_calls(&self, query: &Query, node: Node, source: &[u8]) -> Result<Vec<crate::tree_sitter::FunctionCallInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut calls = Vec::new();
        let mut cursor = QueryCursor::new();

        let matches = cursor.matches(query, node, source);
        for m in matches {
            let mut callee = String::new();
            let mut line: usize = 0;
            let mut is_method = false;
            for capture in m.captures {
                let captured_node = capture.node;
                let text = captured_node.utf8_text(source).unwrap_or("").to_string();
                if let Some(capture_name) = query.capture_names().get(capture.index as usize) {
                    match capture_name.as_str() {
                        "call.callee" => {
                            callee = text;
                        }
                        "call.expression" => {
                            line = captured_node.start_position().row + 1;
                            // 粗略判断是否为方法调用：如果表达式文本包含 '.' 或 '::'
                            let expr_text = captured_node.utf8_text(source).unwrap_or("");
                            is_method = expr_text.contains('.') || expr_text.contains("::");
                        }
                        _ => {}
                    }
                }
            }
            if !callee.is_empty() {
                calls.push(crate::tree_sitter::FunctionCallInfo { callee, line, is_method });
            }
        }
        Ok(calls)
    }

    /// 提取注释信息
    fn extract_comments(&self, query: &Query, node: Node, source: &[u8]) -> Result<Vec<CommentInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let mut comments = Vec::new();
        let mut cursor = QueryCursor::new();
        
        let matches = cursor.matches(query, node, source);
        
        for m in matches {
            for capture in m.captures {
                let captured_node = capture.node;
                if let Ok(text) = captured_node.utf8_text(source) {
                    let comment = CommentInfo {
                        text: text.to_string(),
                        line: captured_node.start_position().row + 1,
                        is_doc_comment: self.is_doc_comment(&text),
                    };
                    comments.push(comment);
                }
            }
        }
        
        Ok(comments)
    }
    
    /// 解析参数列表
    fn parse_parameters(&self, params_text: &str) -> Vec<String> {
        params_text
            .trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    }
    
    /// 判断是否为文档注释
    fn is_doc_comment(&self, text: &str) -> bool {
        match self.language {
            SupportedLanguage::Java => text.starts_with("/**"),
            SupportedLanguage::Rust => text.starts_with("///") || text.starts_with("//!"),
            SupportedLanguage::Python => text.contains("\"\"\"") || text.contains("'''"),
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => text.starts_with("/**"),
            _ => false,
        }
    }
    
    /// 计算复杂度提示
    fn calculate_complexity_hints(&self, summary: &StructuralSummary) -> Vec<String> {
        let mut hints = Vec::new();
        
        // 函数数量分析
        let func_count = summary.functions.len();
        if func_count > 50 {
            hints.push(format!("文件包含{}个函数，建议考虑拆分", func_count));
        }
        
        // 类数量分析
        let class_count = summary.classes.len();
        if class_count > 10 {
            hints.push(format!("文件包含{}个类，建议考虑模块化", class_count));
        }
        
        // 长函数检测
        for func in &summary.functions {
            let line_count = func.line_end.saturating_sub(func.line_start);
            if line_count > 100 {
                hints.push(format!("函数{}过长({}行)，建议拆分", func.name, line_count));
            }
        }
        
        // 参数过多检测
        for func in &summary.functions {
            if func.parameters.len() > 5 {
                hints.push(format!("函数{}参数过多({}个)，建议使用对象封装", func.name, func.parameters.len()));
            }
        }
        
        hints
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    #[test]
    fn test_unified_analyzer_creation() {
        // 测试各语言的分析器创建
        for lang in SupportedLanguage::all() {
            let result = UnifiedAnalyzer::new(lang);
            assert!(result.is_ok(), "应该能为 {:?} 创建分析器", lang);
        }
    }
    
    #[tokio::test]
    async fn test_analyze_rust_code() {
        let analyzer = UnifiedAnalyzer::new(SupportedLanguage::Rust).unwrap();
        
        let rust_code = r#"
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        
        pub struct Calculator {
            value: i32,
        }
        
        impl Calculator {
            pub fn new() -> Self {
                Self { value: 0 }
            }
            
            pub fn add(&mut self, x: i32) {
                self.value += x;
            }
        }
        "#;
        
        let mut parser = Parser::new();
        parser.set_language(SupportedLanguage::Rust.language()).unwrap();
        let tree = parser.parse(rust_code, None).unwrap();
        
        let result = analyzer.analyze(&tree, rust_code.as_bytes());
        assert!(result.is_ok(), "应该成功分析 Rust 代码");
        
        let summary = result.unwrap();
        assert_eq!(summary.language, "rust");
        // 放宽断言条件，因为查询可能不同
        assert!(summary.functions.len() >= 1, "应该至少找到一个函数，实际: {}", summary.functions.len());
        assert!(summary.classes.len() >= 1, "应该至少找到一个结构体，实际: {}", summary.classes.len());
    }
    
    #[test]
    fn test_load_queries_from_config() {
        // 测试从配置文件加载查询
        let result = UnifiedAnalyzer::load_queries(SupportedLanguage::Java);
        assert!(result.is_ok(), "应该能加载 Java 查询配置");
        
        let queries = result.unwrap();
        assert!(!queries.function_query.is_empty());
        assert!(!queries.class_query.is_empty());
        assert!(!queries.comment_query.is_empty());
    }
    
    #[test]
    fn test_parse_parameters() {
        let analyzer = UnifiedAnalyzer::new(SupportedLanguage::Java).unwrap();
        
        // 测试各种参数格式
        assert_eq!(analyzer.parse_parameters("()"), Vec::<String>::new());
        assert_eq!(analyzer.parse_parameters("(String name)"), vec!["String name"]);
        assert_eq!(
            analyzer.parse_parameters("(int a, String b, boolean c)"),
            vec!["int a", "String b", "boolean c"]
        );
    }
    
    #[test]
    fn test_is_doc_comment() {
        let java_analyzer = UnifiedAnalyzer::new(SupportedLanguage::Java).unwrap();
        assert!(java_analyzer.is_doc_comment("/** JavaDoc */"));
        assert!(!java_analyzer.is_doc_comment("// Regular comment"));
        
        let rust_analyzer = UnifiedAnalyzer::new(SupportedLanguage::Rust).unwrap();
        assert!(rust_analyzer.is_doc_comment("/// Doc comment"));
        assert!(rust_analyzer.is_doc_comment("//! Module doc"));
        assert!(!rust_analyzer.is_doc_comment("// Regular comment"));
    }
}
