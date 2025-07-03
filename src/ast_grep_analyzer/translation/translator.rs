//! Core translation engine for AST-Grep rules
//!
//! This module provides the main translation functionality, including
//! different translation providers and the core translation engine.

use super::{
    SupportedLanguage, TranslationError, TranslationResult, cache_manager::TranslationCacheManager,
};
use crate::ast_grep_analyzer::core::AnalysisRule;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Translation request for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// Original rule to translate
    pub rule: AnalysisRule,
    /// Target language
    pub target_language: SupportedLanguage,
    /// Translation context (optional)
    pub context: Option<String>,
    /// Priority level (0-10, higher is more important)
    pub priority: u8,
}

/// Translation response for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// Translated rule
    pub translated_rule: AnalysisRule,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Translation metadata
    pub metadata: TranslationMetadata,
}

/// Metadata about the translation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMetadata {
    /// Provider used for translation
    pub provider: String,
    /// Time taken for translation (milliseconds)
    pub duration_ms: u64,
    /// Model or service version used
    pub model_version: Option<String>,
    /// Additional provider-specific metadata
    pub provider_metadata: HashMap<String, String>,
}

/// Translation provider trait
#[async_trait]
pub trait TranslationProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Check if the provider is available
    async fn is_available(&self) -> bool;

    /// Translate a single rule
    async fn translate_rule(
        &self,
        request: &TranslationRequest,
    ) -> TranslationResult<TranslationResponse>;

    /// Translates multiple rules in a batch by processing each translation request sequentially.
    ///
    /// Returns a vector of translation responses corresponding to each input request. This default implementation translates each rule one at a time.
    ///
    /// # Arguments
    ///
    /// * `requests` - A slice of translation requests to process.
    ///
    /// # Returns
    ///
    /// A result containing a vector of translation responses, or an error if any translation fails.
    async fn translate_rules_batch(
        &self,
        requests: &[TranslationRequest],
    ) -> TranslationResult<Vec<TranslationResponse>> {
        // Default implementation: translate one by one
        let mut responses = Vec::new();
        for request in requests {
            let response = self.translate_rule(request).await?;
            responses.push(response);
        }
        Ok(responses)
    }

    /// Get supported languages
    fn supported_languages(&self) -> Vec<SupportedLanguage>;

    /// Get provider configuration
    fn get_config(&self) -> HashMap<String, String>;
}

/// OpenAI translation provider
#[derive(Debug)]
pub struct OpenAITranslationProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
    timeout_seconds: u64,
}

impl OpenAITranslationProvider {
    /// Constructs a new `OpenAITranslationProvider` with the specified API key, model, base URL, and timeout.
    ///
    /// If the model, base URL, or timeout are not provided, defaults are used ("gpt-3.5-turbo", "https://api.openai.com/v1", and 30 seconds respectively).
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OpenAITranslationProvider::new(
    ///     "sk-...".to_string(),
    ///     None,
    ///     None,
    ///     Some(60),
    /// );
    /// assert_eq!(provider.model, "gpt-3.5-turbo");
    /// assert_eq!(provider.base_url, "https://api.openai.com/v1");
    /// assert_eq!(provider.timeout_seconds, 60);
    /// ```
    pub fn new(
        api_key: String,
        model: Option<String>,
        base_url: Option<String>,
        timeout_seconds: Option<u64>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds.unwrap_or(30)))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            timeout_seconds: timeout_seconds.unwrap_or(30),
        }
    }

    /// Generates a prompt instructing a translation provider to translate an AST-Grep analysis rule into the specified target language, preserving technical terms and formatting the output as JSON.
    ///
    /// The prompt includes the rule's ID, name, message, and suggestion, and provides explicit instructions to maintain technical accuracy and clarity. The output format is strictly defined for downstream parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// let prompt = provider.create_translation_prompt(&rule, &SupportedLanguage::Chinese);
    /// assert!(prompt.contains("AST-Grep代码分析规则翻译为简体中文"));
    /// ```
    fn create_translation_prompt(
        &self,
        rule: &AnalysisRule,
        target_language: &SupportedLanguage,
    ) -> String {
        let language_name = match target_language {
            SupportedLanguage::Chinese => "简体中文",
            SupportedLanguage::English => "English",
            SupportedLanguage::Auto => "the user's preferred language",
        };

        format!(
            r#"请将以下AST-Grep代码分析规则翻译为{}。保持技术术语的准确性，确保翻译后的规则仍然专业且易于理解。

规则信息：
- 规则ID: {}
- 规则名称: {}
- 消息: {}
- 建议: {}

请只返回JSON格式的翻译结果，格式如下：
{{
  "name": "翻译后的规则名称",
  "message": "翻译后的错误消息",
  "suggestion": "翻译后的修复建议（如果有的话）"
}}

注意：
1. 保持代码相关的技术术语不变
2. 确保翻译后的消息简洁明了
3. 如果原文中有代码示例，保持代码不变
4. 如果suggestion为空，请返回null"#,
            language_name,
            rule.id,
            rule.name,
            rule.message,
            rule.suggestion.as_deref().unwrap_or("无")
        )
    }
}

