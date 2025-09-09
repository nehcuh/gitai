# GitAI 开发贡献指南

感谢您对 GitAI 项目的关注！本指南将帮助您了解如何为 GitAI 贡献代码。

## 项目概览

GitAI 是一个 AI 驱动的 Git 工作流助手，提供**即时**、**非强制性**的开发者工具，不会干扰现有工作流程。

### 技术栈

- **语言**：Rust 2021 edition
- **分析**：Tree-sitter 支持 8+ 种编程语言
- **安全**：OpenGrep 集成用于 SAST 扫描
- **AI 集成**：OpenAI 兼容 API 支持（Ollama、GPT、Claude、Qwen）
- **协议**：MCP (Model Context Protocol) 用于 LLM 集成
- **DevOps**：Coding.net API 集成，计划支持 GitHub/Jira

## 开发环境设置

### 前置要求

- Rust 1.70+ (推荐使用 rustup 安装)
- Git 2.0+
- cargo-edit（可选，用于依赖管理）
- cargo-watch（可选，用于开发时自动编译）

### 环境配置

```bash
# 克隆仓库
git clone https://github.com/nehcuh/gitai.git
cd gitai

# 安装开发依赖
cargo build

# 配置 AI 服务（使用 Ollama 示例）
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"

# 可选：配置 DevOps 平台
export GITAI_DEVOPS_TOKEN="your_token"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"
```

## 开发工作流

### 1. 分支管理

- `main` - 稳定分支，包含发布版本
- `develop` - 开发分支，日常开发在此进行
- `feature/*` - 功能分支
- `bugfix/*` - 错误修复分支
- `release/*` - 发布准备分支

### 2. 提交规范

请遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型（type）：
- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `refactor`: 重构（既不是新功能也不是错误修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

示例：
```
feat(mcp): add service dependency management

- Add version() and dependencies() methods to GitAiMcpService trait
- Implement circular dependency detection
- Add comprehensive test coverage

Closes #123
```

## 构建与测试

### 构建命令

```bash
# Debug 构建
cargo build

# Release 构建（优化）
cargo build --release

# 检查编译错误（不生成二进制文件）
cargo check

# 构建特定二进制文件
cargo build --bin gitai
cargo build --bin gitai-mcp

# 功能门控构建
cargo build --release --no-default-features --features minimal
cargo build --release --features full
```

### 测试命令

```bash
# 运行所有测试
cargo test --all-features

# 运行特定模块测试
cargo test tree_sitter
cargo test mcp

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试函数
cargo test test_parse_parameters
```

### 代码质量

```bash
# 格式化代码
cargo fmt --all

# 检查格式（不修改文件）
cargo fmt --all -- --check

# 运行 Clippy 检查
cargo clippy --all-targets --all-features

# 修复简单的 lint 问题
cargo clippy --fix --allow-dirty --allow-staged
```

## 架构指南

### 模块结构

```
src/
├── main.rs           # CLI 入口
├── args.rs           # 命令行参数定义
├── config.rs         # 配置管理
├── lib.rs            # 库接口
│
├── analysis.rs       # 多维度分析协调器
├── review.rs         # 代码评审引擎
├── commit.rs         # 智能提交
├── scan.rs           # 安全扫描集成
│
├── tree_sitter/      # 结构分析
│   ├── analyzer.rs   # 语言无关的分析器
│   └── queries.rs    # Tree-sitter 查询
│
├── mcp/              # MCP 服务器实现
│   ├── manager.rs    # 服务管理器
│   ├── registry.rs   # 服务注册表
│   └── services/     # MCP 服务实现
│
└── infrastructure/   # 基础设施层
    ├── container/    # DI 容器
    └── provider.rs   # 服务提供者
```

### 设计原则

1. **模块化**：每个功能应该是独立的模块
2. **可测试性**：所有公共 API 都应有测试覆盖
3. **错误处理**：使用 Result 类型，提供清晰的错误信息
4. **性能优先**：避免不必要的内存分配和复制
5. **类型安全**：充分利用 Rust 的类型系统

## 调试技巧

### 日志调试

```bash
# 启用调试日志
RUST_LOG=debug cargo run --bin gitai -- review

# 追踪特定模块
RUST_LOG=gitai::analysis=trace cargo run --bin gitai -- commit

# 启用所有 gitai 日志
RUST_LOG=gitai=debug cargo run --bin gitai -- scan
```

### 性能分析

```bash
# 基准测试
cargo bench

# 使用 time 命令
time cargo run --release --bin gitai -- review --tree-sitter

# 使用 perf（Linux）
perf record -g cargo run --release --bin gitai -- review
perf report
```

## 提交 PR

1. Fork 项目仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交更改：`git commit -am 'feat: add new feature'`
4. 推送到分支：`git push origin feature/your-feature`
5. 创建 Pull Request

### PR 检查清单

- [ ] 代码通过所有测试 (`cargo test --all-features`)
- [ ] 代码通过格式检查 (`cargo fmt --all -- --check`)
- [ ] 代码通过 Clippy 检查 (`cargo clippy --all-targets --all-features`)
- [ ] 更新了相关文档
- [ ] 添加了必要的测试
- [ ] commit message 符合规范

## 常见问题

### Q: 如何添加新的 Tree-sitter 语言支持？

1. 在 `Cargo.toml` 中添加语言的 tree-sitter 包
2. 更新 `src/tree_sitter/mod.rs` 中的 `SupportedLanguage` 枚举
3. 在 `assets/queries/` 中添加对应的查询文件
4. 更新功能门控配置

### Q: 如何添加新的 MCP 服务？

1. 在 `src/mcp/services/` 中创建新的服务模块
2. 实现 `GitAiMcpService` trait
3. 在 `ServiceManager` 中注册服务
4. 添加相应的测试

### Q: 如何处理 AI API 错误？

使用统一的错误处理模式：

```rust
use crate::error::{GitAIError, AIError};

// 处理 API 错误
let response = client.complete(request).await
    .map_err(|e| GitAIError::AI(AIError::ApiError(e.to_string())))?;
```

## 获取帮助

- 查看 [架构文档](../architecture/ARCHITECTURE.md)
- 提交 [Issue](https://github.com/nehcuh/gitai/issues)
- 参与 [讨论](https://github.com/nehcuh/gitai/discussions)

## 行为准则

请遵循我们的行为准则，保持友好和专业的交流环境。我们欢迎所有人参与贡献，无论技术水平如何。

感谢您的贡献！🎉
