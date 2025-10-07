# GitAI Documentation Index / 文档索引

Welcome to GitAI's documentation. This page provides a comprehensive index of all documentation, organized by category.

欢迎查阅 GitAI 文档。本页提供所有文档的综合索引，按类别组织。

## 🚀 Getting Started / 快速开始
- **Development Guide / 开发指南** — [WARP.md](../WARP.md) - Complete development environment setup and commands
- **Project Status / 项目状态** — [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) - Current implementation progress
- **Contributing / 贡献指南** — [development/CONTRIBUTING.md](development/CONTRIBUTING.md) - How to contribute to the project
- **Optimization Roadmap / 优化路线图** — [optimization/README.md](optimization/README.md) - Complete optimization plan and task breakdown

## ⚡ Quickstart / 快速开始
- MCP Server (stdio) Quickstart / MCP 服务器（stdio）快速开始

```bash
# AI service (example: local Ollama). Replace with your setup.
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
# Optional external API key
export GITAI_AI_API_KEY="{{OPENAI_OR_OTHER_API_KEY}}"

# Start MCP server via stdio
gitai mcp --transport stdio
```

See also: API Reference → MCP Quickstart (docs/api/API_REFERENCE.md).

## 🏗️ Architecture & Design / 架构与设计
- **Architecture Overview / 架构概览** — [architecture/ARCHITECTURE.md](architecture/ARCHITECTURE.md) - Core design principles and module structure
- **Modular CLI Design / 模块化 CLI 设计** — [archive/historical/CLI_MODULARIZATION_PROGRESS.md](archive/historical/CLI_MODULARIZATION_PROGRESS.md) - CLI handler modularization
- **MCP Service Registry / MCP 服务注册** — [api/mcp-implementation-notes.md](api/mcp-implementation-notes.md) - Model Context Protocol implementation
- **Concurrent Analysis / 并发分析** — [archive/historical/CONCURRENT_ANALYSIS_OPTIMIZATION.md](archive/historical/CONCURRENT_ANALYSIS_OPTIMIZATION.md) - TreeSitter concurrent optimization
- **Configuration Management / 配置管理** — [CONFIG_MANAGEMENT.md](CONFIG_MANAGEMENT.md) - Configuration system design
- **Graph Summarization / 图摘要** — [api/MCP_GRAPH_SUMMARY.md](api/MCP_GRAPH_SUMMARY.md) - Dependency graph summarization

## 🛠️ Features / 功能特性
- **Feature Flags / 功能门控** — [features/FEATURE_FLAGS.md](features/FEATURE_FLAGS.md) - Feature flag system and conditional compilation
- **Code Review Workflow / 代码评审工作流** — [features/REVIEW_WORKFLOW.md](features/REVIEW_WORKFLOW.md) - AI-powered review process
- **Dependency Graph / 依赖图与摘要** — [features/DEPENDENCY_GRAPH.md](features/DEPENDENCY_GRAPH.md) - Graph export and summarization (CLI & MCP)
- **Dependency Analysis in Review / 评审中的依赖分析** — [features/dependency-analysis-in-review.md](features/dependency-analysis-in-review.md) - PageRank in code review
- **Security Scanning / 安全扫描** — [AI_ERA_SECURITY.md](AI_ERA_SECURITY.md) - AI-era security considerations

## 📚 API & Integration / API 与集成
- **API Reference / API 参考** — [api/API_REFERENCE.md](api/API_REFERENCE.md) - Library, MCP, CLI contracts
- **MCP Graph Summary / MCP 图摘要** — [api/MCP_GRAPH_SUMMARY.md](api/MCP_GRAPH_SUMMARY.md) - Graph summary API
- **MCP Service Guide / MCP 服务说明** — [features/MCP_SERVICE.md](features/MCP_SERVICE.md) - Services and tools overview
- **MCP Integration Examples / MCP 集成示例** — [examples/mcp_integration/README.md](examples/mcp_integration/README.md) - Integration examples
- **Resource Download / 资源下载** — [RESOURCE_DOWNLOAD_GUIDE.md](RESOURCE_DOWNLOAD_GUIDE.md) - External resource management

## 🧪 Testing & Quality / 测试与质量
- **Regression Testing / 回归测试** — [development/REGRESSION.md](development/REGRESSION.md) - Test procedures and checklists
- **MCP Integration Tests / MCP 集成测试** — [development/MCP_INTEGRATION_TESTS.md](development/MCP_INTEGRATION_TESTS.md) - MCP service testing
- **Review Output Fix / 评审输出修复** — [review-output-fix.md](review-output-fix.md) - Review module fixes

## ⚙️ Performance Notes / 性能说明
- Current tuning lives in code and CLI flags. See feature docs and CLI help.
- Archived Phase 2 notes have been moved to Archives; see (PHASE2_OPTIMIZATIONS.md) for historical reference.

## 📖 Reference / 参考资料
- **Terminology / 术语表** — [TERMINOLOGY.md](TERMINOLOGY.md) - Project-specific terminology
- **Architecture Details / 架构详情** — [architecture/ARCHITECTURE.md](architecture/ARCHITECTURE.md) - Detailed architecture documentation

## 📁 Archives / 归档文档
Historical and phase-specific documentation / 历史和阶段性文档：

### Historical Progress / 历史进度
- [archive/historical/CLI_MODULARIZATION_PROGRESS.md](archive/historical/CLI_MODULARIZATION_PROGRESS.md)
- [archive/historical/CONCURRENT_ANALYSIS_OPTIMIZATION.md](archive/historical/CONCURRENT_ANALYSIS_OPTIMIZATION.md)
- [archive/historical/OPTIMIZATION_PLAN.md](archive/historical/OPTIMIZATION_PLAN.md)
- [archive/historical/PHASE1_TASKS_TRACKER.md](archive/historical/PHASE1_TASKS_TRACKER.md)
- [archive/historical/PHASE2_IMPACT_SCOPE_ANALYSIS.md](archive/historical/PHASE2_IMPACT_SCOPE_ANALYSIS.md)

### Legacy Documents / 遗留文档
- [archive/ARCHITECTURE_REFACTOR.md](archive/ARCHITECTURE_REFACTOR.md) - Architecture refactoring notes
- [archive/WORKSPACE_MIGRATION.md](archive/WORKSPACE_MIGRATION.md) - Workspace migration guide
- [archive/MCP_OPTIMIZATION_NOTES.md](archive/MCP_OPTIMIZATION_NOTES.md) - Legacy MCP optimization

---

## 📝 Notes / 注意事项

1. Documentation with duplicate names in subdirectories are being consolidated
2. All paths are relative to the `docs/` directory unless specified otherwise
3. For the latest implementation status, see [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md)

If you find broken links or outdated documentation, please submit an issue or PR.

如发现链接失效或文档需要更新，欢迎在 Issue 中反馈或提交 PR。
