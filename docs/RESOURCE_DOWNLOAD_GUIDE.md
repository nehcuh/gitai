# GitAI 资源下载配置指南

## 📦 资源类型

GitAI 需要下载以下资源：
- **OpenGrep 规则库**：安全扫描规则
- **Tree-sitter 语法文件**：代码结构分析
- **配置更新**：最新的配置模板

## 🚀 快速开始

### 1. 初始化配置（首次使用）

```bash
# 使用默认配置初始化
gitai init

# 使用自定义配置源（企业内网）
gitai init --config-url https://internal.company.com/gitai-config.toml

# 离线模式初始化（使用本地资源）
gitai init --offline --resources-dir /path/to/offline-resources
```

### 2. 自动安装和更新

#### 自动安装 OpenGrep（首次扫描时）
```bash
# 自动安装缺失的工具
gitai scan --auto-install

# 同时更新规则库
gitai scan --auto-install --update-rules
```

#### 更新扫描规则
```bash
# 更新安全扫描规则库
gitai update

# 仅检查更新状态
gitai update --check

# JSON 格式输出更新状态
gitai update --check --format json
```

## ⚙️ 配置文件设置

### 1. 配置下载源

编辑 `~/.config/gitai/config.toml`：

```toml
# ============================================================================
# 资源下载配置
# ============================================================================
[sources]
# OpenGrep 规则库（支持 Git 仓库或 HTTP 下载）
rules_url = "https://github.com/nehcuh/gitai-rules.git"

# Tree-sitter 语法文件
tree_sitter_url = "https://github.com/nehcuh/gitai-tree-sitter.git"

# 备用下载源（主源不可用时自动切换）
fallback_sources = [
    "https://gitee.com/nehcuh/gitai-mirror",  # Gitee 镜像（中国大陆）
    # "https://internal.company.com/gitai-resources",  # 企业内部镜像
]

# 自动更新设置
update_check_interval = 86400  # 每 24 小时检查一次更新
auto_update = false  # 是否自动更新（建议手动控制）
```

### 2. 网络配置（企业环境）

```toml
[network]
# HTTP/HTTPS 代理
proxy = "http://proxy.company.com:8080"
# 或 SOCKS5 代理
# proxy = "socks5://127.0.0.1:1080"

# 网络超时（秒）
timeout = 30

# 失败重试次数
retry_times = 3

# 离线模式（完全不进行网络请求）
offline_mode = false
```

### 3. 缓存配置

```toml
[cache]
# 启用缓存
enabled = true

# 缓存目录（资源下载后存储位置）
path = "~/.cache/gitai"

# 最大缓存大小
max_size = "1GB"

# 缓存有效期（604800 秒 = 7 天）
ttl = 604800
```

## 🌍 不同场景的配置示例

### 场景 1：中国大陆用户

```toml
[sources]
# 使用 Gitee 镜像作为主源
rules_url = "https://gitee.com/nehcuh/gitai-rules.git"
tree_sitter_url = "https://gitee.com/nehcuh/gitai-tree-sitter.git"

fallback_sources = [
    "https://github.com/nehcuh/gitai-rules.git",  # GitHub 作为备用
]
```

### 场景 2：企业内网环境

```toml
[sources]
# 使用内部 GitLab/Gitea 镜像
rules_url = "https://gitlab.company.com/mirrors/gitai-rules.git"
tree_sitter_url = "https://gitlab.company.com/mirrors/gitai-tree-sitter.git"

fallback_sources = []  # 不使用外部源

[network]
# 企业代理
proxy = "http://proxy.company.com:8080"

# 如果需要认证
# proxy = "http://username:password@proxy.company.com:8080"
```

### 场景 3：完全离线环境

```toml
[network]
# 启用离线模式
offline_mode = true

[cache]
# 使用共享缓存目录
path = "/shared/gitai-cache"
```

离线环境准备步骤：
```bash
# 1. 在有网络的机器上下载资源
gitai update

# 2. 打包缓存目录
tar -czf gitai-cache.tar.gz ~/.cache/gitai

# 3. 在离线机器上解压
tar -xzf gitai-cache.tar.gz -C /shared/

# 4. 配置离线模式并指向缓存
gitai init --offline --resources-dir /shared/gitai-cache
```

## 📝 命令行使用

### 扫描时的资源管理

```bash
# 扫描前更新规则
gitai scan --update-rules

# 自动安装工具并更新规则
gitai scan --auto-install --update-rules

# 指定规则语言（跳过自动检测）
gitai scan --lang java

# 使用特定超时时间
gitai scan --timeout 600
```

### 检查资源状态

```bash
# 检查更新状态
gitai update --check

# 查看缓存信息
ls -la ~/.cache/gitai/

# 查看规则库
ls -la ~/.cache/gitai/rules/
```

## 🔧 故障排除

### 问题 1：下载失败

**症状**：`Failed to download from primary source`

**解决方案**：
1. 检查网络连接
2. 配置代理（如果在企业网络）
3. 使用备用源

```toml
[sources]
fallback_sources = [
    "https://gitee.com/nehcuh/gitai-mirror",
    "https://mirror2.example.com/gitai",
]
```

### 问题 2：代理认证失败

**症状**：`407 Proxy Authentication Required`

**解决方案**：
```toml
[network]
# 包含用户名和密码
proxy = "http://username:password@proxy.company.com:8080"
```

### 问题 3：离线环境无法使用

**症状**：`No cached rules found in offline mode`

**解决方案**：
1. 先在有网络的环境下载资源
2. 复制整个缓存目录到离线机器
3. 配置正确的缓存路径

### 问题 4：GitHub 访问受限

**症状**：连接 GitHub 超时

**解决方案**：
```toml
[sources]
# 优先使用 Gitee 镜像
rules_url = "https://gitee.com/nehcuh/gitai-rules.git"

# GitHub 作为备用
fallback_sources = [
    "https://github.com/nehcuh/gitai-rules.git",
]

[network]
# 增加超时时间
timeout = 60
retry_times = 5
```

## 📊 资源目录结构

下载后的资源存储在：

```
~/.cache/gitai/
├── rules/                 # OpenGrep 规则库
│   ├── java/             # Java 规则
│   ├── python/           # Python 规则
│   ├── javascript/       # JavaScript 规则
│   └── ...
├── tree-sitter/          # Tree-sitter 语法文件
│   ├── rust/
│   ├── python/
│   └── ...
├── scan_history/         # 扫描历史记录
└── .metadata.json        # 资源元数据
```

## 🔄 更新策略

### 自动更新（推荐用于开发环境）

```toml
[sources]
auto_update = true
update_check_interval = 86400  # 每天检查
```

### 手动更新（推荐用于生产环境）

```toml
[sources]
auto_update = false
```

手动更新命令：
```bash
# 定期手动更新
gitai update

# 或在扫描时更新
gitai scan --update-rules
```

## 🎯 最佳实践

1. **企业环境**：使用内部镜像，配置代理
2. **离线环境**：预先下载资源，使用离线模式
3. **开发环境**：启用自动更新
4. **生产环境**：禁用自动更新，手动控制
5. **CI/CD**：使用缓存目录，避免重复下载

## 📚 相关文档

- [配置文件示例](../config.example.toml)
- [安装指南](../README.md#installation)
- [故障排除](../docs/TROUBLESHOOTING.md)
