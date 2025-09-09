# MCP 服务 (Model Context Protocol Service)

## 功能概述

GitAI 的 MCP 服务实现了完整的 Model Context Protocol，为 LLM 客户端（如 Claude、ChatGPT）提供标准化的工具接口，实现 AI 辅助编程的无缝集成。

## 核心特性

### 1. 标准化协议
- 完整的 MCP 协议实现
- JSON-RPC 2.0 通信
- 工具发现和调用
- 错误处理规范

### 2. 多种传输方式
- **stdio**：标准输入输出（已实现）
- **TCP**：网络套接字（开发中）
- **SSE**：服务器发送事件（开发中）
- **WebSocket**：双向通信（计划中）

### 3. 服务注册表
- 动态服务注册/注销
- 服务依赖管理
- 版本控制
- 健康检查

### 4. 性能监控
- 实时性能统计
- 调用次数跟踪
- 响应时间分析
- 成功率监控

## 架构设计

```
┌─────────────────────────────────────────────┐
│           LLM Client (Claude等)             │
└─────────────────────────────────────────────┘
                    │ MCP Protocol
                    ▼
┌─────────────────────────────────────────────┐
│            GitAI MCP Server                 │
├─────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │ Bridge  │  │Registry │  │ Manager │    │
│  └─────────┘  └─────────┘  └─────────┘    │
├─────────────────────────────────────────────┤
│            MCP Services                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │ Review  │  │ Commit  │  │  Scan   │    │
│  └─────────┘  └─────────┘  └─────────┘    │
│  ┌─────────┐  ┌─────────────────────┐      │
│  │Analysis │  │    Deviation        │      │
│  └─────────┘  └─────────────────────┘      │
└─────────────────────────────────────────────┘
```

## 可用服务

### 1. execute_review - 代码评审
执行全面的代码评审，包括质量分析、安全扫描和任务一致性检查。

**参数：**
- `tree_sitter` (bool): 启用结构分析
- `security_scan` (bool): 启用安全扫描
- `issue_ids` (array): 关联的 Issue ID
- `format` (string): 输出格式 (text/json/markdown)
- `deviation_analysis` (bool): 偏离度分析

### 2. execute_commit - 智能提交
生成智能提交信息并执行提交。

**参数：**
- `message` (string): 提交信息（可选）
- `issue_ids` (array): 关联 Issue
- `add_all` (bool): 添加所有文件
- `review` (bool): 提交前评审
- `dry_run` (bool): 试运行模式

### 3. execute_scan - 安全扫描
执行代码安全扫描，检测潜在漏洞。

**参数：**
- `path` (string): 扫描路径
- `tool` (string): 扫描工具
- `lang` (string): 语言过滤
- `timeout` (int): 超时时间

### 4. execute_analysis - 结构分析
多语言代码结构分析，支持并发处理。

**参数：**
- `path` (string): 分析路径
- `language` (string): 编程语言
- `verbosity` (int): 详细程度 (0-2)

### 5. analyze_deviation - 偏离度分析
分析代码变更与 DevOps 任务的一致性。

**参数：**
- `diff` (string): 代码变更
- `issue_ids` (array): Issue ID 列表

## 配置说明

在 `~/.config/gitai/config.toml` 中配置：

```toml
[mcp]
enabled = true

[mcp.server]
name = "gitai-mcp"
version = "0.1.0"

[mcp.services]
# 启用的服务列表
enabled = ["review", "commit", "scan", "analysis", "deviation"]

# 各服务的配置
[mcp.services.review]
default_language = "auto"
include_security_scan = false
max_issues = 10

[mcp.services.commit]
default_review = false
auto_stage = false

[mcp.services.scan]
default_tool = "opengrep"
default_timeout = 300

[mcp.services.analysis]
enable_concurrency = true
max_workers = 8
cache_enabled = true
```

## 部署方式

### 1. 独立服务器模式

```bash
# 启动独立的 MCP 服务器
gitai-mcp serve

# 或通过 cargo 运行
cargo run --bin gitai-mcp -- serve
```

### 2. 集成模式

```bash
# 通过主程序启动
gitai mcp --transport stdio

# 指定配置文件
gitai mcp --config /path/to/config.toml
```

### 3. Claude Desktop 集成

在 Claude Desktop 配置文件中添加：

```json
{
  "mcpServers": {
    "gitai": {
      "command": "gitai-mcp",
      "args": ["serve"],
      "env": {
        "GITAI_AI_API_URL": "http://localhost:11434/v1/chat/completions",
        "GITAI_AI_MODEL": "qwen2.5:32b"
      }
    }
  }
}
```

## 使用示例

### 示例 1：通过 Claude 进行代码评审

```
User: 请评审当前代码变更

Claude: 我来为您进行代码评审。

[调用 execute_review]

评审结果：
- 代码质量评分：8.5/10
- 发现 2 个潜在问题：
  1. 未处理的异常情况（line 42）
  2. 可能的内存泄漏（line 78）
- 建议改进：
  - 添加错误处理
  - 使用 RAII 模式管理资源
```

