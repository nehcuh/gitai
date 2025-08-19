use std::collections::HashMap;
use tree_sitter::Query;
use crate::errors::{AppError, tree_sitter_init_error, tree_sitter_language_error, tree_sitter_query_compilation_error};
use super::core::{LanguageConfig, LanguageRegistry, create_language_registry};
use super::query_provider::{QueryProvider, QueryType};

/// 统一的语言处理器
pub struct LanguageProcessor {
    registry: LanguageRegistry,
    query_provider: QueryProvider,
    compiled_queries: HashMap<String, HashMap<QueryType, Query>>,
}

impl LanguageProcessor {
    pub fn new() -> Self {
        Self {
            registry: create_language_registry(),
            query_provider: QueryProvider::new(),
            compiled_queries: HashMap::new(),
        }
    }

    /// 初始化所有语言的查询
    pub fn initialize(&mut self) -> Result<(), AppError> {
        // 验证注册表状态并收集语言名称
        let language_names: Vec<String> = self.registry.get_all_languages()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        if language_names.is_empty() {
            return Err(tree_sitter_init_error("No languages configured in registry"));
        }
        
        // 清理现有缓存
        self.compiled_queries.clear();
        
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut failed_languages = Vec::new();
        
        for language in language_names {
            match self.initialize_language_queries(&language) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failure_count += 1;
                    failed_languages.push((language, e));
                }
            }
        }
        
        // 验证初始化结果
        if success_count == 0 {
            let error_details = failed_languages
                .into_iter()
                .map(|(lang, err)| format!("{}: {}", lang, err))
                .collect::<Vec<_>>()
                .join(", ");
            
            return Err(tree_sitter_init_error(format!(
                "Failed to initialize any language queries. Errors: {}", 
                error_details
            )));
        }
        
        // 记录部分失败的情况
        if failure_count > 0 {
            eprintln!("Warning: {} languages failed to initialize, {} succeeded", failure_count, success_count);
            for (language, error) in failed_languages {
                eprintln!("  - {}: {}", language, error);
            }
        }
        
        // 验证缓存一致性
        if self.compiled_queries.len() != success_count {
            eprintln!("Warning: Query cache size mismatch. Expected: {}, Actual: {}", 
                success_count, self.compiled_queries.len());
        }
        
        Ok(())
    }

    /// 初始化单个语言的查询
    fn initialize_language_queries(&mut self, language: &str) -> Result<(), AppError> {
        let config = self.registry.get_config(language)
            .ok_or_else(|| tree_sitter_language_error(language))?;
        
        let mut lang_queries = HashMap::new();
        
        // 编译高亮查询
        if let Ok(Some(query)) = config.get_highlights_query(&self.query_provider) {
            lang_queries.insert(QueryType::Highlights, query);
        }
        
        // 编译注入查询
        if let Ok(Some(query)) = config.get_injections_query(&self.query_provider) {
            lang_queries.insert(QueryType::Injections, query);
        }
        
        // 编译局部变量查询
        if let Ok(Some(query)) = config.get_locals_query(&self.query_provider) {
            lang_queries.insert(QueryType::Locals, query);
        }
        
        if !lang_queries.is_empty() {
            self.compiled_queries.insert(language.to_string(), lang_queries);
        } else {
            return Err(tree_sitter_query_compilation_error(
                language, 
                "No queries could be compiled"
            ));
        }
        
        Ok(())
    }

    /// 获取语言的编译查询
    pub fn get_compiled_query(&self, language: &str, query_type: QueryType) -> Option<&Query> {
        self.compiled_queries
            .get(language)
            .and_then(|queries| queries.get(&query_type))
    }

    /// 检查语言是否支持
    pub fn is_language_supported(&self, language: &str) -> bool {
        self.compiled_queries.contains_key(language)
    }

    /// 获取所有支持的语言
    pub fn get_supported_languages(&self) -> Vec<&str> {
        self.compiled_queries.keys().map(|s| s.as_str()).collect()
    }

    /// 获取语言配置
    pub fn get_language_config(&self, language: &str) -> Option<&LanguageConfig> {
        self.registry.get_config(language)
    }

    /// 根据文件扩展名检测语言
    pub fn detect_language_by_extension(&self, extension: &str) -> Option<&LanguageConfig> {
        self.registry.detect_language_by_extension(extension)
    }
}

impl Default for LanguageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_processor_creation() {
        let processor = LanguageProcessor::new();
        assert!(processor.get_supported_languages().is_empty());
    }

    #[test]
    fn test_language_processor_initialization() {
        let mut processor = LanguageProcessor::new();
        // 初始化应该不会失败，即使没有语言配置
        let result = processor.initialize();
        assert!(result.is_ok());
    }
}