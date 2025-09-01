// AI 服务模块
// TODO: 从 src/ai.rs 迁移

use crate::config::Config;
use gitai_types::Result;

pub struct AIClient {
    config: Config,
}

impl AIClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn generate_commit_message(&self, diff: &str, context: &str) -> Result<String> {
        // TODO: 实际实现
        Ok(format!("feat: auto-generated commit message for {} lines of changes", diff.lines().count()))
    }
    
    pub async fn review_code(&self, diff: &str, context: &str) -> Result<String> {
        // TODO: 实际实现
        Ok("Code review: No issues found.".to_string())
    }
}
