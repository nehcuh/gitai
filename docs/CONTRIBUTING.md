# GitAI 贡献指南

> 🤝 **欢迎参与 GitAI 项目开发！**

感谢您考虑为 GitAI 做出贡献！本指南将帮助您了解如何参与项目开发，从环境搭建到提交代码的完整流程。

## 📋 目录

- [快速开始](#快速开始)
- [开发环境搭建](#开发环境搭建)
- [项目结构](#项目结构)
- [开发规范](#开发规范)
- [贡献流程](#贡献流程)
- [测试指南](#测试指南)
- [文档编写](#文档编写)
- [发布流程](#发布流程)

## 🚀 快速开始

### 贡献类型

欢迎以下类型的贡献：

- 🐛 **Bug 报告和修复**
- ✨ **新功能开发**
- 📝 **文档改进**
- 🔧 **代码优化**
- 🧪 **测试覆盖**
- 🌐 **国际化支持**

### 贡献前检查

在开始贡献之前，请确保：

- [ ] 阅读并理解 [行为准则](CODE_OF_CONDUCT.md)
- [ ] 搜索现有的 [Issues](https://github.com/your-org/gitai/issues) 和 [Pull Requests](https://github.com/your-org/gitai/pulls)
- [ ] 对于重大变更，先创建 Issue 讨论
- [ ] 确保您的贡献符合项目目标

## 🛠️ 开发环境搭建

### 前置要求

| 软件 | 版本要求 | 安装指南 |
|------|----------|----------|
| **Rust** | 1.70.0+ | [rustup.rs](https://rustup.rs/) |
| **Git** | 2.20.0+ | [git-scm.com](https://git-scm.com/) |
| **Node.js** | 16.0.0+ | [nodejs.org](https://nodejs.org/) |
| **Docker** | 20.0.0+ | [docker.com](https://www.docker.com/) |

### 开发环境设置

```bash
# 1. 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 2. 设置 Rust 工具链
rustup toolchain install stable
rustup default stable
rustup component add clippy rustfmt

# 3. 安装开发依赖
cargo install cargo-watch cargo-nextest cargo-tarpaulin

# 4. 安装 pre-commit 钩子
pip install pre-commit
pre-commit install

# 5. 验证环境
cargo check
cargo test
```

### 推荐工具

- **IDE**: VS Code + Rust Analyzer
- **Git**: GitHub CLI (`gh`)
- **调试**: `gdb` 或 `lldb`
- **性能分析**: `perf`, `valgrind`

### VS Code 配置

创建 `.vscode/settings.json`：

```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.allTargets": false,
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true,
    "editor.rulers": [100],
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true
}
```

推荐扩展：
- Rust Analyzer
- CodeLLDB
- GitLens
- Markdown All in One

## 🏗️ 项目结构

```
gitai/
├── src/                    # 源代码
│   ├── main.rs            # 主入口
│   ├── lib.rs             # 库入口
│   ├── config/            # 配置管理
│   ├── handlers/          # 命令处理器
│   │   ├── commit/        # 提交相关
│   │   ├── review/        # 审查相关
│   │   └── scan/          # 扫描相关
│   ├── types/             # 类型定义
│   ├── errors/            # 错误处理
│   ├── utils/             # 工具函数
│   ├── mcp/               # MCP 服务
│   └── tree_sitter_analyzer/  # TreeSitter 分析
├── docs/                   # 文档
├── tests/                  # 集成测试
├── examples/               # 示例代码
├── assets/                 # 资源文件
├── scripts/                # 构建脚本
└── Cargo.toml             # 项目配置
```

### 核心模块说明

| 模块 | 职责 | 入口文件 |
|------|------|----------|
| **handlers** | 命令处理逻辑 | `src/handlers/mod.rs` |
| **config** | 配置管理 | `src/config/mod.rs` |
| **types** | 类型定义 | `src/types/mod.rs` |
| **errors** | 错误处理 | `src/errors/mod.rs` |
| **mcp** | MCP 服务 | `src/mcp/mod.rs` |
| **tree_sitter_analyzer** | 代码分析 | `src/tree_sitter_analyzer/mod.rs` |

## 📝 开发规范

### 代码风格

我们使用 `rustfmt` 和 `clippy` 来保持代码风格的一致性。

#### Rust 代码规范

```rust
// 好的示例
pub struct CommitHandler {
    config: Arc<AppConfig>,
    git_service: GitService,
}

impl CommitHandler {
    /// 创建新的提交处理器
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            git_service: GitService::new(),
        }
    }

    /// 生成提交消息
    pub async fn generate_message(&self, diff: &str) -> Result<String, AppError> {
        // 实现逻辑
        Ok(String::new())
    }
}
```

#### 代码风格要点

- 使用 4 个空格缩进
- 行长度限制为 100 字符
- 使用 `snake_case` 命名变量和函数
- 使用 `PascalCase` 命名结构体和枚举
- 添加详细的文档注释

### 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| **函数** | snake_case | `generate_commit_message` |
| **变量** | snake_case | `user_input` |
| **结构体** | PascalCase | `CommitHandler` |
| **枚举** | PascalCase | `ErrorType` |
| **常量** | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| **模块** | snake_case | `git_handler` |

### 错误处理

```rust
use crate::errors::{AppError, ErrorMessage, ErrorSeverity, ErrorCategory};

// 好的错误处理
pub fn parse_config(path: &str) -> Result<Config, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Enhanced(
            ErrorMessage::new(
                "CONFIG_001",
                "Failed to read configuration file",
                ErrorSeverity::High,
                ErrorCategory::Configuration,
            ).with_details(&format!("Path: {}", path))
        ))?;
    
    // 解析逻辑...
    Ok(Config::default())
}

// 避免 unwrap() 和 expect()
// 使用 ? 操作符进行错误传播
```

### 日志记录

```rust
use tracing::{info, warn, error, debug};

// 结构化日志记录
pub async fn process_commit(&self, args: &CommitArgs) -> Result<(), AppError> {
    info!(
        operation = "process_commit",
        args = ?args,
        "Starting commit processing"
    );
    
    // 处理逻辑...
    
    info!(
        operation = "process_commit",
        duration_ms = timer.elapsed().as_millis(),
        "Commit processing completed"
    );
    
    Ok(())
}
```

### 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_commit_message_generation() {
        // 准备测试数据
        let temp_dir = TempDir::new().unwrap();
        let config = Arc::new(AppConfig::default());
        let handler = CommitHandler::new(config);
        
        // 执行测试
        let result = handler.generate_message("test diff").await;
        
        // 验证结果
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(!message.is_empty());
        assert!(message.contains("test"));
    }
}
```

## 🔄 贡献流程

### 1. 准备工作

```bash
# Fork 项目到您的 GitHub 账号
# 然后克隆您的 fork

git clone https://github.com/YOUR-USERNAME/gitai.git
cd gitai

# 添加上游仓库
git remote add upstream https://github.com/your-org/gitai.git
```

### 2. 创建功能分支

```bash
# 同步最新代码
git checkout main
git pull upstream main

# 创建功能分支
git checkout -b feature/your-feature-name

# 或者修复 bug
git checkout -b fix/issue-number-description
```

### 3. 开发和测试

```bash
# 开发过程中持续测试
cargo watch -x test

# 运行完整测试套件
cargo test
cargo test --release

# 代码格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 测试覆盖率
cargo tarpaulin --out html
```

### 4. 提交变更

```bash
# 添加变更
git add .

# 提交（遵循提交消息规范）
git commit -m "feat: add new commit message generation algorithm

- Implement GPT-based commit message generation
- Add configuration options for temperature and max tokens
- Include unit tests for new functionality
- Update documentation

Closes #123"
```

#### 提交消息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型**：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 重构代码
- `test`: 测试相关
- `chore`: 构建/工具更新

**示例**：
```
feat(commit): add AI-powered commit message generation

- Integrate OpenAI API for intelligent commit message generation
- Add configuration options for model selection and parameters
- Include fallback mechanism for offline usage
- Add comprehensive unit tests

Closes #42
```

### 5. 推送和创建 PR

```bash
# 推送分支
git push origin feature/your-feature-name

# 使用 GitHub CLI 创建 PR
gh pr create --title "feat: add new commit message generation" \
             --body "This PR adds AI-powered commit message generation functionality..."

# 或者在 GitHub 网页界面创建 PR
```

### 6. 代码审查

- 响应审查者的反馈
- 根据建议进行修改
- 保持 PR 更新和简洁

```bash
# 根据反馈进行修改
git add .
git commit -m "address review comments: improve error handling"
git push origin feature/your-feature-name
```

## 🧪 测试指南

### 测试类型

| 测试类型 | 用途 | 命令 |
|----------|------|------|
| **单元测试** | 测试单个函数/方法 | `cargo test` |
| **集成测试** | 测试模块间交互 | `cargo test --test integration` |
| **端到端测试** | 测试完整流程 | `cargo test --test e2e` |
| **性能测试** | 测试性能表现 | `cargo bench` |

### 测试编写指南

```rust
// 单元测试示例
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    fn create_test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig::default())
    }
    
    #[test]
    fn test_config_parsing() {
        let config_content = r#"
        [ai]
        model_name = "test-model"
        temperature = 0.5
        "#;
        
        let config = Config::from_str(config_content).unwrap();
        assert_eq!(config.ai.model_name, "test-model");
        assert_eq!(config.ai.temperature, 0.5);
    }
    
    #[tokio::test]
    async fn test_async_function() {
        let handler = CommitHandler::new(create_test_config());
        let result = handler.generate_message("test diff").await;
        assert!(result.is_ok());
    }
}
```

### 模拟和测试工具

```rust
// 使用 mockall 进行 mock
use mockall::predicate::*;
use mockall::mock;

mock! {
    GitService {}
    
    impl GitService {
        fn get_diff(&self) -> Result<String, GitError>;
        fn commit(&self, message: &str) -> Result<(), GitError>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock_git = MockGitService::new();
    mock_git
        .expect_get_diff()
        .returning(|| Ok("test diff".to_string()));
    
    // 测试逻辑
}
```

### 测试覆盖率

```bash
# 生成测试覆盖率报告
cargo tarpaulin --out html --output-dir coverage

# 查看覆盖率
open coverage/tarpaulin-report.html
```

## 📖 文档编写

### 文档类型

- **API 文档**: 使用 `///` 注释
- **用户文档**: Markdown 格式
- **开发文档**: 在 `docs/` 目录
- **示例代码**: 在 `examples/` 目录

### 文档规范

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
/// 当 AI 服务不可用时，会返回 [`AppError::AI`]。
/// 当 Git 操作失败时，会返回 [`AppError::Git`]。
pub struct CommitHandler {
    config: Arc<AppConfig>,
}
```

### 文档生成

```bash
# 生成 API 文档
cargo doc --open

# 检查文档链接
cargo doc --no-deps

# 测试文档中的示例
cargo test --doc
```

## 🔧 调试和性能分析

### 调试技巧

```bash
# 使用 rust-gdb 调试
rust-gdb target/debug/gitai
(gdb) break main
(gdb) run commit

# 使用 LLDB
rust-lldb target/debug/gitai
(lldb) b main
(lldb) r commit
```

### 性能分析

```bash
# 使用 perf 分析性能
perf record target/release/gitai commit
perf report

# 使用 valgrind 检查内存
valgrind --tool=memcheck target/debug/gitai commit

# 使用 cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bin gitai -- commit
```

### 日志调试

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- commit

# 启用特定模块日志
RUST_LOG=gitai::handlers::commit=debug cargo run -- commit

# 启用错误回溯
RUST_BACKTRACE=1 cargo run -- commit
```

## 📦 发布流程

### 版本管理

我们使用 [SemVer](https://semver.org/) 进行版本管理：

- **主版本 (Major)**: 不兼容的 API 更改
- **次版本 (Minor)**: 向后兼容的功能添加
- **补丁版本 (Patch)**: 向后兼容的 bug 修复

### 发布检查清单

- [ ] 所有测试通过
- [ ] 文档更新
- [ ] 变更日志更新
- [ ] 版本号更新
- [ ] 创建 release tag
- [ ] 发布到 crates.io

### 发布脚本

```bash
#!/bin/bash
# release.sh - 发布脚本

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

# 运行测试
cargo test --release

# 更新版本号
sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# 更新变更日志
echo "## [$VERSION] - $(date +%Y-%m-%d)" >> CHANGELOG.md

# 提交更改
git add .
git commit -m "chore: bump version to $VERSION"

# 创建标签
git tag -a "v$VERSION" -m "Release version $VERSION"

# 推送
git push origin main
git push origin "v$VERSION"

echo "Release $VERSION created successfully!"
```

## 🤝 社区指南

### 行为准则

我们致力于为所有人提供一个友好、安全和包容的环境。请遵守我们的 [行为准则](CODE_OF_CONDUCT.md)。

### 沟通渠道

- **GitHub Issues**: 报告 bug 和请求功能
- **GitHub Discussions**: 一般讨论和问题
- **Discord**: 实时交流 (链接待定)
- **Twitter**: [@gitai_dev](https://twitter.com/gitai_dev)

### 获取帮助

- 📖 阅读 [文档](https://gitai.docs.com)
- 🔍 搜索 [已有 Issues](https://github.com/your-org/gitai/issues)
- 💬 参与 [GitHub Discussions](https://github.com/your-org/gitai/discussions)
- 📧 联系维护者

## 🏆 致谢

感谢所有为 GitAI 做出贡献的开发者们！

### 贡献者

- 查看 [贡献者页面](https://github.com/your-org/gitai/graphs/contributors)
- 所有贡献者将在 README 中列出

### 如何获得认可

- 在 README 中列出重要贡献者
- 在发布说明中感谢贡献者
- 颁发 GitHub 徽章和证书

---

**🎉 感谢您的贡献！** 

每一个贡献都让 GitAI 变得更好。无论是代码、文档、bug 报告还是建议，我们都非常感谢！

如有任何问题，请随时通过 GitHub Issues 或 Discussions 联系我们。