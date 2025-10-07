# GitAI - AI驱动的Git工作流助手

[![Version](https://img.shields.io/badge/version-v2.1.0-blue.svg?style=for-the-badge)](https://github.com/nehcuh/gitai/releases/tag/v2.1.0)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS%20|%20Windows-lightgrey.svg)
[![Status](https://img.shields.io/badge/status-stable-green.svg?style=for-the-badge)](https://github.com/nehcuh/gitai)

> 🤖 **让AI成为你的Git助手** - 即时代码评审、智能提交、安全扫描、架构分析

GitAI 是一个AI驱动的Git工作流增强工具，提供**即时**、**非强制性**的开发辅助。它不改变你现有的Git工作流，而是在你需要时提供智能化的帮助。

**🎯 v2.1.0 现已发布！** - 生产就绪版本，包含完整的企业级功能和186+测试覆盖。

## ✨ 核心功能

### 🔍 智能代码评审 (`gitai review`) - **v2.1.0 企业级实现**
- **深度代码分析**：4988行完整实现，支持多维度数据采集
- **AI提示词引擎**：智能构建分析提示词，支持多种输出格式
- **Tree-sitter集成**：精确的代码复杂度分析和结构化解析
- **智能缓存系统**：内存+磁盘双层缓存，避免重复分析
- **大文件处理**：支持10000+行diff的企业级分析能力

### 🤖 智能提交 (`gitai commit`) - **v2.1.0 全新实现**
- **智能信息生成**：基于文件变更类型自动生成描述性提交信息
- **预提交安全检查**：检测敏感文件和大文件，提供安全建议
- **完整Git集成**：支持暂存、提交、状态检查等完整工作流
- **Issue关联**：自动关联Issue ID到提交信息
- **干运行模式**：支持预览模式，不执行实际提交

### 🛡️ 安全扫描 (`gitai scan`) - **v2.1.0 企业级扫描器**
- **敏感信息检测**：API密钥、数据库连接、密码、JWT密钥检测
- **代码安全检查**：SQL注入、硬编码IP、不安全随机数生成检测
- **智能文件过滤**：自动跳过二进制文件，支持语言特定过滤
- **严重程度分级**：Critical、Error、Warning、Info四级分类
- **详细扫描报告**：包含文件位置、修复建议、统计信息

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
cargo build --release
sudo cp target/release/gitai /usr/local/bin/

# 或使用 cargo install（即将支持）
# cargo install gitai
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

## 🏗️ 架构特点

GitAI 采用模块化的 Workspace 架构，包含9个专门的 crate：

- **gitai-core**: 核心业务逻辑和接口定义 (5000+ 行)
- **gitai-types**: 共享类型和错误定义
- **gitai-analysis**: 代码分析引擎（Tree-sitter、架构影响分析）
- **gitai-security**: 安全扫描功能（OpenGrep集成）
- **gitai-metrics**: 质量度量和趋势分析
- **gitai-mcp**: MCP协议服务器实现 (31 tests)
- **gitai-adapters**: 外部服务适配器（AI、DevOps）
- **gitai-cli**: 命令行界面 (50 tests)
- **gitai-evaluation**: 项目质量评估工具

## 🎯 v2.1.0 技术亮点

### 💡 企业级可靠性
- **熔断器模式**: 完整实现，支持高并发AI调用
- **重试机制**: 智能错误恢复，支持多种重试策略
- **事件驱动架构**: 松耦合的模块设计，高度可扩展
- **异步并发处理**: 高性能异步架构，支持大规模处理

### 🔒 质量保证
- **测试覆盖**: 186+ 测试，核心功能100%通过
- **编译质量**: 零警告构建，严格代码标准
- **错误处理**: 完整的错误类型体系和恢复机制
- **性能监控**: 内置性能指标收集和分析

### ⚡ 性能优化
- **智能缓存**: 内存+磁盘双层缓存系统
- **大文件支持**: 支持10000+行diff分析
- **快速启动**: < 1秒启动时间
- **高效网络**: 优化的API调用和资源管理

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

**当前版本**: v2.1.0 | **项目状态**: 生产就绪 | [发布说明](RELEASE_NOTES_v2.1.0.md) | [项目状态报告](docs/FINAL_PROJECT_STATUS_REPORT.md)
