# GitAI API 文档

> 📖 **完整的 CLI 命令和配置选项参考**

## 📋 目录

- [CLI 命令参考](#cli-命令参考)
- [配置选项](#配置选项)
- [环境变量](#环境变量)
- [返回值和错误码](#返回值和错误码)
- [使用示例](#使用示例)

## 🚀 CLI 命令参考

### 主命令

```bash
gitai [OPTIONS] <SUBCOMMAND>
```

#### 全局选项

| 选项 | 短选项 | 描述 | 默认值 |
|------|--------|------|--------|
| `--config` | `-c` | 指定配置文件路径 | `~/.config/gitai/config.toml` |
| `--verbose` | `-v` | 启用详细输出 | `false` |
| `--quiet` | `-q` | 静默模式 | `false` |
| `--help` | `-h` | 显示帮助信息 | - |
| `--version` | `-V` | 显示版本信息 | - |

### 子命令

#### 1. commit / c - 提交消息生成

```bash
gitai commit [OPTIONS]
```

**功能**: 使用 AI 生成智能提交消息

**选项**:

| 选项 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--message` | `String` | 自定义提交消息 | - |
| `--auto-confirm` | `Bool` | 自动确认提交 | `false` |
| `--tree-sitter` | `Bool` | 使用 TreeSitter 分析 | `false` |
| `--temperature` | `Float` | AI 生成温度 (0.0-1.0) | `0.7` |
| `--prompt` | `String` | 自定义提示词 | - |
| `--issue-id` | `String` | 关联的问题 ID | - |
| `--format` | `String` | 输出格式 (text/json) | `text` |

**示例**:
```bash
# 基础使用
gitai commit

# 自定义温度
gitai commit --temperature 0.5

# 自动确认
gitai commit --auto-confirm

# 使用 TreeSitter 分析
gitai commit --tree-sitter

# 关联问题 ID
gitai commit --issue-id TASK-123
```

#### 2. review / rv - 代码审查

```bash
gitai review [OPTIONS]
```

**功能**: 执行 AI 驱动的代码审查

**选项**:

| 选项 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--space-id` | `String` | DevOps 空间 ID | - |
| `--stories` | `String` | 用户故事 ID 列表 | - |
| `--tasks` | `String` | 任务 ID 列表 | - |
| `--defects` | `String` | 缺陷 ID 列表 | - |
| `--depth` | `String` | 分析深度 (basic/normal/deep) | `normal` |
| `--format` | `String` | 输出格式 (text/json/markdown) | `text` |
| `--output` | `String` | 输出文件路径 | - |
| `--focus` | `String` | 重点关注领域 | - |
| `--no-ai-analysis` | `Bool` | 禁用 AI 分析 | `false` |

**示例**:
```bash
# 基础审查
gitai review

# 与工作项集成
gitai review --space-id 12345 --stories 99,100

# 深度分析
gitai review --depth deep

# 重点关注安全性
gitai review --focus "安全性,性能"

# 输出到文件
gitai review --output review-report.md --format markdown
```

#### 3. scan - 安全扫描

```bash
gitai scan [OPTIONS] [PATH]
```

**功能**: 执行代码安全扫描

**选项**:

| 选项 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--path` | `String` | 扫描路径 | `.` |
| `--full` | `Bool` | 全量扫描 | `false` |
| `--remote` | `Bool` | 使用远程规则 | `false` |
| `--update-rules` | `Bool` | 更新规则 | `false` |
| `--output` | `String` | 输出文件路径 | - |
| `--format` | `String` | 输出格式 (json/text/sarif) | `json` |
| `--rules` | `String` | 自定义规则文件 | - |
| `--exclude` | `String` | 排除模式 | - |

**示例**:
```bash
# 扫描当前目录
gitai scan

# 扫描特定路径
gitai scan src/

# 全量扫描
gitai scan --full

# 更新规则后扫描
gitai scan --update-rules

# 输出到文件
gitai scan --output security-report.json --format json
```

#### 4. translate - 规则翻译

```bash
gitai translate [OPTIONS]
```

**功能**: 翻译安全规则到不同语言

**选项**:

| 选项 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--from-language` | `String` | 源语言 | `javascript` |
| `--to-language` | `String` | 目标语言 | `rust` |
| `--input` | `String` | 输入规则文件/目录 | - |
| `--output` | `String` | 输出目录 | - |
| `--rules-dir` | `String` | 规则目录 | `./rules` |

**示例**:
```bash
# 翻译规则
gitai translate --from-language javascript --to-language rust

# 指定输入输出
gitai translate --input rules/js --output rules/rust --to-language rust
```

#### 5. mcp-server - MCP 服务

```bash
gitai mcp-server [OPTIONS]
```

**功能**: 启动 Model Context Protocol 服务

**选项**:

| 选项 | 类型 | 描述 | 默认值 |
|------|------|------|--------|
| `--port` | `u16` | 服务端口 | `8080` |
| `--host` | `String` | 绑定地址 | `localhost` |
| `--config` | `String` | MCP 配置文件 | - |
| `--log-level` | `String` | 日志级别 | `info` |

**示例**:
```bash
# 启动 MCP 服务
gitai mcp-server

# 自定义端口
gitai mcp-server --port 9000

# 指定配置
gitai mcp-server --config mcp-config.json
```

#### 6. help / h - 帮助信息

```bash
gitai help [SUBCOMMAND]
```

**功能**: 显示命令帮助信息

**示例**:
```bash
# 显示全部帮助
gitai help

# 显示特定命令帮助
gitai help commit
gitai help review
```

## ⚙️ 配置选项

### 配置文件结构

```toml
# ~/.config/gitai/config.toml

[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
api_key = ""
temperature = 0.7
max_tokens = 2048
timeout = 30

[git]
author_name = "Your Name"
author_email = "your.email@example.com"
signing_key = ""
commit_template = ""

[devops]
platform = "coding"
api_base_url = "https://your-company.devops.com"
api_token = ""
default_space_id = ""

[scanner]
rules_dir = "./rules"
exclude_patterns = ["*.test.js", "node_modules/**"]
default_severity = "medium"
enable_remote_rules = false

[mcp]
server_port = 8080
server_host = "localhost"
enable_tree_sitter = true
enable_ai_analysis = true
enable_devops_integration = true

[logging]
level = "info"
format = "text"
file = ""
```

### 配置选项详解

#### AI 配置 [ai]

| 配置项 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| `api_url` | `String` | AI API 端点 | `http://localhost:11434/v1/chat/completions` |
| `model_name` | `String` | 模型名称 | `qwen2.5:7b` |
| `api_key` | `String` | API 密钥 | `""` |
| `temperature` | `Float` | 生成温度 (0.0-1.0) | `0.7` |
| `max_tokens` | `Integer` | 最大令牌数 | `2048` |
| `timeout` | `Integer` | 请求超时时间 (秒) | `30` |

#### Git 配置 [git]

| 配置项 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| `author_name` | `String` | 作者姓名 | 从 git config 读取 |
| `author_email` | `String` | 作者邮箱 | 从 git config 读取 |
| `signing_key` | `String` | 签名密钥 | `""` |
| `commit_template` | `String` | 提交消息模板 | `""` |

#### DevOps 配置 [devops]

| 配置项 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| `platform` | `String` | 平台类型 | `coding` |
| `api_base_url` | `String` | API 基础 URL | `""` |
| `api_token` | `String` | API 令牌 | `""` |
| `default_space_id` | `String` | 默认空间 ID | `""` |

#### 扫描器配置 [scanner]

| 配置项 | 类型 | 描述 | 默认值 |
|--------|------|------|--------|
| `rules_dir` | `String` | 规则目录 | `./rules` |
| `exclude_patterns` | `Array<String>` | 排除模式 | `[]` |
| `default_severity` | `String` | 默认严重程度 | `medium` |
| `enable_remote_rules` | `Bool` | 启用远程规则 | `false` |

## 🌍 环境变量

GitAI 支持以下环境变量：

### AI 相关

| 环境变量 | 描述 | 配置等价 |
|----------|------|----------|
| `GITAI_AI_API_URL` | AI API 端点 | `ai.api_url` |
| `GITAI_AI_MODEL` | AI 模型名称 | `ai.model_name` |
| `GITAI_AI_API_KEY` | AI API 密钥 | `ai.api_key` |
| `GITAI_AI_TEMPERATURE` | 生成温度 | `ai.temperature` |

### DevOps 相关

| 环境变量 | 描述 | 配置等价 |
|----------|------|----------|
| `DEV_DEVOPS_API_BASE_URL` | DevOps API 基础 URL | `devops.api_base_url` |
| `DEV_DEVOPS_API_TOKEN` | DevOps API 令牌 | `devops.api_token` |
| `DEV_DEVOPS_DEFAULT_SPACE_ID` | 默认空间 ID | `devops.default_space_id` |

### 系统相关

| 环境变量 | 描述 | 默认值 |
|----------|------|--------|
| `GITAI_CONFIG_PATH` | 配置文件路径 | `~/.config/gitai/config.toml` |
| `RUST_LOG` | 日志级别 | `info` |
| `RUST_BACKTRACE` | 错误回溯 | `0` |

## 🔢 返回值和错误码

### 返回值

| 返回值 | 描述 |
|--------|------|
| `0` | 成功执行 |
| `1` | 一般错误 |
| `2` | 配置错误 |
| `3` | Git 错误 |
| `4` | AI 服务错误 |
| `5` | 网络错误 |
| `6` | 文件操作错误 |

### 错误码

#### 配置错误 (CONFIG_*)

| 错误码 | 描述 |
|--------|------|
| `CONFIG_001` | 配置文件未找到 |
| `CONFIG_002` | 配置文件解析错误 |
| `CONFIG_003` | 配置验证错误 |

#### Git 错误 (GIT_*)

| 错误码 | 描述 |
|--------|------|
| `GIT_001` | Git 命令执行失败 |
| `GIT_002` | 不是 Git 仓库 |
| `GIT_003` | 没有暂存的更改 |

#### AI 错误 (AI_*)

| 错误码 | 描述 |
|--------|------|
| `AI_001` | AI 请求失败 |
| `AI_002` | AI 响应解析错误 |
| `AI_003` | AI API 错误 |
| `AI_004` | AI 响应为空 |

#### DevOps 错误 (DEVOPS_*)

| 错误码 | 描述 |
|--------|------|
| `DEVOPS_001` | 网络错误 |
| `DEVOPS_002` | 认证错误 |
| `DEVOPS_003` | 资源未找到 |
| `DEVOPS_004` | 速率限制 |

## 📚 使用示例

### 基础工作流

```bash
# 1. 配置 GitAI
cat > ~/.config/gitai/config.toml << EOF
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
temperature = 0.7

[git]
author_name = "Developer"
author_email = "dev@example.com"
EOF

# 2. 修改代码
echo "console.log('Hello, GitAI!');" > app.js

# 3. 添加到暂存区
git add app.js

# 4. 生成提交消息
gitai commit

# 5. 审查代码
gitai review

# 6. 扫描安全问题
gitai scan
```

### 高级工作流

```bash
# 1. 深度代码审查
gitai review --depth deep --focus "安全性,性能,可维护性"

# 2. 与工作项集成
gitai review --space-id 12345 --stories 99,100 --tasks 201

# 3. 生成详细报告
gitai review --format markdown --output review-report.md

# 4. 自动化提交
gitai commit --auto-confirm --tree-sitter

# 5. 批量扫描
gitai scan --full --format sarif --output security.sarif
```

### MCP 服务集成

```bash
# 1. 启动 MCP 服务
gitai mcp-server --port 8080 &

# 2. 使用 MCP 客户端
curl -X POST http://localhost:8080/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "tool": "analyze",
    "args": {
      "code": "fn main() { println!(\"Hello\"); }",
      "language": "rust"
    }
  }'
```

### 批处理脚本

```bash
#!/bin/bash
# auto-commit.sh - 自动化提交脚本

set -e

echo "🚀 开始自动化提交流程..."

# 1. 检查是否有更改
if ! git diff --staged --quiet; then
    echo "📝 发现暂存的更改"
    
    # 2. 执行安全扫描
    echo "🛡️ 执行安全扫描..."
    gitai scan --format json --output scan-result.json
    
    # 3. 检查扫描结果
    if [ -f scan-result.json ]; then
        ISSUES=$(jq '.issues | length' scan-result.json)
        if [ "$ISSUES" -gt 0 ]; then
            echo "⚠️ 发现 $ISSUES 个安全问题，请先修复"
            exit 1
        fi
    fi
    
    # 4. 执行代码审查
    echo "🔍 执行代码审查..."
    gitai review --format json --output review-result.json
    
    # 5. 生成提交消息
    echo "✨ 生成提交消息..."
    gitai commit --auto-confirm --tree-sitter
    
    echo "✅ 提交完成！"
else
    echo "ℹ️ 没有发现暂存的更改"
fi
```

## 🔗 相关文档

- [快速入门指南](QUICK_START.md) - 5分钟上手
- [配置参考](CONFIGURATION_REFERENCE.md) - 详细配置说明
- [故障排除](TROUBLESHOOTING.md) - 解决常见问题
- [MCP API 参考](mcp-api/api-reference.md) - MCP 服务 API
- [开发指南](CONTRIBUTING.md) - 参与开发

---

**📝 注意**: 本文档会随着 GitAI 的更新而持续更新。如有疑问，请查阅最新的在线文档或提交 Issue。