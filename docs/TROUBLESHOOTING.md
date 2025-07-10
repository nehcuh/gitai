# GitAI 故障排除指南

> 🔧 **解决 GitAI 使用过程中的常见问题**

## 📋 目录

- [快速诊断](#快速诊断)
- [安装和配置问题](#安装和配置问题)
- [AI 服务问题](#ai-服务问题)
- [Git 操作问题](#git-操作问题)
- [DevOps 集成问题](#devops-集成问题)
- [性能问题](#性能问题)
- [日志分析](#日志分析)
- [高级诊断](#高级诊断)

## 🚨 快速诊断

### 一键诊断脚本

```bash
#!/bin/bash
# GitAI 健康检查脚本

echo "🔍 GitAI 健康检查"
echo "==================="

# 1. 检查 GitAI 版本
echo "📦 GitAI 版本:"
gitai --version || echo "❌ GitAI 未安装或不在 PATH 中"

# 2. 检查配置文件
echo -e "\n⚙️ 配置文件:"
CONFIG_FILE="$HOME/.config/gitai/config.toml"
if [ -f "$CONFIG_FILE" ]; then
    echo "✅ 配置文件存在: $CONFIG_FILE"
else
    echo "❌ 配置文件不存在: $CONFIG_FILE"
fi

# 3. 检查 Git 状态
echo -e "\n📂 Git 状态:"
if git rev-parse --is-inside-work-tree &>/dev/null; then
    echo "✅ 当前在 Git 仓库中"
    git status --porcelain | head -5
else
    echo "❌ 当前不在 Git 仓库中"
fi

# 4. 检查 AI 服务
echo -e "\n🤖 AI 服务检查:"
if command -v ollama &>/dev/null; then
    echo "✅ Ollama 已安装"
    ollama list | head -3
else
    echo "⚠️ Ollama 未安装"
fi

# 5. 检查网络连接
echo -e "\n🌐 网络检查:"
if curl -s --max-time 5 http://localhost:11434/api/tags &>/dev/null; then
    echo "✅ 本地 Ollama 服务运行正常"
else
    echo "❌ 本地 Ollama 服务无法访问"
fi

echo -e "\n🎯 诊断完成！"
```

### 快速修复检查清单

- [ ] GitAI 已正确安装且在 PATH 中
- [ ] 配置文件存在且格式正确
- [ ] 在 Git 仓库中执行命令
- [ ] AI 服务（如 Ollama）正在运行
- [ ] 网络连接正常
- [ ] 权限设置正确

## 🔧 安装和配置问题

### 问题1: GitAI 命令未找到

**症状**: 
```bash
$ gitai --version
bash: gitai: command not found
```

**原因分析**:
- GitAI 未安装
- 安装路径不在 PATH 中
- 编译失败

**解决方案**:

```bash
# 方法1: 检查编译状态
cd /path/to/gitai
cargo build --release

# 方法2: 手动添加到 PATH
export PATH="$PATH:/path/to/gitai/target/release"
echo 'export PATH="$PATH:/path/to/gitai/target/release"' >> ~/.bashrc

# 方法3: 安装到系统路径
sudo cp target/release/gitai /usr/local/bin/

# 方法4: 使用 cargo install
cargo install --path .
```

### 问题2: 配置文件错误

**症状**:
```bash
$ gitai commit
Error: Configuration error: Failed to parse TOML from file
```

**原因分析**:
- 配置文件语法错误
- 配置文件权限问题
- 配置文件位置错误

**解决方案**:

```bash
# 1. 验证配置文件语法
cat ~/.config/gitai/config.toml | toml-validator

# 2. 重新创建配置文件
mkdir -p ~/.config/gitai
cat > ~/.config/gitai/config.toml << EOF
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
temperature = 0.7

[git]
author_name = "Your Name"
author_email = "your.email@example.com"
EOF

# 3. 检查权限
chmod 600 ~/.config/gitai/config.toml
```

### 问题3: 依赖项缺失

**症状**:
```bash
$ cargo build --release
error: linker `cc` not found
```

**解决方案**:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel

# macOS
xcode-select --install
brew install openssl
```

## 🤖 AI 服务问题

### 问题1: AI 服务连接失败

**症状**:
```bash
$ gitai commit
Error: AI interaction error: AI API request failed
```

**原因分析**:
- AI 服务未启动
- 配置的 API 端点错误
- 网络连接问题
- API 密钥无效

**解决方案**:

```bash
# 1. 检查 Ollama 服务状态
ollama list
ollama serve &

# 2. 验证服务连接
curl -X POST http://localhost:11434/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen2.5:7b",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'

# 3. 检查配置
gitai --config ~/.config/gitai/config.toml commit --verbose

# 4. 使用调试模式
RUST_LOG=debug gitai commit
```

### 问题2: AI 响应质量差

**症状**:
- 生成的提交消息不相关
- 代码审查建议不准确
- 响应内容重复或错误

**解决方案**:

```bash
# 1. 调整 AI 参数
gitai commit --temperature 0.3

# 2. 更换模型
# 在配置文件中修改
[ai]
model_name = "qwen2.5:14b"  # 使用更大的模型

# 3. 使用自定义提示词
gitai commit --prompt "Generate a concise commit message in English"

# 4. 检查上下文长度
# 确保代码差异不会超过模型的上下文限制
git diff --stat
```

### 问题3: API 速率限制

**症状**:
```bash
Error: AI API responded with error 429: Too Many Requests
```

**解决方案**:

```bash
# 1. 检查 API 限制
curl -I https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# 2. 添加重试机制 (在配置中)
[ai]
max_retries = 3
retry_delay = 2

# 3. 切换到本地模型
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
```

## 📂 Git 操作问题

### 问题1: 不是 Git 仓库

**症状**:
```bash
$ gitai commit
Error: Git command error: Not a git repository
```

**解决方案**:

```bash
# 1. 初始化 Git 仓库
git init

# 2. 检查当前目录
pwd
ls -la

# 3. 进入正确的项目目录
cd /path/to/your/project

# 4. 验证 Git 状态
git status
```

### 问题2: 没有暂存的更改

**症状**:
```bash
$ gitai commit
Error: Git command error: No changes staged for commit
```

**解决方案**:

```bash
# 1. 添加文件到暂存区
git add .

# 2. 检查暂存状态
git status

# 3. 查看具体更改
git diff --staged

# 4. 如果需要提交所有更改
git add -A
```

### 问题3: Git 配置缺失

**症状**:
```bash
$ gitai commit
Error: Git configuration missing: user.name or user.email
```

**解决方案**:

```bash
# 1. 设置 Git 用户信息
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# 2. 或在 GitAI 配置中设置
[git]
author_name = "Your Name"
author_email = "your.email@example.com"

# 3. 验证配置
git config --list | grep user
```

## 🔗 DevOps 集成问题

### 问题1: DevOps API 认证失败

**症状**:
```bash
$ gitai review --space-id 12345 --stories 99
Error: DevOps API error: Authentication failed
```

**解决方案**:

```bash
# 1. 检查 API 令牌
echo $DEV_DEVOPS_API_TOKEN

# 2. 验证 API 访问
curl -H "Authorization: token $DEV_DEVOPS_API_TOKEN" \
  "$DEV_DEVOPS_API_BASE_URL/api/projects"

# 3. 更新配置
[devops]
api_token = "your-valid-token"
api_base_url = "https://your-company.devops.com"

# 4. 检查权限
# 确保令牌有访问相应工作项的权限
```

### 问题2: 工作项未找到

**症状**:
```bash
Error: DevOps API error: Work item 99 not found
```

**解决方案**:

```bash
# 1. 验证工作项 ID
curl -H "Authorization: token $DEV_DEVOPS_API_TOKEN" \
  "$DEV_DEVOPS_API_BASE_URL/api/projects/12345/issues/99"

# 2. 检查空间 ID
# 确保 space-id 正确

# 3. 使用正确的 ID 格式
gitai review --space-id 12345 --stories 99,100 --tasks 201
```

### 问题3: 网络连接问题

**症状**:
```bash
Error: DevOps API error: Network request failed
```

**解决方案**:

```bash
# 1. 检查网络连接
ping your-devops-server.com

# 2. 检查防火墙和代理
curl -v $DEV_DEVOPS_API_BASE_URL

# 3. 设置代理 (如果需要)
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080

# 4. 验证 SSL 证书
curl -k $DEV_DEVOPS_API_BASE_URL  # 跳过证书验证测试
```

## ⚡ 性能问题

### 问题1: 处理速度慢

**症状**:
- 提交消息生成时间过长
- 代码审查响应缓慢
- 扫描大型项目耗时

**解决方案**:

```bash
# 1. 启用性能监控
RUST_LOG=info gitai commit

# 2. 使用更快的模型
[ai]
model_name = "qwen2.5:3b"  # 使用较小的模型

# 3. 减少上下文长度
git diff --stat  # 检查差异大小
git add --patch  # 分批提交

# 4. 优化扫描范围
gitai scan src/ --exclude "tests/**"

# 5. 使用本地模型
# 避免网络延迟，使用 Ollama 本地部署
```

### 问题2: 内存使用过高

**症状**:
```bash
$ gitai review --depth deep
Error: Out of memory
```

**解决方案**:

```bash
# 1. 监控内存使用
top -p $(pgrep gitai)

# 2. 分批处理
gitai review --depth normal  # 使用较浅的分析

# 3. 限制并发
# 在配置中设置
[scanner]
max_concurrent_files = 10

# 4. 清理缓存
rm -rf ~/.cache/gitai/
```

### 问题3: 磁盘空间不足

**症状**:
```bash
Error: No space left on device
```

**解决方案**:

```bash
# 1. 检查磁盘空间
df -h

# 2. 清理日志文件
find ~/.cache/gitai -name "*.log" -delete

# 3. 清理临时文件
find /tmp -name "gitai-*" -delete

# 4. 配置日志轮转
[logging]
max_log_size = "10MB"
log_rotation = true
```

## 📊 日志分析

### 启用详细日志

```bash
# 1. 设置日志级别
export RUST_LOG=debug

# 2. 启用回溯
export RUST_BACKTRACE=1

# 3. 运行命令
gitai commit --verbose

# 4. 查看日志文件
tail -f ~/.cache/gitai/gitai.log
```

### 日志级别说明

| 级别 | 描述 | 使用场景 |
|------|------|----------|
| `error` | 错误信息 | 生产环境 |
| `warn` | 警告信息 | 生产环境 |
| `info` | 常规信息 | 默认级别 |
| `debug` | 调试信息 | 开发调试 |
| `trace` | 详细追踪 | 深度调试 |

### 常见日志模式

```bash
# AI 请求失败
ERROR gitai::handlers::ai: AI request failed: connection timeout

# 配置文件问题
WARN gitai::config: Configuration file not found, using defaults

# Git 操作成功
INFO gitai::handlers::git: Commit created successfully: abc123

# 性能监控
DEBUG gitai::handlers::commit: Operation completed in 1.23s
```

## 🔍 高级诊断

### 网络诊断

```bash
# 1. 检查 DNS 解析
nslookup api.openai.com

# 2. 测试连接
telnet api.openai.com 443

# 3. 检查 SSL 证书
openssl s_client -connect api.openai.com:443

# 4. 代理设置
echo $HTTP_PROXY $HTTPS_PROXY
```

### 系统诊断

```bash
# 1. 检查系统资源
htop
iotop

# 2. 检查文件描述符
lsof -p $(pgrep gitai)

# 3. 检查系统调用
strace -p $(pgrep gitai)

# 4. 检查依赖库
ldd $(which gitai)
```

### 调试模式

```bash
# 1. 编译调试版本
cargo build --features debug

# 2. 使用调试器
gdb target/debug/gitai
(gdb) run commit

# 3. 内存检查
valgrind --tool=memcheck target/debug/gitai commit

# 4. 性能分析
perf record target/release/gitai commit
perf report
```

## 📞 获取帮助

### 社区支持

1. **GitHub Issues**: [https://github.com/your-org/gitai/issues](https://github.com/your-org/gitai/issues)
2. **讨论区**: [https://github.com/your-org/gitai/discussions](https://github.com/your-org/gitai/discussions)
3. **文档网站**: [https://gitai.docs.com](https://gitai.docs.com)

### 提交问题时请包含

- [ ] GitAI 版本号 (`gitai --version`)
- [ ] 操作系统信息 (`uname -a`)
- [ ] 错误信息和日志
- [ ] 重现步骤
- [ ] 配置文件 (脱敏后)

### 问题报告模板

```markdown
## 问题描述
简要描述遇到的问题

## 环境信息
- GitAI 版本: 
- 操作系统: 
- Rust 版本: 
- Git 版本: 

## 重现步骤
1. 执行命令: `gitai commit`
2. 错误信息: 
3. 预期结果: 
4. 实际结果: 

## 日志信息
```
RUST_LOG=debug gitai commit
```

## 配置文件
```toml
[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
# ... 其他配置
```

## 其他信息
任何可能相关的额外信息
```

---

**🔧 记住**: 大多数问题都可以通过仔细检查配置文件和日志来解决。如果问题持续存在，请不要犹豫寻求社区帮助！