### 示例 2：智能提交流程

```json
// MCP 请求
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "execute_commit",
    "arguments": {
      "issue_ids": ["#123", "#456"],
      "review": true,
      "add_all": true
    }
  },
  "id": 1
}

// MCP 响应
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "✅ 提交成功\n提交信息：feat(auth): implement OAuth2 integration\n关联 Issue: #123, #456"
      }
    ]
  },
  "id": 1
}
```

### 示例 3：并发文件分析

```python
# Python 客户端示例
import json
import subprocess

def call_mcp_tool(tool_name, arguments):
    request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        },
        "id": 1
    }
    
    process = subprocess.Popen(
        ["gitai-mcp", "serve"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    
    response = process.communicate(input=json.dumps(request))[0]
    return json.loads(response)

# 执行并发分析
result = call_mcp_tool("execute_analysis", {
    "path": "/path/to/project",
    "language": "rust",
    "verbosity": 2
})

print(f"分析完成：{result['result']['content'][0]['text']}")
```

## 服务依赖管理

### 依赖定义

```rust
// 服务可以声明依赖关系
impl GitAiMcpService for ReviewService {
    fn dependencies(&self) -> Vec<ServiceDependency> {
        vec![
            ServiceDependency {
                service_name: "analysis".to_string(),
                version_req: VersionReq::parse(">=0.1.0").unwrap(),
                optional: false,
            },
            ServiceDependency {
                service_name: "scan".to_string(),
                version_req: VersionReq::parse("*").unwrap(),
                optional: true,
            },
        ]
    }
}
```

### 启动顺序

服务注册表会自动进行拓扑排序，确保依赖服务先启动：

```
1. analysis (无依赖)
2. scan (无依赖)
3. review (依赖 analysis, scan)
4. commit (依赖 review)
```

## 性能优化

### 1. 并发处理
- 多文件并行分析
- 异步 I/O 操作
- 连接池管理

### 2. 缓存策略
- 服务实例缓存
- 结果缓存
- 配置缓存

### 3. 资源控制
- Semaphore 限制并发
- 超时控制
- 内存限制

## 错误处理

### 错误类型

```rust
pub enum McpError {
    InvalidParams(String),      // 参数错误
    ToolNotFound(String),       // 工具未找到
    ExecutionFailed(String),    // 执行失败
    Timeout(String),           // 超时
    ConfigError(String),       // 配置错误
    ServiceError(String),      // 服务错误
    DependencyError(String),   // 依赖错误
}
```

### 错误响应格式

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid parameters",
    "data": {
      "details": "Missing required parameter: path"
    }
  },
  "id": 1
}
```

## 监控和调试

### 日志级别

```bash
# 启用调试日志
RUST_LOG=debug gitai-mcp serve

# 只记录 MCP 相关日志
RUST_LOG=gitai::mcp=debug gitai-mcp serve

# 追踪特定服务
RUST_LOG=gitai::mcp::services::review=trace gitai-mcp serve
```

### 性能统计

```bash
# 查看性能统计
gitai mcp --stats

# 输出示例：
MCP 服务统计：
- 总调用次数：1,234
- 成功率：98.5%
- 平均响应时间：250ms
- 最慢响应：2.3s (execute_scan)
- 最快响应：10ms (analyze_deviation)
```

### 健康检查

```bash
# 健康检查端点
curl http://localhost:8080/health

# 响应
{
  "status": "healthy",
  "services": {
    "review": "ready",
    "commit": "ready",
    "scan": "ready",
    "analysis": "ready"
  },
  "uptime": 3600,
  "version": "0.1.0"
}
```

## 安全考虑

### 1. 认证授权
- API 密钥验证
- JWT token 支持
- 权限控制

### 2. 输入验证
- 参数类型检查
- 路径遍历防护
- 命令注入防护

### 3. 资源限制
- 请求大小限制
- 执行时间限制
- 并发连接限制

## 故障排除

### 问题：服务启动失败

**解决方案：**
1. 检查配置文件语法
2. 验证 AI 服务连接
3. 确认端口未被占用
4. 查看详细日志

### 问题：工具调用超时

**解决方案：**
1. 增加超时配置
2. 优化被调用的操作
3. 检查网络连接
4. 使用缓存

### 问题：依赖服务不可用

**解决方案：**
1. 检查服务注册状态
2. 验证依赖版本兼容性
3. 手动启动依赖服务
4. 查看依赖链

## 最佳实践

### 1. 服务设计
- 保持服务单一职责
- 明确定义接口契约
- 实现优雅降级
- 添加重试机制

### 2. 性能优化
- 使用批处理减少调用
- 实现结果缓存
- 异步处理长时间任务
- 监控性能指标

### 3. 可维护性
- 完善的日志记录
- 清晰的错误信息
- 版本兼容性管理
- 文档及时更新

## 未来展望

- [ ] WebSocket 传输支持
- [ ] GraphQL 接口
- [ ] 服务网格集成
- [ ] 分布式追踪
- [ ] 自动扩缩容
