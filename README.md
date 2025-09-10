# GitAI - AI驱动的Git工作流助手

[![Version](https://img.shields.io/badge/version-v1.1.0-blue.svg?style=for-the-badge)](https://github.com/nehcuh/gitai/releases/tag/v1.1.0)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS%20|%20Windows-lightgrey.svg)
[![Status](https://img.shields.io/badge/status-stable-green.svg?style=for-the-badge)](https://github.com/nehcuh/gitai)

> 🤖 **让AI成为你的Git助手** - 即时代码评审、智能提交、安全扫描、架构分析

GitAI 是一个AI驱动的Git工作流增强工具，提供**即时**、**非强制性**的开发辅助。它不改变你现有的Git工作流，而是在你需要时提供智能化的帮助。

## ✨ 核心功能

### 🔍 智能代码评审 (`gitai review`)
- **多维度分析**：结合代码结构、安全扫描、DevOps任务上下文
- **智能缓存**：避免重复分析，提高响应速度
- **灵活配置**：可选启用Tree-sitter分析、安全扫描、偏离度检测

### 🤖 智能提交 (`gitai commit`)
- **AI生成提交信息**：基于代码变更自动生成规范的提交信息
- **Issue关联**：自动添加Issue前缀，支持DevOps平台集成
- **测试模式**：`--dry-run` 预览提交信息而不实际提交

### 🛡️ 安全扫描 (`gitai scan`)
- **高性能扫描**：集成OpenGrep，支持30+种编程语言
- **智能规则管理**：自动下载和更新安全规则库
- **自动安装**：`--auto-install` 一键安装扫描引擎

### 🌐 MCP服务器 (`gitai mcp`)
- **完整MCP协议支持**：实现Model Context Protocol服务器
- **四大核心服务**：代码评审、智能提交、安全扫描、代码分析
- **LLM集成**：与Claude、GPT等LLM客户端无缝集成

### 📊 架构分析 (`gitai graph`)
- **依赖图导出**：生成Graphviz DOT格式的可视化图
- **智能摘要**：社区压缩、路径采样、预算自适应裁剪
- **LLM友好**：专为大语言模型优化的输出格式

### 📈 质量追踪 (`gitai metrics`)
- **持续监控**：自动记录代码质量指标快照
- **趋势分析**：识别质量改善或恶化趋势
- **可视化报告**：生成Markdown/HTML格式的分析报告

## 🚀 快速开始

### 安装

```bash
# 从源码安装（推荐）
git clone https://github.com/nehcuh/gitai.git
cd gitai
cargo build --release --features default
sudo cp target/release/gitai /usr/local/bin/

# 或使用 cargo install
cargo install gitai
```

### 初始化配置

```bash
# 交互式配置向导
gitai init

# 检查配置状态
gitai config check
```

### 基本使用

```bash
# 智能代码评审
gitai review

# AI生成提交信息
gitai commit

# 安全扫描
gitai scan

# 启动MCP服务器
gitai mcp --transport stdio
```

## 🎯 功能门控

GitAI 支持灵活的功能门控，可根据需求定制构建：

```bash
# 最小构建 (10MB)
cargo build --release --no-default-features --features minimal

# 默认构建 (12MB)
cargo build --release --features default

# 完整构建 (22MB)
cargo build --release --features full
```

详见 [功能门控指南](docs/features/FEATURE_FLAGS.md)。

## 📚 文档

- 文档索引: docs/README.md
- 架构设计: docs/architecture/ARCHITECTURE.md
- MCP 服务: docs/features/MCP_SERVICE.md
- API 参考: docs/api/API_REFERENCE.md
- MCP 图摘要: docs/api/MCP_GRAPH_SUMMARY.md
- 依赖图与摘要: docs/features/DEPENDENCY_GRAPH.md
- 开发指南: docs/development/CONTRIBUTING.md

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！请查看[贡献指南](docs/development/CONTRIBUTING.md)了解详情。

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

---

**当前版本**: v1.1.0 | **项目状态**: 稳定版 | [查看更新日志](CHANGELOG.md)
