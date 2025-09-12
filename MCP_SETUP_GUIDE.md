# GitAI MCP 配置指南

## 快速开始

### 1. 基础配置（已完成）

配置文件位置：`~/.config/gitai/config.toml`

当前已启用的MCP服务：
- `review` - 代码评审
- `commit` - 智能提交  
- `scan` - 安全扫描
- `analysis` - 代码分析
- `dependency` - 依赖分析
- `deviation` - 偏离度分析

### 2. 配置AI服务

#### 选项A：使用本地Ollama（推荐）

1. 安装Ollama：
```bash
# macOS
brew install ollama

# 启动Ollama
ollama serve
```

2. 下载模型：
```bash
# 推荐模型
ollama pull qwen2.5:32b
# 或者
ollama pull codellama:13b
ollama pull deepseek-coder:33b
```

3. 修改配置文件 `~/.config/gitai/config.toml`：
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"  # 或你下载的其他模型
# api_key = ""  # Ollama不需要
temperature = 0.3  # 代码任务建议低温度
```

#### 选项B：使用OpenAI API

```toml
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"  # 或 gpt-3.5-turbo
api_key = "sk-your-api-key"
temperature = 0.3
```

#### 选项C：使用Claude API

```toml
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
temperature = 0.3
```

### 3. 配置DevOps平台（可选）

#### Coding.net配置
```toml
[devops]
platform = "coding"
base_url = "https://your-team.coding.net"
token = "your-personal-access-token"
project = "your-team/your-project"
space_id = 12345  # 你的空间ID
```

#### GitHub配置（即将支持）
```toml
[devops]
platform = "github"
base_url = "https://api.github.com"
token = "ghp_your_personal_access_token"
project = "owner/repo"
```

### 4. 启动MCP服务器

#### 方式1：独立运行（推荐用于Claude Desktop）

```bash
# 使用stdio传输（默认）
./target/release/gitai mcp --transport stdio

# 使用TCP传输
./target/release/gitai mcp --transport tcp --addr 127.0.0.1:8711

# 使用HTTP/SSE传输
./target/release/gitai mcp --transport sse --addr 127.0.0.1:8711
```

#### 方式2：配置Claude Desktop

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "gitai": {
      "command": "/Users/huchen/Projects/gitai/target/release/gitai",
      "args": ["mcp", "--transport", "stdio"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

或者使用绝对路径：

```json
{
  "mcpServers": {
    "gitai": {
      "command": "/usr/local/bin/gitai",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

### 5. 测试MCP服务

#### 测试服务器启动
```bash
# 启动服务器并发送测试请求
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | ./target/release/gitai mcp --transport stdio
```

#### 测试工具列表
```bash
# 获取可用工具
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | ./target/release/gitai mcp --transport stdio
```

### 6. 使用MCP工具

在Claude或其他支持MCP的客户端中，你可以使用以下工具：

#### execute_review - 代码评审
```json
{
  "name": "execute_review",
  "arguments": {
    "path": ".",
    "format": "text",
    "security_scan": true,
    "tree_sitter": false,
    "issue_ids": ["#123"]
  }
}
```

#### execute_commit - 智能提交
```json
{
  "name": "execute_commit",
  "arguments": {
    "message": "自定义提交信息",
    "issue_ids": ["#123", "#456"],
    "add_all": false,
    "dry_run": true,
    "review": false
  }
}
```

#### execute_scan - 安全扫描
```json
{
  "name": "execute_scan",
  "arguments": {
    "path": ".",
    "lang": "rust",
    "timeout": 300,
    "tool": "opengrep"
  }
}
```

#### execute_analysis - 代码分析
```json
{
  "name": "execute_analysis",
  "arguments": {
    "path": "src",
    "language": "rust",
    "verbosity": 1
  }
}
```

#### execute_dependency_graph - 依赖图
```json
{
  "name": "execute_dependency_graph",
  "arguments": {
    "path": ".",
    "format": "dot",
    "depth": 3,
    "verbosity": 1
  }
}
```

### 7. 故障排除

#### 问题：MCP服务器无法启动
```bash
# 检查配置
./target/release/gitai config check

# 启用调试日志
RUST_LOG=debug ./target/release/gitai mcp --transport stdio
```

#### 问题：AI服务连接失败
```bash
# 测试Ollama连接
curl http://localhost:11434/api/tags

# 测试OpenAI连接
curl -H "Authorization: Bearer $GITAI_AI_API_KEY" https://api.openai.com/v1/models
```

#### 问题：工具调用失败
```bash
# 检查工具是否可用
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/gitai mcp --transport stdio | jq .
```

### 8. 环境变量（可选）

你也可以通过环境变量覆盖配置：

```bash
# AI配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
export GITAI_AI_API_KEY="your-key"

# DevOps配置
export GITAI_DEVOPS_TOKEN="your-token"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"
export GITAI_DEVOPS_SPACE_ID="12345"

# 启动MCP服务器
./target/release/gitai mcp --transport stdio
```

### 9. 安装到系统（推荐）

```bash
# 安装到系统路径
sudo cp ./target/release/gitai /usr/local/bin/
sudo cp ./target/release/gitai-mcp /usr/local/bin/

# 验证安装
gitai --help
gitai-mcp --help
```

### 10. 完整示例：配置Ollama + Claude Desktop

1. 安装并启动Ollama：
```bash
brew install ollama
ollama serve
ollama pull qwen2.5:32b
```

2. 更新GitAI配置：
```bash
# 编辑配置
vim ~/.config/gitai/config.toml

# 修改[ai]部分为：
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"
temperature = 0.3
```

3. 配置Claude Desktop：
```bash
# 创建配置文件
cat > ~/Library/Application\ Support/Claude/claude_desktop_config.json << 'EOF'
{
  "mcpServers": {
    "gitai": {
      "command": "/usr/local/bin/gitai",
      "args": ["mcp", "--transport", "stdio"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
EOF
```

4. 重启Claude Desktop，在对话中就可以使用GitAI的工具了！

## 常用命令速查

```bash
# 初始化配置
gitai init

# 检查配置
gitai config check

# 启动MCP服务器
gitai mcp --transport stdio

# 代码评审（命令行）
gitai review --security-scan

# 智能提交（命令行）
gitai commit --issue-id "#123"

# 安全扫描（命令行）
gitai scan --auto-install

# 项目质量评估
gitai evaluate --format json
```

## 支持的客户端

- ✅ Claude Desktop (macOS/Windows)
- ✅ Continue (VS Code)
- ✅ 任何支持MCP协议的LLM客户端
- 🚧 自定义Web界面（开发中）

---

更多信息请参考：
- [MCP协议规范](https://modelcontextprotocol.io)
- [GitAI文档](docs/features/MCP_SERVICE.md)
