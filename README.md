# GitAI - AI驱动的Git工作流助手

[![Version](https://img.shields.io/badge/version-v1.0.0-blue.svg?style=for-the-badge)](https://github.com/nehcuh/gitai/releases/tag/v1.0.0)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
![Platform](https://img.shields.io/badge/platform-Linux%20|%20macOS%20|%20Windows-lightgrey.svg)
[![Status](https://img.shields.io/badge/status-stable-green.svg?style=for-the-badge)](https://github.com/nehcuh/gitai)

> 🤖 **让AI成为你的Git助手** - 即时代码评审、智能提交、Git命令解释

GitAI 是一组AI驱动的Git辅助工具，专注于在开发过程中提供即时帮助。不同于强制性的开发流程，GitAI的设计理念是**非强制性**和**即时性** - 在你需要的任何时候提供AI辅助，不改变你现有的Git工作流。

## 🚀 项目概览

### 🎯 设计理念
- **即时性** - 在开发过程的任何时刻都能使用，获得即时反馈
- **非强制性** - 所有功能都是可选的，不改变现有的Git工作流
- **工具性** - 是工具集合，不是强制流程，用户主动选择使用
- **兼容性** - 完全兼容原生Git命令，不破坏用户习惯

### 💡 使用场景
- **代码评审** - 随时检查代码质量，获得改进建议
- **智能提交** - AI生成规范的提交信息，支持Issue关联
- **Git学习** - AI解释复杂Git命令的输出结果
- **安全扫描** - 集成OpenGrep进行代码安全检查
- **DevOps集成** - 关联开发任务，分析需求覆盖率

> **版本状态**：v1.0.0 稳定版已发布！所有核心功能完全可用，包括完整的 MCP 集成、代码评审、智能提交、安全扫描等。

## 🎯 功能门控（新功能）

GitAI 现在支持**功能门控**，允许您根据需求定制构建，优化二进制文件大小：

- **最小构建** (10MB)：仅核心功能，适合轻量级使用
- **默认构建** (12MB)：包含 AI + 常用语言支持
- **完整构建** (22MB)：所有功能和语言支持

```bash
# 构建最小版本
cargo build --release --no-default-features --features minimal

# 构建特定语言支持
cargo build --release --no-default-features --features tree-sitter-rust

# 构建完整版本
cargo build --release --features full
```

详细说明请参考 [功能门控指南](docs/FEATURE_FLAGS.md)。

## 📚 文档导航

- 本README - 完整的功能介绍和使用指南
- [docs/README.md](docs/README.md) - 中文文档索引
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - 技术架构和设计理念
- [docs/FEATURE_FLAGS.md](docs/FEATURE_FLAGS.md) - 功能门控使用指南
- [docs/dependency-analysis-in-review.md](docs/dependency-analysis-in-review.md) - 依赖分析与 PageRank 在评审中的应用
- [docs/REVIEW_WORKFLOW.md](docs/REVIEW_WORKFLOW.md) - 基于场景的评审工作流与流程图
- [docs/REGRESSION.md](docs/REGRESSION.md) - 回归测试手册

快速开始请参考下方章节。

## 🧰 开发工具

- tools/build-variants.sh — 构建不同功能门控变体的便捷脚本
- tools/test-features.sh — 批量测试不同 feature 组合的脚本
- tools/migrate_types.sh — 类型迁移/批处理脚本（按需使用）

使用建议：
- 执行前请阅读脚本头部说明
- 可在本地或 CI 中调用，配合功能门控进行快速验证

## ✨ 核心功能

### 🔍 智能代码评审 (`gitai review`)
- **即时反馈**：在任何开发阶段都能检查代码质量，获得改进建议
- **多维度分析**：结合代码结构、安全扫描、DevOps任务上下文进行综合评估
- **智能缓存**：避免重复分析，提高响应速度
- **灵活配置**：可选择启用安全扫描、Tree-sitter结构分析、偏离度分析等
- **非强制性**：提供建议供参考，不强制修改代码

### 🤖 智能提交助手 (`gitai commit`)
- **AI生成提交信息**：基于代码变更自动生成规范、简洁的提交信息
- **Issue关联**：自动添加Issue前缀，符合DevOps平台规范
- **上下文感知**：结合代码评审结果和DevOps任务信息生成更准确的提交信息
- **灵活使用**：支持自定义消息与AI生成的结合
- **测试模式**：支持`--dry-run`预览提交信息而不实际提交

### 🧠 智能 Git 代理
- **即时解释**：使用`--ai`标志让AI解释任何Git命令的输出结果
- **学习辅助**：帮助理解复杂的Git命令和错误信息
- **零破坏性**：默认不启用AI解释，完全兼容原生Git行为
- **错误分析**：自动检测Git错误并提供AI分析和解决方案

### 🔗 DevOps 集成
- **任务上下文**：自动获取Issue信息，分析代码变更与需求的一致性
- **偏离度分析**：评估代码实现是否解决了指定Issue，提供覆盖率分析
- **平台支持**：支持Coding.net、GitHub等DevOps平台的Issue集成
- **规范化提交**：自动生成符合团队规范的提交信息格式

### 🛡️ 代码安全扫描 (`gitai scan`)
- **高性能扫描**：集成OpenGrep引擎，支持30+种编程语言的安全规则
- **智能规则管理**：自动下载和更新安全规则库，支持自定义规则源
- **灵活配置**：支持全量扫描、增量扫描、远程扫描等多种模式
- **自动安装**：一键自动安装OpenGrep（`--auto-install`），提供清晰的安装指引
- **多格式输出**：支持文本、JSON等多种输出格式，便于集成到CI/CD
- **性能优化**：智能缓存和语言检测，提供基准测试模式

> **配置提示**：使用`gitai update --check`检查规则状态，环境变量`GITAI_RULES_URL`可覆盖默认规则源。

### 🌐 MCP 服务器集成 (`gitai mcp`)
- **完整MCP协议支持**：实现Model Context Protocol (MCP) 服务器，支持与LLM客户端的无缝集成
- **四大核心服务**：提供代码评审、智能提交、安全扫描、代码分析四个核心工具
- **错误处理优化**：完善的错误类型系统和用户友好的错误信息
- **性能监控**：内置性能统计和监控功能，实时跟踪工具调用情况
- **配置验证**：完整的配置验证机制，确保服务器稳定运行
- **日志记录**：结构化日志系统，便于调试和监控

**启动MCP服务器**：
```bash
# 启动stdio传输的MCP服务器
gitai mcp --transport stdio

# 独立运行MCP服务器
gitai-mcp serve
```

> **MCP工具列表**：
> - `execute_review` - 代码评审
> - `execute_commit` - 智能提交  
> - `execute_scan` - 安全扫描
> - `execute_analysis` - 代码分析

### 📊 架构质量趋势追踪 (`gitai metrics`)
- **持续监控**：自动记录每次代码分析的质量指标快照
- **多维度指标**：追踪技术债务、代码复杂度、API稳定性、架构耦合度等关键指标
- **趋势分析**：识别质量改善或恶化趋势，及时发现异常
- **智能预测**：基于历史数据预测未来趋势，提前预警潜在问题
- **可视化报告**：生成直观的Markdown/HTML报告，包含图表和详细分析
- **数据导出**：支持CSV/JSON格式导出，便于进一步分析或集成

#### 详细使用指南

##### 1. 记录质量快照 (`metrics record`)
```bash
# 记录当前代码库的质量指标快照
gitai metrics record

# 记录特定路径的质量快照
gitai metrics record --path=./src

# 添加标签和备注
gitai metrics record --tag=v1.0.0 --note="发布前的质量基线"

# 强制记录（即使最近已有快照）
gitai metrics record --force
```

##### 2. 趋势分析 (`metrics analyze`)
```bash
# 分析最近30天的质量趋势
gitai metrics analyze --days=30

# 分析特定日期范围
gitai metrics analyze --from=2024-01-01 --to=2024-03-31

# 分析特定指标的趋势
gitai metrics analyze --metrics=complexity,debt --days=7

# 包含异常检测和预测
gitai metrics analyze --detect-anomalies --predict-trend
```

##### 3. 生成报告 (`metrics report`)
```bash
# 生成Markdown格式的趋势报告
gitai metrics report --output=quality-report.md

# 生成HTML格式的可视化报告
gitai metrics report --format=html --output=report.html

# 生成最近季度的报告
gitai metrics report --period=quarter --output=q1-report.md

# 包含图表和详细分析
gitai metrics report --with-charts --verbose --output=detailed-report.md
```

##### 4. 查看快照列表 (`metrics list`)
```bash
# 列出所有质量快照
gitai metrics list

# 列出最近10个快照
gitai metrics list --limit=10

# 按日期范围过滤
gitai metrics list --from=2024-01-01 --to=2024-12-31

# 按标签过滤
gitai metrics list --tag=release

# 详细模式（显示所有指标）
gitai metrics list --verbose
```

##### 5. 比较快照 (`metrics compare`)
```bash
# 比较两个快照的差异
gitai metrics compare --snapshot1=2024-01-01 --snapshot2=2024-02-01

# 比较当前与基线
gitai metrics compare --baseline=v1.0.0 --current

# 比较并生成差异报告
gitai metrics compare --id1=abc123 --id2=def456 --output=diff.md

# 只比较特定指标
gitai metrics compare --metrics=complexity,coverage --id1=latest --id2=previous
```

##### 6. 清理历史数据 (`metrics clean`)
```bash
# 清理30天前的快照
gitai metrics clean --older-than=30d

# 保留最近100个快照，删除其余
gitai metrics clean --keep-recent=100

# 清理特定标签的快照
gitai metrics clean --tag=test --confirm

# 预览将要删除的数据（不实际删除）
gitai metrics clean --older-than=90d --dry-run
```

##### 7. 数据导出 (`metrics export`)
```bash
# 导出为CSV格式
gitai metrics export --format=csv --output=metrics.csv

# 导出为JSON格式
gitai metrics export --format=json --output=metrics.json

# 导出特定时间范围的数据
gitai metrics export --from=2024-01-01 --to=2024-12-31 --output=yearly.csv

# 导出用于其他工具的格式
gitai metrics export --format=prometheus --output=metrics.txt
```

#### 实际应用场景

##### 持续集成中的质量门控
```yaml
# GitHub Actions 示例
- name: Record Quality Metrics
  run: |
    gitai metrics record --tag="PR-${{ github.event.number }}"
    gitai metrics analyze --days=7 --detect-anomalies
    
    # 如果质量下降超过阈值，则失败
    gitai metrics compare --baseline=main --current \
      --fail-on-regression --threshold=5
```

##### 定期质量报告
```bash
# 每周质量报告（可加入cron）
#!/bin/bash
gitai metrics record --tag=weekly
gitai metrics analyze --days=7 --predict-trend
gitai metrics report --period=week --output="reports/week-$(date +%Y%W).md"
gitai metrics export --format=csv --output="data/week-$(date +%Y%W).csv"
```

##### 版本发布质量基线
```bash
# 发布前记录基线
gitai metrics record --tag="release-v2.0.0" --note="Release candidate baseline"

# 发布后对比
gitai metrics compare --baseline="release-v1.0.0" --current="release-v2.0.0" \
  --output=release-comparison.md
```

## ⚡ 即时辅助工具的使用方式

GitAI的设计理念是**即时性**和**非强制性** - 在你需要的时候提供帮助，不强制改变你的工作流。

### 开发过程中的任意时刻

```bash
# 写代码过程中，随时检查质量
gitai review

# 准备提交时，生成规范的提交信息  
gitai commit --issue-id="#123"

# 提交前，进行全面安全检查
gitai review --security-scan --block-on-critical

# 学习Git命令时，获得AI解释
gitai --ai log --oneline -10

# 需要安全扫描时，独立使用
gitai scan --lang=java
```

### 非强制性的设计

- **提供建议，不强制执行** - `gitai review`给出改进建议，但由你决定是否采纳
- **兼容现有工作流** - 可以继续使用原生`git commit`，GitAI只是提供辅助
- **渐进式采用** - 可以从一个功能开始尝试，逐步使用更多功能
- **可选增强** - 所有增强功能都是可选的，通过命令行参数控制

### 与传统工具的区别

```
传统工具：强制流程
编码 → 预检查 → 扫描 → 评审 → 提交 (必须按流程执行)

GitAI：即时辅助
编码 → [随时使用gitai review] → [使用gitai commit] → 提交 (完全由你控制)
```

### 🔗 安全扫描与代码评审集成

- **一键安全评审**：在代码评审时自动运行安全扫描 (`--security-scan`)
- **智能风险评估**：自动计算整体风险等级，提供重点关注建议
- **问题分类展示**：按严重程度分类显示安全问题，突出高风险问题
- **可选阻止机制**：可配置在发现严重安全问题时阻止合并 (`--block-on-critical`)
- **上下文感知建议**：基于具体问题类型提供定制化的安全改进建议

## 🚀 快速开始

### 安装

```bash
# 从源码编译安装
git clone https://github.com/your-org/gitai.git
cd gitai
cargo build --release

# 将二进制文件添加到 PATH
cp target/release/gitai /usr/local/bin/
```

安装完成后，务必查看本地的 (配置文件)[~/.config/gitai/config.toml], 将其中的 AI 配置为自己能够使用的 AI 服务，默认配置使用的是 ollama 的服务，如果本地没有安装的话会无法使用 AI 服务

### 基本使用

```bash
# 智能代码评审
gitai review

# AI 生成提交信息
gitai commit

# 关联 issue ID 的智能提交
gitai commit --issue-id="#123,#354"

# 代码安全扫描
gitai scan

# 启用安全扫描的代码评审
gitai review --security-scan

# 指定扫描工具的安全评审
gitai review --security-scan --scan-tool=opengrep

# 发现严重问题时阻止合并
gitai review --security-scan --block-on-critical

# 启用全局 AI 模式
gitai --ai status

# 启动MCP服务器
gitai mcp --transport stdio

# 质量趋势追踪
gitai metrics record                 # 记录当前质量快照
gitai metrics analyze --days=30      # 分析最近30天趋势
gitai metrics report --output=report.md  # 生成趋势报告

# 获取帮助
gitai help
```

## 📋 详细功能指南

### 🔍 智能代码评审

#### 基础评审
```bash
# 评审当前暂存的变更
gitai review

# 评审特定提交
gitai review --commit1=HEAD~1 --commit2=HEAD

# 比较两个提交
gitai review --commit1=abc123 --commit2=def456
```

#### Tree-sitter 支持
```bash
# 启用 Tree-sitter 进行结构分析（推荐与 --full 一起使用）
gitai review --tree-sitter

# 与安全扫描结合
gitai review --tree-sitter --security-scan --scan-tool=opengrep
```

#### 评审模式与典型场景
- 基础评审（快速检查变更，默认模式）
  ```bash
  gitai review
  ```
- 全量架构评审（依赖图 + PageRank + 架构影响 + 可选安全扫描）
  ```bash
  gitai review --full --tree-sitter --security-scan --scan-tool=opengrep
  ```
- 带 DevOps 任务上下文的全量评审
  ```bash
  gitai review --full --issue-id="#123,#456"
  ```
- DevOps 偏离度分析（专注需求符合度，使用 deviation 模板）
  ```bash
  gitai review --deviation-analysis --issue-id="#123"
  ```
- CI 严格模式（发现高危问题时降低评分/阻止合并）
  ```bash
  gitai review --full --security-scan --block-on-critical
  ```

#### 评审流程图（简化）
```mermaid
flowchart LR
  A[gitai review] --> B{模式}
  B -->|基础| C[结构/安全(可选)] --> D[review 模板] --> E[AI/回退] --> F[输出]
  B -->|--full| G[依赖图 + PageRank + 影响范围] --> H[注入 Dependency Insights] --> E
  B -->|--deviation-analysis| I[deviation 模板] --> E
```

#### Issue 关联（当前支持）
```bash
# 通过 issue-id 参数关联
gitai review --issue-id="#123"

# 多个 issues 关联
gitai review --issue-id="#123,#456"

# 偏离度分析（开发中）
gitai review --deviation-analysis
```

#### 高级配置
```bash
# 集成工作项分析
gitai review --space-id=726226 --stories=99

# 生成不同格式的报告
gitai review --format=json --output=report.json
gitai review --format=markdown --output=report.md
gitai review --format=html --output=report.html
```

### 🤖 智能提交助手

#### 基础 AI 提交
```bash
# 生成 AI 提交信息
gitai commit

# 使用别名
gitai cm
```

#### AI 增强提交
```bash
# AI 自动生成提交信息
gitai commit

# 结合代码评审
gitai commit --review

# 使用别名
gitai cm
```

#### 自定义提交信息
```bash
# 保留用户信息 + AI 增强
gitai commit -m "feat: 添加用户认证功能"

# 多行提交信息
gitai commit -m "feat: 用户认证

- 实现登录/登出功能
- 添加密码加密
- 集成会话管理"
```

#### Issue ID 关联提交
```bash
# 关联单个 issue
gitai commit --issue-id="#123"

# 关联多个 issues
gitai commit --issue-id="#123,#354"

# 不带井号前缀（自动添加）
gitai commit --issue-id="123,354"

# 结合自定义消息
gitai commit --issue-id="#123" -m "修复登录问题"

# 与其他功能组合使用
gitai commit --issue-id="#123,#354" -t -a -m "实现新功能"
```

生成的提交信息示例：
```
#123,#354 feat: 实现用户认证功能

- 添加登录/登出接口
- 集成JWT令牌验证
- 完善权限控制机制
```

#### 智能文件暂存
```bash
# 自动暂存 + 智能提交
gitai commit -a

# 结合其他选项
gitai commit -a -t -m "修复认证漏洞"
```

#### 审查结果集成
```bash
# 提交时自动集成审查结果
gitai commit --review

# 与其他功能组合
gitai commit -a -t --review -m "实现用户管理功能"
```

### 🧠 智能 Git 代理

#### 全局 AI 模式
```bash
# 所有输出都通过 AI 解释
gitai --ai status
gitai --ai log --oneline -10
gitai --ai diff HEAD~1

# 复杂命令的 AI 解释
gitai --ai merge --strategy=recursive origin/main
```

#### 智能错误处理
```bash
# 显式开启 AI 解释；错误时亦会给出建议，同时保留原始输出
gitai --ai status
gitai --ai push origin
gitai --ai rebase main
```

#### 显式禁用 AI（默认即禁用）
```bash
# 默认不启用 AI，如使用了 alias 等强制开启时，可用 --noai 显式禁用
gitai --noai status
gitai --noai commit -m "正常提交"
```

### 📊 评审报告示例

#### 文本格式输出
```
========== 增强型 AI 代码评审报告 ==========

📊 **总体评分**: 85/100

## 📋 需求实现一致性分析
✅ 用户故事 #99: 用户登录功能
- 完整性评分: 90/100
- 准确性评分: 85/100
- 实现状态: 基本完成
- 缺失功能: 密码强度验证
- 额外实现: 自动登录记住功能

## 🔧 代码质量分析
- 整体质量: 85/100
- 可维护性: 80/100
- 性能评估: 90/100
- 安全性评估: 75/100

## 🌳 Tree-sitter 结构分析
- 新增函数: authenticate_user(), validate_token()
- 修改函数: login_handler() - 复杂度增加
- 新增类型: UserSession, AuthError
- 依赖变更: 新增 bcrypt, jsonwebtoken

## ⚠️ 发现的问题
1. 🔴 **Critical** - SQL 注入风险
   📍 位置: src/auth.rs:42
   💡 建议: 使用参数化查询

2. 🟡 **Medium** - 缺少输入验证
   📍 位置: src/handlers/login.rs:15
   💡 建议: 添加邮箱格式验证

## 💡 改进建议
1. **加强安全防护** (优先级: High)
   - 实现 SQL 注入防护
   - 添加 CSRF 保护
   - 工作量估算: 1-2 天

2. **优化性能** (优先级: Medium)
   - 实现连接池
   - 添加缓存机制
   - 工作量估算: 2-3 天
```

#### JSON 格式输出
```json
{
  "overall_score": 85,
  "requirement_analysis": {
    "user_stories": [
      {
        "id": 99,
        "title": "用户登录功能",
        "completeness_score": 90,
        "accuracy_score": 85,
        "missing_features": ["密码强度验证"],
        "extra_implementations": ["自动登录记住功能"]
      }
    ]
  },
  "code_quality": {
    "overall": 85,
    "maintainability": 80,
    "performance": 90,
    "security": 75
  },
  "tree_sitter_analysis": {
    "new_functions": ["authenticate_user", "validate_token"],
    "modified_functions": ["login_handler"],
    "new_types": ["UserSession", "AuthError"],
    "dependency_changes": ["bcrypt", "jsonwebtoken"]
  },
  "issues": [
    {
      "severity": "Critical",
      "type": "Security",
      "description": "SQL 注入风险",
      "location": "src/auth.rs:42",
      "suggestion": "使用参数化查询"
    }
  ]
}
```

## ⚙️ 配置

### Shell 别名推荐（非必需）
```bash
# Zsh/Bash
alias ga='gitai --ai'      # 显式启用 AI 解释
alias gnx='gitai --noai'   # 显式禁用 AI（即使用原生 git 行为）

# Fish
alias ga 'gitai --ai'
alias gnx 'gitai --noai'
```

### 环境变量
```bash
# AI 服务配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
export GITAI_AI_API_KEY="your_openai_key"  # 可选
```

### 配置文件 (`~/.config/gitai/config.toml`)
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"
temperature = 0.3
api_key = "your_api_key"  # 可选

[scan]
default_path = "/path/to/project"  # 可选，默认为当前目录
timeout = 300
jobs = 4

[mcp]
enabled = true

[mcp.server]
name = "gitai-mcp"
version = "0.1.0"

[mcp.services]
enabled = ["review", "commit", "scan", "analysis"]

[mcp.services.review]
default_language = "auto"
include_security_scan = false

[mcp.services.commit]
auto_stage = false
include_review = false

[mcp.services.scan]
default_tool = "opengrep"
default_timeout = 300

[mcp.services.analysis]
verbosity = 1
```

### Prompt 模板自定义
```bash
# 初始化模板目录（如不存在则创建，并写入默认模板）
gitai prompts init

# 查看可用模板
gitai prompts list

# 查看某个模板内容
gitai prompts show --name commit

# 从远程/预置源更新模板（非强制，按需使用）
gitai prompts update

# 模板目录结构（可手工编辑）
~/.config/gitai/prompts/
├── commit.md                # 提交信息生成模板
├── review.md                # 代码评审模板
```

## 🛠️ 支持的技术栈

### 编程语言支持
- **Rust** 🦀 - 完整支持，包括宏分析
- **Python** 🐍 - 支持语法分析和依赖检测
- **JavaScript/TypeScript** 📜 - 支持 ES6+ 和 JSX
- **Java** ☕ - 支持 Spring Boot 和企业级框架
- **Go** 🐹 - 支持 goroutine 和并发分析
- **C/C++** ⚡ - 支持现代 C++ 特性

### DevOps 平台支持
- **Coding.net** - 腾讯云开发者平台（完整支持）
- **Jira** - Atlassian 项目管理（开发中）
- **Azure DevOps** - 微软开发平台（规划中）
- **GitHub Issues** - GitHub 项目管理（规划中）

### AI 模型支持
- **Ollama** - 本地大语言模型（推荐）
- **OpenAI** - GPT-3.5/4 系列
- **Claude** - Anthropic AI 模型
- **Qwen** - 阿里云通义千问
- **自定义 API** - 兼容 OpenAI 格式的任何服务

## 📊 使用场景

### 个人开发者
```bash
# 每日开发流程
gitai review                        # 代码质量检查
gitai commit -a --issue-id="#123"  # 关联 issue 的智能提交
gitai --ai push origin main         # 推送时的智能提示

# 处理多个相关 issues
gitai commit --issue-id="#123,#456" -m "修复登录和权限问题"
```

### 团队协作
```bash
# 团队评审流程
gitai review --format=html --output=team-review.html

# 标准化提交流程（关联工作项）
gitai commit --review --issue-id="#STORY-123" -m "实现用户认证功能"

# 团队协作中的 issue 追踪
gitai commit --issue-id="#BUG-456,#TASK-789" --review

# 发布分支的批量关联
gitai commit --issue-id="#FEAT-001,#FEAT-002,#BUG-003" -m "发布 v1.2.0"
```

### CI/CD 集成
```yaml
# GitHub Actions 示例
- name: AI Code Review
  run: |
    gitai review --format=json --output=ci-review.json
    
    # 如果需要关联 issues
    gitai review --issue-id="${{ github.event.pull_request.body }}" \
      --format=json --output=ci-review.json
```

### 代码质量治理
```bash
# 技术债务分析
gitai review

# 安全审计
gitai review --security-scan --format=markdown \
  --output=security-audit-$(date +%Y%m%d).md

# 代码质量分析
gitai review --format=json --output=quality-report.json
```

## 🧪 高级功能

### 批量处理
```bash
# 批量分析多个提交
for commit in $(git log --oneline -10 --format=%h); do
  gitai review --commit1=$commit --commit2=$commit~1 \
    --format=json --output=review-$commit.json
done

# 批量生成提交信息（历史重写）
git rebase -i HEAD~10 --exec "gitai commit --amend"
```

### 自定义分析规则
```bash
# 指定语言分析
gitai review --language=rust

# 指定输出格式
gitai review --format=json

# 保存分析结果
gitai review --output=review-report.md
```

### 集成外部工具
```bash
# 与 git hooks 集成
echo 'gitai commit -a --review' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# 与编辑器集成（VS Code）
echo '{"command": "gitai commit"}' > .vscode/tasks.json
```

## 🔧 故障排除

### 常见问题

**Q: DevOps API 连接失败**
```bash
# 验证 API 配置
curl -H "Authorization: token $GITAI_DEVOPS_TOKEN" \
  "$GITAI_DEVOPS_BASE_URL/api/user"

# 检查网络连接
gitai review --dry-run --space-id=726226
```

**Q: AI 服务响应慢**
```bash
# 使用更轻量的模型
export GITAI_AI_MODEL="qwen2.5:7b"

# 调整分析深度
gitai review

# 检查本地 AI 服务
curl http://localhost:11434/api/tags
```

**Q: Tree-sitter 分析失败**
```bash
# Tree-sitter 功能尚在开发中
# 当前可使用基础 AI 分析
gitai review

# 启用详细日志调试
RUST_LOG=debug gitai review
```

**Q: 提交信息生成质量不佳**
```bash
# 自定义提示词模板
cp ~/.config/gitai/prompts/commit.md \
   ~/.config/gitai/prompts/commit.md.backup

# 编辑模板
vim ~/.config/gitai/prompts/commit.md

# 测试新模板
gitai commit --dry-run
```

### OpenGrep 安装说明
```bash
# 推荐方式（需要安装 Rust 工具链 https://rustup.rs ）
cargo install opengrep

# 若执行后无法找到命令，添加 cargo bin 到 PATH（临时）
export PATH="$HOME/.cargo/bin:$PATH"

# 使用 gitai 自动安装（内部也是调用 cargo）
gitai scan --auto-install --update-rules
```

### 调试模式
```bash
# 启用详细日志
RUST_LOG=debug gitai review

# 跟踪 AI 请求
RUST_LOG=gitai::handlers::ai=trace gitai commit

# 性能分析
time gitai review --tree-sitter
```

## 🏗️ 项目架构

```
gitai/
├── src/
│   ├── handlers/
│   │   ├── review.rs              # 代码评审核心逻辑
│   │   ├── commit.rs              # 智能提交处理
│   │   ├── intelligent_git.rs     # 智能 Git 代理
│   │   ├── ai.rs                  # AI 服务集成
│   │   ├── analysis.rs            # 需求分析引擎
│   │   ├── git.rs                 # Git 命令封装
│   │   └── help.rs                # 帮助系统
│   ├── clients/
│   │   └── devops_client.rs       # DevOps API 客户端
│   ├── tree_sitter_analyzer/      # 语法分析器
│   │   ├── analyzer.rs            # 主分析器
│   │   └── core.rs                # 核心分析逻辑
│   ├── types/
│   │   ├── ai.rs                  # AI 相关类型
│   │   ├── devops.rs              # DevOps 数据类型
│   │   ├── git.rs                 # Git 相关类型
│   │   └── general.rs             # 通用类型
│   ├── config.rs                  # 配置管理
│   ├── errors.rs                  # 错误处理
│   ├── utils.rs                   # 工具函数
│   └── main.rs                    # 主入口
├── docs/
│   ├── prds/                      # 产品需求文档
│   └── stories/                   # 用户故事
├── assets/                        # 配置模板和资源
├── examples/                      # 使用示例
└── tests/                         # 测试套件
```

## 🤝 贡献指南

### 开发环境搭建
```bash
# 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建项目
cargo build

# 运行测试
cargo test

# 启用调试模式
RUST_LOG=debug cargo run -- help
```

### 提交规范
```bash
# 使用 GitAI 自己生成提交信息
cargo run -- commit --review

# 手动提交的格式规范
git commit -m "feat(review): 添加 Tree-sitter 增强分析

- 实现多语言语法分析
- 支持自定义分析深度
- 优化分析性能
- 添加缓存机制

Closes #123"
```

### 开发流程
1. **Fork 项目**
2. **创建特性分支** (`git checkout -b feature/amazing-feature`)
3. **开发和测试** (`cargo test`)
4. **使用 GitAI 评审** (`gitai review`)
5. **智能提交** (`gitai commit --review`)
6. **创建 Pull Request**

### 代码规范
- 使用 `rustfmt` 格式化代码
- 遵循 Rust 官方编码规范
- 为新功能添加测试
- 更新相关文档

## 📝 版本发布历史

### 🎉 v1.0.0 (2024-12-23) - **稳定版发布**

#### ✨ 主要特性
- **MCP 集成** 🚀 完整的 Model Context Protocol 支持，实现与 LLM 客户端的无缝集成
- **性能优化** ⚡ 扫描速度提升 20-30%，语言检测速度提升 5 倍
- **代码质量** ✅ 通过所有 Clippy 检查，代码质量达到生产标准
- **安全加固** 🔒 修复测试脚本安全漏洞，防止命令注入攻击
- **错误处理** 🛡️ 统一错误处理系统，提供用户友好的错误信息和解决建议
- **日志优化** 📊 支持彩色输出和详细时间戳，提升调试体验

#### 🔧 功能改进
- 优化缓存机制，减少重复计算
- 改进语言检测算法，提高准确性
- 增强 MCP 服务器稳定性
- 完善配置验证机制
- 提升整体用户体验

#### 📦 已完成功能
- [x] 智能代码评审
- [x] AI 提交信息生成
- [x] 安全扫描集成
- [x] DevOps 平台集成
- [x] 智能 Git 代理
- [x] Issue ID 关联提交
- [x] MCP 服务器
- [x] 统一错误处理
- [x] 性能优化
- [x] 安全加固

---

## 📈 版本路线图

### v1.1.x - 增强功能
- [ ] 更多 DevOps 平台支持 (GitHub Issues, Jira, Azure DevOps)
- [ ] Tree-sitter 深度分析完整实现
- [ ] 自定义分析规则引擎
- [ ] 实时代码分析
- [ ] IDE 插件支持 (VS Code, IntelliJ)

### v1.2.x - 企业功能
- [ ] 团队协作功能
- [ ] 分析报告看板
- [ ] 代码质量趋势分析
- [ ] 自定义 AI 模型训练
- [ ] 企业级部署方案

### v2.0.x - 下一代架构
- [ ] 高可用集群支持
- [ ] 详细的权限管理
- [ ] 完整的 API 生态
- [ ] 插件系统架构
- [ ] 多语言界面支持

## 🌟 特色亮点

### 🚀 性能优化
- **并发处理**：多文件并行分析
- **智能缓存**：Tree-sitter 分析结果缓存
- **增量分析**：只分析变更的代码部分
- **流式处理**：大文件的分块处理

### 🔒 安全特性
- **本地优先**：代码分析可完全在本地进行
- **数据脱敏**：自动过滤敏感信息
- **权限控制**：细粒度的 API 访问控制
- **审计日志**：详细的操作记录

### 🎯 用户体验
- **智能补全**：命令和参数的智能提示
- **渐进式增强**：从简单到复杂的功能升级
- **多语言支持**：中英文界面和提示
- **可视化报告**：丰富的图表和统计信息

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE) 开源。

## 🙏 致谢

感谢以下开源项目和社区的支持：

- [Tree-sitter](https://tree-sitter.github.io/) - 强大的语法分析框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [Clap](https://docs.rs/clap/) - 命令行参数解析
- [Serde](https://serde.rs/) - 序列化框架
- [Reqwest](https://docs.rs/reqwest/) - HTTP 客户端
- [Ollama](https://ollama.ai/) - 本地大语言模型服务

## 📞 联系我们

- **问题反馈**: [GitHub Issues](https://github.com/your-org/gitai/issues)
- **功能建议**: [GitHub Discussions](https://github.com/your-org/gitai/discussions)
- **邮件联系**: gitai@your-org.com
- **官方文档**: [gitai.dev](https://gitai.dev)

---

**GitAI** - 让每一次提交都更智能，让每一行代码都更优雅 🚀

*让 AI 成为你最好的编程伙伴*
