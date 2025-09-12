// AI 服务模块（OpenAI/Ollama 兼容，Anthropic 简单兼容）

use crate::config::Config;
use gitai_types::Result;

#[derive(Clone, Debug)]
enum Provider {
    OpenAICompat,
    Anthropic,
}

/// 统一 AI 客户端
pub struct AIClient {
    config: Config,
    http: reqwest::Client,
    provider: Provider,
}

impl AIClient {
    /// 创建 AI 客户端
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("gitai-core-ai/0.1")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let provider = Self::detect_provider(&config.ai.api_url);
        Self {
            config,
            http,
            provider,
        }
    }

    fn detect_provider(api_url: &str) -> Provider {
        let url = api_url.to_ascii_lowercase();
        if url.contains("anthropic.com") || url.ends_with("/v1/messages") {
            Provider::Anthropic
        } else {
            // 默认按 OpenAI/Ollama 兼容的 chat completions
            Provider::OpenAICompat
        }
    }

    /// 生成提交信息（真实实现，失败时降级到本地摘要，不抛错）
    pub async fn generate_commit_message(&self, diff: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "You are an assistant that writes Conventional Commit messages.\n\
             Summarize the following diff into a single-line subject.\n\
             Constraints:\n\
             - Use English.\n\
             - Max 72 characters.\n\
             - Use conventional type (feat|fix|docs|refactor|chore|test|perf).\n\
             - No trailing period.\n\
             Context:\n{}\n\nDiff:\n{}",
            context, diff
        );
        let system = "You write concise, conventional commit subjects only.";
        match self.send_chat(&prompt, Some(system)).await {
            Ok(text) => Ok(text.trim().to_string()),
            Err(e) => Ok(format!(
                "feat: auto-generated commit message (fallback) [{} lines, reason: {}]",
                diff.lines().count(),
                e
            )),
        }
    }

    /// 代码评审（真实实现，失败时降级到本地摘要，不抛错）
    pub async fn review_code(&self, diff: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "You are a senior code reviewer. Provide a structured review with:\n\
             - Key issues (security, correctness, performance)\n\
             - Actionable suggestions\n\
             - Risk assessment (low/medium/high)\n\
             Context:\n{}\n\nDiff:\n{}",
            context, diff
        );
        let system = "Be precise and pragmatic. Prefer bullet points.";
        match self.send_chat(&prompt, Some(system)).await {
            Ok(text) => Ok(text),
            Err(e) => Ok(format!(
                "[AI降级] 无法调用 AI 服务（{}）。以下为上下文：\n\n{}\n\n(已省略 diff)",
                e, context
            )),
        }
    }

    async fn send_chat(
        &self,
        user_content: &str,
        system_prompt: Option<&str>,
    ) -> std::result::Result<String, String> {
        match self.provider {
            Provider::OpenAICompat => self.send_openai_compat(user_content, system_prompt).await,
            Provider::Anthropic => self.send_anthropic(user_content, system_prompt).await,
        }
    }

    async fn send_openai_compat(
        &self,
        user_content: &str,
        system_prompt: Option<&str>,
    ) -> std::result::Result<String, String> {
        let url = &self.config.ai.api_url;
        let mut messages = vec![];
        if let Some(sys) = system_prompt {
            messages.push(serde_json::json!({"role":"system","content":sys}));
        }
        messages.push(serde_json::json!({"role":"user","content":user_content}));
        let body = serde_json::json!({
            "model": self.config.ai.model,
            "temperature": self.config.ai.temperature,
            "messages": messages,
        });
        let mut req = self
            .http
            .post(url)
            .header("Content-Type", "application/json");
        if let Some(ref key) = self.config.ai.api_key {
            if !key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
        }
        let resp = req
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("http error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("http status: {}", resp.status()));
        }
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("json error: {}", e))?;
        // Try OpenAI schema: choices[0].message.content
        if let Some(content) = v
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
        {
            return Ok(content.to_string());
        }
        // Some providers (older) return choices[0].text
        if let Some(content) = v
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("text"))
            .and_then(|c| c.as_str())
        {
            return Ok(content.to_string());
        }
        Err("unexpected response schema (openai compat)".to_string())
    }

    async fn send_anthropic(
        &self,
        user_content: &str,
        system_prompt: Option<&str>,
    ) -> std::result::Result<String, String> {
        let url = &self.config.ai.api_url;
        let mut messages = vec![];
        let mut content = String::new();
        if let Some(sys) = system_prompt {
            content.push_str(sys);
            content.push_str("\n\n");
        }
        content.push_str(user_content);
        messages.push(serde_json::json!({"role":"user","content": content}));
        let body = serde_json::json!({
            "model": self.config.ai.model,
            "temperature": self.config.ai.temperature,
            "max_tokens": 1200,
            "messages": messages,
        });
        let mut req = self
            .http
            .post(url)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01");
        if let Some(ref key) = self.config.ai.api_key {
            if !key.is_empty() {
                req = req.header("x-api-key", key);
            }
        }
        let resp = req
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| format!("http error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("http status: {}", resp.status()));
        }
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("json error: {}", e))?;
        // Anthropic: content[0].text
        if let Some(text) = v
            .get("content")
            .and_then(|arr| arr.get(0))
            .and_then(|blk| blk.get("text"))
            .and_then(|t| t.as_str())
        {
            return Ok(text.to_string());
        }
        Err("unexpected response schema (anthropic)".to_string())
    }
}
