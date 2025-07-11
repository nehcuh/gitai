# GitAI 配置参考

> ⚙️ **详细的配置选项和参数说明**

## 📋 目录

- [配置文件格式](#配置文件格式)
- [AI 服务配置](#ai-服务配置)
- [Git 配置](#git-配置)
- [DevOps 集成配置](#devops-集成配置)
- [安全扫描配置](#安全扫描配置)
- [MCP 服务配置](#mcp-服务配置)
- [日志配置](#日志配置)
- [性能配置](#性能配置)
- [环境变量](#环境变量)
- [配置验证](#配置验证)

## 📄 配置文件格式

### 默认配置文件位置

| 操作系统 | 配置文件路径 |
|----------|--------------|
| **Linux** | `~/.config/gitai/config.toml` |
| **macOS** | `~/.config/gitai/config.toml` |
| **Windows** | `%APPDATA%\gitai\config.toml` |

### 配置文件结构

```toml
# GitAI 配置文件
# 版本: 1.0.0
# 文档: https://gitai.docs.com/configuration

[ai]
# AI 服务相关配置

[git]
# Git 操作相关配置

[devops]
# DevOps 平台集成配置

[scanner]
# 安全扫描配置

[mcp]
# MCP 服务配置

[logging]
# 日志系统配置

[performance]
# 性能优化配置

[security]
# 安全相关配置
```

## 🤖 AI 服务配置

### 基础配置

```toml
[ai]
# AI 服务 API 端点
api_url = "http://localhost:11434/v1/chat/completions"

# 使用的 AI 模型名称
model_name = "qwen2.5:7b"

# API 密钥（可选，某些服务需要）
api_key = ""

# 生成温度 (0.0-1.0)
# 较低值生成更确定的结果，较高值生成更多样化的结果
temperature = 0.7

# 最大生成令牌数
max_tokens = 2048

# 请求超时时间（秒）
timeout = 30

# 最大重试次数
max_retries = 3

# 重试间隔（秒）
retry_delay = 2

# 启用流式响应
stream = false

# 系统提示词
system_prompt = """
你是一个专业的代码助手，请根据代码变更生成简洁、准确的提交消息。
提交消息应该遵循 Conventional Commits 规范。
"""
```

### 模型特定配置

#### OpenAI 配置

```toml
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model_name = "gpt-4"
api_key = "sk-your-api-key-here"
temperature = 0.7
max_tokens = 2048

# OpenAI 特定参数
top_p = 1.0
frequency_penalty = 0.0
presence_penalty = 0.0
```

#### Anthropic Claude 配置

```toml
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model_name = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
temperature = 0.7
max_tokens = 2048

# Claude 特定参数
top_k = 40
```

#### Ollama 配置

```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
# Ollama 通常不需要 API 密钥
api_key = ""
temperature = 0.7
max_tokens = 2048

# Ollama 特定参数
num_ctx = 4096      # 上下文窗口大小
num_predict = -1    # 预测令牌数 (-1 表示无限制)
repeat_penalty = 1.1 # 重复惩罚
```

### 高级 AI 配置

```toml
[ai]
# 批处理配置
batch_size = 10
batch_timeout = 60

# 缓存配置
enable_cache = true
cache_ttl = 3600    # 缓存过期时间（秒）
cache_size = 1000   # 缓存条目数

# 负载均衡配置
[[ai.endpoints]]
url = "http://ai-server-1:11434/v1/chat/completions"
weight = 1
timeout = 30

[[ai.endpoints]]
url = "http://ai-server-2:11434/v1/chat/completions"
weight = 2
timeout = 30

# 降级配置
[ai.fallback]
enabled = true
endpoint = "http://fallback-server:11434/v1/chat/completions"
model = "qwen2.5:3b"  # 使用较小的模型作为降级
```

## 📂 Git 配置

### 基础配置

```toml
[git]
# Git 作者信息
author_name = "Your Name"
author_email = "your.email@example.com"

# Git 签名密钥
signing_key = ""

# 提交消息模板
commit_template = """
{{type}}{{scope}}: {{description}}

{{body}}

{{footer}}
"""

# 自动添加文件到暂存区
auto_add = false

# 自动推送到远程仓库
auto_push = false

# 默认分支
default_branch = "main"

# 远程仓库名称
remote_name = "origin"
```

### 提交消息配置

```toml
[git.commit]
# 提交消息格式
format = "conventional"  # conventional, simple, detailed

# 提交类型映射
[git.commit.types]
feat = "新功能"
fix = "修复"
docs = "文档"
style = "格式"
refactor = "重构"
test = "测试"
chore = "构建/工具"

# 作用域映射
[git.commit.scopes]
ui = "用户界面"
api = "API接口"
db = "数据库"
config = "配置"
```

### 高级 Git 配置

```toml
[git]
# 工作目录配置
work_tree = "."
git_dir = ".git"

# 子模块处理
handle_submodules = true

# 大文件处理
lfs_enabled = true

# 钩子配置
[git.hooks]
pre_commit = true
pre_push = true
commit_msg = true

# 合并策略
[git.merge]
strategy = "recursive"
auto_resolve = false
```

## 🔗 DevOps 集成配置

### 基础配置

```toml
[devops]
# 平台类型
platform = "coding"  # coding, azure, jira, github

# API 基础 URL
api_base_url = "https://your-company.devops.com"

# API 访问令牌
api_token = "your-token-here"

# 默认空间/项目 ID
default_space_id = "12345"

# 请求超时
timeout = 30

# 最大重试次数
max_retries = 3
```

### 平台特定配置

#### Coding DevOps 配置

```toml
[devops]
platform = "coding"
api_base_url = "https://your-team.coding.net"
api_token = "your-coding-token"
default_space_id = "12345"

# Coding 特定配置
[devops.coding]
team_domain = "your-team"
project_name = "your-project"
```

#### Azure DevOps 配置

```toml
[devops]
platform = "azure"
api_base_url = "https://dev.azure.com/your-org"
api_token = "your-pat-token"
default_space_id = "your-project"

# Azure 特定配置
[devops.azure]
organization = "your-org"
project = "your-project"
api_version = "6.0"
```

#### Jira 配置

```toml
[devops]
platform = "jira"
api_base_url = "https://your-domain.atlassian.net"
api_token = "your-jira-token"
default_space_id = "PROJECT"

# Jira 特定配置
[devops.jira]
username = "your-email@example.com"
project_key = "PROJ"
```

### 工作项配置

```toml
[devops.work_items]
# 工作项类型映射
[devops.work_items.types]
story = "用户故事"
task = "任务"
bug = "缺陷"
epic = "史诗"

# 状态映射
[devops.work_items.statuses]
todo = "待办"
in_progress = "进行中"
done = "已完成"
blocked = "阻塞"

# 优先级映射
[devops.work_items.priorities]
high = "高"
medium = "中"
low = "低"
```

## 🛡️ 代码扫描配置

GitAI 使用分层配置架构，结合了 GitAI 高层策略配置和 ast-grep 底层执行配置。

### 配置架构概述

```
📊 GitAI 配置层 (config.toml)
├── 🎯 控制扫描行为和策略
├── 📥 管理规则下载和更新
├── 📁 设置存储路径
└── ⏰ 配置更新频率

     ⬇️ 使用

🔧 ast-grep 配置层 (sgconfig.yml)  
├── 📂 定义规则目录结构
├── 🛠️ 配置扫描引擎参数
├── 🧪 设置测试环境
└── 📋 指定规则查找路径
```

### GitAI 扫描配置 (高层配置)

```toml
[scan]
# 扫描结果存储路径
results_path = "~/.gitai/scan-results"

[scan.rule_manager]
# 规则缓存目录
cache_path = "~/.config/gitai/scan-rules"

# 规则源仓库 URL
url = "https://github.com/coderabbitai/ast-grep-essentials"

# 规则缓存有效期（小时）
ttl_hours = 24

# 是否自动更新规则
auto_update = true
```

### ast-grep 配置 (底层配置)

自动下载的规则包中包含 `sgconfig.yml` 文件：

```yaml
# ~/.config/gitai/scan-rules/ast-grep-essentials/sgconfig.yml
---
ruleDirs:
  - rules        # 规则文件目录
utilDirs:
  - utils        # 工具函数目录
testConfigs:
  - testDir: tests  # 测试文件目录
```

### 完整扫描配置示例

```toml
[scan]
# 扫描结果存储路径
results_path = "~/.gitai/scan-results"

[scan.rule_manager]
# 规则缓存目录（规则自动下载到此处）
cache_path = "~/.config/gitai/scan-rules"

# 规则源仓库（默认使用 coderabbitai/ast-grep-essentials）
url = "https://github.com/coderabbitai/ast-grep-essentials"

# 规则缓存TTL（24小时后检查更新）
ttl_hours = 24

# 自动更新规则
auto_update = true

# 网络超时时间（秒）
network_timeout = 30

# 最大重试次数
max_retries = 3
```

### 自定义规则源配置

```toml
[scan.rule_manager]
# 使用自定义规则仓库
url = "https://github.com/your-org/custom-ast-grep-rules"

# 更频繁的更新检查
ttl_hours = 6

# 自定义缓存路径
cache_path = "~/my-custom-scan-rules"
```

### 扫描命令选项

```bash
# 基础扫描命令
gitai scan                    # 增量扫描当前目录
gitai scan --full             # 全量扫描当前目录
gitai scan --path /project    # 扫描指定路径
gitai scan --update-rules     # 强制更新规则后扫描

# 输出和格式化
gitai scan --output results.json     # 保存结果到文件
gitai scan --remote                  # 使用远程扫描服务（如果配置）
```

### 扫描规则组织结构

下载的规则按语言分类存储：

```
~/.config/gitai/scan-rules/ast-grep-essentials/
├── sgconfig.yml              # ast-grep 主配置
├── rules/                    # 规则文件目录
│   ├── c/                   # C 语言规则
│   ├── cpp/                 # C++ 语言规则
│   ├── java/                # Java 语言规则
│   ├── javascript/          # JavaScript 规则
│   ├── python/              # Python 语言规则
│   ├── rust/                # Rust 语言规则
│   └── go/                  # Go 语言规则
├── utils/                   # 工具函数
└── tests/                   # 测试文件
```

### 支持的编程语言

| 语言 | 规则数量 | 支持状态 | 主要检查项 |
|------|----------|----------|------------|
| **Python** | 30+ | ✅ 完全支持 | 安全漏洞、性能、最佳实践 |
| **JavaScript** | 40+ | ✅ 完全支持 | XSS、注入、ES6+ 语法 |
| **Rust** | 25+ | ✅ 完全支持 | 内存安全、并发、性能 |
| **Java** | 35+ | ✅ 完全支持 | 安全、Spring Boot、JPA |
| **Go** | 20+ | ✅ 完全支持 | 并发、错误处理、性能 |
| **C/C++** | 30+ | ✅ 完全支持 | 内存管理、缓冲区溢出 |

### 规则更新机制

```toml
[scan.rule_manager]
# 自动检查更新的触发条件：
# 1. TTL 过期（默认24小时）
# 2. 手动强制更新 (--update-rules)
# 3. 本地规则缓存不存在

# 更新过程：
# 1. 检查远程仓库最新 commit hash
# 2. 与本地缓存的 hash 比较
# 3. 如需更新则 git pull 或 git clone
# 4. 验证下载的规则文件（80%有效性阈值）
# 5. 原子性替换本地规则
```

### 故障排除和维护

```bash
# 检查扫描状态
gitai scan --help

# 重新下载规则
gitai update-scan-rules

# 检查 ast-grep 安装状态
gitai check-ast-grep

# 安装或更新 ast-grep
gitai install-ast-grep

# 清理规则缓存（将触发重新下载）
rm -rf ~/.config/gitai/scan-rules
```

### 自定义扫描行为

虽然 GitAI 主要通过下载的规则包进行扫描，您也可以：

1. **修改 sgconfig.yml**（不推荐，会被更新覆盖）
2. **使用自定义规则源**（推荐）：

```toml
[scan.rule_manager]
# 指向您的私有规则仓库
url = "https://github.com/your-company/security-rules"
cache_path = "~/.config/gitai/custom-rules"
```

3. **fork 官方规则仓库** 并自定义规则内容

## 🔗 MCP 服务配置

### 基础配置

```toml
[mcp]
# 服务器端口
server_port = 8080

# 绑定地址
server_host = "localhost"

# 最大连接数
max_connections = 1000

# 连接超时（秒）
connection_timeout = 30

# 请求超时（秒）
request_timeout = 60

# 启用的服务
enabled_services = [
    "tree_sitter",
    "ai_analysis",
    "devops_integration"
]
```

### 服务特定配置

#### TreeSitter 服务配置

```toml
[mcp.tree_sitter]
# 启用 TreeSitter 服务
enabled = true

# 支持的语言
supported_languages = [
    "rust",
    "javascript",
    "typescript",
    "python",
    "java",
    "go"
]

# 缓存配置
cache_enabled = true
cache_size = 1000
cache_ttl = 3600

# 性能配置
max_file_size = 1048576  # 1MB
parse_timeout = 30
```

#### AI 分析服务配置

```toml
[mcp.ai_analysis]
# 启用 AI 分析服务
enabled = true

# 分析类型
analysis_types = [
    "code_quality",
    "security",
    "performance",
    "refactoring"
]

# 批处理配置
batch_size = 5
batch_timeout = 120

# 缓存配置
cache_enabled = true
cache_size = 500
cache_ttl = 1800
```

### 高级 MCP 配置

```toml
[mcp]
# TLS 配置
[mcp.tls]
enabled = false
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"

# 认证配置
[mcp.auth]
enabled = false
auth_type = "bearer"  # bearer, basic, api_key
api_key = "your-api-key"

# 限流配置
[mcp.rate_limit]
enabled = true
requests_per_minute = 1000
burst_size = 100

# 监控配置
[mcp.monitoring]
enabled = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
```

## 📊 日志配置

### 基础配置

```toml
[logging]
# 日志级别
level = "info"  # trace, debug, info, warn, error

# 日志格式
format = "text"  # text, json, pretty

# 输出文件
file = ""  # 空表示输出到控制台

# 日志轮转
rotation = "daily"  # daily, hourly, size

# 最大日志文件大小
max_size = "100MB"

# 保留日志文件数量
max_files = 10

# 时间格式
time_format = "%Y-%m-%d %H:%M:%S"

# 是否显示源代码位置
show_source = true

# 是否显示线程信息
show_thread = false
```

### 日志过滤配置

```toml
[logging.filter]
# 模块级别过滤
[logging.filter.modules]
"gitai::handlers" = "debug"
"gitai::ai" = "info"
"gitai::git" = "warn"

# 关键字过滤
excluded_keywords = [
    "password",
    "token",
    "secret",
    "key"
]

# 敏感信息替换
[logging.filter.replacements]
"api_key=\\S+" = "api_key=****"
"password=\\S+" = "password=****"
```

### 高级日志配置

```toml
[logging]
# 异步日志
async_logging = true

# 缓冲区大小
buffer_size = 1000

# 刷新间隔（毫秒）
flush_interval = 1000

# 结构化日志字段
[logging.structured]
service = "gitai"
version = "1.0.0"
environment = "production"

# 第三方日志系统集成
[logging.integrations]
# ELK Stack
elasticsearch_url = "http://localhost:9200"
logstash_host = "localhost:5044"

# Syslog
syslog_enabled = false
syslog_facility = "local0"
```

## ⚡ 性能配置

### 基础配置

```toml
[performance]
# 最大并发请求数
max_concurrent_requests = 100

# 连接池大小
connection_pool_size = 50

# 请求超时（秒）
request_timeout = 30

# 工作线程数
worker_threads = 8

# 缓存配置
cache_size = "512MB"
cache_ttl = 3600

# 内存限制
memory_limit = "2GB"
```

### 缓存配置

```toml
[performance.cache]
# 缓存后端
backend = "memory"  # memory, redis, file

# Redis 配置
redis_url = "redis://localhost:6379"
redis_pool_size = 10

# 文件缓存配置
file_cache_dir = "/tmp/gitai-cache"
file_cache_compression = true

# 缓存策略
[performance.cache.policies]
ai_responses = { ttl = 3600, size = 1000 }
git_diffs = { ttl = 1800, size = 500 }
analysis_results = { ttl = 7200, size = 200 }
```

### 优化配置

```toml
[performance.optimization]
# 启用压缩
compression_enabled = true
compression_level = 6

# 预加载
preload_models = true
preload_rules = true

# 并发控制
max_concurrent_ai_requests = 5
max_concurrent_git_operations = 10

# 资源限制
[performance.limits]
max_file_size = "10MB"
max_diff_size = "5MB"
max_code_length = 100000
```

## 🌍 环境变量

### 系统环境变量

```bash
# 配置文件路径
export GITAI_CONFIG_PATH="$HOME/.config/gitai/config.toml"

# 日志级别
export RUST_LOG="info"

# 错误回溯
export RUST_BACKTRACE="1"

# 工作目录
export GITAI_WORK_DIR="$PWD"
```

### AI 服务环境变量

```bash
# AI 服务配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:7b"
export GITAI_AI_API_KEY="your-api-key"
export GITAI_AI_TEMPERATURE="0.7"
export GITAI_AI_MAX_TOKENS="2048"
export GITAI_AI_TIMEOUT="30"
```

### DevOps 环境变量

```bash
# DevOps 配置
export DEV_DEVOPS_API_BASE_URL="https://your-company.devops.com"
export DEV_DEVOPS_API_TOKEN="your-token"
export DEV_DEVOPS_DEFAULT_SPACE_ID="12345"
export DEV_DEVOPS_TIMEOUT="30"
```

### 其他环境变量

```bash
# 代理设置
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
export NO_PROXY="localhost,127.0.0.1"

# 并发设置
export GITAI_MAX_CONCURRENT="10"

# 缓存设置
export GITAI_CACHE_DIR="$HOME/.cache/gitai"
export GITAI_CACHE_SIZE="1000"
```

### 环境变量优先级

1. **命令行参数** (最高优先级)
2. **环境变量**
3. **配置文件**
4. **默认值** (最低优先级)

## ✅ 配置验证

### 验证命令

```bash
# 验证配置文件
gitai config --validate

# 显示当前配置
gitai config --show

# 测试配置
gitai config --test

# 生成默认配置
gitai config --generate > ~/.config/gitai/config.toml
```

### 配置验证规则

```toml
# 验证规则示例
[validation]
# 必需字段
required_fields = [
    "ai.api_url",
    "ai.model_name",
    "git.author_name",
    "git.author_email"
]

# 值范围验证
[validation.ranges]
"ai.temperature" = { min = 0.0, max = 1.0 }
"ai.max_tokens" = { min = 1, max = 32768 }
"mcp.server_port" = { min = 1024, max = 65535 }

# 格式验证
[validation.formats]
"git.author_email" = "email"
"devops.api_base_url" = "url"
"ai.api_url" = "url"
```

### 常见配置错误

```bash
# 错误: 无效的 TOML 格式
Error: Configuration parse error: invalid TOML syntax at line 10

# 错误: 缺少必需字段
Error: Missing required field: ai.api_url

# 错误: 值超出范围
Error: ai.temperature must be between 0.0 and 1.0

# 错误: 无效的 URL 格式
Error: Invalid URL format in devops.api_base_url
```

## 🔧 配置示例

### 开发环境配置

```toml
# development.toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
temperature = 0.8
max_tokens = 1024

[git]
author_name = "Developer"
author_email = "dev@example.com"

[logging]
level = "debug"
format = "pretty"

[performance]
max_concurrent_requests = 10
cache_size = "100MB"
```

### 生产环境配置

```toml
# production.toml
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model_name = "gpt-4"
api_key = "${OPENAI_API_KEY}"
temperature = 0.7
max_tokens = 2048
timeout = 60

[git]
author_name = "GitAI Bot"
author_email = "gitai@company.com"

[logging]
level = "info"
format = "json"
file = "/var/log/gitai/gitai.log"

[performance]
max_concurrent_requests = 100
cache_size = "1GB"
worker_threads = 16

[mcp]
server_port = 8080
server_host = "0.0.0.0"
max_connections = 1000

[security]
enable_authentication = true
allowed_origins = ["https://company.com"]
```

---

**⚙️ 现在您可以根据需要自定义 GitAI 的各种配置选项了！**

记住在修改配置后使用 `gitai config --validate` 命令验证配置的正确性。