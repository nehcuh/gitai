# GitAI 快速入门指南

> 🚀 **5分钟快速上手 GitAI** - 从安装到使用的最短路径

## 📋 快速概览

**GitAI** 是一个 AI 驱动的 Git 工作流助手，提供智能提交消息生成、代码审查、安全扫描等功能。

### 🎯 主要功能
- 🤖 **AI 提交消息生成** - 自动生成高质量提交消息
- 🔍 **智能代码审查** - AI 驱动的代码审查和建议
- 🛡️ **安全扫描** - 代码安全漏洞检测
- 🔧 **MCP 服务** - 支持 Model Context Protocol 集成

## ⚡ 快速安装

### 方法一：从源码编译（推荐）

```bash
# 1. 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 2. 编译安装
cargo build --release

# 3. 添加到 PATH（可选）
cp target/release/gitai ~/.local/bin/
```

### 方法二：预编译包

```bash
# 下载最新 release
wget https://github.com/your-org/gitai/releases/latest/download/gitai-linux-x64.tar.gz
tar -xzf gitai-linux-x64.tar.gz
sudo mv gitai /usr/local/bin/
```

## 🔧 最小化配置

创建配置文件 `~/.config/gitai/config.toml`：

```toml
[ai]
# 使用 Ollama 本地模型（推荐新手）
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
temperature = 0.7

[git]
# 基础 Git 配置
author_name = "Your Name"
author_email = "your.email@example.com"
```

## 🎯 核心功能快速体验

### 1. 智能提交消息生成 ⭐

```bash
# 添加文件到暂存区
git add .

# 使用 AI 生成提交消息
gitai commit

# 或使用简短命令
gitai c
```

**示例输出：**
```
✨ AI 生成的提交消息：
feat: implement user authentication system

- Add JWT token validation middleware
- Implement user registration and login endpoints
- Add password hashing with bcrypt
- Update database schema for user table

✓ 是否使用此消息？ (y/N)
```

### 2. 代码审查 ⭐

```bash
# 审查当前更改
gitai review

# 或使用简短命令
gitai rv
```

**示例输出：**
```
🔍 代码审查结果：

📊 总体评分: 85/100

✅ 优点：
- 代码结构清晰，符合 Rust 最佳实践
- 错误处理完善，使用了 Result 类型
- 文档注释详细

⚠️ 建议改进：
- 考虑添加单元测试覆盖
- 部分函数可以进一步优化性能
- 建议添加日志记录
```

### 3. 安全扫描 ⭐

```bash
# 扫描当前项目
gitai scan

# 扫描特定路径
gitai scan src/
```

**示例输出：**
```
🛡️ 安全扫描结果：

🔍 扫描了 45 个文件，发现 2 个问题：

⚠️ 中等风险：
- SQL 注入风险 @ src/database.rs:123
- 硬编码密钥 @ src/config.rs:45

💡 建议：
- 使用参数化查询防止 SQL 注入
- 将敏感信息移到环境变量中
```

## 🤖 AI 模型配置

### 使用 Ollama（推荐新手）

```bash
# 1. 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 2. 下载中文模型
ollama pull qwen2.5:7b

# 3. 启动 Ollama 服务
ollama serve
```

### 使用 OpenAI API

```toml
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model_name = "gpt-4"
api_key = "sk-your-api-key-here"
```

### 使用其他 API 服务

```toml
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model_name = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
```

## 📚 基础使用场景

### 场景1：日常开发提交

```bash
# 修改代码后
git add .
gitai commit

# 一键提交（跳过确认）
gitai commit --auto-confirm
```

### 场景2：代码审查前检查

```bash
# 在提交前先审查
gitai review

# 根据建议修改代码
# 然后提交
gitai commit
```

### 场景3：安全检查

```bash
# 定期安全扫描
gitai scan --format json > security-report.json

# 更新安全规则
gitai scan --update-rules
```

## 🎨 高级功能预览

### TreeSitter 分析

```bash
# 使用 TreeSitter 进行深度代码分析
gitai commit --tree-sitter

# 查看 TreeSitter 统计信息
gitai analyze --tree-sitter-stats
```

### 与工作项集成

```bash
# 结合 DevOps 工作项进行审查
gitai review --space-id 12345 --stories 99,100

# 生成与工作项关联的提交消息
gitai commit --issue-id TASK-123
```

### MCP 服务

```bash
# 启动 MCP 服务
gitai mcp-server

# 使用 MCP 客户端
gitai mcp-client --tool analyze --input "code content"
```

## 🚨 常见问题速览

### Q1: 提示 "AI 服务连接失败"

**解决方案：**
```bash
# 检查 Ollama 服务状态
ollama list

# 重启 Ollama 服务
ollama serve
```

### Q2: 提交消息质量不理想

**解决方案：**
```bash
# 调整 AI 参数
gitai commit --temperature 0.5

# 使用自定义提示词
gitai commit --prompt "生成简洁的提交消息"
```

### Q3: 配置文件位置

**默认位置：**
- Linux/macOS: `~/.config/gitai/config.toml`
- Windows: `%APPDATA%\gitai\config.toml`

## 🔗 下一步

现在您已经掌握了 GitAI 的基础使用！

### 📖 深入学习
- [完整用户指南](../README.md) - 详细功能说明
- [API 参考文档](API_DOCUMENTATION.md) - 完整命令参考
- [配置指南](CONFIGURATION_REFERENCE.md) - 高级配置选项

### 🛠️ 高级功能
- [故障排除](TROUBLESHOOTING.md) - 解决常见问题
- [部署指南](DEPLOYMENT_GUIDE.md) - 生产环境部署
- [开发指南](CONTRIBUTING.md) - 参与项目开发

### 💬 获取帮助
- [GitHub Issues](https://github.com/your-org/gitai/issues) - 报告问题
- [讨论区](https://github.com/your-org/gitai/discussions) - 交流讨论
- [文档站点](https://gitai.docs.com) - 在线文档

---

**🎉 恭喜！您已经成功上手 GitAI！**

开始享受 AI 驱动的 Git 工作流带来的便利吧！