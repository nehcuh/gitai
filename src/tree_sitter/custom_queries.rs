// 用户自定义查询模板支持
// 允许用户在 ~/.gitai/queries/ 目录下添加自定义的 Tree-sitter 查询模板

use crate::tree_sitter::{SupportedLanguage, unified_analyzer::LanguageQueries};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// 用户自定义查询管理器
pub struct CustomQueryManager {
    /// 用户查询目录
    user_queries_dir: PathBuf,
    /// 已加载的自定义查询
    custom_queries: HashMap<String, LanguageQueries>,
    /// 是否启用用户查询
    enabled: bool,
}

/// 查询配置元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// 语言名称
    pub language: String,
    /// 查询版本
    pub version: Option<String>,
    /// 查询描述
    pub description: Option<String>,
    /// 是否覆盖默认查询
    pub override_default: bool,
    /// 扩展现有查询
    pub extends: Option<String>,
}

/// 完整的查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomQueryConfig {
    /// 元数据
    pub metadata: QueryMetadata,
    /// 查询内容
    pub queries: LanguageQueries,
}

impl CustomQueryManager {
    /// 创建新的自定义查询管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let user_queries_dir = dirs::home_dir()
            .ok_or("无法找到用户主目录")?
            .join(".gitai")
            .join("queries");
        
        // 如果目录不存在，创建它
        if !user_queries_dir.exists() {
            std::fs::create_dir_all(&user_queries_dir)?;
            log::info!("创建用户查询目录: {:?}", user_queries_dir);
            
            // 创建示例配置文件
            Self::create_example_config(&user_queries_dir)?;
        }
        
        let mut manager = Self {
            user_queries_dir,
            custom_queries: HashMap::new(),
            enabled: true,
        };
        
        // 加载所有自定义查询
        manager.load_all_custom_queries()?;
        
