# 🌐 GitAI Translation System Architecture

This document provides a technical overview of the GitAI translation system architecture, including component design, data flow, and implementation details.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Components](#core-components)
3. [Translation Flow](#translation-flow)
4. [Cache System](#cache-system)
5. [Performance Considerations](#performance-considerations)
6. [Adding New Languages](#adding-new-languages)
7. [Translation Provider Interface](#translation-provider-interface)
8. [Technical Implementation Details](#technical-implementation-details)

## Architecture Overview

The GitAI translation system follows a modular design with the following high-level components:

- **TranslationManager**: Central coordinator that handles language detection, provider selection, and translation operations
- **RuleLocalizer**: Specialized component for translating AST-Grep rules and pattern-based content
- **CacheManager**: Handles caching of translation results to improve performance
- **Translator**: Interface implemented by different translation providers (OpenAI, etc.)
- **SupportedLanguage**: Enum representing the supported languages (currently `En`, `Zh`, and `Auto`)

![Translation System Architecture](../assets/translation-architecture.png)

## Core Components

### TranslationManager

The `TranslationManager` is the main entry point for the translation system:

```rust
pub struct TranslationManager {
    config: TranslationConfig,
    target_language: SupportedLanguage,
    initialized: bool,
    // Implementation details omitted
}
```

Key responsibilities:
- Initializes the translation system based on configuration
- Resolves the target language (from CLI args, environment, or config)
- Routes translation requests to appropriate handlers
- Manages translation resources and lifecycle

### RuleLocalizer

The `RuleLocalizer` specializes in translating AST-Grep rules and code analysis results:

```rust
pub struct RuleLocalizer {
    cache: Arc<Mutex<HashMap<String, String>>>,
    target_language: SupportedLanguage,
    // Implementation details omitted
}
```

Key responsibilities:
- Translates rule metadata (name, description, message)
- Preserves pattern syntax during translation
- Handles code analysis result localization

### CacheManager

The `CacheManager` provides caching facilities to reduce translation API calls:

```rust
pub struct TranslationCacheManager {
    cache_dir: PathBuf,
    enabled: bool,
    // Implementation details omitted
}
```

Key responsibilities:
- Stores and retrieves translations from persistent cache
- Implements cache invalidation strategies
- Provides thread-safe cache access

### Translator

The `Translator` trait defines the interface for translation providers:

```rust
pub trait Translator: Send + Sync {
    fn translate(
        &self, 
        text: &str, 
        source_lang: &SupportedLanguage, 
        target_lang: &SupportedLanguage
    ) -> TranslationResult<String>;
    
    // Additional methods omitted
}
```

## Translation Flow

The translation process follows these steps:

1. **Initialization**:
   - Load configuration from config file and environment variables
   - Determine target language (CLI arg > env var > config > system locale)
   - Initialize translation providers and cache

2. **Request Processing**:
   - Incoming text is checked against cache
   - If cache miss, route to appropriate translation provider
   - Apply any specialized translation rules (for code, patterns, etc.)
   - Store results in cache

3. **Result Integration**:
   - Translated content is integrated into command output
   - Format-specific handlers ensure proper rendering in different outputs (text, JSON, etc.)

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Command    │    │ Translation │    │ Translation │
│  Handler    │───▶│   Manager   │───▶│   Provider  │
└─────────────┘    └─────────────┘    └─────────────┘
                         │                    │
                         ▼                    ▼
                  ┌─────────────┐    ┌─────────────┐
                  │    Cache    │    │  Formatted  │
                  │   Manager   │    │   Output    │
                  └─────────────┘    └─────────────┘
```

## Cache System

The translation cache system is designed for efficiency and persistence:

- **Storage Format**: JSON files organized by language pair and content hash
- **Cache Key Generation**: SHA-256 hash of source text + context
- **Directory Structure**:
  ```
  ~/.cache/gitai/translation/
  ├── en-zh/                  # English to Chinese translations
  │   ├── <hash1>.json
  │   ├── <hash2>.json
  │   └── ...
  ├── zh-en/                  # Chinese to English translations
  │   ├── <hash3>.json
  │   └── ...
  └── metadata.json           # Cache metadata and stats
  ```

- **Cache Entry Format**:
  ```json
  {
    "source_text": "Original text content",
    "translated_text": "Translated content",
    "source_language": "en",
    "target_language": "zh",
    "timestamp": 1623456789,
    "provider": "openai",
    "model": "gpt-3.5-turbo"
  }
  ```

- **Invalidation Strategy**: 
  - Time-based expiration (configurable)
  - Version-based invalidation on major updates
  - Manual invalidation via `--force-scan`

## Performance Considerations

The translation system is designed with performance in mind:

- **Measured Impact**: Translation adds approximately 5ms overhead for scanning 36 files when using cache
- **Batch Translation**: Groups similar content to reduce API calls
- **Asynchronous Translation**: Non-blocking translation for improved UI responsiveness
- **Cache Warming**: Preemptive translation of common messages
- **Content Filtering**: Only translates user-facing content, preserves code and syntax

## Adding New Languages

To add support for a new language:

1. Update the `SupportedLanguage` enum in `translation/mod.rs`:
   ```rust
   pub enum SupportedLanguage {
       En,
       Zh,
       // Add new language here
       Es,  // Example: Spanish
       Auto,
   }
   ```

2. Add language detection logic in `resolve_target_language()`:
   ```rust
   fn resolve_target_language(lang_code: &str) -> SupportedLanguage {
       match lang_code.to_lowercase().as_str() {
           "en" => SupportedLanguage::En,
           "zh" => SupportedLanguage::Zh,
           "es" => SupportedLanguage::Es,  // Add mapping here
           "auto" => detect_system_language(),
           _ => SupportedLanguage::En,  // Default
       }
   }
   ```

3. Update translation providers to support the new language pair

## Translation Provider Interface

Custom translation providers must implement the `Translator` trait:

```rust
pub trait Translator: Send + Sync {
    fn translate(
        &self, 
        text: &str, 
        source_lang: &SupportedLanguage, 
        target_lang: &SupportedLanguage
    ) -> TranslationResult<String>;
    
    fn batch_translate(
        &self,
        texts: &[String],
        source_lang: &SupportedLanguage,
        target_lang: &SupportedLanguage
    ) -> TranslationResult<Vec<String>> {
        // Default implementation uses individual translate calls
        // Override for better performance
    }
    
    fn name(&self) -> &str;
    
    fn supports_language_pair(
        &self,
        source_lang: &SupportedLanguage,
        target_lang: &SupportedLanguage
    ) -> bool;
}
```

Provider registration:

```rust
pub fn register_provider(&mut self, provider: Box<dyn Translator>) -> TranslationResult<()> {
    let provider_name = provider.name().to_string();
    self.providers.insert(provider_name.clone(), provider);
    Ok(())
}
```

## Technical Implementation Details

### Error Handling

The translation system uses a specialized error type:

```rust
pub enum TranslationError {
    ConfigError(String),
    ProviderError(String),
    CacheError(String),
    UnsupportedLanguage(String),
    NetworkError(String),
    // Other error variants...
}

pub type TranslationResult<T> = Result<T, TranslationError>;
```

### Thread Safety

Translation components are designed for concurrent use:

- `TranslationManager` is thread-safe and can be shared across command handlers
- Cache access is protected by `Arc<Mutex<T>>` for concurrent read/write operations
- Translation providers must implement `Send + Sync` traits

### Configuration Schema

The translation configuration schema:

```toml
[translation]
enabled = true
default_language = "zh"  # zh|en|auto
cache_enabled = true
provider = "openai"
cache_dir = "~/.cache/gitai/translation"
cache_max_age_days = 30  # Optional
cache_max_size_mb = 100  # Optional

[translation.provider_settings]
api_key = "your-translation-api-key"
model = "gpt-3.5-turbo"
timeout_seconds = 10     # Optional
max_retries = 3          # Optional
endpoint = "https://custom-translation-api.example.com/v1/translate"  # Optional
```

### Metrics and Telemetry

The translation system collects the following performance metrics:

- Translation request count by language pair
- Cache hit/miss ratio
- Average translation latency
- Total translation volume
- API errors by provider

These metrics can be viewed with the `--translation-perf-stats` flag.

---

## Advanced Usage

### Custom Translation Rules

For specialized domain-specific translations, custom rules can be defined:

```toml
[translation.custom_rules]
"performance issue" = "性能问题"
"security vulnerability" = "安全漏洞"
"code smell" = "代码异味"
```

### Plugin Architecture

The translation system supports a plugin architecture for extension:

```rust
pub trait TranslationPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&mut self, config: &TranslationConfig) -> TranslationResult<()>;
    fn pre_translate(&self, text: &str) -> TranslationResult<String>;
    fn post_translate(&self, text: &str) -> TranslationResult<String>;
}
```

Custom plugins can be registered to perform pre/post-processing on translation text.

---

This document provides a technical overview of the GitAI translation system architecture. For implementation details, refer to the source code in the `src/ast_grep_analyzer/translation/` directory.