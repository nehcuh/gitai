use gitai_core::config::Config;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 提示词模板管理器
pub struct PromptManager {
    config: Config,
}

/// 提示词渲染上下文
#[derive(Debug, Clone)]
pub struct PromptContext {
    pub variables: HashMap<String, String>,
}

impl PromptContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn with_variable(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }
}

impl Default for PromptContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 获取提示词模板目录路径
    fn get_prompts_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("gitai")
            .join("prompts")
    }

    /// 获取assets目录下的提示词路径
    fn get_assets_prompts_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("prompts")
    }

    /// 获取模板文件路径
    fn get_template_path(&self, template_name: &str) -> Option<PathBuf> {
        let prompts_dir = self.get_prompts_dir();
        let assets_dir = self.get_assets_prompts_dir();

        // 优先级: 用户配置目录 > assets目录
        let search_dirs = vec![prompts_dir, assets_dir];

        for base_dir in search_dirs {
            let template_path = base_dir.join(format!("{template_name}.md"));
            if template_path.exists() {
                return Some(template_path);
            }
        }

        None
    }

    /// 加载提示词模板
    pub fn load_template(
        &self,
        template_name: &str,
        _language: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let template_path = self
            .get_template_path(template_name)
            .ok_or_else(|| format!("未找到提示词模板: {template_name}"))?;

        log::debug!("加载提示词模板: {template_path:?}");

        fs::read_to_string(&template_path)
            .map_err(|e| format!("读取提示词模板失败: {} - {e}", template_path.display()).into())
    }

    /// 渲染提示词模板
    pub fn render_template(&self, template_content: &str, context: &PromptContext) -> String {
        let mut rendered = template_content.to_string();

        // 替换变量
        for (key, value) in &context.variables {
            let placeholder = format!("{{{key}}}");
            rendered = rendered.replace(&placeholder, value);
        }

        rendered
    }

    /// 加载并渲染提示词模板
    pub fn load_and_render(
        &self,
        template_name: &str,
        context: &PromptContext,
        language: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let template_content = self.load_template(template_name, language)?;
        Ok(self.render_template(&template_content, context))
    }

    /// 获取语言设置
    pub fn get_language(&self) -> Option<&str> {
        // 从配置中获取语言设置，如果没有则检查环境变量
        if let Some(ref lang) = self.config.language {
            Some(lang.as_str())
        } else {
            std::env::var("LANG").ok().and_then(|lang| {
                if lang.starts_with("zh") {
                    Some("cn")
                } else {
                    None
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir;  // 暂时未使用

    #[test]
    fn test_prompt_context() {
        let context = PromptContext::new()
            .with_variable("diff", "test diff")
            .with_variable("tree_sitter_summary", "test summary");

        assert_eq!(
            context.variables.get("diff"),
            Some(&"test diff".to_string())
        );
        assert_eq!(
            context.variables.get("tree_sitter_summary"),
            Some(&"test summary".to_string())
        );
    }

    #[test]
    fn test_template_rendering() {
        let config = Config::default();
        let manager = PromptManager::new(config);

        let template = "代码变更：\n```diff\n{diff}\n```\n\n{tree_sitter_summary}";
        let context = PromptContext::new()
            .with_variable("diff", "+ added line")
            .with_variable("tree_sitter_summary", "函数分析结果");

        let rendered = manager.render_template(template, &context);
        assert!(rendered.contains("+ added line"));
        assert!(rendered.contains("函数分析结果"));
    }

    #[test]
    fn test_language_detection() {
        let config = Config {
            language: Some("cn".to_string()),
            ..Default::default()
        };
        let manager = PromptManager::new(config);

        assert_eq!(manager.get_language(), Some("cn"));
    }

    #[test]
    fn test_template_loading() {
        let config = Config::default();
        let manager = PromptManager::new(config);

        // 这个测试不会有实际文件，所以期望失败
        let result = manager.load_template("nonexistent", None);
        assert!(result.is_err());
    }
}