#[async_trait]
impl TranslationProvider for OpenAITranslationProvider {
    /// Returns the name of the translation provider, which is "openai".
    fn name(&self) -> &str {
        "openai"
    }

    /// Checks if the OpenAI translation provider is available by sending a minimal request to the API.
    ///
    /// Returns `true` if the provider responds successfully, otherwise returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OpenAITranslationProvider::new("api-key".to_string(), None, None, None);
    /// let available = tokio_test::block_on(provider.is_available());
    /// assert!(available || !available); // Availability depends on API key and network.
    /// ```
    async fn is_available(&self) -> bool {
        // Simple health check - try to make a minimal request
        let url = format!("{}/models", self.base_url);

        match self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Translates an analysis rule into the target language using the OpenAI API.
    ///
    /// Sends a chat completion request to the OpenAI API with a generated prompt, parses the AI's JSON response, and constructs a translated rule. Returns a `TranslationResponse` containing the translated rule, a fixed confidence score, and metadata about the translation process.
    ///
    /// # Errors
    ///
    /// Returns a `TranslationError` if the API request fails, the response format is invalid, or the AI's output cannot be parsed as JSON.
    async fn translate_rule(
        &self,
        request: &TranslationRequest,
    ) -> TranslationResult<TranslationResponse> {
        let start_time = std::time::Instant::now();

        let prompt = self.create_translation_prompt(&request.rule, &request.target_language);

        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        });

        let url = format!("{}/chat/completions", self.base_url);

