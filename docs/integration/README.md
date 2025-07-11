# 集成指南

> 🔧 **GitAI 集成文档 - MCP 服务和第三方集成**

欢迎来到 GitAI 集成指南！这里包含了将 GitAI 服务集成到各种 AI 客户端和工具中所需的文档。

## 📚 文档列表

### 🌐 通用集成

| 文档 | 描述 | 阅读时间 |
|------|------|----------|
| [**MCP集成指南**](MCP_INTEGRATION_GUIDE.md) | 通用MCP服务配置和使用方法 | 30 分钟 |

### 🎯 特定客户端

| 文档 | 描述 | 阅读时间 |
|------|------|----------|
| [**ChatWise配置**](MCP_CHATWISE_SETUP.md) | ChatWise客户端专用配置指南 | 15 分钟 |

## 🔧 支持的集成方式

### 📡 MCP协议集成
- **标准兼容**: 遵循 Model Context Protocol 规范
- **多客户端支持**: ChatWise, Claude Desktop, 等
- **服务化架构**: 独立MCP服务器进程

### 🛠️ 可用服务

| 服务 | 功能 | 说明 |
|------|------|------|
| `gitai_review` | AI代码审查 | 深度代码分析和改进建议 |
| `gitai_status` | Git状态查询 | 实时获取仓库状态信息 |
| `gitai_diff` | 代码差异分析 | 智能比较代码变更 |
| `gitai_commit` | 智能提交 | AI生成提交消息和自动提交 |
| `gitai_scan` | 安全扫描 | 代码安全漏洞检测 |

## 🎯 集成场景

### 👤 个人开发者
- IDE集成 (VS Code, IntelliJ)
- AI助手集成 (ChatWise, Claude)
- 命令行工具链

### 🏢 团队协作
- CI/CD流程集成
- 代码审查工具集成
- 项目管理平台集成

### 🏭 企业级应用
- DevOps平台集成
- 安全扫描集成
- 质量管理集成

## 🚀 快速开始

### 1. 启动MCP服务
```bash
./target/release/mcp_server
```

### 2. 配置客户端
```json
{
  "name": "gitai-mcp-server",
  "command": "/path/to/gitai/target/release/mcp_server"
}
```

### 3. 使用服务
- 代码评审: `gitai_review`
- 智能提交: `gitai_commit`
- 安全扫描: `gitai_scan`

## 🔗 相关文档

- [用户指南](../user-guide/) - 了解基础功能
- [开发指南](../developer-guide/) - 理解技术实现
- [运维指南](../operations/) - 服务部署配置

## 📊 集成特性

### ⚡ 高性能
- 智能缓存机制
- 并发请求处理
- 流式数据传输

### 🔒 安全性
- API密钥管理
- 权限控制
- 数据脱敏

### 🌍 兼容性
- 跨平台支持
- 多语言分析
- 标准协议

---

**🔌 开始集成**: 建议从 [MCP集成指南](MCP_INTEGRATION_GUIDE.md) 开始！