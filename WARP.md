# WARP.md

本文件为 WARP (warp.dev) 在此仓库中工作时提供指南。

## 项目概览

GitAI 是一个 AI 驱动的 Git 工作流助手，提供**即时**、**非强制性**的开发者工具，不会干扰现有工作流程。它结合多维度代码分析与 AI 洞察力，提升开发生产力。

### 核心理念
- **即时辅助**：在开发过程中随时可用
- **非强制性**：所有功能都是可选的，用户自主选择何时使用
- **完全兼容**：与现有 Git 工作流无缝配合

### 关键能力
- **智能代码评审**：结合 Tree-sitter 结构分析、安全扫描和 DevOps 任务上下文的多维分析
- **智能提交**：AI 生成的提交信息，自动关联 Issue 并集成 DevOps
- **安全扫描**：基于 OpenGrep 的安全分析，支持自动安装和规则管理
- **MCP 服务器**：Model Context Protocol 服务器，实现与 LLM 的无缝集成
- **质量指标**：架构质量跟踪，包含趋势分析和报告

### 技术栈
- **语言**：Rust 2021 edition
- **分析**：Tree-sitter 支持 8+ 种编程语言
- **安全**：OpenGrep 集成用于 SAST 扫描
- **AI 集成**：OpenAI 兼容 API 支持（Ollama、GPT、Claude、Qwen）
- **协议**：MCP (Model Context Protocol) 用于 LLM 集成
- **DevOps**：Coding.net API 集成，计划支持 GitHub/Jira

## 开发命令

### 构建命令
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check for compilation errors without building
cargo check

# Build specific binary
cargo build --bin gitai
cargo build --bin gitai-mcp

# Binary locations after build:
# - target/debug/gitai (debug)
# - target/release/gitai (release)
# - target/debug/gitai-mcp (MCP server)
```

### 测试
```bash
# Run all unit tests
cargo test

# Run tests with all features enabled
cargo test --all-features

# Run tests with output capture disabled
cargo test -- --nocapture

# Run specific test
cargo test config_test

# Integration tests are located in:
# - tests/mcp_integration/ (MCP protocol tests)
# - Unit tests are embedded in source files with #[cfg(test)]
```

### 代码质量
```bash
# Format code
cargo fmt --all

# Check formatting without changing files
cargo fmt --all -- --check

# Run Clippy linter
cargo clippy --all-targets

# Clippy with stricter warnings (CI configuration)
cargo clippy --all-targets -- -D warnings

# Fix simple linting issues automatically
cargo fix --lib -p gitai
```

### 运行 GitAI

#### 基本命令
```bash
# Initialize GitAI configuration
cargo run --bin gitai -- init

# AI-powered code review
cargo run --bin gitai -- review

# Code review with security scanning
cargo run --bin gitai -- review --security-scan

# Smart commit with AI-generated message
cargo run --bin gitai -- commit

# Smart commit with issue linking
cargo run --bin gitai -- commit --issue-id "#123,#456"

# Security scanning with OpenGrep
cargo run --bin gitai -- scan --auto-install --update-rules

# Start MCP server for LLM integration
cargo run --bin gitai -- mcp --transport stdio

# Quality metrics recording and analysis
cargo run --bin gitai -- metrics record
cargo run --bin gitai -- metrics analyze --days 30
```

#### 调试命令
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin gitai -- review

# Trace specific module
RUST_LOG=gitai::analysis=trace cargo run --bin gitai -- commit

# Enable all gitai logs
RUST_LOG=gitai=debug cargo run --bin gitai -- scan

# Performance analysis
time cargo run --bin gitai -- review --tree-sitter
```

### 环境配置
```bash
# Required for AI functionality (example with Ollama)
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"

# Optional: OpenAI API key for external AI services
export GITAI_AI_API_KEY="your_openai_api_key"

# Optional: DevOps platform integration
export GITAI_DEVOPS_TOKEN="your_devops_token"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"

# Optional: Custom rules for security scanning
# Official OpenGrep rules repo:
#   Homepage: https://github.com/opengrep/opengrep-rules
#   Tarball (recommended): https://github.com/opengrep/opengrep-rules/archive/refs/heads/main.tar.gz
export GITAI_RULES_URL="https://github.com/opengrep/opengrep-rules/archive/refs/heads/main.tar.gz"
```

## 架构概览

### 多维度分析引擎

GitAI 的核心优势在于能够融合多种分析维度：

```
Code Changes (git diff)
    ↓
┌─────────────────────────────────────────────────┐
│            Data Collection Layer               │
├─────────────────────────────────────────────────┤
│ • Tree-sitter (Structure Analysis)             │
│ • OpenGrep (Security Scanning)                 │
│ • DevOps APIs (Task Context)                   │
│ • Git History (Change Patterns)                │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│            AI Fusion Layer                     │
├─────────────────────────────────────────────────┤
│ • Context-aware prompt generation              │
│ • Multi-model AI integration                   │
│ • Caching and optimization                     │
└─────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────┐
│            Output Layer                        │
├─────────────────────────────────────────────────┤
│ • Code review reports (text/json/html)         │
│ • Smart commit messages                        │
│ • Security findings                            │
│ • Quality metrics                              │
└─────────────────────────────────────────────────┘
```

