# GitAI 开发指南

## 项目概述

GitAI 是一个基于 Rust 开发的 AI 驱动的 Git 工具套件，专注于提供智能代码审查、自动化提交消息生成和增强的 Git 命令解释功能。

## 核心架构

### 模块组织

```
src/
├── main.rs                   # CLI 入口点
├── lib.rs                    # 库导出
├── cli/                      # CLI 相关功能
├── git/                      # Git核心操作（项目基础）
├── ai/                       # AI能力模块（横向支持）
├── review/                   # 代码审查功能（Git + AI）
├── analysis/                 # 代码分析模块（Git + AST-Grep）
├── translation/              # 全局翻译功能（AI驱动）
├── devops/                   # DevOps 集成（Git扩展）
├── config/                   # 配置管理
└── common/                   # 通用功能
```

### 关键特性

1. **Git 核心**: 以 Git 操作为基础，扩展智能能力
2. **AI 驱动**: 横向AI模块为各功能提供智能支持
3. **多语言支持**: 全局翻译系统支持中英文界面
4. **AST-Grep 集成**: 基于 AST-Grep 的代码分析
5. **DevOps 集成**: 扩展 Git 工作流与 DevOps 平台集成
6. **模块化设计**: 清晰的职责分离和依赖关系

## 开发环境设置

### 必要依赖

```bash
# 安装 Rust (推荐使用 rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装额外工具
cargo install cargo-watch cargo-udeps
```

### 项目设置

```bash
# 克隆项目
git clone <repository-url>
cd gitai

# 构建项目
cargo build

# 运行测试
cargo test

# 开发模式运行
RUST_LOG=debug cargo run -- help
```

## 开发工作流

### 1. 代码风格

```bash
# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy

# 检查未使用的依赖
cargo +nightly udeps
```

### 2. 测试策略

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行测试并显示输出
cargo test -- --nocapture

# 运行集成测试
cargo test --test integration_commit_test
```

### 3. 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- review

# 特定模块日志
RUST_LOG=gitai::handlers::commit=debug cargo run -- commit
```

## 核心模块开发指南

### 1. CLI 模块 (`src/cli/`)

**职责**: 处理命令行参数解析和用户交互

```rust
// 示例: 添加新的 CLI 命令
pub struct NewCommand {
    pub arg1: String,
    pub arg2: Option<String>,
}

impl NewCommand {
    pub fn execute(&self, config: &AppConfig) -> Result<(), AppError> {
        // 实现命令逻辑
    }
}
```

### 2. Git 模块 (`src/git/`) - 项目基础

**职责**: 提供所有Git基础操作，是项目的核心基础

```rust
// 示例: 添加新的 Git 操作
pub fn new_git_operation(repo_path: &Path) -> Result<GitResult, GitError> {
    // 实现 Git 操作
}

// Git 差异分析
pub fn analyze_diff(repo_path: &Path) -> Result<GitDiff, GitError> {
    // 实现差异分析
}
```

### 3. AI 模块 (`src/ai/`) - 横向能力模块

**职责**: 为其他功能模块提供AI支持

```rust
// 示例: 添加新的 AI 功能
pub async fn new_ai_feature(client: &AIClient, input: &str) -> Result<AIResponse, AIError> {
    let messages = vec![ChatMessage::user(input)];
    client.chat(messages).await
}

// AI 分析接口
pub trait AIAnalyzer {
    async fn analyze(&self, content: &str) -> Result<AnalysisResult, AIError>;
}
```

### 4. Review 模块 (`src/review/`) - Git + AI + AST-Grep

**职责**: 智能代码审查功能

```rust
// 示例: 实现代码审查
pub struct ReviewEngine {
    git_ops: GitOperations,
    ai_client: AIClient,
    ast_analyzer: AstGrepAnalyzer,
}

impl ReviewEngine {
    pub async fn review_changes(&self, repo_path: &Path) -> Result<ReviewReport, ReviewError> {
        let diff = self.git_ops.get_diff(repo_path)?;
        let ast_analysis = self.ast_analyzer.analyze(&diff)?;
        let ai_insights = self.ai_client.analyze_code(&diff).await?;
        // 生成审查报告
    }
}
```

### 5. 分析模块 (`src/analysis/`) - Git + AST-Grep

**职责**: 代码分析功能

```rust
// 示例: 添加新的分析规则
pub fn analyze_with_new_rule(code: &str, language: &str) -> Result<AnalysisResult, AnalysisError> {
    // 实现分析逻辑
}
```

### 6. 翻译模块 (`src/translation/`) - 全局AI驱动

**职责**: 全局翻译功能，使用 AI 提示词

```rust
// 示例: 使用翻译服务
let translator = Translator::new(ai_client, config)?;
let result = translator.translate("Hello", SupportedLanguage::Chinese).await?;
```

