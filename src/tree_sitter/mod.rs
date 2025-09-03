pub mod analyzer;
pub mod cache;
pub mod custom_queries;
pub mod queries;
pub mod unified_analyzer;

use cache::{CacheKey, TreeSitterCache};
use std::collections::HashMap;
use tree_sitter::{Language, Parser};

/// 支持的编程语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Java,
    Rust,
    C,
    Cpp,
    Python,
    Go,
    JavaScript,
    TypeScript,
}

impl SupportedLanguage {
    /// 从文件扩展名推断语言
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "java" => Some(Self::Java),
            "rs" => Some(Self::Rust),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            "py" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            _ => None,
        }
    }

    /// 从语言名称推断语言
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "java" => Some(Self::Java),
            "rust" | "rs" => Some(Self::Rust),
            "c" => Some(Self::C),
            "cpp" | "c++" => Some(Self::Cpp),
            "python" | "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "javascript" | "js" => Some(Self::JavaScript),
            "typescript" | "ts" => Some(Self::TypeScript),
            _ => None,
        }
    }

    /// 获取语言名称（用于下载queries）
    pub fn name(&self) -> &'static str {
        match self {
            Self::Java => "java",
            Self::Rust => "rust",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Python => "python",
            Self::Go => "go",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
        }
    }

    /// 获取Tree-sitter语言对象
    pub fn language(&self) -> Option<Language> {
        match self {
            #[cfg(feature = "tree-sitter-java")]
            Self::Java => Some(tree_sitter_java::language()),
            #[cfg(not(feature = "tree-sitter-java"))]
            Self::Java => None,

            #[cfg(feature = "tree-sitter-rust")]
            Self::Rust => Some(tree_sitter_rust::language()),
            #[cfg(not(feature = "tree-sitter-rust"))]
            Self::Rust => None,

            #[cfg(feature = "tree-sitter-c")]
            Self::C => Some(tree_sitter_c::language()),
            #[cfg(not(feature = "tree-sitter-c"))]
            Self::C => None,

            #[cfg(feature = "tree-sitter-cpp")]
            Self::Cpp => Some(tree_sitter_cpp::language()),
            #[cfg(not(feature = "tree-sitter-cpp"))]
            Self::Cpp => None,

            #[cfg(feature = "tree-sitter-python")]
            Self::Python => Some(tree_sitter_python::language()),
            #[cfg(not(feature = "tree-sitter-python"))]
            Self::Python => None,

            #[cfg(feature = "tree-sitter-go")]
            Self::Go => Some(tree_sitter_go::language()),
            #[cfg(not(feature = "tree-sitter-go"))]
            Self::Go => None,

            #[cfg(feature = "tree-sitter-javascript")]
            Self::JavaScript => Some(tree_sitter_javascript::language()),
            #[cfg(not(feature = "tree-sitter-javascript"))]
            Self::JavaScript => None,

            #[cfg(feature = "tree-sitter-typescript")]
            Self::TypeScript => Some(tree_sitter_typescript::language_typescript()),
            #[cfg(not(feature = "tree-sitter-typescript"))]
            Self::TypeScript => None,
        }
    }

    /// 获取所有支持的语言
    pub fn all() -> Vec<Self> {
        vec![
            Self::Java,
            Self::Rust,
            Self::C,
            Self::Cpp,
            Self::Python,
            Self::Go,
            Self::JavaScript,
            Self::TypeScript,
        ]
    }
}

/// Tree-sitter管理器
pub struct TreeSitterManager {
    parsers: HashMap<SupportedLanguage, Parser>,
    queries_manager: queries::QueriesManager,
    cache: Option<TreeSitterCache>,
}

impl TreeSitterManager {
    /// 创建新的管理器
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut parsers = HashMap::new();
        let queries_manager = queries::QueriesManager::new()?;

        // 初始化所有语言的解析器（仅已启用的）
        for lang in SupportedLanguage::all() {
            if let Some(language) = lang.language() {
                let mut parser = Parser::new();
                parser.set_language(language)?;
                parsers.insert(lang, parser);
            }
        }

        // 确保queries已下载
        queries_manager.ensure_queries_downloaded().await?;

        // 初始化缓存 (100项，1小时过期)
        let cache = TreeSitterCache::new(100, 3600).ok();