### 模块结构

#### 核心模块
- **`src/main.rs`**: CLI entry point and command routing
- **`src/args.rs`**: Command-line argument definitions using clap
- **`src/config.rs`**: Configuration management for ~/.config/gitai/config.toml
- **`src/lib.rs`**: Library interface and re-exports

#### 分析模块  
- **`src/analysis.rs`**: Multi-dimensional analysis coordinator
- **`src/review.rs`**: Code review execution engine
- **`src/commit.rs`**: Smart commit message generation
- **`src/scan.rs`**: OpenGrep security scanning integration
- **`src/tree_sitter/`**: Structure analysis (8 language support)

#### 集成模块
- **`src/ai.rs`**: AI service integration (OpenAI-compatible APIs)
- **`src/devops.rs`**: DevOps platform API clients
- **`src/mcp/`**: Model Context Protocol server implementation
- **`src/metrics/`**: Quality tracking and trend analysis

#### 支持模块
- **`src/git.rs`**: Git command execution and parsing
- **`src/config_init.rs`**: Configuration initialization
- **`src/resource_manager.rs`**: Resource downloading and caching
- **`src/prompts.rs`**: AI prompt template management

### MCP（Model Context Protocol）集成

MCP 服务器用于与 LLM 客户端进行无缝集成：

```
┌─────────────────────────────────────┐
│          LLM Client                 │
│      (Claude, OpenAI, etc.)         │
└─────────────────────────────────────┘
                 │ MCP Protocol
                 ▼
┌─────────────────────────────────────┐
│        GitAI MCP Server             │
├─────────────────────────────────────┤
│ • execute_review                    │
│ • execute_commit                    │  
│ • execute_scan                      │
│ • execute_analysis                  │
└─────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│       GitAI Core Engine             │
└─────────────────────────────────────┘
```

#### MCP 服务
- **Review Service**: Code quality analysis with security scanning
- **Commit Service**: Smart commit message generation with issue linking
- **Scan Service**: Security vulnerability detection
- **Analysis Service**: Tree-sitter structure analysis

### 缓存策略
- **Review Cache**: `~/.cache/gitai/review_cache/` (MD5-based cache keys)
- **Scan History**: `~/.cache/gitai/scan_history/` (JSON scan results)  
- **Tree-sitter Cache**: In-memory LRU cache with disk persistence
- **Rules Cache**: `~/.cache/gitai/rules/` (OpenGrep security rules)

## 配置与设置

### 初始化
```bash
# Initialize configuration with default settings
gitai init

# Initialize with custom config URL (for enterprise)
gitai init --config-url https://your-org.com/gitai-config.toml

# Initialize in offline mode
gitai init --offline
```

### 配置文件结构

主配置存放于 `~/.config/gitai/config.toml`：

```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"
temperature = 0.3
api_key = "your_api_key"  # Optional

[scan]
default_path = "."
timeout = 300
jobs = 4

[devops]
platform = "coding"                    # coding, github, gitlab
base_url = "https://your-org.coding.net"
token = "your_devops_token"
project = "your-team/your-project"
timeout = 30

[mcp]
enabled = true

[mcp.services]
enabled = ["review", "commit", "scan", "analysis"]

[mcp.services.review]
default_language = "auto"
include_security_scan = false

[mcp.services.scan]
default_tool = "opengrep"
default_timeout = 300
```

### AI 模型配置

#### Ollama（本地开发推荐）
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"  # or "codellama", "llama2", etc.
temperature = 0.3
```

#### OpenAI
```toml
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key = "sk-your-openai-key"
temperature = 0.3
```

#### Claude（通过 API）
```toml
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
temperature = 0.3
```

### 资源目录
- **Configuration**: `~/.config/gitai/`
- **Cache**: `~/.cache/gitai/`
- **Rules**: `~/.cache/gitai/rules/`
- **Prompts**: `~/.config/gitai/prompts/`
- **Tree-sitter**: `~/.cache/gitai/tree-sitter/`

## 支持的语言与技术

### 编程语言（Tree-sitter 支持）
| Language   | Extension | Tree-sitter Parser |
|------------|-----------|-------------------|
| Rust       | `.rs`     | tree-sitter-rust  |
| Java       | `.java`   | tree-sitter-java  |
| Python     | `.py`     | tree-sitter-python|
| JavaScript | `.js`     | tree-sitter-javascript |
| TypeScript | `.ts`     | tree-sitter-typescript |
| Go         | `.go`     | tree-sitter-go    |
| C          | `.c`, `.h`| tree-sitter-c     |
| C++        | `.cpp`, `.hpp` | tree-sitter-cpp |

### DevOps 平台集成
- **Coding.net** ✅（完全支持）
- **GitHub Issues** 🔄（计划中）  
- **Jira** 🔄（开发中）
- **Azure DevOps** 📋（路线图）

### AI 模型支持
- **Ollama** ✅ (Local LLMs, recommended)
- **OpenAI** ✅ (GPT-3.5, GPT-4 series)
- **Claude** ✅ (Anthropic API)
- **Qwen** ✅ (Alibaba Cloud)
- **Custom APIs** ✅ (OpenAI-compatible endpoints)

### 安全扫描
- **OpenGrep** ✅ (Primary engine, 30+ language rules)
- **Custom Rules** ✅ (YAML/JSON rule definitions)
- **Auto-installation** ✅ (Cargo-based tool installation)
- **Rule Updates** ✅ (Automatic rule repository sync)

## 测试与调试

### 单元测试
```bash
# Run all unit tests
cargo test

