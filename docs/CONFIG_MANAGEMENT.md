# GitAI 配置管理指南

## 目录
1. [快速开始](#快速开始)
2. [配置文件结构](#配置文件结构)
3. [初始化配置](#初始化配置)
4. [资源管理](#资源管理)
5. [离线模式](#离线模式)
6. [企业部署](#企业部署)
7. [故障排除](#故障排除)

## 快速开始

### 首次使用

GitAI 在首次运行时会自动初始化配置：

```bash
# 自动初始化（使用默认配置）
gitai init

# 查看初始化状态
gitai config check
```

### 配置目录结构

GitAI 使用以下目录结构：

```
~/.config/gitai/          # 配置目录
├── config.toml          # 主配置文件
├── prompts/             # AI提示词模板
│   ├── commit-generator.md
│   └── review.md
└── .version             # 版本信息文件

~/.cache/gitai/           # 缓存目录
├── rules/               # OpenGrep规则缓存
│   └── .metadata.json
├── tree-sitter/         # Tree-sitter语法文件
│   └── .metadata.json
└── config/              # 远程配置缓存
    └── .metadata.json
```

## 配置文件结构

### 完整配置示例

```toml
# GitAI Configuration File
version = "1.0.0"

[sources]
# 资源下载源（可配置为内网镜像）
config_url = "https://raw.githubusercontent.com/nehcuh/gitai/main/assets/config.example.toml"
rules_url = "https://github.com/nehcuh/gitai-rules.git"
tree_sitter_url = "https://github.com/nehcuh/gitai-tree-sitter.git"

# 备用源（主源不可用时自动切换）
fallback_sources = [
    "https://gitee.com/nehcuh/gitai-mirror",
    "https://internal.company.com/gitai-resources"  # 企业内网镜像
]

# 更新策略
update_check_interval = 86400  # 24小时检查一次
auto_update = false            # 是否自动更新

[ai]
# AI服务配置
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"
temperature = 0.3
api_key = ""  # OpenAI兼容API需要
timeout = 30

[scan]
# 代码扫描配置
default_path = "."
timeout = 300
jobs = 4
rules_path = "~/.cache/gitai/rules"
auto_install_opengrep = true

[network]
# 网络配置
proxy = ""  # 例如："http://proxy.company.com:8080"
timeout = 30
retry_times = 3
offline_mode = false

[cache]
# 缓存配置
enabled = true
path = "~/.cache/gitai"
max_size = "1GB"
ttl = 604800  # 7天
```

## 初始化配置

### 基本初始化

```bash
# 交互式初始化
gitai init

# 从指定URL初始化（企业内网）
gitai init --config-url https://internal.company.com/gitai-config.toml

# 离线模式初始化
gitai init --offline

# 开发模式（使用项目内资源）
gitai init --dev
```

### 环境变量支持

可以通过环境变量覆盖配置：

```bash
# 设置配置源
export GITAI_CONFIG_URL="https://internal.company.com/config.toml"
export GITAI_RULES_URL="https://internal.company.com/rules.git"

# 启用离线模式
export GITAI_OFFLINE=true

# 设置代理
export GITAI_PROXY="http://proxy.company.com:8080"

# 覆盖AI配置
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:7b"
```

## 资源管理

### 查看资源状态

```bash
# 检查配置和资源状态
gitai config check

# 显示当前配置
gitai config show
gitai config show --format=json
gitai config show --format=toml
```

### 更新资源

```bash
# 更新所有资源（规则、语法文件等）
gitai config update

# 强制更新（忽略更新间隔）
gitai config update --force

# 单独更新规则
gitai update --check
gitai update
```

### 清理缓存

```bash
# 清理过期缓存
gitai config clean

# 重置配置到默认值
gitai config reset
gitai config reset --no-backup  # 不创建备份
```

## 离线模式

### 准备离线包

在有网络的环境准备离线资源包：

```bash
# 下载所有必要资源
gitai config update --force

# 打包资源
cd ~/.cache/gitai
tar czf gitai-offline-resources.tar.gz rules/ tree-sitter/
```

### 离线环境部署

```bash
# 解压离线包
tar xzf gitai-offline-resources.tar.gz -C ~/.cache/gitai/

# 启用离线模式
export GITAI_OFFLINE=true

# 或在配置文件中设置
# [network]
# offline_mode = true
```

### 离线模式使用

```bash
# 所有命令都会使用本地缓存
gitai --offline review
gitai --offline scan
gitai --offline commit
```

## 企业部署

### 1. 搭建内网镜像

创建企业内部的配置和资源镜像：

```bash
# 克隆官方资源
git clone https://github.com/nehcuh/gitai-rules.git
git clone https://github.com/nehcuh/gitai-tree-sitter.git

# 部署到内网Git服务器
git remote add internal https://git.company.com/tools/gitai-rules.git
git push internal main
```

### 2. 创建企业配置模板

`company-config.toml`:
```toml
version = "1.0.0"

[sources]
# 使用内网镜像
config_url = "https://git.company.com/tools/gitai-config/raw/main/config.toml"
rules_url = "https://git.company.com/tools/gitai-rules.git"
tree_sitter_url = "https://git.company.com/tools/gitai-tree-sitter.git"

[network]
# 企业代理设置
proxy = "http://proxy.company.com:8080"

[ai]
# 企业内部AI服务
api_url = "https://ai.company.com/v1/chat/completions"
api_key = "${AI_API_KEY}"  # 从环境变量读取
```

### 3. 批量部署脚本

```bash
#!/bin/bash
# deploy-gitai.sh

# 下载GitAI二进制
wget https://git.company.com/tools/gitai/releases/latest/gitai
chmod +x gitai

# 使用企业配置初始化
./gitai init --config-url https://git.company.com/tools/gitai-config/config.toml

# 验证安装
./gitai config check
```

### 4. Docker部署

```dockerfile
FROM rust:latest

# 安装GitAI
RUN cargo install gitai

# 使用企业配置
ENV GITAI_CONFIG_URL=https://git.company.com/tools/gitai-config/config.toml
ENV GITAI_PROXY=http://proxy.company.com:8080

# 初始化配置
RUN gitai init

ENTRYPOINT ["gitai"]
```

## 故障排除

### 常见问题

#### 1. 配置文件找不到

```bash
# 检查配置路径
echo $HOME/.config/gitai/config.toml

# 重新初始化
gitai init
```

#### 2. 网络连接失败

```bash
# 检查代理设置
echo $GITAI_PROXY

# 使用离线模式
gitai --offline review

# 或设置备用源
export GITAI_RULES_URL="https://gitee.com/nehcuh/gitai-rules.git"
```

#### 3. 资源下载失败

```bash
# 查看详细日志
RUST_LOG=debug gitai config update

# 手动下载资源
cd ~/.cache/gitai/rules
git clone https://github.com/nehcuh/gitai-rules.git .
```

#### 4. 配置迁移问题

```bash
# 备份当前配置
cp ~/.config/gitai/config.toml ~/.config/gitai/config.toml.backup

# 重置到默认配置
gitai config reset

# 恢复自定义设置
# 手动编辑 ~/.config/gitai/config.toml
```

### 调试模式

```bash
# 启用调试日志
export RUST_LOG=debug
gitai config check

# 查看完整配置
gitai config show --format=toml

# 验证资源完整性
ls -la ~/.cache/gitai/rules/
ls -la ~/.cache/gitai/tree-sitter/
```

### 版本兼容性

GitAI 会自动检测并迁移旧版本配置：

1. **自动迁移**：检测到旧版本配置时自动升级
2. **备份机制**：迁移前自动创建 `.backup` 文件
3. **版本跟踪**：通过 `.version` 文件跟踪配置版本

### 获取帮助

```bash
# 查看帮助信息
gitai --help
gitai init --help
gitai config --help

# 查看版本信息
gitai --version

# 提交问题
# https://github.com/nehcuh/gitai/issues
```

## 高级配置

### 自定义规则源

如果需要使用自定义的安全扫描规则：

```toml
[sources]
rules_url = "https://git.company.com/security/custom-rules.git"

[scan]
rules_path = "~/.cache/gitai/custom-rules"
```

### 多环境配置

使用不同的配置文件：

```bash
# 开发环境
export GITAI_CONFIG_FILE=~/.config/gitai/config.dev.toml

# 生产环境
export GITAI_CONFIG_FILE=~/.config/gitai/config.prod.toml
```

### 性能优化

```toml
[cache]
# 增加缓存大小
max_size = "5GB"

# 延长缓存时间
ttl = 2592000  # 30天

[scan]
# 增加并行任务数
jobs = 8

[network]
# 减少超时时间
timeout = 10
retry_times = 1
```

## 最佳实践

1. **定期更新资源**：建议每周更新一次规则库
   ```bash
   gitai config update
   ```

2. **使用版本控制**：将企业配置文件纳入版本控制
   ```bash
   git add company-config.toml
   git commit -m "chore: update GitAI configuration"
   ```

3. **监控资源使用**：定期清理缓存
   ```bash
   gitai config clean
   du -sh ~/.cache/gitai/
   ```

4. **安全配置**：敏感信息使用环境变量
   ```bash
   export GITAI_AI_API_KEY="your-secret-key"
   ```

5. **备份配置**：定期备份配置文件
   ```bash
   cp -r ~/.config/gitai/ ~/backup/gitai-config-$(date +%Y%m%d)/
   ```
