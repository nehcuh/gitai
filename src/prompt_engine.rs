use gitai_core::config::Config;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// 简化的提示词引擎 - 遵循Linus的简单性原则
pub struct PromptEngine {
    templates: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct YamlConfig {
    templates: HashMap<String, String>,
}

impl PromptEngine {
    /// 创建新的提示词引擎
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut templates = Self::get_default_templates();

        // 尝试加载用户配置
        if let Some(config_path) = Self::get_config_path(config) {
            if let Ok(user_templates) = Self::load_user_templates(&config_path).await {
                templates.extend(user_templates);
            }
        }

        Ok(Self { templates })
    }

    /// 获取配置文件路径
    fn get_config_path(config: &Config) -> Option<PathBuf> {
        if let Some(ref prompts_config) = config.prompts {
            if let Some(ref dir) = prompts_config.directory {
                return Some(PathBuf::from(dir).join("prompts.yaml"));
            }
        }

        // 默认路径
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("gitai").join("prompts.yaml"))
    }

    /// 加载用户自定义模板
    async fn load_user_templates(
        path: &PathBuf,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let content = tokio::fs::read_to_string(path).await?;
        let yaml_config: YamlConfig = serde_yaml::from_str(&content)?;
        Ok(yaml_config.templates)
    }

    /// 渲染提示词 - 简单的变量替换
    pub fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| format!("模板 '{}' 未找到", template_name))?;

        let mut result = template.clone();
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// 检查模板是否存在
    pub fn has_template(&self, template_name: &str) -> bool {
        self.templates.contains_key(template_name)
    }

    /// 获取默认模板 - 极简版本
    fn get_default_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();

        templates.insert("architectural_analysis".to_string(), 
            "分析以下{{language}}代码的架构一致性：\n\n{{code}}\n\n重点关注：职责分离、耦合度、内聚性、设计模式。返回JSON格式的分析结果。".to_string());

        templates.insert("requirement_validation".to_string(), 
            "验证以下代码是否满足需求：\n需求：{{issue_description}}\n代码：\n{{language}}\n{{code}}\n\n分析需求覆盖度和实现质量。返回JSON格式结果。".to_string());

        templates.insert("security_analysis".to_string(), 
            "分析以下{{language}}代码的安全风险：\n\n{{code}}\n\n重点关注：注入攻击、XSS、输入验证、权限控制。返回JSON格式的安全分析结果。".to_string());

        templates.insert("quality_analysis".to_string(), 
            "分析以下{{language}}代码的质量：\n\n{{code}}\n\n重点关注：代码结构、命名规范、错误处理、可维护性。返回JSON格式的质量分析结果。".to_string());

        templates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_engine_creation() {
        let config = Config::default();
        let engine = PromptEngine::new(&config).await.unwrap();
        assert!(engine.has_template("architectural_analysis"));
        assert!(engine.has_template("security_analysis"));
    }

    #[test]
    fn test_template_rendering() {
        let mut templates = HashMap::new();
        templates.insert(
            "test".to_string(),
            "Hello {{name}}, language: {{language}}".to_string(),
        );
        let engine = PromptEngine { templates };

        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());
        context.insert("language".to_string(), "Rust".to_string());

        let result = engine.render("test", &context).unwrap();
        assert_eq!(result, "Hello World, language: Rust");
    }

    #[test]
    fn test_missing_template() {
        let engine = PromptEngine {
            templates: HashMap::new(),
        };
        let result = engine.render("nonexistent", &HashMap::new());
        assert!(result.is_err());
    }
}
