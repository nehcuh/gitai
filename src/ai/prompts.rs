// AI 提示词管理模块
// TODO: 将从现有代码迁移提示词管理功能

use crate::common::{AppResult, AppError};
use std::collections::HashMap;
use std::path::Path;

/// 提示词管理器
pub struct PromptManager {
    prompts: HashMap<String, String>,
}

impl PromptManager {
    /// Creates a new `PromptManager` with an empty prompt collection.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = PromptManager::new();
    /// assert!(manager.get("nonexistent").is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }

    /// Loads a prompt template from a file and stores it under the given name.
    ///
    /// Reads the file at the specified path and inserts its contents into the prompt collection with the provided name.
    /// Returns an error if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = PromptManager::new();
    /// manager.load_from_file("greeting", "prompts/greeting.txt")?;
    /// assert!(manager.get("greeting").is_some());
    /// ```
    pub fn load_from_file(&mut self, name: &str, path: impl AsRef<Path>) -> AppResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::io(format!("读取提示词文件失败: {}", e)))?;
        
        self.prompts.insert(name.to_string(), content);
        Ok(())
    }

    /// Returns a reference to the prompt template associated with the given name, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = PromptManager::new();
    /// manager.prompts.insert("greeting".to_string(), "Hello, {name}!".to_string());
    /// assert_eq!(manager.get("greeting"), Some(&"Hello, {name}!".to_string()));
    /// assert_eq!(manager.get("farewell"), None);
    /// ```
    pub fn get(&self, name: &str) -> Option<&String> {
        self.prompts.get(name)
    }

    /// Builds a prompt string by substituting variables into a named template.
    ///
    /// Retrieves the prompt template associated with `name` and replaces all placeholders of the form `{key}` with corresponding values from the `variables` map. Returns an error if the prompt template is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut manager = PromptManager::new();
    /// manager.prompts.insert("greet".to_string(), "Hello, {name}!".to_string());
    /// let mut vars = HashMap::new();
    /// vars.insert("name".to_string(), "Alice".to_string());
    /// let prompt = manager.build_prompt("greet", &vars).unwrap();
    /// assert_eq!(prompt, "Hello, Alice!");
    /// ```
    pub fn build_prompt(&self, name: &str, variables: &HashMap<String, String>) -> AppResult<String> {
        let template = self.prompts.get(name)
            .ok_or_else(|| AppError::generic(format!("未找到提示词: {}", name)))?;

        let mut result = template.clone();
        for (key, value) in variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }

        Ok(result)
    }
}

impl Default for PromptManager {
    /// Creates a new `PromptManager` instance with an empty prompt collection.
    ///
    /// This method is called when using `PromptManager::default()`.
    ///
    /// # Examples
    ///
    /// ```
    /// let manager = PromptManager::default();
    /// assert!(manager.get("nonexistent").is_none());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}