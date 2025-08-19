/// 简化的 Tree-sitter 查询管理器
/// 
/// 这个模块提供了一个轻量级的查询管理系统，去除了不必要的网络下载和缓存复杂性。
/// 查询直接内联在代码中，提供稳定可靠的语法分析能力。

use std::collections::HashMap;
use tree_sitter::Query;
use crate::errors::{AppError, tree_sitter_query_compilation_error};

/// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    /// 语法高亮查询
    Highlights,
    /// 注入查询
    Injections,
    /// 局部变量查询
    Locals,
}

/// 查询定义 - 内联的查询字符串
pub struct QueryDefinition {
    /// 查询内容
    pub content: &'static str,
    /// 查询描述
    pub description: &'static str,
}

/// 语言查询配置
pub struct LanguageQueries {
    /// 语言名称
    pub language: &'static str,
    /// 查询定义
    pub queries: HashMap<QueryType, QueryDefinition>,
}

/// 简化的查询提供器
pub struct QueryProvider {
    /// 语言查询映射
    language_queries: HashMap<&'static str, LanguageQueries>,
}

impl QueryProvider {
    /// 创建新的查询提供器
    pub fn new() -> Self {
        let mut provider = Self {
            language_queries: HashMap::new(),
        };
        
        provider.register_default_queries();
        provider
    }
    
    /// 注册默认的查询定义
    fn register_default_queries(&mut self) {
        // Rust 查询
        self.language_queries.insert("rust", LanguageQueries {
            language: "rust",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_item)
  (struct_item)
  (enum_item)
  (trait_item)
  (impl_item)
  (mod_item)
  (use_item)
  (let_declaration)
  (const_item)
  (static_item)
  (type_item)
  (macro_invocation)
  (attribute_item)
  (field_identifier)
  (identifier)
] @variable
"#,
                    description: "Rust 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "Rust 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (let_declaration
    pattern: (identifier) @definition.local
  )
)
(
  (parameter
    pattern: (identifier) @definition.parameter
  )
)
"#,
                    description: "Rust 局部变量查询",
                });
                queries
            },
        });
        
        // Java 查询
        self.language_queries.insert("java", LanguageQueries {
            language: "java",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (class_declaration)
  (method_declaration)
  (field_declaration)
  (interface_declaration)
  (constructor_declaration)
  (enum_declaration)
  (identifier)
] @variable
"#,
                    description: "Java 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "Java 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (local_variable_declaration
    (variable_declarator
      name: (identifier) @definition.local
    )
  )
)
(
  (formal_parameter
    (identifier) @definition.parameter
  )
)
"#,
                    description: "Java 局部变量查询",
                });
                queries
            },
        });
        
        // Python 查询
        self.language_queries.insert("python", LanguageQueries {
            language: "python",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_definition)
  (class_definition)
  (identifier)
] @variable
"#,
                    description: "Python 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "Python 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (assignment
    left: (identifier) @definition.local
  )
)
(
  (function_definition
    parameters: (parameters
      (identifier) @definition.parameter
    )
  )
)
"#,
                    description: "Python 局部变量查询",
                });
                queries
            },
        });
        
        // Go 查询
        self.language_queries.insert("go", LanguageQueries {
            language: "go",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_declaration)
  (type_declaration)
  (method_declaration)
  (struct_type)
  (interface_type)
  (identifier)
] @variable
"#,
                    description: "Go 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "Go 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (short_var_declaration
    left: (expression_list
      (identifier) @definition.local
    )
  )
)
(
  (function_declaration
    parameters: (parameter_list
      (parameter_declaration
        name: (identifier) @definition.parameter
      )
    )
  )
)
"#,
                    description: "Go 局部变量查询",
                });
                queries
            },
        });
        
        // JavaScript 查询
        self.language_queries.insert("javascript", LanguageQueries {
            language: "javascript",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_declaration)
  (function_expression)
  (class_declaration)
  (method_definition)
  (identifier)
] @variable
"#,
                    description: "JavaScript 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "JavaScript 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (variable_declaration
    (variable_declarator
      name: (identifier) @definition.local
    )
  )
)
(
  (function_declaration
    parameters: (formal_parameters
      (identifier) @definition.parameter
    )
  )
)
"#,
                    description: "JavaScript 局部变量查询",
                });
                queries
            },
        });
        
        // C 查询
        self.language_queries.insert("c", LanguageQueries {
            language: "c",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_definition)
  (struct_specifier)
  (enum_specifier)
  (declaration)
  (identifier)
] @variable
"#,
                    description: "C 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "C 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (declaration
    declarator: (identifier) @definition.local
  )
)
(
  (function_definition
    parameters: (parameter_list
      (parameter_declaration
        declarator: (identifier) @definition.parameter
      )
    )
  )
)
"#,
                    description: "C 局部变量查询",
                });
                queries
            },
        });
        
        // C++ 查询
        self.language_queries.insert("cpp", LanguageQueries {
            language: "cpp",
            queries: {
                let mut queries = HashMap::new();
                queries.insert(QueryType::Highlights, QueryDefinition {
                    content: r#"
[
  (function_definition)
  (class_specifier)
  (struct_specifier)
  (enum_specifier)
  (declaration)
  (identifier)
] @variable
"#,
                    description: "C++ 语法高亮查询",
                });
                queries.insert(QueryType::Injections, QueryDefinition {
                    content: "",
                    description: "C++ 注入查询",
                });
                queries.insert(QueryType::Locals, QueryDefinition {
                    content: r#"
(
  (declaration
    declarator: (identifier) @definition.local
  )
)
(
  (function_definition
    parameters: (parameter_list
      (parameter_declaration
        declarator: (identifier) @definition.parameter
      )
    )
  )
)
"#,
                    description: "C++ 局部变量查询",
                });
                queries
            },
        });
    }
    
    /// 获取查询内容
    pub fn get_query_content(&self, language: &str, query_type: QueryType) -> Option<&'static str> {
        self.language_queries
            .get(language)
            .and_then(|queries| queries.queries.get(&query_type))
            .map(|definition| definition.content)
    }
    
    /// 获取查询描述
    pub fn get_query_description(&self, language: &str, query_type: QueryType) -> Option<&'static str> {
        self.language_queries
            .get(language)
            .and_then(|queries| queries.queries.get(&query_type))
            .map(|definition| definition.description)
    }
    
    /// 检查语言是否支持
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.language_queries.contains_key(language)
    }
    
    /// 获取支持的语言列表
    pub fn get_supported_languages(&self) -> Vec<&'static str> {
        self.language_queries.keys().copied().collect()
    }
    
    /// 编译查询
    pub fn compile_query(&self, language: &str, query_type: QueryType, language_fn: fn() -> tree_sitter::Language) -> Result<Query, AppError> {
        let content = self.get_query_content(language, query_type)
            .ok_or_else(|| tree_sitter_query_compilation_error(
                language, 
                format!("Query not found for type: {:?}", query_type)
            ))?;
        
        let ts_language = language_fn();
        Query::new(&ts_language, content)
            .map_err(|e| tree_sitter_query_compilation_error(language, e.to_string()))
    }
    
    /// 获取默认查询内容（fallback）
    pub fn get_default_query_content(&self, query_type: QueryType) -> &'static str {
        match query_type {
            QueryType::Highlights => "(identifier) @variable",
            QueryType::Injections => "",
            QueryType::Locals => "",
        }
    }
}