### 7. DevOps 模块 (`src/devops/`) - Git工作流扩展

**职责**: DevOps平台集成，扩展Git工作流

```rust
// 示例: DevOps集成
pub struct DevOpsIntegration {
    git_ops: GitOperations,
    platform_client: DevOpsClient,
}

impl DevOpsIntegration {
    pub async fn sync_work_items(&self, repo_path: &Path) -> Result<(), DevOpsError> {
        let commits = self.git_ops.get_recent_commits(repo_path)?;
        self.platform_client.update_work_items(commits).await?;
        Ok(())
    }
}
```

## 配置管理

### 配置文件结构

```toml
[ai]
api_url = "http://localhost:1234/v1/chat/completions"
model_name = "model-name"
temperature = 0.7

[translation]
enabled = true
use_ai = true
default_language = "auto"

[ast_grep]
enabled = true
analysis_depth = "medium"
```

### 配置加载

```rust
// 加载配置
let config = AppConfig::load()?;

// 使用配置
if config.translation.enabled {
    // 执行翻译相关操作
}
```

## 错误处理

### 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Git 操作错误: {0}")]
    Git(#[from] GitError),
    
    #[error("AI 服务错误: {0}")]
    AI(#[from] AIError),
}
```

### 错误处理最佳实践

```rust
// 使用 Result 类型
pub fn risky_operation() -> Result<String, AppError> {
    // 可能失败的操作
    let result = some_operation()
        .map_err(|e| AppError::Custom(format!("操作失败: {}", e)))?;
    Ok(result)
}

// 错误上下文
use thiserror::Error;

#[derive(Error, Debug)]
#[error("处理文件 {file} 时发生错误")]
pub struct FileError {
    file: String,
    #[source]
    source: std::io::Error,
}
```

## 测试指南

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_function_name() {
        let temp_dir = TempDir::new().unwrap();
        let result = function_to_test(temp_dir.path());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function_to_test().await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

```rust
// tests/integration_test.rs
use gitai::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    // 设置测试环境
    // 执行完整工作流
    // 验证结果
}
```

### Mock 和测试工具

```rust
// 使用 httpmock 进行 HTTP 测试
use httpmock::prelude::*;

#[tokio::test]
async fn test_api_call() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/api/test");
        then.status(200).json_body(json!({"result": "success"}));
    });

    // 执行测试
    mock.assert();
}
```

## 性能优化

### 1. 异步编程

```rust
// 使用 tokio 进行异步操作
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let tasks = vec![
        tokio::spawn(async_task_1()),
        tokio::spawn(async_task_2()),
    ];
    
    let results = futures::future::join_all(tasks).await;
    // 处理结果
}
```

### 2. 缓存策略

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V> 
where 
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub async fn get_or_insert<F, Fut>(&self, key: K, factory: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = V>,
    {
        // 缓存实现
    }
}
```

### 3. 内存管理

```rust
// 使用 Arc 进行共享所有权
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedResource {
    data: Arc<ExpensiveData>,
}

// 使用 Cow 进行写时复制
use std::borrow::Cow;

pub fn process_data(data: Cow<str>) -> String {
    // 处理数据
}
```

## 贡献指南

### 1. 分支策略

```bash
# 创建功能分支
git checkout -b feat/new-feature

# 提交更改
git commit -m "feat: 添加新功能"

# 推送分支
git push origin feat/new-feature
```

### 2. 提交消息规范

```
feat: 添加新功能
fix: 修复 bug
docs: 更新文档
style: 代码格式化
refactor: 重构代码
test: 添加测试
chore: 其他更改
```

### 3. 代码审查清单

- [ ] 代码风格符合项目规范
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 错误处理得当
- [ ] 性能考虑合理

## 故障排除

### 常见问题

1. **编译错误**: 检查 Rust 版本和依赖
2. **测试失败**: 确保环境配置正确
3. **性能问题**: 使用分析工具定位瓶颈
4. **配置问题**: 检查配置文件格式

### 调试技巧

```bash
# 使用 rust-lldb 调试
rust-lldb target/debug/gitai

# 使用 perf 分析性能
perf record target/release/gitai
perf report
```

## 发布流程

### 1. 版本发布

```bash
# 更新版本号
cargo bump minor

# 创建发布标签
git tag -a v0.2.0 -m "Release v0.2.0"

# 构建发布版本
cargo build --release

# 推送标签
git push origin v0.2.0
```

### 2. 文档更新

```bash
# 生成 API 文档
cargo doc --no-deps --open

# 更新 README
# 更新 CHANGELOG
```

## 资源链接

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 文档](https://docs.rs/tokio/)
- [AST-Grep 文档](https://ast-grep.github.io/)
- [项目 Issues](https://github.com/your-repo/issues)

---

这个开发指南将帮助新的开发者快速上手 GitAI 项目，并提供了完整的开发工作流程指导。