# GitAI - AI 增强的 Git 代码评审工具

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

GitAI 是一个强大的 AI 增强代码评审工具，结合 Git 代码变更分析与 DevOps 工作项管理，提供智能的需求一致性评估和代码质量分析。

## ✨ 核心特性

### 🎯 智能需求一致性分析
- **工作项集成**：自动获取 DevOps 平台的用户故事、任务和缺陷信息
- **需求对比**：AI 分析代码实现与需求描述的匹配度
- **量化评估**：提供 0-100 分的需求实现完整性和准确性评分
- **偏离检测**：识别代码实现与业务需求的偏离点

### 🔍 多维度代码质量评估
- **结构分析**：评估代码架构、设计模式和模块化程度
- **性能评估**：识别潜在的性能瓶颈和优化机会
- **安全性检查**：检测安全漏洞和风险点
- **可维护性评估**：分析代码可读性、可扩展性和可测试性

### 🚨 智能问题识别与建议
- **风险分级**：按 Critical/High/Medium/Low 分类问题严重程度
- **精准定位**：提供具体的文件位置和代码行号
- **改进建议**：生成可执行的修复建议和优先级排序
- **影响评估**：评估修复对业务和技术的预期影响

### 🛠️ 强大的技术分析
- **TreeSitter 语法分析**：支持 Rust、Java、Python、Go、JavaScript、C/C++ 等多种语言
- **语义理解**：识别函数、类型、接口变更和可见性修改
- **变更模式分析**：理解代码变更的模式和影响范围

## 🚀 快速开始

### 基础代码评审
```bash
# 分析当前暂存的代码变更
gitai review

# 使用 TreeSitter 进行深度分析
gitai review --tree-sitter
```

### AI 增强分析（结合工作项）
```bash
# 分析用户故事实现情况
gitai review --space-id=726226 --stories=99,100,101

# 分析任务完成情况
gitai review --space-id=726226 --tasks=200,201

# 分析缺陷修复情况
gitai review --space-id=726226 --defects=301,302

# 混合工作项综合分析
gitai review --space-id=726226 --stories=99 --tasks=200 --defects=301
```

### 高级配置
```bash
# 深度分析 + 特定关注点
gitai review --space-id=726226 --stories=99 \
  --depth=deep \
  --focus="安全性,性能,可维护性"

# 生成 JSON 格式报告
gitai review --space-id=726226 --stories=99 \
  --format=json \
  --output=analysis-report.json

# 生成 Markdown 报告
gitai review --space-id=726226 --stories=99 \
  --format=markdown \
  --output=review-report.md
```

## 📊 分析报告示例

### 文本格式输出
```
========== 增强型 AI 代码评审报告 ==========

📊 **总体评分**: 85/100

## 📋 需求实现一致性分析
- 完整性评分: 80/100
- 准确性评分: 90/100
- 缺失功能:
  - 错误处理机制
- 额外实现:
  - 详细日志记录

## 🔧 代码质量分析
- 整体质量: 85/100
- 可维护性: 80/100
- 性能评估: 75/100
- 安全性评估: 90/100

## ⚠️ 发现的偏离和问题
1. 🟡 **Logic Error** - 缺少空值检查
   📍 位置: src/main.rs:42
   💡 建议: 添加输入验证

## 💡 改进建议
1. **改进错误处理** (优先级: 1)
   - 描述: 添加更完善的错误处理机制
   - 预期影响: 提高系统稳定性
   - 工作量估算: Medium

## 🎯 风险评估
- 🟡 风险等级: Medium
- 业务影响: 中等业务影响
- 缓解策略:
  - 增加测试覆盖
  - 实现超时和重试机制
```

## ⚙️ 安装与配置

### 安装
```bash
# 从源码编译安装
git clone https://github.com/your-org/gitai.git
cd gitai
cargo build --release

# 或使用 cargo 安装
cargo install gitai
```

### 环境配置
```bash
# DevOps API 配置
export DEV_DEVOPS_API_BASE_URL="https://codingcorp.devops.xxx.com.cn"
export DEV_DEVOPS_API_TOKEN="your_devops_api_token"

# AI 模型配置（可选，默认使用本地 Ollama）
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen3:32b-q8_0"
```

### 配置文件 (`~/.config/gitai/config.toml`)
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen3:32b-q8_0"
temperature = 0.7
api_key = "your_api_key"  # 可选

[account]
devops_platform = "coding"
base_url = "https://codingcorp.devops.xxx.com.cn"
token = "your_devops_token"

