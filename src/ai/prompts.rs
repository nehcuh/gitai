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
    /// 创建新的提示词管理器
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }

    /// 从文件加载提示词
    pub fn load_from_file(&mut self, name: &str, path: impl AsRef<Path>) -> AppResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::io(format!("读取提示词文件失败: {}", e)))?;
        
        self.prompts.insert(name.to_string(), content);
        Ok(())
    }

    /// 获取提示词
    pub fn get(&self, name: &str) -> Option<&String> {
        self.prompts.get(name)
    }

    /// 构建带变量的提示词
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
    fn default() -> Self {
        Self::new()
    }
}