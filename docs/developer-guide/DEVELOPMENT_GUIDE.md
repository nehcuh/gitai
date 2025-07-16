# GitAI 开发指南

> 🛠️ **完整的开发环境搭建和开发流程指南**

## 📋 目录

- [开发环境搭建](#开发环境搭建)
- [项目结构详解](#项目结构详解)
- [开发工作流](#开发工作流)
- [调试和测试](#调试和测试)
- [代码规范](#代码规范)
- [架构设计](#架构设计)
- [扩展开发](#扩展开发)
- [性能优化](#性能优化)

## 🚀 开发环境搭建

### 前置要求

| 工具 | 版本 | 用途 |
|------|------|------|
| **Rust** | 1.75+ | 主要编程语言 |
| **Node.js** | 18+ | 前端工具链 |
| **Git** | 2.30+ | 版本控制 |
| **Docker** | 20+ | 容器化开发 |
| **VS Code** | 最新 | 推荐 IDE |

### 开发环境安装

```bash
# 1. 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 安装 Rust 组件
rustup component add clippy rustfmt rust-analyzer
rustup target add wasm32-unknown-unknown  # 可选：WASM 支持

# 3. 安装开发工具
cargo install cargo-watch cargo-nextest cargo-tarpaulin cargo-udeps
cargo install bacon  # 持续构建工具
cargo install cargo-expand  # 宏展开工具

# 4. 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 5. 设置开发环境
make setup-dev  # 或手动执行后续步骤
```

### VS Code 配置

创建 `.vscode/settings.json`：

```json
{
    "rust-analyzer.server.path": "~/.cargo/bin/rust-analyzer",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.completion.autoimport.enable": true,
    "rust-analyzer.inlayHints.parameterHints.enable": true,
    "rust-analyzer.inlayHints.typeHints.enable": true,
    
    "editor.formatOnSave": true,
    "editor.rulers": [100],
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true,
    
    "terminal.integrated.env.osx": {
        "RUST_BACKTRACE": "1"
    },
    "terminal.integrated.env.linux": {
        "RUST_BACKTRACE": "1"
    }
}
```

创建 `.vscode/launch.json`：

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug GitAI",
            "cargo": {
                "args": ["build", "--bin=gitai"],
                "filter": {
                    "name": "gitai",
                    "kind": "bin"
                }
            },
            "args": ["commit", "--verbose"],
            "cwd": "${workspaceFolder}"
        },
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Tests",
            "cargo": {
                "args": ["test", "--no-run"],
                "filter": {
                    "name": "gitai",
                    "kind": "lib"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

推荐扩展：
- **rust-analyzer** - Rust 语言服务器
- **CodeLLDB** - 调试器
- **Error Lens** - 错误高亮
- **GitLens** - Git 增强
- **Thunder Client** - API 测试

### 开发工具配置

#### Makefile

```makefile
# Makefile
.PHONY: setup-dev build test lint clean bench

setup-dev:
	rustup component add clippy rustfmt
	cargo install cargo-watch cargo-nextest
	pre-commit install

build:
	cargo build

build-release:
	cargo build --release

test:
	cargo nextest run

test-coverage:
	cargo tarpaulin --out html --output-dir coverage

lint:
	cargo fmt --check
	cargo clippy -- -D warnings

fix:
	cargo fmt
	cargo clippy --fix

bench:
	cargo bench --bench core_benchmark

clean:
	cargo clean
	rm -rf target/ coverage/

watch:
	cargo watch -x check -x test -x run

dev:
	cargo watch -x 'run -- commit --verbose'
```

#### Pre-commit 配置

创建 `.pre-commit-config.yaml`：

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --
        language: system
        types: [rust]
        
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
        
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        types: [rust]
        pass_filenames: false
```

## 🏗️ 项目结构详解

### 目录结构

```
gitai/
├── src/                        # 源代码
│   ├── main.rs                # 主入口
│   ├── lib.rs                 # 库入口
│   ├── config/                # 配置管理
│   │   ├── mod.rs             # 配置模块
│   │   ├── app.rs             # 应用配置
│   │   └── validation.rs      # 配置验证
│   ├── handlers/              # 命令处理器
│   │   ├── mod.rs             # 处理器模块
│   │   ├── commit.rs          # 提交处理
│   │   ├── review.rs          # 审查处理
│   │   ├── scan.rs            # 扫描处理
│   │   └── translate.rs       # 翻译处理
│   ├── types/                 # 类型定义
│   │   ├── mod.rs             # 类型模块
│   │   ├── common.rs          # 通用类型
│   │   ├── api.rs             # API 类型
│   │   └── config.rs          # 配置类型
│   ├── errors/                # 错误处理
│   │   ├── mod.rs             # 错误模块
│   │   ├── enhanced.rs        # 增强错误处理
│   │   └── utils.rs           # 错误工具
│   ├── utils/                 # 工具函数
│   │   ├── mod.rs             # 工具模块
│   │   ├── git.rs             # Git 工具
│   │   ├── file.rs            # 文件工具
│   │   └── string.rs          # 字符串工具
│   ├── mcp/                   # MCP 服务
│   │   ├── mod.rs             # MCP 模块
│   │   ├── server.rs          # MCP 服务器
│   │   └── services/          # MCP 服务实现
│   │       ├── mod.rs         # 服务模块
│   │       ├── tree_sitter.rs # TreeSitter 服务
│   │       ├── ai_analysis.rs # AI 分析服务
│   │       └── devops.rs      # DevOps 服务
│   ├── tree_sitter_analyzer/  # TreeSitter 分析器
│   │   ├── mod.rs             # 分析器模块
│   │   ├── enhanced.rs        # 增强分析器
│   │   └── languages/         # 语言支持
│   │       ├── mod.rs         # 语言模块
│   │       ├── rust.rs        # Rust 支持
│   │       ├── javascript.rs  # JavaScript 支持
│   │       └── python.rs      # Python 支持
│   └── logging/               # 日志系统
│       ├── mod.rs             # 日志模块
│       └── config.rs          # 日志配置
├── tests/                     # 集成测试
│   ├── integration/           # 集成测试
│   │   ├── mod.rs             # 测试模块
│   │   ├── commit_test.rs     # 提交测试
│   │   └── review_test.rs     # 审查测试
│   └── fixtures/              # 测试数据
│       ├── sample_repo/       # 示例仓库
│       └── config/            # 测试配置
├── benches/                   # 基准测试
│   └── core_benchmark.rs      # 核心基准测试
├── docs/                      # 文档
├── examples/                  # 示例代码
├── assets/                    # 资源文件
├── scripts/                   # 构建脚本
├── Cargo.toml                 # 项目配置
├── Cargo.lock                 # 依赖锁定
├── Makefile                   # 构建脚本
└── README.md                  # 项目说明
```

### 核心模块说明

#### 1. 配置模块 (`src/config/`)

```rust
// src/config/mod.rs
pub mod app;
pub mod validation;

pub use app::AppConfig;
pub use validation::ConfigValidator;

// 配置加载和验证
pub fn load_config(path: Option<&str>) -> Result<AppConfig, ConfigError> {
    let config = AppConfig::load(path)?;
    ConfigValidator::validate(&config)?;
    Ok(config)
}
```

#### 2. 处理器模块 (`src/handlers/`)

```rust
// src/handlers/mod.rs
pub mod commit;
pub mod review;
pub mod scan;
pub mod translate;

pub use commit::CommitHandler;
pub use review::ReviewHandler;
pub use scan::ScanHandler;
pub use translate::TranslateHandler;

// 统一的处理器接口
pub trait CommandHandler {
    type Args;
    type Output;
    
    async fn handle(&self, args: Self::Args) -> Result<Self::Output, AppError>;
}
```

#### 3. 类型模块 (`src/types/`)

```rust
// src/types/mod.rs
pub mod common;
pub mod api;
pub mod config;

pub use common::*;
pub use api::*;
pub use config::*;

// 通用结果类型
pub type Result<T> = std::result::Result<T, AppError>;
```

## 🔄 开发工作流

### 日常开发流程

```bash
# 1. 同步最新代码
git pull origin main

# 2. 创建功能分支
git checkout -b feature/new-functionality

# 3. 开发过程中持续测试
cargo watch -x check -x test

# 4. 提交前检查
make lint
make test
make bench
make build-release

# 5. 提交代码
git add .
git commit -m "feat: add new functionality"

# 6. 推送并创建 PR
git push origin feature/new-functionality
```

### 测试驱动开发

```bash
# 1. 先写测试
cargo test new_feature_test -- --ignored

# 2. 实现功能
cargo watch -x 'test new_feature_test'

# 3. 运行完整测试套件
cargo nextest run

# 4. 检查测试覆盖率
cargo tarpaulin --out html
```

### 代码审查流程

```bash
# 1. 创建 PR 后的自动检查
.github/workflows/ci.yml  # CI/CD 流程

# 2. 本地审查检查
cargo clippy -- -D warnings
cargo fmt --check
cargo audit

# 3. 性能测试
cargo bench --bench core_benchmark
```

## 🧪 调试和测试

### 调试技巧

#### 1. 使用 `dbg!` 宏

```rust
// 调试变量值
let result = some_function();
dbg!(&result);

// 调试表达式
let processed = dbg!(input.trim().to_lowercase());
```

#### 2. 使用 `tracing` 调试

```rust
use tracing::{debug, info, warn, error, span, Level};

#[tracing::instrument]
pub async fn process_commit(args: &CommitArgs) -> Result<String, AppError> {
    let span = span!(Level::INFO, "process_commit", args = ?args);
    let _enter = span.enter();
    
    debug!("Starting commit processing");
    
    // 处理逻辑
    
    info!("Commit processing completed");
    Ok(result)
}
```

#### 3. 使用 GDB/LLDB 调试

```bash
# 编译调试版本
cargo build

# 使用 GDB
gdb target/debug/gitai
(gdb) set args commit --verbose
(gdb) break main
(gdb) run

# 使用 LLDB
lldb target/debug/gitai
(lldb) settings set target.run-args commit --verbose
(lldb) b main
(lldb) run
```

### 测试策略

#### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    fn create_test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            ai: AiConfig {
                model_name: "test-model".to_string(),
                temperature: 0.7,
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    #[tokio::test]
    async fn test_commit_message_generation() {
        let config = create_test_config();
        let handler = CommitHandler::new(config);
        
        let result = handler.generate_message("test diff").await;
        
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(!message.is_empty());
    }
    
    #[test]
    fn test_config_validation() {
        let config = AppConfig::default();
        let result = ConfigValidator::validate(&config);
        assert!(result.is_ok());
    }
}
```

#### 2. 集成测试

```rust
// tests/integration/commit_test.rs
use gitai::handlers::CommitHandler;
use gitai::config::AppConfig;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_commit_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 初始化 Git 仓库
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    
    // 创建测试文件
    std::fs::write(repo_path.join("test.rs"), "fn main() {}").unwrap();
    
    // 添加到暂存区
    Command::new("git")
        .args(["add", "test.rs"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    
    // 测试提交消息生成
    let config = Arc::new(AppConfig::default());
    let handler = CommitHandler::new(config);
    
    let result = handler.generate_message("test diff").await;
    assert!(result.is_ok());
}
```

#### 3. 性能测试

```rust
// benches/core_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_basic_operations(c: &mut Criterion) {
    // 基准测试：基本字符串操作
    c.bench_function("string_clone", |b| {
        let s = "Hello, World!".to_string();
        b.iter(|| {
            let cloned = s.clone();
            black_box(cloned)
        })
    });

    // 基准测试：向量操作
    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..100 {
                v.push(i);
            }
            black_box(v)
        })
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    let sample_code = r#"
    fn hello_world() {
        println!("Hello, world!");
        let x = 42;
        let y = x * 2;
        if y > 50 {
            println!("y is greater than 50");
        }
    }
    "#;

    // 基准测试：字符串处理
    c.bench_function("string_processing", |b| {
        b.iter(|| {
            let lines: Vec<&str> = sample_code.lines().collect();
            let filtered: Vec<&str> = lines.into_iter()
                .filter(|line| !line.trim().is_empty())
                .collect();
            black_box(filtered)
        })
    });
}

criterion_group!(benches, benchmark_basic_operations, benchmark_string_operations);
criterion_main!(benches);
```

### 基准测试配置

在 `Cargo.toml` 中添加基准测试配置：

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "core_benchmark"
harness = false
```

### 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench core_benchmark

# 只编译不运行
cargo bench --no-run

# 生成详细报告
cargo bench -- --verbose
```

**基准测试最佳实践：**

1. **基准测试应该被设计为独立的、可重复的测试**
2. **使用 `black_box` 防止编译器优化掉测试代码**
3. **测试真实的用例场景，而不是微优化**
4. **定期运行基准测试以跟踪性能变化**
5. **在 CI/CD 中集成基准测试，监控性能回归**

**常见问题解决：**

如果遇到 `cargo bench` 编译错误，可能是因为测试代码存在问题。可以：
- 使用 `cargo bench --bench core_benchmark` 只运行特定基准测试
- 使用 `cargo bench --no-run` 只编译不运行
- 检查测试代码是否有编译错误

## 📏 代码规范

### Rust 代码规范

#### 1. 命名规范

```rust
// 好的命名示例
pub struct CommitHandler {
    config: Arc<AppConfig>,
    ai_service: AiService,
}

impl CommitHandler {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            ai_service: AiService::new(),
        }
    }
    
    pub async fn generate_commit_message(&self, diff: &str) -> Result<String, AppError> {
        // 实现逻辑
    }
}

// 常量命名
const MAX_RETRY_COUNT: u32 = 3;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
```

#### 2. 错误处理

```rust
// 好的错误处理
pub fn parse_config(path: &str) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io {
            message: format!("Failed to read config file: {}", path),
            source: e,
        })?;
    
    let config: Config = toml::from_str(&content)
        .map_err(|e| AppError::ConfigParse {
            message: "Invalid TOML format".to_string(),
            source: e,
        })?;
    
    validate_config(&config)?;
    Ok(config)
}

// 避免使用 unwrap() 和 expect()
// 使用 ? 操作符进行错误传播
```

#### 3. 文档注释

```rust
/// 提交处理器
/// 
/// 负责处理 Git 提交相关的操作，包括消息生成和提交执行。
/// 
/// # 示例
/// 
/// ```rust
/// use gitai::handlers::CommitHandler;
/// use gitai::config::AppConfig;
/// use std::sync::Arc;
/// 
/// let config = Arc::new(AppConfig::default());
/// let handler = CommitHandler::new(config);
/// 
/// // 生成提交消息
/// let message = handler.generate_message("diff content").await?;
/// println!("Generated message: {}", message);
/// ```
/// 
/// # 错误
/// 
/// 当 AI 服务不可用时，会返回 [`AppError::AiService`]。
/// 当 Git 操作失败时，会返回 [`AppError::Git`]。
pub struct CommitHandler {
    /// 应用配置
    config: Arc<AppConfig>,
    /// AI 服务实例
    ai_service: AiService,
}

impl CommitHandler {
    /// 创建新的提交处理器
    /// 
    /// # 参数
    /// 
    /// - `config`: 应用配置
    /// 
    /// # 返回
    /// 
    /// 新的提交处理器实例
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            ai_service: AiService::new(),
        }
    }
    
    /// 生成提交消息
    /// 
    /// # 参数
    /// 
    /// - `diff`: Git 差异内容
    /// 
    /// # 返回
    /// 
    /// 生成的提交消息
    /// 
    /// # 错误
    /// 
    /// 当 AI 服务请求失败时返回错误
    pub async fn generate_message(&self, diff: &str) -> Result<String, AppError> {
        // 实现逻辑
        Ok(String::new())
    }
}
```

### 代码格式化

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check

# 自动修复 clippy 警告
cargo clippy --fix

# 严格检查
cargo clippy -- -D warnings
```

## 🏛️ 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
├─────────────────────────────────────────────────────────────┤
│  Command Handlers                                           │
│  ├── CommitHandler                                          │
│  ├── ReviewHandler                                          │
│  ├── ScanHandler                                            │
│  └── TranslateHandler                                       │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                     Service Layer                           │
├─────────────────────────────────────────────────────────────┤
│  Core Services                                              │
│  ├── AiService                                              │
│  ├── GitService                                             │
│  ├── DevOpsService                                          │
│  └── SecurityService                                        │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                      MCP Layer                              │
├─────────────────────────────────────────────────────────────┤
│  MCP Services                                               │
│  ├── TreeSitterService                                     │
│  ├── AiAnalysisService                                     │
│  └── DevOpsIntegrationService                              │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                      │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure                                             │
│  ├── Configuration                                          │
│  ├── Error Handling                                         │
│  ├── Logging                                                │
│  └── Utilities                                              │
└─────────────────────────────────────────────────────────────┘
```

### 服务接口设计

```rust
// 服务接口定义
#[async_trait]
pub trait AiService {
    async fn generate_commit_message(&self, diff: &str) -> Result<String, AiError>;
    async fn analyze_code(&self, code: &str) -> Result<CodeAnalysis, AiError>;
    async fn review_changes(&self, changes: &[Change]) -> Result<ReviewResult, AiError>;
}

#[async_trait]
pub trait GitService {
    fn get_staged_diff(&self) -> Result<String, GitError>;
    fn commit(&self, message: &str) -> Result<(), GitError>;
    fn get_repository_info(&self) -> Result<RepoInfo, GitError>;
}

#[async_trait]
pub trait DevOpsService {
    async fn get_work_items(&self, space_id: &str) -> Result<Vec<WorkItem>, DevOpsError>;
    async fn update_work_item(&self, id: &str, update: &WorkItemUpdate) -> Result<(), DevOpsError>;
}
```

## 🔧 扩展开发

### 添加新的命令处理器

```rust
// src/handlers/analyze.rs
use crate::config::AppConfig;
use crate::errors::AppError;
use crate::types::AnalyzeArgs;
use async_trait::async_trait;
use std::sync::Arc;

pub struct AnalyzeHandler {
    config: Arc<AppConfig>,
}

impl AnalyzeHandler {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CommandHandler for AnalyzeHandler {
    type Args = AnalyzeArgs;
    type Output = String;
    
    async fn handle(&self, args: Self::Args) -> Result<Self::Output, AppError> {
        // 实现分析逻辑
        Ok("Analysis completed".to_string())
    }
}
```

### 添加新的 MCP 服务

```rust
// src/mcp/services/code_metrics.rs
use crate::mcp::McpService;
use crate::types::*;
use rmcp::*;
use serde_json::Value;

pub struct CodeMetricsService {
    // 服务状态
}

impl CodeMetricsService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl McpService for CodeMetricsService {
    fn name(&self) -> &'static str {
        "code_metrics"
    }
    
    async fn get_tools(&self) -> Result<Vec<Tool>, McpError> {
        Ok(vec![
            Tool {
                name: "calculate_metrics".to_string(),
                description: "Calculate code metrics".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string"},
                        "language": {"type": "string"}
                    },
                    "required": ["code", "language"]
                }),
            }
        ])
    }
    
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value, McpError> {
        match name {
            "calculate_metrics" => {
                // 实现指标计算逻辑
                Ok(serde_json::json!({
                    "lines_of_code": 100,
                    "complexity": 5,
                    "maintainability_index": 80
                }))
            }
            _ => Err(McpError::ToolNotFound(name.to_string()))
        }
    }
}
```

### 添加新的语言支持

```rust
// src/tree_sitter_analyzer/languages/go.rs
use tree_sitter::{Language, Parser};
use crate::tree_sitter_analyzer::LanguageAnalyzer;

extern "C" {
    fn tree_sitter_go() -> Language;
}

pub struct GoAnalyzer;

impl LanguageAnalyzer for GoAnalyzer {
    fn language() -> Language {
        unsafe { tree_sitter_go() }
    }
    
    fn file_extensions() -> &'static [&'static str] {
        &["go"]
    }
    
    fn analyze_specific_patterns(&self, code: &str) -> Vec<Pattern> {
        // Go 特定的模式分析
        vec![]
    }
}
```

## ⚡ 性能优化

### 异步优化

```rust
// 并发处理
use tokio::task::JoinSet;

pub async fn process_files_concurrently(files: Vec<String>) -> Result<Vec<ProcessResult>, AppError> {
    let mut set = JoinSet::new();
    
    for file in files {
        set.spawn(async move {
            process_single_file(file).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result??);
    }
    
    Ok(results)
}

// 流式处理
use futures::stream::{self, StreamExt};

pub async fn process_files_stream(files: Vec<String>) -> Result<Vec<ProcessResult>, AppError> {
    let results: Result<Vec<_>, _> = stream::iter(files)
        .map(|file| process_single_file(file))
        .buffer_unordered(10)  // 限制并发数
        .collect()
        .await;
    
    results
}
```

### 缓存优化

```rust
// 使用 LRU 缓存
use lru::LruCache;
use std::sync::Mutex;

pub struct CachedAnalyzer {
    cache: Mutex<LruCache<String, AnalysisResult>>,
}

impl CachedAnalyzer {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }
    
    pub async fn analyze(&self, code: &str) -> Result<AnalysisResult, AppError> {
        let key = format!("{:x}", md5::compute(code));
        
        // 检查缓存
        if let Some(cached) = self.cache.lock().unwrap().get(&key) {
            return Ok(cached.clone());
        }
        
        // 执行分析
        let result = self.perform_analysis(code).await?;
        
        // 缓存结果
        self.cache.lock().unwrap().put(key, result.clone());
        
        Ok(result)
    }
}
```

### 内存优化

```rust
// 使用 Arc 共享数据
use std::sync::Arc;

pub struct SharedConfig {
    config: Arc<AppConfig>,
}

impl SharedConfig {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
    
    pub fn get_config(&self) -> Arc<AppConfig> {
        Arc::clone(&self.config)
    }
}

// 使用 Cow 避免不必要的克隆
use std::borrow::Cow;

pub fn process_text(input: &str) -> Cow<str> {
    if input.contains("need_processing") {
        // 需要处理时才克隆
        Cow::Owned(input.replace("need_processing", "processed"))
    } else {
        // 不需要处理时借用
        Cow::Borrowed(input)
    }
}
```

## 📊 开发工具和脚本

### 开发脚本

```bash
#!/bin/bash
# scripts/dev.sh - 开发辅助脚本

set -e

case "$1" in
    "setup")
        echo "Setting up development environment..."
        rustup component add clippy rustfmt
        cargo install cargo-watch cargo-nextest
        ;;
    "test")
        echo "Running tests..."
        cargo nextest run
        ;;
    "coverage")
        echo "Generating coverage report..."
        cargo tarpaulin --out html --output-dir coverage
        ;;
    "release")
        echo "Building release..."
        cargo build --release
        ;;
    *)
        echo "Usage: $0 {setup|test|coverage|release}"
        exit 1
        ;;
esac
```

### 性能分析脚本

```bash
#!/bin/bash
# scripts/profile.sh - 性能分析脚本

# 构建 release 版本
cargo build --release

# 生成火焰图
cargo flamegraph --bin gitai -- commit --verbose

# 运行基准测试
cargo bench --bench core_benchmark

# 内存分析
valgrind --tool=massif target/release/gitai commit --verbose
```

---

**🎯 现在您已经掌握了 GitAI 的完整开发流程！**

开始您的开发之旅，为 GitAI 添加新功能，改进现有代码，让它变得更加强大和实用！