impl Default for QueryProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局查询提供器实例
static QUERY_PROVIDER: std::sync::OnceLock<QueryProvider> = std::sync::OnceLock::new();

/// 获取全局查询提供器
pub fn get_query_provider() -> &'static QueryProvider {
    QUERY_PROVIDER.get_or_init(QueryProvider::new)
}

/// 便捷函数：获取查询内容
pub fn get_query_content(language: &str, query_type: QueryType) -> Option<&'static str> {
    get_query_provider().get_query_content(language, query_type)
}

/// 便捷函数：检查语言支持
pub fn is_language_supported(language: &str) -> bool {
    get_query_provider().is_language_supported(language)
}

/// 便捷函数：获取支持的语言
pub fn get_supported_languages() -> Vec<&'static str> {
    get_query_provider().get_supported_languages()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_provider_creation() {
        let provider = QueryProvider::new();
        assert!(provider.is_language_supported("rust"));
        assert!(provider.is_language_supported("java"));
        assert!(!provider.is_language_supported("invalid"));
    }
    
    #[test]
    fn test_get_query_content() {
        let provider = QueryProvider::new();
        let rust_highlights = provider.get_query_content("rust", QueryType::Highlights);
        assert!(rust_highlights.is_some());
        assert!(rust_highlights.unwrap().contains("function_item"));
        
        let invalid = provider.get_query_content("invalid", QueryType::Highlights);
        assert!(invalid.is_none());
    }
    
    #[test]
    fn test_get_supported_languages() {
        let provider = QueryProvider::new();
        let languages = provider.get_supported_languages();
        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"java"));
        assert!(languages.contains(&"python"));
    }
    
    #[test]
    fn test_global_query_provider() {
        let provider = get_query_provider();
        assert!(provider.is_language_supported("rust"));
        
        let content = get_query_content("rust", QueryType::Highlights);
        assert!(content.is_some());
        
        let supported = get_supported_languages();
        assert!(!supported.is_empty());
    }
    
    #[test]
    fn test_default_query_content() {
        let provider = QueryProvider::new();
        let default = provider.get_default_query_content(QueryType::Highlights);
        assert_eq!(default, "(identifier) @variable");
        
        let default_injections = provider.get_default_query_content(QueryType::Injections);
        assert_eq!(default_injections, "");
    }
}