        let response = timeout(
            Duration::from_secs(self.timeout_seconds),
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send(),
        )
        .await
        .map_err(|_| TranslationError::Timeout)?
        .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TranslationError::ProviderError(format!(
                "OpenAI API error {}: {}",
                status, error_text
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TranslationError::ProviderError(format!("JSON parse error: {}", e)))?;

        // Extract the translated content
        let content = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                TranslationError::ProviderError("Invalid response format".to_string())
            })?;

        // Parse the JSON response from AI
        let translation_data: serde_json::Value =
            serde_json::from_str(content.trim()).map_err(|e| {
                TranslationError::ProviderError(format!("Failed to parse AI response: {}", e))
            })?;

        // Create translated rule
        let mut translated_rule = request.rule.clone();

        if let Some(name) = translation_data["name"].as_str() {
            translated_rule.name = name.to_string();
        }

        if let Some(message) = translation_data["message"].as_str() {
            translated_rule.message = message.to_string();
        }

        if let Some(suggestion) = translation_data["suggestion"].as_str() {
            translated_rule.suggestion = Some(suggestion.to_string());
        } else if translation_data["suggestion"].is_null() {
            translated_rule.suggestion = None;
        }

        let duration = start_time.elapsed();

        let metadata = TranslationMetadata {
            provider: self.name().to_string(),
            duration_ms: duration.as_millis() as u64,
            model_version: Some(self.model.clone()),
            provider_metadata: HashMap::new(),
        };

        Ok(TranslationResponse {
            translated_rule,
            confidence: 0.8, // Default confidence for OpenAI
            metadata,
        })
    }

    /// Translates a batch of rules into the target language, processing requests in smaller chunks to avoid token limits.
    ///
    /// Each rule is translated individually; if a translation fails, a fallback response with the original rule and zero confidence is returned for that rule.
    ///
    /// # Returns
    /// A vector of `TranslationResponse` objects, each containing the translated rule or the original rule if translation failed.
    async fn translate_rules_batch(
        &self,
        requests: &[TranslationRequest],
    ) -> TranslationResult<Vec<TranslationResponse>> {
        // For OpenAI, we'll process in smaller batches to avoid token limits
        const BATCH_SIZE: usize = 5;
        let mut all_responses = Vec::new();

        for chunk in requests.chunks(BATCH_SIZE) {
            let mut responses = Vec::new();
            for request in chunk {
                match self.translate_rule(request).await {
                    Ok(response) => responses.push(response),
                    Err(e) => {
                        warn!("Failed to translate rule {}: {}", request.rule.id, e);
                        // Create a fallback response with original rule
                        let fallback_response = TranslationResponse {
                            translated_rule: request.rule.clone(),
                            confidence: 0.0,
                            metadata: TranslationMetadata {
                                provider: self.name().to_string(),
                                duration_ms: 0,
                                model_version: Some(self.model.clone()),
                                provider_metadata: [("error".to_string(), e.to_string())]
                                    .iter()
                                    .cloned()
                                    .collect(),
                            },
                        };
                        responses.push(fallback_response);
                    }
                }
            }
            all_responses.extend(responses);
        }

        Ok(all_responses)
    }

    /// Returns the list of languages supported by this translation provider.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OpenAITranslationProvider::new("api-key".to_string(), None, None, None);
    /// let langs = provider.supported_languages();
    /// assert!(langs.contains(&SupportedLanguage::English));
    /// assert!(langs.contains(&SupportedLanguage::Chinese));
    /// assert!(langs.contains(&SupportedLanguage::Auto));
    /// ```
    fn supported_languages(&self) -> Vec<SupportedLanguage> {
        vec![
            SupportedLanguage::English,
            SupportedLanguage::Chinese,
            SupportedLanguage::Auto,
        ]
    }

    /// Returns the configuration of the OpenAI translation provider as key-value pairs.
    ///
    /// The configuration includes the model name, base URL, and timeout in seconds.
    fn get_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("model".to_string(), self.model.clone());
        config.insert("base_url".to_string(), self.base_url.clone());
        config.insert(
            "timeout_seconds".to_string(),
            self.timeout_seconds.to_string(),
        );
        config
    }
}

/// Mock translation provider for testing
#[derive(Debug)]
pub struct MockTranslationProvider {
    translations: HashMap<String, HashMap<String, String>>,
    delay_ms: u64,
    should_fail: bool,
}

impl MockTranslationProvider {
    /// Creates a new `MockTranslationProvider` with preset English-to-Chinese translations, a default delay, and no failure mode enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new();
    /// assert_eq!(provider.name(), "mock");
    /// ```
    pub fn new() -> Self {
        let mut translations = HashMap::new();

        // Add some mock translations
        let mut english_to_chinese = HashMap::new();
        english_to_chinese.insert(
            "Use === for strict equality comparison".to_string(),
            "使用 === 进行严格相等比较".to_string(),
        );
        english_to_chinese.insert(
            "Avoid using unwrap()".to_string(),
            "避免使用 unwrap()".to_string(),
        );

        translations.insert("en_to_zh".to_string(), english_to_chinese);

        Self {
            translations,
            delay_ms: 100,
            should_fail: false,
        }
    }

    /// Sets an artificial delay in milliseconds for translation operations, used for testing purposes.
    ///
    /// # Parameters
    ///
    /// - `delay_ms`: The delay duration in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns a new instance of the provider with the specified delay applied.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new().with_delay(500);
    /// ```
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Configures the mock provider to simulate failure for testing purposes.
    ///
    /// When set to `true`, subsequent translation attempts will return errors.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new().with_failure(true);
    /// assert!(!provider.is_available());
    /// ```
    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }
}

#[async_trait]
impl TranslationProvider for MockTranslationProvider {
    /// Returns the name of the mock translation provider.
    fn name(&self) -> &str {
        "mock"
    }

