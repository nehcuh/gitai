// AI 客户端接口 - 待实现
// TODO: 将从现有代码迁移 AI 客户端功能

use crate::common::{AppResult, ChatMessage};

/// AI 客户端抽象接口
#[async_trait::async_trait]
pub trait AIClient: Send + Sync {
    /// 发送聊天消息
    async fn chat(&self, messages: Vec<ChatMessage>) -> AppResult<String>;
    
    /// 检查 AI 服务是否可用
    async fn is_available(&self) -> bool;
    
    /// 获取模型信息
    fn get_model_info(&self) -> String;
}

/// AI 客户端构建器
pub struct AIClientBuilder;

impl AIClientBuilder {
    /// 从配置创建 AI 客户端
    pub fn from_config(_config: &crate::config::AppConfig) -> AppResult<Box<dyn AIClient>> {
        // TODO: 实现实际的客户端创建逻辑
        todo!("AI 客户端实现待迁移")
    }
}