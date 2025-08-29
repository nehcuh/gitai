# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

GitAI is an AI-driven Git workflow assistant that provides **instant**, **non-mandatory** developer tools without disrupting existing workflows. It combines multi-dimensional code analysis with AI-powered insights to enhance development productivity.

### Core Philosophy
- **Instant assistance**: Available at any moment during development
- **Non-mandatory**: All features are optional, users choose when to engage
- **Full compatibility**: Works seamlessly with existing Git workflows

### Key Capabilities
- **Intelligent Code Review**: Multi-dimensional analysis combining Tree-sitter structure analysis, security scanning, and DevOps task context
- **Smart Commits**: AI-generated commit messages with automatic issue linking and DevOps integration
- **Security Scanning**: OpenGrep-powered security analysis with auto-installation and rule management
- **MCP Server**: Model Context Protocol server for seamless LLM integration
- **Quality Metrics**: Architectural quality tracking with trend analysis and reporting

### Technology Stack
- **Language**: Rust 2021 edition
- **Analysis**: Tree-sitter for 8+ programming languages
- **Security**: OpenGrep integration for SAST scanning
- **AI Integration**: OpenAI-compatible API support (Ollama, GPT, Claude, Qwen)
- **Protocols**: MCP (Model Context Protocol) for LLM integration
- **DevOps**: API integrations for Coding.net and planned GitHub/Jira support

## Development Commands

### Build Commands
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

### Testing
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

### Code Quality
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

### Running GitAI

#### Basic Commands
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

#### Debugging Commands
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

### Environment Setup
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
export GITAI_RULES_URL="https://your-rules-repo/rules.tar.gz"
```

## Architecture Overview

### Multi-dimensional Analysis Engine

GitAI's core strength is its ability to combine multiple analysis dimensions:

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

### Module Structure

#### Core Modules
- **`src/main.rs`**: CLI entry point and command routing
- **`src/args.rs`**: Command-line argument definitions using clap
- **`src/config.rs`**: Configuration management for ~/.config/gitai/config.toml
- **`src/lib.rs`**: Library interface and re-exports

#### Analysis Modules  
- **`src/analysis.rs`**: Multi-dimensional analysis coordinator
- **`src/review.rs`**: Code review execution engine
- **`src/commit.rs`**: Smart commit message generation
- **`src/scan.rs`**: OpenGrep security scanning integration
- **`src/tree_sitter/`**: Structure analysis (8 language support)

#### Integration Modules
- **`src/ai.rs`**: AI service integration (OpenAI-compatible APIs)
- **`src/devops.rs`**: DevOps platform API clients
- **`src/mcp/`**: Model Context Protocol server implementation
- **`src/metrics/`**: Quality tracking and trend analysis

#### Support Modules
- **`src/git.rs`**: Git command execution and parsing
- **`src/config_init.rs`**: Configuration initialization
- **`src/resource_manager.rs`**: Resource downloading and caching
- **`src/prompts.rs`**: AI prompt template management

### MCP (Model Context Protocol) Integration

The MCP server enables seamless integration with LLM clients:

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

#### MCP Services
- **Review Service**: Code quality analysis with security scanning
- **Commit Service**: Smart commit message generation with issue linking
- **Scan Service**: Security vulnerability detection
- **Analysis Service**: Tree-sitter structure analysis

### Caching Strategy
- **Review Cache**: `~/.cache/gitai/review_cache/` (MD5-based cache keys)
- **Scan History**: `~/.cache/gitai/scan_history/` (JSON scan results)  
- **Tree-sitter Cache**: In-memory LRU cache with disk persistence
- **Rules Cache**: `~/.cache/gitai/rules/` (OpenGrep security rules)

## Configuration & Setup

### Initial Setup
```bash
# Initialize configuration with default settings
gitai init

# Initialize with custom config URL (for enterprise)
gitai init --config-url https://your-org.com/gitai-config.toml

