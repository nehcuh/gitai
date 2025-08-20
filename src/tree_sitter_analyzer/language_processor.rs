use std::collections::HashMap;
use tree_sitter::Query;
use crate::errors::AppError;
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
        let languages = self.registry.get_all_languages();
        let languages_count = languages.len();
        
        if languages_count == 0 {
            return Err(AppError::TreeSitter(crate::errors::TreeSitterError::InitializationFailed { 
                reason: "No languages configured in registry".to_string() 
            }));
        }
        
        self.compiled_queries.clear();
        
        let mut failed = Vec::new();
        
        // 先收集所有语言名称
        let language_names: Vec<String> = languages.iter().map(|&s| s.to_string()).collect();
        
        for language in language_names {
            if let Err(e) = self.initialize_language_queries(&language) {
                failed.push((language, e));
            }
        }
        
        // 如果所有语言都失败了，返回错误
        if failed.len() == languages_count {
            let error_details = failed.iter()
                .map(|(lang, err)| format!("{}: {}", lang, err))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AppError::TreeSitter(crate::errors::TreeSitterError::InitializationFailed { 
                reason: format!("All languages failed to initialize: {}", error_details) 
            }));
        }
        
        // 记录失败的语言但不阻止初始化
        if !failed.is_empty() {
            eprintln!("Warning: {} languages failed to initialize", failed.len());
            for (language, error) in failed {
                eprintln!("  - {}: {}", language, error);
            }
        }
        
        Ok(())
    }

    /// 初始化单个语言的查询
    fn initialize_language_queries(&mut self, language: &str) -> Result<(), AppError> {
        let config = self.registry.get_config(language)
            .ok_or_else(|| AppError::TreeSitter(crate::errors::TreeSitterError::LanguageNotSupported { 
                language: language.to_string() 
            }))?;
        
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
        
        if lang_queries.is_empty() {
            return Err(AppError::TreeSitter(crate::errors::TreeSitterError::QueryCompilationFailed { 
                language: language.to_string(), 
                error: "No queries could be compiled".to_string() 
            }));
        }
        
        self.compiled_queries.insert(language.to_string(), lang_queries);
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

    /// 简化的语言检测方法
    pub fn detect_language(&self, file_path: &std::path::Path) -> Option<String> {
        file_path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.detect_language_by_extension(ext))
            .map(|config| config.name.to_string())
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