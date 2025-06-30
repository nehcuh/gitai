# GitAI - AI 驱动的智能 Git 工具套件

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

GitAI 是一个革命性的 AI 驱动 Git 工具套件，将人工智能深度集成到你的 Git 工作流中。从智能代码评审到自动提交信息生成，再到 Git 命令的智能解释，GitAI 让你的开发体验更加智能、高效。

## ✨ 核心功能

### 🔍 智能代码评审 (`gitai review`)
- **AI 驱动分析**：深度理解代码变更的业务逻辑和技术影响
- **AstGrep 语法分析**：支持 Rust、Java、Python、Go、JavaScript、C/C++ 等多种语言
- **DevOps 集成**：自动获取工作项需求，分析代码与需求的一致性
- **多维度评估**：代码质量、安全性、性能、可维护性全方位分析
- **量化评分**：提供 0-100 分的客观评分和详细改进建议

### 🤖 智能提交助手 (`gitai commit`)
- **AI 生成提交信息**：基于代码变更自动生成规范的提交信息
- **AstGrep 增强**：分析代码结构，生成更具上下文的描述
- **Issue ID 关联**：自动在提交信息前添加关联的 issue 编号前缀
- **审查结果集成**：自动整合代码评审要点到提交信息
- **自定义信息支持**：保留用户输入，AI 提供补充建议
- **智能文件暂存**：自动识别和暂存相关文件

### 🧠 智能 Git 代理
- **全局 AI 模式** (`--ai`)：所有 Git 命令输出都通过 AI 解释
- **智能错误处理**：自动检测错误并提供 AI 分析和解决方案
- **命令学习助手**：帮助理解复杂的 Git 命令和输出
- **无缝集成**：完全兼容原生 Git 命令和工作流

### 🎯 DevOps 工作流集成
- **需求追踪**：自动关联代码变更与用户故事、任务、缺陷
- **完整性验证**：验证代码实现是否满足业务需求
- **偏离检测**：识别代码实现与原始需求的偏差
- **团队协作**：支持多平台 DevOps 工具集成

### 🌐 多语言翻译支持 (`--lang`)
- **AST-Grep 扫描翻译**：支持中英文双语的代码扫描结果输出
- **全局语言设置**：通过 `--lang=zh|en|auto` 参数控制所有命令的输出语言
- **智能语言检测**：`auto` 模式自动检测系统语言环境
- **配置化管理**：支持通过配置文件设置默认翻译偏好
- **性能优化**：翻译功能对工具性能影响微乎其微
- **向后兼容**：翻译功能默认禁用，不影响现有工作流

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

# 启用全局 AI 模式
gitai --ai status

# 多语言支持
gitai --lang=zh scan src/          # 中文输出代码扫描
gitai --lang=en review            # 英文输出代码评审
gitai scan --lang=auto src/       # 自动检测语言

# 获取帮助
gitai help
```

## 📋 详细功能指南

### 🌐 多语言翻译功能

#### 全局语言设置
```bash
# 设置中文输出（适用于所有子命令）
gitai --lang=zh <command>

# 设置英文输出
gitai --lang=en <command>

# 自动检测系统语言（基于LANG环境变量）
gitai --lang=auto <command>

# 通过环境变量设置语言
export GITAI_TRANSLATION_LANGUAGE=zh
gitai <command>  # 将使用中文输出
```

#### AST-Grep 代码扫描翻译
```bash
# 中文代码扫描
gitai scan --lang=zh src/
# 输出: 🔍 AST-Grep 扫描完成
#       📂 扫描了 36 个文件
#       ⚠️  发现 5 个问题

# 英文代码扫描
gitai scan --lang=en src/
# 输出: 🔍 AST-Grep Scan Complete
#       📂 Scanned 36 files
#       ⚠️  Found 5 issues

# 带统计信息的中文扫描
gitai scan --lang=zh src/ --stats
# 输出详细的中文统计信息

# 格式化输出的多语言扫描
gitai scan --lang=zh src/ --format=json --output=scan-zh.json
gitai scan --lang=en src/ --format=json --output=scan-en.json

# 性能优化的扫描（使用翻译缓存）
gitai scan --lang=zh src/ --use-cache  # 使用缓存，36个文件仅需额外5ms
```

#### 子命令语言参数
```bash
# Review 命令本地语言设置
gitai review --lang=zh --focus="性能问题"
gitai review --lang=en --focus="performance issues"

# 扫描命令的语言特定输出
gitai scan --lang=zh --verbose --max-issues=10
gitai scan --lang=en --format=json --output=results.json