# Initialize in offline mode
gitai init --offline
```

### Configuration File Structure

The main configuration is stored in `~/.config/gitai/config.toml`:

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

### AI Model Configuration

#### Ollama (Recommended for local development)
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

#### Claude (via API)
```toml
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
temperature = 0.3
```

### Resource Directories
- **Configuration**: `~/.config/gitai/`
- **Cache**: `~/.cache/gitai/`
- **Rules**: `~/.cache/gitai/rules/`
- **Prompts**: `~/.config/gitai/prompts/`
- **Tree-sitter**: `~/.cache/gitai/tree-sitter/`

## Supported Languages & Technologies

### Programming Languages (Tree-sitter Support)
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

### DevOps Platform Integration
- **Coding.net** ✅ (Fully supported)
- **GitHub Issues** 🔄 (Planned)  
- **Jira** 🔄 (In development)
- **Azure DevOps** 📋 (Roadmap)

### AI Model Support
- **Ollama** ✅ (Local LLMs, recommended)
- **OpenAI** ✅ (GPT-3.5, GPT-4 series)
- **Claude** ✅ (Anthropic API)
- **Qwen** ✅ (Alibaba Cloud)
- **Custom APIs** ✅ (OpenAI-compatible endpoints)

### Security Scanning
- **OpenGrep** ✅ (Primary engine, 30+ language rules)
- **Custom Rules** ✅ (YAML/JSON rule definitions)
- **Auto-installation** ✅ (Cargo-based tool installation)
- **Rule Updates** ✅ (Automatic rule repository sync)

## Testing & Debugging

### Unit Tests
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

### Integration Tests
```bash
# MCP integration tests (requires Python)
cd tests/mcp_integration
python test_direct_mcp.py
python test_mcp_scan.py

# End-to-end workflow tests
cargo test --test integration_tests
```

### Debugging & Troubleshooting

#### Debug Logging
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

#### Performance Analysis
```bash
# Benchmark scanning performance
time gitai scan --benchmark --no-history

# Profile memory usage
valgrind --tool=massif target/release/gitai review

# Analyze Tree-sitter caching efficiency
RUST_LOG=gitai::tree_sitter::cache=debug gitai review --tree-sitter
```

#### Common Issues

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

## Quick Reference

### Common Development Workflows

**Code Review Workflow:**
```bash
# 1. Quick code quality check
gitai review

# 2. Comprehensive review with security
gitai review --security-scan --tree-sitter

# 3. Review with DevOps context
gitai review --issue-id "#123" --deviation-analysis
```

**Smart Commit Workflow:**
```bash
# 1. AI-generated commit message
gitai commit

# 2. Link to specific issues
gitai commit --issue-id "#123,#456"

# 3. Review before committing
gitai commit --review --all
```

**Security Scanning Workflow:**
```bash
# 1. Quick security scan
gitai scan

# 2. Full scan with latest rules
gitai scan --update-rules --full

# 3. Language-specific scanning
gitai scan --lang java --timeout 600
```

**Quality Metrics Workflow:**
```bash
# 1. Record current quality snapshot
gitai metrics record

# 2. Analyze quality trends
gitai metrics analyze --days 30

# 3. Generate quality report
gitai metrics report --format html --output quality-report.html
```

### Key File Locations
- **Main config**: `~/.config/gitai/config.toml`
- **AI prompts**: `~/.config/gitai/prompts/`
- **Security rules**: `~/.cache/gitai/rules/`
- **Review cache**: `~/.cache/gitai/review_cache/`
- **Scan history**: `~/.cache/gitai/scan_history/`

### Related Documentation
- **Architecture details**: `docs/ARCHITECTURE.md` 
- **Feature overview**: `README.md`
- **Regression testing**: `docs/REGRESSION.md`
- **Configuration design**: `docs/CONFIG_MANAGEMENT.md`
- **MCP implementation**: `docs/mcp-implementation-notes.md`