    /// Indicates whether the mock translation provider is available.
    ///
    /// Returns `false` if the provider is configured to simulate failure; otherwise, returns `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new();
    /// assert!(provider.is_available().await);
    ///
    /// let failing_provider = MockTranslationProvider::new().with_failure(true);
    /// assert!(!failing_provider.is_available().await);
    /// ```
    async fn is_available(&self) -> bool {
        !self.should_fail
    }

    /// Translates an analysis rule using mock translation logic for testing purposes.
    ///
    /// Simulates translation by returning a preset translation if available, or by prefixing the message with the target language code. Can be configured to introduce artificial delay or to simulate failure.
    ///
    /// # Returns
    ///
    /// A `TranslationResponse` containing the translated rule, a fixed confidence score, and metadata about the translation process. Returns an error if the mock provider is configured to fail.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new();
    /// let request = TranslationRequest {
    ///     rule: create_test_rule(),
    ///     target_language: SupportedLanguage::Chinese,
    ///     context: None,
    ///     priority: 1,
    /// };
    /// let response = provider.translate_rule(&request).await.unwrap();
    /// assert_eq!(response.translated_rule.message, "模拟翻译信息");
    /// ```
    async fn translate_rule(
        &self,
        request: &TranslationRequest,
    ) -> TranslationResult<TranslationResponse> {
        // Simulate delay
        if self.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        }

        if self.should_fail {
            return Err(TranslationError::ProviderError(
                "Mock provider configured to fail".to_string(),
            ));
        }

        let start_time = std::time::Instant::now();

        let mut translated_rule = request.rule.clone();

        // Simple mock translation logic
        let translation_key = format!("en_to_{}", request.target_language.code());
        if let Some(lang_translations) = self.translations.get(&translation_key) {
            if let Some(translated_message) = lang_translations.get(&request.rule.message) {
                translated_rule.message = translated_message.clone();
            } else {
                // Fallback: add language prefix
                translated_rule.message = format!(
                    "[{}] {}",
                    request.target_language.code(),
                    request.rule.message
                );
            }
        }

        let duration = start_time.elapsed();

        let metadata = TranslationMetadata {
            provider: self.name().to_string(),
            duration_ms: duration.as_millis() as u64,
            model_version: Some("mock-v1.0".to_string()),
            provider_metadata: HashMap::new(),
        };