# Commit 命令的语言设置
gitai commit --lang=zh  # 生成中文提交信息
gitai commit --lang=en  # 生成英文提交信息

# 智能Git操作语言设置
gitai git --lang=zh status  # 增强的中文git状态输出
gitai git --lang=en log     # 增强的英文git日志输出
```

#### 翻译配置示例
```toml
# ~/.config/gitai/config.toml
[translation]
enabled = true
default_language = "zh"  # zh|en|auto
cache_enabled = true
provider = "openai"      # openai|azure|custom
cache_dir = "~/.cache/gitai/translation"
# cache_max_age_days = 30  # 可选: 缓存最大保留天数
# cache_max_size_mb = 100  # 可选: 缓存最大大小(MB)

[translation.provider_settings]
api_key = "your-translation-api-key"
model = "gpt-3.5-turbo"  # 或 "gpt-4" 等
# timeout_seconds = 10    # 可选: API调用超时时间
# max_retries = 3         # 可选: 失败重试次数
# endpoint = "https://custom-translation-api.example.com/v1/translate"  # 自定义翻译API
```

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

#### AstGrep 增强分析
```bash
# 启用 AstGrep 深度分析
gitai review --ast-grep

# 指定分析深度
gitai review --ast-grep --depth=1  # 基础分析
gitai review --ast-grep --depth=2  # 中级分析
gitai review --ast-grep --depth=3  # 深度分析
```

#### DevOps 工作项集成
```bash
# 分析用户故事实现
gitai review --space-id=726226 --stories=99,100,101

# 分析任务完成情况
gitai review --space-id=726226 --tasks=200,201

# 分析缺陷修复
gitai review --space-id=726226 --defects=301,302

# 混合工作项分析
gitai review --space-id=726226 --stories=99 --tasks=200 --defects=301
```

#### 高级配置
```bash
# 指定分析深度和关注点
gitai review --space-id=726226 --stories=99 \
  --depth=deep \
  --focus="安全性,性能,可维护性"

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

#### Tree-sitter 增强提交
```bash
# 启用 Tree-sitter 分析
gitai commit --tree-sitter

# 指定分析级别
gitai commit -t -l 2

# 完整参数形式
gitai commit --tree-sitter --level 3
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
# 正常执行，错误时提供 AI 解释
gitai status         # 如果有错误，自动提供解决方案
gitai push origin    # 推送失败时，AI 分析原因和解决方法
gitai rebase main    # 冲突时，AI 提供解决建议
```

#### 禁用 AI 功能
```bash
# 完全禁用 AI，使用原生 Git
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

### 环境变量
```bash
# DevOps API 配置
export GITAI_DEVOPS_PLATFORM="coding"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"
export GITAI_DEVOPS_TOKEN="your_api_token"

# AI 服务配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
export GITAI_AI_API_KEY="your_openai_key"  # 可选
```

### 配置文件 (`~/.config/gitai/config.toml`)
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:32b"
temperature = 0.3
api_key = "your_api_key"  # 可选

[account]
devops_platform = "coding"
base_url = "https://your-org.coding.net"
token = "your_devops_token"
timeout = 30000
retry_count = 3

[tree_sitter]
enabled = true
analysis_depth = "medium"
cache_enabled = true
languages = ["rust", "python", "javascript", "typescript", "go", "java", "c", "cpp"]

[review]
auto_save = true
storage_path = "~/review_results"
format = "markdown"
max_age_hours = 168
include_in_commit = true

[translation]
enabled = true
default_language = "auto"  # zh|en|auto
cache_enabled = true
provider = "openai"
cache_dir = "~/.cache/gitai/translation"

[translation.provider_settings]
api_key = "your_translation_api_key"
model = "gpt-3.5-turbo"
```

### Prompt 模板自定义
```bash
# 自定义提示词模板
~/.config/gitai/prompts/
├── commit-generator.md      # 提交信息生成模板
├── general-helper.md        # 通用 AI 助手模板
├── review.md               # 代码评审模板
├── translator.md           # 翻译模板
└── commit-deviation.md     # 偏离检测模板
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
gitai review --tree-sitter          # 代码质量检查
gitai commit -a -t --issue-id="#123" # 关联 issue 的智能提交
gitai --ai push origin main         # 推送时的智能提示

# 处理多个相关 issues
gitai commit --issue-id="#123,#456" -m "修复登录和权限问题"
```

