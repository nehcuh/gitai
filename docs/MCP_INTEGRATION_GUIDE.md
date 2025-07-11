# GitAI MCP 服务集成指南

> 🔧 **Model Context Protocol (MCP) 服务配置和使用指南**

## 📋 概览

GitAI 支持通过 Model Context Protocol (MCP) 提供服务化的AI代码分析能力。MCP是一个标准化的协议，允许AI客户端（如ChatWise、Claude Desktop等）与外部工具进行交互。

### ✨ MCP服务特性

- 🤖 **AI代码评审** - 深度代码分析和改进建议
- 📊 **Git状态查询** - 实时获取仓库状态信息
- 🔍 **代码差异分析** - 智能比较代码变更
- 💬 **智能提交** - AI生成的提交消息和自动提交
- 🛡️ **安全扫描** - 代码安全漏洞检测（支持缓存）
- 🌍 **多仓库支持** - 支持指定不同Git仓库路径

## 🚀 快速开始

### 1. 编译MCP服务器

```bash
# 进入项目目录
cd /path/to/gitai

# 编译MCP服务器
cargo build --release

# 验证编译结果
./target/release/mcp_server --help
```

### 2. 启动MCP服务器（独立模式）

```bash
# 启动服务器
./target/release/mcp_server

# 输出示例：
# MCP Server started successfully
# Listening for connections...
```

## 🎯 MCP客户端配置

### ChatWise 配置

在ChatWise的MCP设置中添加以下配置：

```json
{
  "name": "gitai-mcp-server",
  "command": "/path/to/gitai/target/release/mcp_server",
  "args": [],
  "env": {
    "RUST_LOG": "info"
  }
}
```

### Claude Desktop 配置

在 `~/.claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "gitai": {
      "command": "/path/to/gitai/target/release/mcp_server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 其他MCP客户端

GitAI MCP服务遵循标准MCP协议，兼容所有支持MCP的客户端。配置方式类似，指定可执行文件路径即可。

## 🛠️ 可用的MCP工具

### 1. gitai_review - AI代码评审

**功能**：执行AI驱动的代码评审，提供详细的分析和改进建议。

**参数**：
```json
{
  "depth": "medium",           // 可选: shallow, medium, deep
  "focus": "性能优化,安全性",    // 可选: 特定关注领域
  "language": "rust",          // 可选: 限制分析语言
  "format": "markdown",        // 可选: 输出格式 (text, markdown, json)
  "path": "/path/to/repo"      // 可选: 指定Git仓库路径
}
```

**使用示例**：
```json
// 基础评审（使用当前目录）
{}

// 深度评审with特定关注点
{
  "depth": "deep",
  "focus": "安全性,性能,可维护性",
  "format": "markdown"
}

// 指定仓库路径的评审
{
  "path": "/Users/username/projects/my-repo",
  "depth": "medium"
}
```

### 2. gitai_status - Git状态查询

**功能**：获取Git仓库的当前状态信息。

**参数**：
```json
{
  "detailed": true,           // 可选: 是否返回详细状态
  "path": "/path/to/repo"     // 可选: 指定Git仓库路径
}
```

**使用示例**：
```json
// 基础状态查询
{"detailed": false}

// 详细状态查询
{"detailed": true}

// 查询特定仓库状态
{
  "path": "/Users/username/projects/my-repo",
  "detailed": true
}
```

### 3. gitai_diff - 代码差异分析

**功能**：获取和分析代码变更差异。

**参数**：
```json
{
  "staged": true,              // 可选: 显示已暂存的更改
  "file_path": "src/main.rs",  // 可选: 特定文件路径
  "path": "/path/to/repo"      // 可选: 指定Git仓库路径
}
```

**使用示例**：
```json
// 查看暂存区差异
{"staged": true}

// 查看特定文件差异
{
  "file_path": "src/main.rs",
  "staged": false
}

// 跨仓库差异查询
{
  "path": "/Users/username/projects/my-repo",
  "staged": true
}
```

### 4. gitai_commit - 智能提交

**功能**：AI生成提交信息并执行提交操作。

**参数**：
```json
{
  "message": "feat: add feature",  // 可选: 自定义提交信息
  "auto_stage": true,             // 可选: 自动暂存文件
  "tree_sitter": true,            // 可选: 启用语法分析
  "issue_id": "#123",             // 可选: 关联issue ID
  "path": "/path/to/repo"         // 可选: 指定Git仓库路径
}
```

**使用示例**：
```json
// AI自动生成提交信息
{}

// 带issue关联的智能提交
{
  "auto_stage": true,
  "tree_sitter": true,
  "issue_id": "#123,#456"
}

// 自定义信息+AI增强
{
  "message": "fix: 修复登录问题",
  "tree_sitter": true
}
```

### 5. gitai_scan - 安全扫描

**功能**：执行代码安全和质量扫描，支持智能缓存。

**参数**：
```json
{
  "path": ".",                 // 可选: 指定扫描路径
  "full_scan": true,           // 可选: 是否执行全量扫描
  "update_rules": false,       // 可选: 是否更新扫描规则
  "show_results": true,        // 可选: 是否展示详细扫描结果
  "repository_path": "/path/to/repo"  // 可选: 指定Git仓库路径
}
```

**返回模式**：

- **基础模式** (`show_results: false` 或未设置)：
  - 扫描状态和基本统计
  - 简要问题汇总
  - 结果文件保存路径

- **详细模式** (`show_results: true`)：
  - 完整的扫描结果分析
  - 安全问题详细列表
  - 严重性分布统计
  - 具体修复建议

**缓存特性**：
- 自动缓存扫描结果（24小时有效期）
- 相同参数扫描优先使用缓存
- 大幅提升重复扫描响应速度

**使用示例**：
```json
// 基础快速扫描
{"show_results": false}