        Ok(manager)
    }
    
    /// 创建示例配置文件
    fn create_example_config(dir: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let example_path = dir.join("example_custom_query.toml");
        
        if !example_path.exists() {
            let example_content = r#"# GitAI 自定义 Tree-sitter 查询模板示例
# 
# 将此文件复制并重命名为 <language>_queries.toml 来为特定语言添加自定义查询
# 例如: rust_queries.toml, python_queries.toml, my_language_queries.toml

[metadata]
# 语言名称 (必须)
language = "example"

# 版本信息 (可选)
version = "1.0.0"

# 描述信息 (可选)
description = "示例自定义查询配置"

# 是否覆盖默认查询 (默认为 false)
# true: 完全替换默认查询
# false: 与默认查询合并
override_default = false

# 扩展现有语言的查询 (可选)
# 如果设置，将基于指定语言的查询进行扩展
# extends = "rust"

[queries]
# 函数查询模式
function_query = """
(function_item
  name: (identifier) @function.name
  parameters: (parameters) @function.parameters
) @function.definition
"""

# 类/结构体查询模式
class_query = """
(struct_item
  name: (type_identifier) @class.name
) @class.definition
"""

# 注释查询模式
comment_query = """
(line_comment) @comment
(block_comment) @comment
"""

# 提示：
# - 使用 Tree-sitter 查询语法
# - @捕获名称 用于提取特定节点
# - 可以参考官方 Tree-sitter 文档了解更多查询语法
# - 测试查询可以使用 tree-sitter CLI 工具
"#;
            std::fs::write(&example_path, example_content)?;
            log::info!("创建示例查询配置: {:?}", example_path);
        }
        
        Ok(())
    }
    
    /// 加载所有自定义查询
    pub fn load_all_custom_queries(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut loaded_count = 0;
        
        // 扫描目录中的所有 .toml 文件
        for entry in std::fs::read_dir(&self.user_queries_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                // 跳过示例文件
                if path.file_name().and_then(|s| s.to_str()) == Some("example_custom_query.toml") {
                    continue;
                }
                
                match self.load_custom_query_file(&path) {
                    Ok(config) => {
                        self.custom_queries.insert(
                            config.metadata.language.clone(),
                            config.queries.clone()
                        );
                        loaded_count += 1;
                        log::info!(
                            "加载自定义查询: {} ({})", 
                            config.metadata.language,
                            path.display()
                        );
                    }
                    Err(e) => {
                        log::warn!("加载自定义查询失败 {:?}: {}", path, e);
                    }
                }
            }
        }
        
        if loaded_count > 0 {
            log::info!("成功加载 {} 个自定义查询配置", loaded_count);
        }
        
        Ok(())
    }
    
    /// 加载单个自定义查询文件
    fn load_custom_query_file(&self, path: &Path) -> Result<CustomQueryConfig, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        let config: CustomQueryConfig = toml::from_str(&content)?;
        
        // 验证配置
        self.validate_config(&config)?;
        
        Ok(config)
    }
    
    /// 验证配置
    fn validate_config(&self, config: &CustomQueryConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查语言名称是否为空
        if config.metadata.language.is_empty() {
            return Err("语言名称不能为空".into());
        }
        
        // 检查查询是否至少有一个非空
        if config.queries.function_query.is_empty() 
            && config.queries.class_query.is_empty() 
            && config.queries.comment_query.is_empty() {
            return Err("至少需要定义一个查询".into());
        }
        
        Ok(())
    }
    
    /// 获取语言的自定义查询
    pub fn get_custom_queries(&self, language: &str) -> Option<&LanguageQueries> {
        if !self.enabled {
            return None;
        }
        
        self.custom_queries.get(language)
    }
    
    /// 获取或合并查询
    pub fn get_queries_for_language(
        &self, 
        language: SupportedLanguage,
        default_queries: Option<LanguageQueries>
    ) -> Result<LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        let lang_name = language.name();
        
        // 检查是否有自定义查询
        if let Some(custom) = self.custom_queries.get(lang_name) {
            // 如果有默认查询，可能需要合并
            if let Some(default) = default_queries {
                // 这里简单地使用自定义查询覆盖
                // 未来可以实现更复杂的合并策略
                return Ok(self.merge_queries(&default, custom));
            } else {
                return Ok(custom.clone());
            }
        }
        
        // 检查是否有针对未知语言的自定义查询
        // 例如：用户可能为 GitAI 不原生支持的语言添加了查询
        if let Some(custom) = self.custom_queries.get(&format!("custom_{}", lang_name)) {
            return Ok(custom.clone());
        }
        
        // 使用默认查询
        default_queries.ok_or_else(|| format!("没有找到 {} 语言的查询配置", lang_name).into())
    }
    
    /// 合并查询（自定义优先）
    fn merge_queries(&self, default: &LanguageQueries, custom: &LanguageQueries) -> LanguageQueries {
        LanguageQueries {
            function_query: if !custom.function_query.is_empty() {
                custom.function_query.clone()
            } else {
                default.function_query.clone()
            },
            class_query: if !custom.class_query.is_empty() {
                custom.class_query.clone()
            } else {
                default.class_query.clone()
            },
            comment_query: if !custom.comment_query.is_empty() {
                custom.comment_query.clone()
            } else {
                default.comment_query.clone()
            },
            call_query: custom.call_query.clone().or_else(|| default.call_query.clone()),
        }
    }
    
    /// 重新加载自定义查询
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.custom_queries.clear();
        self.load_all_custom_queries()?;
        log::info!("自定义查询已重新加载");
        Ok(())
    }
    
    /// 启用/禁用自定义查询
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        log::info!("自定义查询 {}", if enabled { "已启用" } else { "已禁用" });
    }
    
    /// 列出所有已加载的自定义查询
    pub fn list_custom_queries(&self) -> Vec<String> {
        self.custom_queries.keys().cloned().collect()
    }
    
    /// 添加新的自定义查询语言
    pub fn add_custom_language(
        &mut self,
        name: &str,
        queries: LanguageQueries
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 创建配置
        let config = CustomQueryConfig {
            metadata: QueryMetadata {
                language: name.to_string(),
                version: Some("1.0.0".to_string()),
                description: Some(format!("Custom queries for {}", name)),
                override_default: true,
                extends: None,
            },
            queries,
        };
        
        // 保存到文件
        let file_path = self.user_queries_dir.join(format!("{}_queries.toml", name));
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&file_path, content)?;
        
        // 添加到内存缓存
        self.custom_queries.insert(name.to_string(), config.queries);
        
        log::info!("添加自定义语言查询: {} -> {:?}", name, file_path);
        Ok(())
    }
    
    /// 验证查询语法
    pub fn validate_query_syntax(
        &self,
        language: SupportedLanguage,
        query: &str
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tree_sitter::Query;
        
        let lang = language.language();
        Query::new(lang, query)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_query_manager_creation() {
        let manager = CustomQueryManager::new();
        assert!(manager.is_ok(), "应该能创建自定义查询管理器");
    }
    
    #[test]
    fn test_merge_queries() {
        let manager = CustomQueryManager::new().unwrap();
        
        let default = LanguageQueries {
            function_query: "default_function".to_string(),
            class_query: "default_class".to_string(),
            comment_query: "default_comment".to_string(),
            call_query: None,
        };
        
        let custom = LanguageQueries {
            function_query: "custom_function".to_string(),
            class_query: String::new(),
            comment_query: "custom_comment".to_string(),
            call_query: None,
        };
        
        let merged = manager.merge_queries(&default, &custom);
        
        assert_eq!(merged.function_query, "custom_function");
        assert_eq!(merged.class_query, "default_class"); // 使用默认值
        assert_eq!(merged.comment_query, "custom_comment");
    }
    
    #[test]
    fn test_validate_config() {
        let manager = CustomQueryManager::new().unwrap();
        
        // 有效配置
        let valid_config = CustomQueryConfig {
            metadata: QueryMetadata {
                language: "test".to_string(),
                version: None,
                description: None,
                override_default: false,
                extends: None,
            },
            queries: LanguageQueries {
                function_query: "test".to_string(),
                class_query: String::new(),
                comment_query: String::new(),
                call_query: None,
            },
        };
        
        assert!(manager.validate_config(&valid_config).is_ok());
        
        // 无效配置 - 空语言名
        let invalid_config = CustomQueryConfig {
            metadata: QueryMetadata {
                language: String::new(),
                version: None,
                description: None,
                override_default: false,
                extends: None,
            },
            queries: LanguageQueries {
                function_query: "test".to_string(),
                class_query: String::new(),
                comment_query: String::new(),
                call_query: None,
            },
        };
        
        assert!(manager.validate_config(&invalid_config).is_err());
    }
}