### 团队协作
```bash
# 团队评审流程
gitai review --space-id=PROJECT_ID --stories=STORY_IDS \
  --format=html --output=team-review.html

# 标准化提交流程（关联工作项）
gitai commit -t --review --issue-id="#STORY-123" -m "实现用户认证功能"

# 团队协作中的 issue 追踪
gitai commit --issue-id="#BUG-456,#TASK-789" -t --review

# 发布分支的批量关联
gitai commit --issue-id="#FEAT-001,#FEAT-002,#BUG-003" -m "发布 v1.2.0"
```

### CI/CD 集成
```yaml
# GitHub Actions 示例
- name: AI Code Review
  run: |
    gitai review --space-id=${{ vars.SPACE_ID }} \
      --stories=${{ github.event.pull_request.body }} \
      --format=json --output=ci-review.json

    gitai commit -a -t --review
```

### 多语言团队协作
```bash
# 中文团队使用中文输出
gitai --lang=zh review --focus="性能优化"
gitai --lang=zh scan src/ --stats --verbose

# 国际团队使用英文输出
gitai --lang=en review --focus="security issues"
gitai --lang=en scan --format=json --output=scan-results.json

# 多语言环境自动适配
gitai --lang=auto commit -a -t
gitai --lang=auto scan src/ --parallel

# 团队标准化：统一使用英文输出便于协作
gitai --lang=en review --format=json --output=team-review.json
gitai --lang=en scan --format=sarif --output=security-scan.sarif
```

### 代码质量治理
```bash
# 技术债务分析
gitai review --focus="技术债务,可维护性" --depth=deep

# 多语言代码扫描报告
gitai --lang=zh scan src/ --stats --max-issues=50 > 代码质量报告.txt
gitai --lang=en scan src/ --format=json --output=quality-report.json
```

# 安全审计
gitai review --focus="安全性" --tree-sitter --format=markdown \
  --output=security-audit-$(date +%Y%m%d).md

# 性能分析
gitai review --focus="性能,优化" --tree-sitter
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
# 使用自定义关注点
gitai review --focus="内存安全,并发安全,API 设计"

# 特定文件类型分析
gitai review --tree-sitter --files="*.rs,*.py"