// 详细全量扫描
{
  "full_scan": true,
  "show_results": true,
  "update_rules": true
}

// 跨仓库扫描
{
  "repository_path": "/Users/username/projects/my-repo",
  "path": "src/",
  "show_results": true
}
```

## 💡 使用场景和最佳实践

### 开发工作流集成

```bash
# 典型的MCP辅助开发流程

# 1. 查看当前状态
gitai_status: {"detailed": true}

# 2. 查看变更
gitai_diff: {"staged": true}

# 3. 代码评审
gitai_review: {"depth": "medium", "focus": "安全性"}

# 4. 安全扫描
gitai_scan: {"show_results": true}

# 5. 智能提交
gitai_commit: {"auto_stage": true, "tree_sitter": true}
```

### 跨仓库代码分析

```json
// 分析多个项目
{
  "path": "/Users/username/projects/frontend-repo",
  "depth": "deep",
  "focus": "性能优化"
}

{
  "path": "/Users/username/projects/backend-repo", 
  "depth": "deep",
  "focus": "安全性"
}
```

### CI/CD集成

MCP服务可以集成到CI/CD流程中：

```yaml
# GitHub Actions 示例
- name: GitAI Code Review
  run: |
    # 通过MCP客户端调用GitAI服务
    mcp-client gitai_review '{"depth": "deep", "format": "json"}' > review.json
    mcp-client gitai_scan '{"full_scan": true, "show_results": true}' > scan.json
```

## 🔧 配置和环境

### 环境变量

```bash
# AI服务配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
export GITAI_AI_API_KEY="your_api_key"  # 可选

# DevOps集成
export GITAI_DEVOPS_PLATFORM="coding"
export GITAI_DEVOPS_BASE_URL="https://your-org.coding.net"
export GITAI_DEVOPS_TOKEN="your_api_token"

# MCP服务配置
export RUST_LOG="info"
export GITAI_MCP_TIMEOUT="30000"
```

### 配置文件

MCP服务使用与CLI相同的配置文件 `~/.config/gitai/config.toml`：

```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:32b"
temperature = 0.3

[scan]
results_path = "~/.gitai/scan-results"
cache_enabled = true
cache_ttl_hours = 24

[mcp]
timeout_ms = 30000
max_concurrent_requests = 5
```

## 🚨 故障排除

### 常见问题

**Q: MCP工具调用失败**
```bash
# 解决方案：
# 1. 确保在正确的Git仓库目录中或指定了正确的path参数
# 2. 确认有已暂存的文件（git status应显示暂存文件）
# 3. 检查MCP服务器日志

# 验证步骤：
git status
git add .
# 然后重试MCP调用
```

**Q: 空的评审结果**
```bash
# 解决方案：
# 1. 确认git add已正确暂存文件
# 2. 检查是否在Git仓库根目录或指定了正确的path
# 3. 验证文件确实有变更

git diff --cached  # 应该显示暂存的变更
```

**Q: MCP服务器连接问题**
```bash
# 解决方案：
# 1. 确认服务器正在运行
./target/release/mcp_server

# 2. 检查客户端MCP配置
# 3. 验证可执行文件路径正确
which /path/to/gitai/target/release/mcp_server
```

**Q: AI服务连接超时**
```bash
# 解决方案：
# 1. 检查AI服务是否运行
curl http://localhost:11434/api/tags

# 2. 调整超时设置
export GITAI_MCP_TIMEOUT="60000"

# 3. 使用更轻量的模型
export GITAI_AI_MODEL="qwen2.5:7b"
```

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug ./target/release/mcp_server

# 跟踪MCP通信
RUST_LOG=gitai::mcp=trace ./target/release/mcp_server

# 性能分析
time ./target/release/mcp_server
```

## 📊 性能优化

### 缓存策略

- **扫描结果缓存**：24小时自动缓存
- **AI响应缓存**：相同请求复用结果
- **配置缓存**：减少重复配置读取

### 并发控制

- **最大并发请求**：默认5个
- **请求队列**：超出限制自动排队
- **超时保护**：防止长时间阻塞

### 资源管理

- **内存优化**：流式处理大文件
- **连接池**：复用AI服务连接
- **优雅关闭**：确保请求完整处理

## 🔗 相关文档

- [MCP协议规范](https://spec.modelcontextprotocol.io/)
- [GitAI配置指南](CONFIGURATION_REFERENCE.md)
- [API文档](API_DOCUMENTATION.md)
- [故障排除指南](TROUBLESHOOTING.md)

---

**💡 提示**: MCP服务让GitAI的AI能力可以轻松集成到任何支持MCP的客户端中，实现无缝的智能代码分析体验！