        Ok(Self {
            parsers,
            queries_manager,
            cache,
        })
    }

    /// 获取指定语言的解析器
    pub fn get_parser(&mut self, language: SupportedLanguage) -> Option<&mut Parser> {
        self.parsers.get_mut(&language)
    }

    /// 获取查询管理器
    pub fn queries(&self) -> &queries::QueriesManager {
        &self.queries_manager
    }

    /// 分析代码结构（支持缓存）
    pub fn analyze_structure(
        &mut self,
        code: &str,
        language: SupportedLanguage,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!(
            "开始分析 {:?} 语言代码，代码长度: {} 字符",
            language,
            code.len()
        );

        // 检查缓存
        if let Some(ref cache) = self.cache {
            let cache_key = CacheKey::from_content(code, language.name());
            if let Some(cached_summary) = cache.get(&cache_key) {
                log::info!("使用缓存的分析结果 - {language:?} 语言");
                return Ok(cached_summary);
            }
        }

        let parser = self.get_parser(language).ok_or_else(|| {
            let error = format!("Parser not found for language {language:?}");
            log::error!("{error}");
            error
        })?;

        let tree = parser.parse(code, None).ok_or_else(|| {
            let error = format!("Failed to parse {language:?} code");
            log::error!("{error}");
            error
        })?;

        log::debug!("Tree 解析成功，根节点: {}", tree.root_node().kind());

        // 使用新的统一分析器
        let analyzer = unified_analyzer::UnifiedAnalyzer::new(language).map_err(|e| {
            log::error!("Failed to create UnifiedAnalyzer for {language:?}: {e}");
            e
        })?;

        let result = analyzer.analyze(&tree, code.as_bytes()).map_err(|e| {
            log::error!("Failed to analyze structure for {language:?}: {e}");
            e
        })?;

        log::info!(
            "结构分析成功：{:?} 语言，函数: {}, 类: {}, 注释: {}",
            language,
            result.functions.len(),
            result.classes.len(),
            result.comments.len()
        );

        // 保存到缓存
        if let Some(ref cache) = self.cache {
            let cache_key = CacheKey::from_content(code, language.name());
            if let Err(e) = cache.set(cache_key, result.clone()) {
                log::warn!("缓存保存失败: {e}");
            }
        }

        Ok(result)
    }
}

/// 代码结构摘要
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StructuralSummary {
    /// 单语言模式的语言标识（保持向后兼容）
    pub language: String,
    /// 多语言模式的分析结果
    pub language_summaries: std::collections::HashMap<String, LanguageSummary>,
    /// 单语言模式的分析结果（保持向后兼容）
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub complexity_hints: Vec<String>,
    pub calls: Vec<FunctionCallInfo>,
}

/// 单个语言的分析结果
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LanguageSummary {
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub complexity_hints: Vec<String>,
    pub calls: Vec<FunctionCallInfo>,
    /// 该语言涉及的文件数量
    pub file_count: usize,
}

impl StructuralSummary {
    /// 创建单语言模式的结构摘要
    pub fn single_language(language: String, summary: LanguageSummary) -> Self {
        let mut result = Self {
            language: language.clone(),
            language_summaries: std::collections::HashMap::new(),
            functions: summary.functions.clone(),
            classes: summary.classes.clone(),
            imports: summary.imports.clone(),
            exports: summary.exports.clone(),
            comments: summary.comments.clone(),
            complexity_hints: summary.complexity_hints.clone(),
            calls: summary.calls.clone(),
        };
        result.language_summaries.insert(language, summary);
        result
    }

    /// 创建多语言模式的结构摘要
    pub fn multi_language(language_summaries: std::collections::HashMap<String, LanguageSummary>) -> Self {
        let mut result = Self {
            language: "multi-language".to_string(),
            language_summaries,
            ..Default::default()
        };
        
        // 合并所有语言的结果以保持向后兼容
        for summary in result.language_summaries.values() {
            result.functions.extend(summary.functions.clone());
            result.classes.extend(summary.classes.clone());
            result.imports.extend(summary.imports.clone());
            result.exports.extend(summary.exports.clone());
            result.comments.extend(summary.comments.clone());
            result.complexity_hints.extend(summary.complexity_hints.clone());
            result.calls.extend(summary.calls.clone());
        }
        
        result
    }

    /// 获取所有检测到的语言
    pub fn detected_languages(&self) -> Vec<&str> {
        if self.language_summaries.is_empty() {
            vec![&self.language]
        } else {
            self.language_summaries.keys().map(|s| s.as_str()).collect()
        }
    }

    /// 检查是否为多语言模式
    pub fn is_multi_language(&self) -> bool {
        self.language_summaries.len() > 1
    }
}

