// AI 聊天接口模块

use crate::common::{AppResult, ChatMessage};
use crate::ai::AIClient;

/// AI 聊天会话
pub struct ChatSession {
    client: Box<dyn AIClient>,
    messages: Vec<ChatMessage>,
}

impl ChatSession {
    /// 创建新的聊天会话
    pub fn new(client: Box<dyn AIClient>) -> Self {
        Self {
            client,
            messages: Vec::new(),
        }
    }

    /// 添加系统消息
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::system(content));
    }

    /// 发送用户消息并获取回复
    pub async fn send_message(&mut self, content: impl Into<String>) -> AppResult<String> {
        self.messages.push(ChatMessage::user(content));
        
        let response = self.client.chat(self.messages.clone()).await?;
        self.messages.push(ChatMessage::assistant(&response));
        
        Ok(response)
    }

    /// 清空会话历史
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// 获取会话历史
    pub fn get_history(&self) -> &[ChatMessage] {
        &self.messages
    }
}