[tree_sitter]
enabled = true
analysis_depth = "medium"
cache_enabled = true
languages = ["rust", "java", "python", "go", "javascript", "c", "cpp"]
```

## 📚 详细使用指南

### 分析深度级别
- **`--depth=basic`**: 快速分析，关注主要问题
- **`--depth=normal`**: 标准分析，平衡速度和深度（默认）
- **`--depth=deep`**: 深度分析，详细检查所有方面

### 关注点定制
```bash
--focus="安全性"           # 重点关注安全问题
--focus="性能,可维护性"    # 关注多个领域
--focus="错误处理"         # 专项分析
```

### 支持的 DevOps 平台
- **Coding.net**: 腾讯云开发者平台
- **Jira**: Atlassian 项目管理工具（开发中）
- **Azure DevOps**: 微软开发平台（开发中）

### 支持的编程语言
- Rust 🦀
- Java ☕
- Python 🐍
- Go 🐹
- JavaScript/TypeScript 📜
- C/C++ ⚡

## 🎯 使用场景

### 开发团队日常评审
```bash
# 每日代码评审
gitai review --tree-sitter --depth=normal

# 发布前质量检查
gitai review --depth=deep --format=html --output=release-review.html
```

### DevOps 流程集成
```bash
# CI/CD 管道中的自动化评审
gitai review --space-id=$SPACE_ID --stories=$STORY_IDS \
  --format=json --output=ci-analysis.json

# 需求验收检查
gitai review --space-id=$SPACE_ID --stories=$STORY_ID \
  --focus="功能完整性,用户体验"
```

### 代码质量治理
```bash
# 技术债务分析
gitai review --focus="可维护性,技术债务" --depth=deep

# 安全审计
gitai review --focus="安全性" --format=markdown --output=security-audit.md
```

## 🔧 故障排除

### 常见问题

**Q: DevOps API 连接失败？**
```bash
# 验证连接和权限
curl -H "Authorization: token $DEV_DEVOPS_API_TOKEN" \
  "$DEV_DEVOPS_API_BASE_URL/external/collaboration/api/project/$SPACE_ID/issues/$ISSUE_ID"
```

**Q: AI 分析响应慢？**
```bash
# 使用较轻的分析深度
gitai review --depth=basic

# 检查本地 AI 服务状态
curl http://localhost:11434/api/tags
```

**Q: TreeSitter 分析失败？**
```bash
# 启用调试模式
RUST_LOG=debug gitai review --tree-sitter

# 回退到简化分析
gitai review  # 不使用 --tree-sitter 参数
```

## 🏗️ 项目架构

```
gitai/
├── src/
│   ├── handlers/
│   │   ├── analysis.rs     # AI 分析引擎 🆕
│   │   ├── review.rs       # 代码评审核心
│   │   ├── ai.rs          # AI 交互处理
│   │   └── git.rs         # Git 命令处理
│   ├── clients/
│   │   └── devops_client.rs # DevOps API 客户端 🆕
│   ├── types/
│   │   ├── ai.rs          # AI 分析类型 🆕
│   │   ├── devops.rs      # DevOps 数据类型 🆕
│   │   └── git.rs         # Git 相关类型
│   ├── tree_sitter_analyzer/ # 语法分析器
│   └── config.rs          # 配置管理
├── docs/                  # 详细文档
├── examples/              # 使用示例
└── assets/               # 配置模板
```

## 🤝 贡献指南

我们欢迎社区贡献！请遵循以下步骤：

1. **Fork 项目**
2. **创建特性分支** (`git checkout -b feature/amazing-feature`)
3. **提交更改** (`git commit -m 'Add amazing feature'`)
4. **推送到分支** (`git push origin feature/amazing-feature`)
5. **创建 Pull Request**

### 开发环境搭建
```bash
# 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 安装依赖
cargo build

# 运行测试
cargo test

# 启用调试日志
RUST_LOG=debug cargo run -- review --help
```

## 📈 路线图

### 近期目标 (v0.2.0)
- [ ] 更多 DevOps 平台支持 (Jira, Azure DevOps)
- [ ] 自定义分析规则配置
- [ ] 增量分析优化
- [ ] IDE 插件支持

### 中期目标 (v0.3.0)
- [ ] 实时代码分析
- [ ] 团队协作功能
- [ ] 分析结果缓存机制
- [ ] 多语言提示词优化

### 长期目标 (v1.0.0)
- [ ] 企业级部署支持
- [ ] 高级分析报告
- [ ] 机器学习模型优化
- [ ] 云原生架构

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE) 开源。

## 🙏 致谢

感谢以下开源项目的支持：
- [Tree-sitter](https://tree-sitter.github.io/) - 语法分析
- [Tokio](https://tokio.rs/) - 异步运行时
- [Clap](https://docs.rs/clap/) - 命令行解析
- [Serde](https://serde.rs/) - 序列化框架

---

**GitAI** - 让代码评审更智能，让开发更高效 🚀