impl LanguageSummary {
    /// 从旧的 StructuralSummary 转换
    pub fn from_structural_summary(summary: &StructuralSummary) -> Self {
        Self {
            language: summary.language.clone(),
            functions: summary.functions.clone(),
            classes: summary.classes.clone(),
            imports: summary.imports.clone(),
            exports: summary.exports.clone(),
            comments: summary.comments.clone(),
            complexity_hints: summary.complexity_hints.clone(),
            calls: summary.calls.clone(),
            file_count: 1,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub is_async: bool,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub fields: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub is_abstract: bool,
    pub extends: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallInfo {
    pub callee: String,
    pub line: usize,
    pub is_method: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentInfo {
    pub text: String,
    pub line: usize,
    pub is_doc_comment: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_language_from_extension() {
        assert_eq!(
            SupportedLanguage::from_extension("java"),
            Some(SupportedLanguage::Java)
        );
        assert_eq!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            SupportedLanguage::from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("go"),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(
            SupportedLanguage::from_extension("c"),
            Some(SupportedLanguage::C)
        );
        assert_eq!(
            SupportedLanguage::from_extension("cpp"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(SupportedLanguage::from_extension("unknown"), None);
    }

    #[test]
    fn test_supported_language_name() {
        assert_eq!(SupportedLanguage::Java.name(), "java");
        assert_eq!(SupportedLanguage::Rust.name(), "rust");
        assert_eq!(SupportedLanguage::Python.name(), "python");
        assert_eq!(SupportedLanguage::JavaScript.name(), "javascript");
        assert_eq!(SupportedLanguage::TypeScript.name(), "typescript");
        assert_eq!(SupportedLanguage::Go.name(), "go");
        assert_eq!(SupportedLanguage::C.name(), "c");
        assert_eq!(SupportedLanguage::Cpp.name(), "cpp");
    }

    #[test]
    fn test_supported_language_all() {
        let all_langs = SupportedLanguage::all();
        assert_eq!(all_langs.len(), 8);
        assert!(all_langs.contains(&SupportedLanguage::Java));
        assert!(all_langs.contains(&SupportedLanguage::Rust));
        assert!(all_langs.contains(&SupportedLanguage::Python));
        assert!(all_langs.contains(&SupportedLanguage::JavaScript));
        assert!(all_langs.contains(&SupportedLanguage::TypeScript));
        assert!(all_langs.contains(&SupportedLanguage::Go));
        assert!(all_langs.contains(&SupportedLanguage::C));
        assert!(all_langs.contains(&SupportedLanguage::Cpp));
    }

    #[tokio::test]
    async fn test_tree_sitter_manager_creation() {
        let result = TreeSitterManager::new().await;
        assert!(result.is_ok(), "TreeSitterManager creation should succeed");

        let mut manager = result.unwrap();

        // 测试是否可以获取各种语言的解析器
        for lang in SupportedLanguage::all() {
            let parser = manager.get_parser(lang);
            assert!(
                parser.is_some(),
                "Should be able to get parser for {lang:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_analyze_empty_code() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let result = manager.analyze_structure("", SupportedLanguage::Java);
        assert!(result.is_ok(), "Should handle empty code gracefully");

        let summary = result.unwrap();
        assert_eq!(summary.language, "java");
        assert_eq!(summary.functions.len(), 0);
        assert_eq!(summary.classes.len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_simple_java_code() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let java_code = r#"
        public class Test {
            public void hello() {
                System.out.println("Hello");
            }
        }
        "#;

        let result = manager.analyze_structure(java_code, SupportedLanguage::Java);
        assert!(result.is_ok(), "Should successfully analyze Java code");

        let summary = result.unwrap();
        assert_eq!(summary.language, "java");
        // 简单验证解析结果存在（但不强制要求数量）
        // Length is always >= 0, no need to check
    }

    #[tokio::test]
    async fn test_analyze_simple_rust_code() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let rust_code = r#"
        pub struct TestStruct {
            field: String,
        }
        
        impl TestStruct {
            pub fn new() -> Self {
                Self { field: String::new() }
            }
        }
        "#;

        let result = manager.analyze_structure(rust_code, SupportedLanguage::Rust);
        assert!(result.is_ok(), "Should successfully analyze Rust code");

        let summary = result.unwrap();
        assert_eq!(summary.language, "rust");
        // 简单验证解析结果存在
        // Length is always >= 0, no need to check
    }

    #[tokio::test]
    async fn test_analyze_multiple_languages() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let test_codes = vec![
            (
                SupportedLanguage::Java,
                "public class Test { void hello() {} }",
            ),
            (SupportedLanguage::Rust, "pub fn hello() {}"),
            (SupportedLanguage::Python, "def hello(): pass"),
            (SupportedLanguage::JavaScript, "function hello() {}"),
            (SupportedLanguage::Go, "func hello() {}"),
        ];

        for (lang, code) in test_codes {
            let result = manager.analyze_structure(code, lang);
            assert!(result.is_ok(), "Should successfully analyze {lang:?} code");

            let summary = result.unwrap();
            assert_eq!(summary.language, lang.name());
        }
    }
}
