# GitAI 安装部署指南

> 📦 **完整的安装、配置和部署指南**

## 📋 目录

- [系统要求](#系统要求)
- [安装方法](#安装方法)
- [配置设置](#配置设置)
- [AI 服务部署](#ai-服务部署)
- [验证安装](#验证安装)
- [生产环境部署](#生产环境部署)
- [故障排除](#故障排除)

## 🔧 系统要求

### 最低要求

| 组件 | 要求 |
|------|------|
| **操作系统** | Linux, macOS, Windows |
| **CPU** | x86_64 或 ARM64 |
| **内存** | 2GB RAM (推荐 4GB+) |
| **磁盘空间** | 500MB 可用空间 |
| **网络** | 访问 AI 服务的网络连接 |

### 推荐要求

| 组件 | 推荐配置 |
|------|----------|
| **操作系统** | Ubuntu 20.04+, macOS 12+, Windows 10+ |
| **CPU** | 4核心以上 |
| **内存** | 8GB RAM (用于本地 AI 模型) |
| **磁盘空间** | 10GB+ (用于 AI 模型存储) |
| **网络** | 高速互联网连接 |

### 软件依赖

| 软件 | 版本要求 | 用途 |
|------|----------|------|
| **Rust** | 1.70.0+ | 编译构建 |
| **Git** | 2.20.0+ | 版本控制 |
| **OpenSSL** | 1.1.1+ | 网络安全 |
| **pkg-config** | 0.29+ | 依赖管理 |

## 🚀 安装方法

### 方法1: 从源码编译（推荐）

这是最可靠的安装方法，适合所有平台。

#### 步骤1: 安装 Rust

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 步骤2: 安装系统依赖

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

**CentOS/RHEL:**
```bash
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel git
```

**macOS:**
```bash
# 安装 Homebrew (如果没有)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装依赖
brew install git openssl pkg-config
```

**Windows:**
```powershell
# 使用 Chocolatey
choco install git openssl

# 或使用 winget
winget install Git.Git
winget install OpenSSL.OpenSSL
```

#### 步骤3: 克隆和编译

```bash
# 克隆项目
git clone https://github.com/your-org/gitai.git
cd gitai

# 编译 release 版本
cargo build --release

# 验证编译
./target/release/gitai --version
```

#### 步骤4: 安装到系统

```bash
# 方法1: 复制到系统路径
sudo cp target/release/gitai /usr/local/bin/

# 方法2: 使用 cargo install
cargo install --path .

# 方法3: 添加到用户目录
mkdir -p ~/.local/bin
cp target/release/gitai ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 方法2: 预编译二进制文件

```bash
# 下载最新版本
LATEST_VERSION=$(curl -s https://api.github.com/repos/your-org/gitai/releases/latest | jq -r .tag_name)
wget "https://github.com/your-org/gitai/releases/download/${LATEST_VERSION}/gitai-linux-x64.tar.gz"

# 解压和安装
tar -xzf gitai-linux-x64.tar.gz
sudo mv gitai /usr/local/bin/
sudo chmod +x /usr/local/bin/gitai
```

### 方法3: 包管理器安装

#### 使用 Homebrew (macOS/Linux)

```bash
# 添加 tap
brew tap your-org/gitai

# 安装
brew install gitai
```

#### 使用 Snap (Linux)

```bash
sudo snap install gitai
```

#### 使用 Cargo

```bash
cargo install gitai
```

### 方法4: 容器化部署

```bash
# 使用 Docker
docker pull your-org/gitai:latest
docker run -v $(pwd):/workspace your-org/gitai:latest --help

# 创建别名
echo 'alias gitai="docker run -v $(pwd):/workspace your-org/gitai:latest"' >> ~/.bashrc
```

## ⚙️ 配置设置

### 创建配置目录

```bash
# Linux/macOS
mkdir -p ~/.config/gitai

# Windows
mkdir %APPDATA%\gitai
```

### 基础配置文件

创建 `~/.config/gitai/config.toml`：

```toml
# GitAI 配置文件
# 文档: https://gitai.docs.com/configuration

[ai]
# AI 服务配置
api_url = "http://localhost:11434/v1/chat/completions"
model_name = "qwen2.5:7b"
api_key = ""  # 如果使用 OpenAI 等服务，填入 API 密钥
temperature = 0.7
max_tokens = 2048
timeout = 30

[git]
# Git 配置
author_name = "Your Name"
author_email = "your.email@example.com"
signing_key = ""
commit_template = ""

[devops]
# DevOps 集成配置
platform = "coding"
api_base_url = ""
api_token = ""
default_space_id = ""

[scanner]
# 安全扫描配置
rules_dir = "./rules"
exclude_patterns = ["*.test.js", "node_modules/**", "target/**"]
default_severity = "medium"
enable_remote_rules = false

[mcp]
# MCP 服务配置
server_port = 8080
server_host = "localhost"
enable_tree_sitter = true
enable_ai_analysis = true
enable_devops_integration = true

[logging]
# 日志配置
level = "info"
format = "text"
file = ""
```

### 环境变量配置

创建 `~/.gitai_env`：

```bash
# AI 服务配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:7b"
export GITAI_AI_API_KEY=""
export GITAI_AI_TEMPERATURE="0.7"

# DevOps 配置
export DEV_DEVOPS_API_BASE_URL="https://your-company.devops.com"
export DEV_DEVOPS_API_TOKEN="your-token"
export DEV_DEVOPS_DEFAULT_SPACE_ID="12345"

# 系统配置
export GITAI_CONFIG_PATH="$HOME/.config/gitai/config.toml"
export RUST_LOG="info"
```

加载环境变量：

```bash
# 添加到 shell 配置
echo 'source ~/.gitai_env' >> ~/.bashrc
source ~/.bashrc
```

## 🤖 AI 服务部署

### 选项1: Ollama 本地部署（推荐）

```bash
# 1. 安装 Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 2. 启动 Ollama 服务
ollama serve &

# 3. 下载推荐模型
ollama pull qwen2.5:7b

# 4. 测试模型
ollama run qwen2.5:7b "Hello, test message"

# 5. 列出可用模型
ollama list
```

### 选项2: OpenAI API

```bash
# 1. 获取 API 密钥
# 访问 https://platform.openai.com/api-keys

# 2. 配置 API 密钥
export OPENAI_API_KEY="sk-your-api-key"

# 3. 更新配置文件
cat >> ~/.config/gitai/config.toml << EOF
[ai]
api_url = "https://api.openai.com/v1/chat/completions"
model_name = "gpt-3.5-turbo"
api_key = "$OPENAI_API_KEY"
EOF
```

### 选项3: 自建 AI 服务

```bash
# 使用 vLLM 部署
pip install vllm

# 启动服务
python -m vllm.entrypoints.openai.api_server \
    --model qwen/Qwen2.5-7B-Instruct \
    --port 8000

# 配置 GitAI
[ai]
api_url = "http://localhost:8000/v1/chat/completions"
model_name = "qwen/Qwen2.5-7B-Instruct"
```

### 选项4: 云服务部署

```bash
# 使用 Azure OpenAI
[ai]
api_url = "https://your-resource.openai.azure.com/openai/deployments/gpt-35-turbo/chat/completions?api-version=2024-02-01"
model_name = "gpt-35-turbo"
api_key = "your-azure-key"

# 使用 Anthropic Claude
[ai]
api_url = "https://api.anthropic.com/v1/messages"
model_name = "claude-3-sonnet-20240229"
api_key = "your-anthropic-key"
```

## ✅ 验证安装

### 基础验证

```bash
# 1. 检查版本
gitai --version

# 2. 显示帮助
gitai --help

# 3. 检查配置
gitai config --show

# 4. 测试基础功能
cd /tmp
git init test-repo
cd test-repo
echo "console.log('Hello, GitAI!');" > app.js
git add app.js
gitai commit --dry-run
```

### 完整功能测试

```bash
#!/bin/bash
# GitAI 功能测试脚本

echo "🧪 GitAI 功能测试"
echo "=================="

# 创建测试仓库
TEST_DIR="/tmp/gitai-test-$(date +%s)"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

git init
git config user.name "Test User"
git config user.email "test@example.com"

# 测试1: 提交消息生成
echo "📝 测试1: 提交消息生成"
echo "console.log('Test');" > test.js
git add test.js
gitai commit --dry-run
echo "✅ 提交消息生成测试完成"

# 测试2: 代码审查
echo -e "\n🔍 测试2: 代码审查"
gitai review --no-ai-analysis
echo "✅ 代码审查测试完成"

# 测试3: 安全扫描
echo -e "\n🛡️ 测试3: 安全扫描"
gitai scan --path . --format json
echo "✅ 安全扫描测试完成"

# 测试4: MCP 服务
echo -e "\n🔗 测试4: MCP 服务"
gitai mcp-server --help
echo "✅ MCP 服务测试完成"

# 清理
cd /
rm -rf "$TEST_DIR"

echo -e "\n🎉 所有测试完成！"
```

## 🏭 生产环境部署

### 单机部署

```bash
# 1. 创建专用用户
sudo useradd -m -s /bin/bash gitai
sudo su - gitai

# 2. 安装 GitAI
curl -L https://github.com/your-org/gitai/releases/latest/download/gitai-linux-x64.tar.gz | tar xz
sudo mv gitai /usr/local/bin/

# 3. 配置系统服务
sudo tee /etc/systemd/system/gitai-mcp.service << EOF
[Unit]
Description=GitAI MCP Server
After=network.target

[Service]
Type=simple
User=gitai
WorkingDirectory=/home/gitai
ExecStart=/usr/local/bin/gitai mcp-server --port 8080 --host 0.0.0.0
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 4. 启动服务
sudo systemctl daemon-reload
sudo systemctl enable gitai-mcp
sudo systemctl start gitai-mcp
```

### 容器化部署

```dockerfile
# Dockerfile
FROM rust:1.70 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/gitai /usr/local/bin/

EXPOSE 8080
CMD ["gitai", "mcp-server", "--host", "0.0.0.0", "--port", "8080"]
```

```bash
# 构建镜像
docker build -t gitai:latest .

# 运行容器
docker run -d \
    --name gitai-server \
    -p 8080:8080 \
    -v /path/to/config:/root/.config/gitai \
    gitai:latest
```

### Kubernetes 部署

```yaml
# gitai-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gitai-deployment
spec:
  replicas: 3
  selector:
    matchLabels:
      app: gitai
  template:
    metadata:
      labels:
        app: gitai
    spec:
      containers:
      - name: gitai
        image: gitai:latest
        ports:
        - containerPort: 8080
        env:
        - name: GITAI_AI_API_URL
          value: "http://ollama-service:11434/v1/chat/completions"
        - name: GITAI_AI_MODEL
          value: "qwen2.5:7b"
        volumeMounts:
        - name: config
          mountPath: /root/.config/gitai
      volumes:
      - name: config
        configMap:
          name: gitai-config

---
apiVersion: v1
kind: Service
metadata:
  name: gitai-service
spec:
  selector:
    app: gitai
  ports:
    - protocol: TCP
      port: 80
      targetPort: 8080
  type: LoadBalancer
```

```bash
# 部署
kubectl apply -f gitai-deployment.yaml

# 检查状态
kubectl get pods -l app=gitai
kubectl get svc gitai-service
```

### 高可用部署

```bash
# 使用 Nginx 负载均衡
upstream gitai_backend {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 80;
    server_name gitai.your-domain.com;

    location / {
        proxy_pass http://gitai_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 🔒 安全配置

### SSL/TLS 配置

```bash
# 1. 生成证书
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# 2. 配置 HTTPS
[mcp]
server_port = 8443
tls_cert_file = "/path/to/cert.pem"
tls_key_file = "/path/to/key.pem"
```

### 访问控制

```bash
# 1. 配置防火墙
sudo ufw allow 8080/tcp
sudo ufw enable

# 2. 配置访问控制
[mcp]
allowed_origins = ["https://your-domain.com"]
allowed_ips = ["192.168.1.0/24"]
```

### 日志审计

```bash
# 配置日志审计
[logging]
level = "info"
format = "json"
file = "/var/log/gitai/audit.log"
audit_enabled = true
```

## 🚨 故障排除

### 常见问题

1. **编译失败**
   ```bash
   # 更新 Rust
   rustup update
   
   # 清理缓存
   cargo clean
   
   # 重新编译
   cargo build --release
   ```

2. **权限问题**
   ```bash
   # 检查权限
   ls -la /usr/local/bin/gitai
   
   # 修复权限
   sudo chmod +x /usr/local/bin/gitai
   ```

3. **网络问题**
   ```bash
   # 测试网络连接
   curl -v http://localhost:11434/api/tags
   
   # 检查防火墙
   sudo ufw status
   ```

### 性能优化

```bash
# 1. 编译优化
RUSTFLAGS="-C target-cpu=native" cargo build --release

# 2. 内存优化
export RUST_MIN_STACK=8388608

# 3. 日志优化
export RUST_LOG=warn
```

## 📊 监控和维护

### 健康检查

```bash
# 创建健康检查脚本
cat > /usr/local/bin/gitai-health-check.sh << 'EOF'
#!/bin/bash
if ! curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "GitAI service is down"
    systemctl restart gitai-mcp
fi
EOF

chmod +x /usr/local/bin/gitai-health-check.sh

# 添加到 crontab
echo "*/5 * * * * /usr/local/bin/gitai-health-check.sh" | crontab -
```

### 日志轮转

```bash
# 配置 logrotate
sudo tee /etc/logrotate.d/gitai << EOF
/var/log/gitai/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0644 gitai gitai
    postrotate
        systemctl reload gitai-mcp
    endscript
}
EOF
```

## 🔄 升级和更新

### 升级 GitAI

```bash
# 1. 备份配置
cp -r ~/.config/gitai ~/.config/gitai.backup

# 2. 下载新版本
wget https://github.com/your-org/gitai/releases/latest/download/gitai-linux-x64.tar.gz

# 3. 停止服务
sudo systemctl stop gitai-mcp

# 4. 更新二进制文件
sudo tar -xzf gitai-linux-x64.tar.gz -C /usr/local/bin/

# 5. 重启服务
sudo systemctl start gitai-mcp

# 6. 验证升级
gitai --version
```

### 回滚操作

```bash
# 1. 恢复旧版本
sudo cp /usr/local/bin/gitai.backup /usr/local/bin/gitai

# 2. 恢复配置
cp -r ~/.config/gitai.backup ~/.config/gitai

# 3. 重启服务
sudo systemctl restart gitai-mcp
```

---

**🎉 恭喜！您已成功完成 GitAI 的安装和部署！**

如有任何问题，请参考 [故障排除指南](TROUBLESHOOTING.md) 或联系社区获取帮助。