# 排除特定目录
gitai review --exclude="tests/*,examples/*"
```

### 集成外部工具
```bash
# 与 git hooks 集成
echo 'gitai commit -a -t --review' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# 与编辑器集成（VS Code）
echo '{"command": "gitai commit -t"}' > .vscode/tasks.json
```

### 🌐 翻译功能高级用法
```bash
# 翻译缓存管理
gitai scan --lang=zh src/ --force-scan  # 强制重新翻译
gitai scan --lang=en src/ --use-cache   # 使用翻译缓存
rm -rf ~/.cache/gitai/translation/*     # 清理翻译缓存

# 批量多语言报告生成
gitai --lang=zh scan src/ --format=json --output=report-zh.json
gitai --lang=en scan src/ --format=json --output=report-en.json

# 团队多语言工作流
# 中文团队内部评审
gitai --lang=zh review --focus="代码质量,性能" --format=markdown > 内部评审.md

# 英文国际团队评审
gitai --lang=en review --focus="security,maintainability" --format=json > team-review.json

# 自动语言环境适配脚本
#!/bin/bash
LANG_ENV=$(echo $LANG | cut -d'_' -f1)
if [[ "$LANG_ENV" == "zh" ]]; then
    gitai --lang=zh scan src/ --stats
else
    gitai --lang=en scan src/ --stats
fi

# 配置文件驱动的多语言支持
# 根据项目配置自动选择输出语言
gitai scan src/ --lang=$(grep default_language ~/.config/gitai/config.toml | cut -d'"' -f2)

# CI/CD 集成翻译功能
# .github/workflows/code-analysis.yml
name: Code Analysis
on: [push, pull_request]
jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install GitAI
        run: curl -sSL https://example.com/install-gitai.sh | bash
      - name: Analyze Code (Chinese)
        run: gitai --lang=zh scan . --format=markdown > scan-zh.md
      - name: Analyze Code (English)
        run: gitai --lang=en scan . --format=markdown > scan-en.md
      - name: Upload Reports
        uses: actions/upload-artifact@v3
        with:
          name: analysis-reports
          path: scan-*.md

# 高级缓存管理
gitai scan --lang=zh src/ --translation-cache-info  # 显示缓存统计
gitai scan --lang=zh src/ --warm-translation-cache  # 预热翻译缓存
gitai scan --lang=zh src/ --translation-perf-stats  # 显示翻译性能数据

# 性能测试与优化
time gitai --lang=zh scan src/ --use-cache          # 测量使用缓存的性能
time gitai --lang=zh scan src/ --force-scan         # 测量强制翻译的性能
```

## 🔧 故障排除

### 常见问题

#### 翻译功能问题

**问题: 翻译不生效，输出仍为默认语言**
- 检查配置文件中的 `translation.enabled = true`
- 确认 `--lang` 参数设置正确
- 验证 API 密钥有效性
- 尝试使用 `--force-scan` 参数强制刷新翻译

```bash
# 诊断命令
gitai --lang=zh scan src/ --verbose --debug-translation
```

**问题: 翻译速度慢**
- 启用缓存 `translation.cache_enabled = true`
- 检查网络连接
- 使用 `--use-cache` 参数优化性能
- 考虑使用更快的翻译模型

**问题: 翻译缓存错误**
- 清理缓存目录 `rm -rf ~/.cache/gitai/translation/*`
- 检查缓存目录权限
- 确保缓存目录存在 `mkdir -p ~/.cache/gitai/translation`

**问题: 翻译API密钥错误**
- 检查环境变量 `GITAI_TRANSLATION_API_KEY`
- 确认配置文件中的 API 密钥正确
- 验证 API 密钥有足够的权限和额度

```bash
# 翻译调试命令
export GITAI_DEBUG=true
export GITAI_TRANSLATION_DEBUG=true
gitai --lang=zh scan src/
```

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
gitai review --depth=basic

# 检查本地 AI 服务
curl http://localhost:11434/api/tags
```

**Q: Tree-sitter 分析失败**
```bash
# 启用详细日志
RUST_LOG=debug gitai review --tree-sitter

# 检查语言支持
gitai review --tree-sitter --languages=rust,python

# 回退到基础分析
gitai review  # 不使用 --tree-sitter
```

**Q: 提交信息生成质量不佳**
```bash
# 自定义提示词模板
cp ~/.config/gitai/prompts/commit-generator.md \
   ~/.config/gitai/prompts/commit-generator.md.backup

# 编辑模板
vim ~/.config/gitai/prompts/commit-generator.md

# 测试新模板
gitai commit --dry-run
```

### 调试模式

#### 翻译调试

要调试翻译问题，可以使用以下环境变量和参数:

```bash
# 启用详细日志
export GITAI_DEBUG=true
export GITAI_TRANSLATION_DEBUG=true

# 显示翻译API请求和响应
gitai --lang=zh scan src/ --trace-translation

# 检查翻译性能
gitai scan --lang=zh src/ --translation-perf-stats

# 查看翻译缓存状态
gitai scan --lang=zh src/ --translation-cache-info
```

以下是常见的翻译问题及其调试方法:

1. **API连接问题**: 使用 `--trace-translation` 查看API请求详情
2. **缓存不一致**: 使用 `--force-scan` 强制刷新缓存
3. **性能瓶颈**: 使用 `--translation-perf-stats` 分析性能瓶颈
```bash
# 启用详细日志
RUST_LOG=debug gitai review

# 跟踪 AI 请求
RUST_LOG=gitai::handlers::ai=trace gitai commit

# 性能分析
time gitai review --tree-sitter --depth=deep
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
cargo run -- commit -t --review

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
4. **使用 GitAI 评审** (`gitai review --tree-sitter`)
5. **智能提交** (`gitai commit -t --review`)
6. **创建 Pull Request**

### 代码规范
- 使用 `rustfmt` 格式化代码
- 遵循 Rust 官方编码规范
- 为新功能添加测试
- 更新相关文档

## 📈 版本路线图

### v0.1.x - 基础功能
- [x] 基础代码评审
- [x] AI 提交信息生成
- [x] Tree-sitter 语法分析
- [x] DevOps 集成
- [x] 智能 Git 代理
- [x] Issue ID 关联提交

### v0.2.x - 增强功能
- [ ] 更多 DevOps 平台支持 (Jira, Azure DevOps)
- [ ] 自定义分析规则引擎
- [ ] 实时代码分析
- [ ] IDE 插件支持 (VS Code, IntelliJ)

### v0.3.x - 企业功能
- [ ] 团队协作功能
- [ ] 分析报告看板
- [ ] 代码质量趋势分析
- [ ] 自定义 AI 模型训练

### v1.0.x - 产品级
- [ ] 企业级部署方案
- [ ] 高可用集群支持
- [ ] 详细的权限管理
- [ ] 完整的 API 生态

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
