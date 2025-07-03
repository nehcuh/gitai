# 翻译模块重构设计文档

## 当前问题分析

### 1. 架构问题
- 翻译功能嵌入在 `ast_grep_analyzer/translation/` 中，职责不清
- 翻译服务与AST分析耦合，难以在其他模块中复用
- 缺乏统一的翻译配置管理

### 2. 实现问题
- 未充分利用已有的 `translator.md` 提示词
- 翻译实现与项目整体AI架构不一致
- 缺乏统一的错误处理和日志记录

## 新的翻译架构设计

### 1. 模块结构
```
src/translation/
├── mod.rs              # 模块导出
├── translator.rs       # 核心翻译器
├── prompts.rs          # 提示词管理
├── cache.rs            # 翻译缓存
├── config.rs           # 翻译配置
└── types.rs            # 翻译相关类型
```

### 2. 核心组件

#### 2.1 翻译器 (Translator)
```rust
pub struct Translator {
    ai_client: Arc<dyn AIClient>,
    cache: Arc<TranslationCache>,
    config: TranslationConfig,
    prompt_manager: PromptManager,
}

impl Translator {
    pub async fn translate(&self, content: &str, target_lang: SupportedLanguage) -> Result<TranslationResult, TranslationError>;
    pub async fn translate_with_context(&self, content: &str, context: &str, target_lang: SupportedLanguage) -> Result<TranslationResult, TranslationError>;
    pub async fn auto_translate(&self, content: &str) -> Result<TranslationResult, TranslationError>;
}
```

#### 2.2 提示词管理 (PromptManager)
```rust
pub struct PromptManager {
    base_prompt: String,
    templates: HashMap<String, String>,
}

impl PromptManager {
    pub fn load_from_file(path: &Path) -> Result<Self, PromptError>;
    pub fn build_prompt(&self, content: &str, target_lang: SupportedLanguage, context: Option<&str>) -> String;
    pub fn reload_prompts(&mut self) -> Result<(), PromptError>;
}
```

#### 2.3 翻译缓存 (TranslationCache)
```rust
pub struct TranslationCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl TranslationCache {
    pub fn get(&self, key: &str) -> Option<TranslationResult>;
    pub fn set(&self, key: &str, value: TranslationResult);
    pub fn clear_expired(&self);
}
```

### 3. 类型定义

#### 3.1 支持的语言
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLanguage {
    Chinese,
    English,
    Auto,
}
```

#### 3.2 翻译结果
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub literal_translation: String,
    pub evaluation: String,
    pub free_translation: String,
    pub detected_language: SupportedLanguage,
    pub target_language: SupportedLanguage,
    pub confidence: f32,
}
```

#### 3.3 翻译配置
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub use_ai: bool,
    pub default_language: SupportedLanguage,
    pub cache_enabled: bool,
    pub cache_ttl: u64,
    pub prompt_file: PathBuf,
    pub max_content_length: usize,
    pub timeout: Duration,
}
```

## 与AI配置的集成

### 1. 统一AI客户端
翻译器将使用统一的AI客户端接口，复用项目中的AI配置：

```rust
pub trait AIClient: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, AIError>;
    async fn translate(&self, prompt: &str, content: &str) -> Result<String, AIError>;
}
```

### 2. 配置继承
翻译配置将继承AI配置中的关键参数：
- API端点和密钥
- 超时设置
- 重试策略
- 模型参数

## 使用示例

### 1. 基本翻译
```rust
let translator = Translator::new(ai_client, config)?;
let result = translator.translate("Hello, world!", SupportedLanguage::Chinese).await?;
println!("翻译结果: {}", result.free_translation);
```

### 2. 自动语言检测
```rust
let result = translator.auto_translate("这是一个测试").await?;
println!("检测到的语言: {:?}", result.detected_language);
println!("翻译结果: {}", result.free_translation);
```

### 3. 带上下文的翻译
```rust
let context = "这是一个Git提交消息";
let result = translator.translate_with_context(
    "fix: resolve memory leak in parser",
    context,
    SupportedLanguage::Chinese
).await?;
```

## 集成点

### 1. 命令行界面
- 全局 `--lang` 参数将使用新的翻译系统
- 所有命令的输出都可以通过翻译系统处理

### 2. 其他模块集成
- Review模块: 审查结果翻译
- Commit模块: 提交消息翻译
- AI分析结果: 分析报告翻译
- 错误消息: 错误信息翻译

## 迁移计划

### Phase 1: 基础架构
1. 创建新的翻译模块结构
2. 实现核心翻译器接口
3. 集成AI客户端

### Phase 2: 功能迁移
1. 从现有ast_grep翻译模块迁移功能
2. 更新配置管理
3. 实现缓存机制

### Phase 3: 全局集成
1. 更新所有模块以使用新的翻译服务
2. 实现CLI翻译支持
3. 优化性能和错误处理

## 性能考虑

### 1. 缓存策略
- 基于内容哈希的缓存键
- TTL过期机制
- 内存使用限制

### 2. 异步处理
- 非阻塞翻译API调用
- 批量翻译支持
- 连接池管理

### 3. 错误处理
- 网络错误重试
- 降级策略(缓存备份)
- 优雅的错误报告

## 测试策略

### 1. 单元测试
- 翻译器核心功能
- 缓存机制
- 提示词管理

### 2. 集成测试
- AI客户端集成
- 配置加载
- 错误场景

### 3. 性能测试
- 并发翻译请求
- 缓存命中率
- 内存使用情况

## 文档更新

### 1. API文档
- 翻译器接口文档
- 配置参数说明
- 使用示例

### 2. 用户文档
- 翻译功能使用指南
- 配置文件更新指南
- 故障排除文档

这个重构设计将使翻译功能更加模块化、可复用，并与项目的整体架构保持一致。