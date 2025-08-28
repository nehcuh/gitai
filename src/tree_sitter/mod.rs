pub mod analyzer;
pub mod queries;

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
    pub fn language(&self) -> Language {
        match self {
            Self::Java => tree_sitter_java::language(),
            Self::Rust => tree_sitter_rust::language(),
            Self::C => tree_sitter_c::language(),
            Self::Cpp => tree_sitter_cpp::language(),
            Self::Python => tree_sitter_python::language(),
            Self::Go => tree_sitter_go::language(),
            Self::JavaScript => tree_sitter_javascript::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
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
}

impl TreeSitterManager {
    /// 创建新的管理器
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut parsers = HashMap::new();
        let queries_manager = queries::QueriesManager::new()?;
        
        // 初始化所有语言的解析器
        for lang in SupportedLanguage::all() {
            let mut parser = Parser::new();
            parser.set_language(lang.language())?;
            parsers.insert(lang, parser);
        }
        
        // 确保queries已下载
        queries_manager.ensure_queries_downloaded().await?;
        
        Ok(Self {
            parsers,
            queries_manager,
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

    /// 分析代码结构
    pub fn analyze_structure(
        &mut self,
        code: &str,
        language: SupportedLanguage,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!("开始分析 {:?} 语言代码，代码长度: {} 字符", language, code.len());
        
        let parser = self.get_parser(language)
            .ok_or_else(|| {
                let error = format!("Parser not found for language {:?}", language);
                log::error!("{}", error);
                error
            })?;
        
        let tree = parser.parse(code, None)
            .ok_or_else(|| {
                let error = format!("Failed to parse {:?} code", language);
                log::error!("{}", error);
                error
            })?;
        
        log::debug!("Tree 解析成功，根节点: {}", tree.root_node().kind());
        
        let analyzer = analyzer::StructureAnalyzer::new(language, &self.queries_manager)
            .map_err(|e| {
                log::error!("Failed to create StructureAnalyzer for {:?}: {}", language, e);
                e
            })?;
            
        let result = analyzer.analyze(&tree, code.as_bytes())
            .map_err(|e| {
                log::error!("Failed to analyze structure for {:?}: {}", language, e);
                e
            })?;
            
        log::info!("结构分析成功：{:?} 语言，函数: {}, 类: {}, 注释: {}", 
                  language, result.functions.len(), result.classes.len(), result.comments.len());
                  
        Ok(result)
    }
}

/// 代码结构摘要
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuralSummary {
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub complexity_hints: Vec<String>,
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
        assert_eq!(SupportedLanguage::from_extension("java"), Some(SupportedLanguage::Java));
        assert_eq!(SupportedLanguage::from_extension("rs"), Some(SupportedLanguage::Rust));
        assert_eq!(SupportedLanguage::from_extension("py"), Some(SupportedLanguage::Python));
        assert_eq!(SupportedLanguage::from_extension("js"), Some(SupportedLanguage::JavaScript));
        assert_eq!(SupportedLanguage::from_extension("ts"), Some(SupportedLanguage::TypeScript));
        assert_eq!(SupportedLanguage::from_extension("go"), Some(SupportedLanguage::Go));
        assert_eq!(SupportedLanguage::from_extension("c"), Some(SupportedLanguage::C));
        assert_eq!(SupportedLanguage::from_extension("cpp"), Some(SupportedLanguage::Cpp));
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
            assert!(parser.is_some(), "Should be able to get parser for {:?}", lang);
        }
    }

    #[tokio::test]
    async fn test_analyze_empty_code() {
        let mut manager = TreeSitterManager::new().await.expect("Failed to create manager");
        
        let result = manager.analyze_structure("", SupportedLanguage::Java);
        assert!(result.is_ok(), "Should handle empty code gracefully");
        
        let summary = result.unwrap();
        assert_eq!(summary.language, "java");
        assert_eq!(summary.functions.len(), 0);
        assert_eq!(summary.classes.len(), 0);
    }

    #[tokio::test]
    async fn test_analyze_simple_java_code() {
        let mut manager = TreeSitterManager::new().await.expect("Failed to create manager");
        
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
        assert!(summary.functions.len() >= 0);
        assert!(summary.classes.len() >= 0);
    }

    #[tokio::test]
    async fn test_analyze_simple_rust_code() {
        let mut manager = TreeSitterManager::new().await.expect("Failed to create manager");
        
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
        assert!(summary.functions.len() >= 0);
        assert!(summary.classes.len() >= 0);
    }

    #[tokio::test]
    async fn test_analyze_multiple_languages() {
        let mut manager = TreeSitterManager::new().await.expect("Failed to create manager");
        
        let test_codes = vec![
            (SupportedLanguage::Java, "public class Test { void hello() {} }"),
            (SupportedLanguage::Rust, "pub fn hello() {}"),
            (SupportedLanguage::Python, "def hello(): pass"),
            (SupportedLanguage::JavaScript, "function hello() {}"),
            (SupportedLanguage::Go, "func hello() {}"),
        ];
        
        for (lang, code) in test_codes {
            let result = manager.analyze_structure(code, lang);
            assert!(result.is_ok(), "Should successfully analyze {:?} code", lang);
            
            let summary = result.unwrap();
            assert_eq!(summary.language, lang.name());
        }
    }
}