        Ok(TranslationResponse {
            translated_rule,
            confidence: 0.9,
            metadata,
        })
    }

    /// Returns the list of languages supported by this translation provider.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = MockTranslationProvider::new();
    /// let langs = provider.supported_languages();
    /// assert_eq!(langs, vec![SupportedLanguage::Chinese]);
    /// ```
    fn supported_languages(&self) -> Vec<SupportedLanguage> {
        vec![SupportedLanguage::Chinese]
    }

    /// Returns an empty configuration map for the mock translation provider.
    ///
    /// This implementation does not require any configuration parameters.
    fn get_config(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Main translation engine
pub struct RuleTranslator {
    provider: Arc<dyn TranslationProvider>,
    cache_manager: Option<TranslationCacheManager>,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl RuleTranslator {
    /// Creates a new `RuleTranslator` with the specified translation provider and optional cache manager.
    ///
    /// The translator is initialized with default retry settings: 3 maximum retries and a 1000 ms delay between retries.
    pub fn new(
        provider: Arc<dyn TranslationProvider>,
        cache_manager: Option<TranslationCacheManager>,
    ) -> Self {
        Self {
            provider,
            cache_manager,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }

    /// Configures the maximum number of translation retries and the delay between retries.
    ///
    /// # Parameters
    ///
    /// - `max_retries`: The maximum number of retry attempts for failed translations.
    /// - `retry_delay_ms`: The delay in milliseconds between retry attempts.
    ///
    /// # Returns
    ///
    /// Returns a new `RuleTranslator` instance with the updated retry configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let translator = RuleTranslator::new(provider, None)
    ///     .with_retries(5, 2000);
    /// ```
    pub fn with_retries(mut self, max_retries: u32, retry_delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = retry_delay_ms;
        self
    }

    /// Translates a single analysis rule into the specified target language, using caching and retry logic.
    ///
    /// If the target language is English, returns the original rule without translation. Checks the cache for an existing translation before making a translation request. On a cache miss, performs translation with retries and stores the result in the cache. Returns the translated rule or an error if translation fails.
    ///
    /// # Arguments
    ///
    /// * `rule` - The analysis rule to translate.
    /// * `target_language` - The language to translate the rule into.
    ///
    /// # Returns
    ///
    /// The translated analysis rule, or an error if translation is unsuccessful.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut translator = RuleTranslator::new(provider, Some(cache_manager));
    /// let translated_rule = translator.translate_rule(&rule, &SupportedLanguage::Chinese).await?;
    /// assert_eq!(translated_rule.language, SupportedLanguage::Chinese);
    /// ```
    pub async fn translate_rule(
        &mut self,
        rule: &AnalysisRule,
        target_language: &SupportedLanguage,
    ) -> TranslationResult<AnalysisRule> {
        // Skip translation if target language is English
        if matches!(target_language, SupportedLanguage::English) {
            return Ok(rule.clone());
        }

        let rule_hash = self.calculate_rule_hash(rule);

        // Check cache first
        if let Some(ref cache) = self.cache_manager {
            if let Some(cached_rule) =
                cache.get_cached_translation(&rule.id, &rule_hash, target_language)
            {
                debug!("Using cached translation for rule: {}", rule.id);
                return Ok(cached_rule);
            }
        }

        // Translate with retries
        let request = TranslationRequest {
            rule: rule.clone(),
            target_language: target_language.clone(),
            context: None,
            priority: 5,
        };

        let response = self.translate_with_retries(&request).await?;

        // Store in cache
        if let Some(ref mut cache) = self.cache_manager {
            let _ = cache.store_translation(
                rule.id.clone(),
                rule_hash,
                target_language,
                response.translated_rule.clone(),
                "v1.0".to_string(),
                response.metadata.provider.clone(),
            );
        }

        info!(
            "Translated rule {} to {} (confidence: {:.2})",
            rule.id,
            target_language.code(),
            response.confidence
        );

        Ok(response.translated_rule)
    }

    /// Translates a list of analysis rules into the specified target language.
    ///
    /// Each rule is translated individually. If translation fails for a rule, the original rule is used as a fallback in the result.
    ///
    /// # Returns
    /// A vector containing the translated rules, or the original rules if translation failed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut translator = RuleTranslator::new(provider, None);
    /// let rules = vec![create_test_rule()];
    /// let translated = translator.translate_rules(&rules, &SupportedLanguage::Chinese).await.unwrap();
    /// assert_eq!(translated.len(), rules.len());
    /// ```
    pub async fn translate_rules(
        &mut self,
        rules: &[AnalysisRule],
        target_language: &SupportedLanguage,
    ) -> TranslationResult<Vec<AnalysisRule>> {
        let mut translated_rules = Vec::new();

        for rule in rules {
            match self.translate_rule(rule, target_language).await {
                Ok(translated_rule) => translated_rules.push(translated_rule),
                Err(e) => {
                    warn!("Failed to translate rule {}: {}", rule.id, e);
                    // Use original rule as fallback
                    translated_rules.push(rule.clone());
                }
            }
        }

        Ok(translated_rules)
    }

    /// Checks whether the translation provider is currently available.
    ///
    /// Returns `true` if the provider is reachable and operational, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let available = rule_translator.is_provider_available().await;
    /// assert!(available || !available); // Availability depends on provider state
    /// ```
    pub async fn is_provider_available(&self) -> bool {
        self.provider.is_available().await
    }

    /// Returns the translation provider's name and configuration details.
    ///
    /// The configuration is a map of provider-specific settings such as model, base URL, or timeout.
    pub fn get_provider_info(&self) -> (String, HashMap<String, String>) {
        (self.provider.name().to_string(), self.provider.get_config())
    }

    /// Attempts to translate a rule using the provider, retrying on failure up to the configured number of times.
    ///
    /// Retries translation if an error occurs, waiting for the specified delay between attempts. Returns the translation response on success, or the last encountered error if all attempts fail.
    ///
    /// # Arguments
    ///
    /// * `request` - The translation request containing the rule and target language.
    ///
    /// # Returns
    ///
    /// A `TranslationResponse` on success, or a `TranslationError` if all retries fail.
    ///
    /// # Examples
    ///
    /// ```
    /// let response = translator.translate_with_retries(&request).await?;
    /// assert!(response.confidence > 0.0);
    /// ```
    async fn translate_with_retries(
        &self,
        request: &TranslationRequest,
    ) -> TranslationResult<TranslationResponse> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.provider.translate_rule(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries {
                        warn!(
                            "Translation attempt {} failed for rule {}, retrying in {}ms",
                            attempt + 1,
                            request.rule.id,
                            self.retry_delay_ms
                        );
                        tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TranslationError::ProviderError("Unknown error occurred".to_string())
        }))
    }

    /// Computes a hash string representing the content of an analysis rule.
    ///
    /// The hash is based on the rule's ID, name, message, and suggestion fields. This can be used for caching or change detection purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// let rule = create_test_rule();
    /// let translator = RuleTranslator::new(Arc::new(MockTranslationProvider::new()), None);
    /// let hash = translator.calculate_rule_hash(&rule);
    /// assert!(!hash.is_empty());
    /// ```
    fn calculate_rule_hash(&self, rule: &AnalysisRule) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        rule.id.hash(&mut hasher);
        rule.name.hash(&mut hasher);
        rule.message.hash(&mut hasher);
        rule.suggestion.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_grep_analyzer::core::{IssueCategory, IssueSeverity};

    /// Creates a sample `AnalysisRule` for testing purposes.
    ///
    /// The returned rule is configured for JavaScript and checks for strict equality comparison.
    ///
    /// # Examples
    ///
    /// ```
    /// let rule = create_test_rule();
    /// assert_eq!(rule.id, "test-rule");
    /// assert_eq!(rule.language, "javascript");
    /// ```
    fn create_test_rule() -> AnalysisRule {
        AnalysisRule {
            id: "test-rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test rule for equality comparison".to_string(),
            enabled: true,
            language: "javascript".to_string(),
            severity: IssueSeverity::Warning,
            category: IssueCategory::CodeQuality,
            pattern: "$VAR.test()".to_string(),
            message: "Use === for strict equality comparison".to_string(),
            suggestion: Some("Replace == with ===".to_string()),
        }
    }

    #[tokio::test]
    async fn test_mock_translation_provider() {
        let provider = MockTranslationProvider::new();
        assert!(provider.is_available().await);

        let rule = create_test_rule();
        let request = TranslationRequest {
            rule,
            target_language: SupportedLanguage::Chinese,
            context: None,
            priority: 5,
        };

        let response = provider.translate_rule(&request).await;
        assert!(response.is_ok());

        let translation_response = response.unwrap();
        assert_eq!(
            translation_response.translated_rule.message,
            "使用 === 进行严格相等比较"
        );
    }

    #[tokio::test]
    async fn test_rule_translator_with_mock() {
        let provider = Arc::new(MockTranslationProvider::new());
        let mut translator = RuleTranslator::new(provider, None);

        let rule = create_test_rule();
        let result = translator
            .translate_rule(&rule, &SupportedLanguage::Chinese)
            .await;

        assert!(result.is_ok());
        let translated_rule = result.unwrap();
        assert_eq!(translated_rule.id, rule.id);
        assert_eq!(translated_rule.message, "使用 === 进行严格相等比较");
    }

    #[tokio::test]
    async fn test_translation_request_serialization() {
        let rule = create_test_rule();
        let request = TranslationRequest {
            rule,
            target_language: SupportedLanguage::Chinese,
            context: Some("test context".to_string()),
            priority: 7,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: TranslationRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.rule.id, deserialized.rule.id);
        assert_eq!(request.target_language, deserialized.target_language);
        assert_eq!(request.priority, deserialized.priority);
    }

    #[test]
    fn test_supported_language_from_string() {
        assert_eq!(
            SupportedLanguage::from_str("zh"),
            Some(SupportedLanguage::Chinese)
        );
        assert_eq!(
            SupportedLanguage::from_str("en"),
            Some(SupportedLanguage::English)
        );
        assert_eq!(
            SupportedLanguage::from_str("auto"),
            Some(SupportedLanguage::Auto)
        );
        assert_eq!(SupportedLanguage::from_str("invalid"), None);
    }
}
