// AI 聊天接口模块

use crate::common::{AppResult, ChatMessage};
use crate::ai::AIClient;

/// AI 聊天会话
pub struct ChatSession {
    client: Box<dyn AIClient>,
    messages: Vec<ChatMessage>,
}

impl ChatSession {
    /// Creates a new chat session with the given AI client.
    ///
    /// Initializes an empty message history for the session.
    ///
    /// # Examples
    ///
    /// ```
    /// let client: Box<dyn AIClient> = Box::new(MyAIClient::default());
    /// let session = ChatSession::new(client);
    /// assert!(session.get_history().is_empty());
    /// ```
    pub fn new(client: Box<dyn AIClient>) -> Self {
        Self {
            client,
            messages: Vec::new(),
        }
    }

    /// Appends a system message to the chat session's message history.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut session = ChatSession::new(Box::new(MockAIClient::new()));
    /// session.add_system_message("Welcome to the chat!");
    /// assert_eq!(session.get_history().last().unwrap().role, "system");
    /// ```
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(ChatMessage::system(content));
    }

    /// Sends a user message to the AI client and returns the assistant's reply.
    ///
    /// Appends the user message to the conversation history, sends the full history to the AI client asynchronously,
    /// appends the assistant's response to the history, and returns the response text.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{ChatSession, AIClient, AppResult};
    /// # async fn example(mut session: ChatSession) -> AppResult<()> {
    /// let reply = session.send_message("Hello!").await?;
    /// println!("Assistant replied: {}", reply);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(&mut self, content: impl Into<String>) -> AppResult<String> {
        self.messages.push(ChatMessage::user(content));
        
        let response = self.client.chat(self.messages.clone()).await?;
        self.messages.push(ChatMessage::assistant(&response));
        
        Ok(response)
    }

    /// Removes all messages from the chat session history.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut session = ChatSession::new(Box::new(MockAIClient::new()));
    /// session.add_system_message("Welcome!");
    /// session.clear();
    /// assert!(session.get_history().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Returns a slice of the current chat message history.
    ///
    /// # Examples
    ///
    /// ```
    /// let history = session.get_history();
    /// assert!(history.is_empty());
    /// ```
    pub fn get_history(&self) -> &[ChatMessage] {
        &self.messages
    }
}