# Run tests for specific module
cargo test tree_sitter
cargo test mcp
cargo test analysis

# Run with verbose output
cargo test -- --nocapture

# Test specific function
cargo test test_parse_commit_config
```

### 集成测试
```bash
# MCP integration tests (requires Python)
cd tests/mcp_integration
python test_direct_mcp.py
python test_mcp_scan.py

# End-to-end workflow tests
cargo test --test integration_tests
```

### 调试与故障排除

#### 调试日志
```bash
# Enable debug logs for all modules
RUST_LOG=debug gitai review

# Trace specific module
RUST_LOG=gitai::tree_sitter=trace gitai review --tree-sitter

# AI request debugging
RUST_LOG=gitai::ai=debug gitai commit

# MCP server debugging  
RUST_LOG=debug gitai mcp --transport stdio
```

#### 性能分析
```bash
# Benchmark scanning performance
time gitai scan --benchmark --no-history

# Profile memory usage
valgrind --tool=massif target/release/gitai review

# Analyze Tree-sitter caching efficiency
RUST_LOG=gitai::tree_sitter::cache=debug gitai review --tree-sitter
```

#### 常见问题

**Q: "AI service connection failed"**
```bash
# Check AI service status
curl http://localhost:11434/api/tags  # for Ollama
curl -H "Authorization: Bearer $GITAI_AI_API_KEY" https://api.openai.com/v1/models

# Test with debug logging
RUST_LOG=gitai::ai=debug gitai commit --dry-run
```

**Q: "OpenGrep not found"**
```bash
# Auto-install OpenGrep
gitai scan --auto-install

# Manual installation
cargo install opengrep

# Check installation
which opengrep
opengrep --version
```

**Q: "Configuration file not found"**
```bash
# Initialize configuration
gitai init

# Check configuration status
gitai config check

# Reset to default configuration
gitai config reset
```

**Q: "Tree-sitter parsing failed"**
```bash
# Enable Tree-sitter debugging
RUST_LOG=gitai::tree_sitter=debug gitai review --tree-sitter

# Clear Tree-sitter cache
rm -rf ~/.cache/gitai/tree-sitter/

# Test specific language
gitai review --language=rust --tree-sitter
```

## 速查

### 常见开发工作流

**代码评审工作流：**
```bash
# 1. Quick code quality check
gitai review

# 2. Comprehensive review with security
gitai review --security-scan --tree-sitter

# 3. Review with DevOps context
gitai review --issue-id "#123" --deviation-analysis
```

**智能提交工作流：**
```bash
# 1. AI-generated commit message
gitai commit

# 2. Link to specific issues
gitai commit --issue-id "#123,#456"

# 3. Review before committing
gitai commit --review --all
```

**安全扫描工作流：**
```bash
# 1. Quick security scan
gitai scan

# 2. Full scan with latest rules
gitai scan --update-rules --full

# 3. Language-specific scanning
gitai scan --lang java --timeout 600
```

**质量指标工作流：**
```bash
# 1. Record current quality snapshot
gitai metrics record

# 2. Analyze quality trends
gitai metrics analyze --days 30

# 3. Generate quality report
gitai metrics report --format html --output quality-report.html
```

### 关键文件位置
- **Main config**: `~/.config/gitai/config.toml`
- **AI prompts**: `~/.config/gitai/prompts/`
- **Security rules**: `~/.cache/gitai/rules/`
- **Review cache**: `~/.cache/gitai/review_cache/`
- **Scan history**: `~/.cache/gitai/scan_history/`

### 相关文档
- **Architecture details**: `docs/ARCHITECTURE.md` 
- **Feature overview**: `README.md`
- **Regression testing**: `docs/REGRESSION.md`
- **Configuration design**: `docs/CONFIG_MANAGEMENT.md`
- **MCP implementation**: `docs/mcp-implementation-notes.md`
