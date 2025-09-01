# GitAI 配置适配性优化设计文档

## 1. 概述

本文档描述 GitAI 项目的配置管理系统重构方案，旨在提升项目在不同环境（公网、内网、离线）下的适配性和可移植性。

## 2. 核心目标

1. **灵活的配置源** - 支持从多个源获取配置（远程URL、本地文件、默认模板）
2. **零配置启动** - 首次运行时自动初始化必要的配置和资源
3. **离线支持** - 支持在无网络环境下正常运行
4. **企业友好** - 便于在企业内网环境部署和使用
5. **向后兼容** - 支持旧版本配置的自动迁移

## 3. 配置架构设计

### 3.1 配置优先级（从高到低）

```
命令行参数 > 环境变量 > 用户配置文件 > 默认配置
```

### 3.2 目录结构

```
gitai/
├── assets/
│   ├── config.example.toml          # 默认配置模板
│   ├── prompts/                     # 默认提示词模板
│   └── rules/                       # 默认规则（可选的离线包）
│
~/.config/gitai/                     # 用户配置目录
├── config.toml                      # 用户配置文件
├── prompts/                         # 用户提示词
└── .version                         # 配置版本文件

~/.cache/gitai/                      # 缓存目录
├── rules/                           # OpenGrep 规则缓存
│   └── .metadata.json              # 规则元数据（版本、来源、更新时间）
├── tree-sitter/                    # Tree-sitter 语法文件缓存
│   └── .metadata.json              # 语法文件元数据
└── config/                         # 远程配置缓存
    └── .metadata.json              # 配置元数据
```

## 4. 配置文件结构

### 4.1 增强的配置文件 (config.example.toml)

```toml
# GitAI Configuration File
version = "1.0.0"  # 配置文件版本

[sources]
# 资源下载源配置
config_url = "https://raw.githubusercontent.com/nehcuh/gitai/main/assets/config.example.toml"
rules_url = "https://github.com/nehcuh/gitai-rules.git"
tree_sitter_url = "https://github.com/nehcuh/gitai-tree-sitter.git"

# 备用源（当主源不可用时使用）
fallback_sources = [
    "https://gitee.com/nehcuh/gitai-mirror",
    "https://internal.company.com/gitai-resources"
]

# 更新策略
update_check_interval = 86400  # 24小时检查一次更新
auto_update = false            # 是否自动更新资源

[ai]
api_url = "http://localhost:11434/v1/chat/completions"
model = "qwen2.5:32b"
temperature = 0.3
api_key = ""  # 可选
timeout = 30

[scan]
default_path = "."
timeout = 300
jobs = 4
# OpenGrep 规则配置
rules_path = "~/.cache/gitai/rules"  # 规则存储路径
auto_install_opengrep = true         # 自动安装 OpenGrep

[tree_sitter]
enabled = false  # 当前为实验性功能
grammar_path = "~/.cache/gitai/tree-sitter"
supported_languages = ["rust", "python", "javascript", "go", "java"]

[cache]
enabled = true
path = "~/.cache/gitai"
max_size = "1GB"
ttl = 604800  # 7天

[network]
proxy = ""  # HTTP/HTTPS 代理
timeout = 30
retry_times = 3
offline_mode = false  # 离线模式

[mcp]
enabled = true

[mcp.server]
name = "gitai-mcp"
version = "0.1.0"

[logging]
level = "info"  # debug, info, warn, error
file = "~/.cache/gitai/gitai.log"
max_size = "10MB"
max_backups = 3
```

## 5. 核心模块设计

### 5.1 ConfigInitializer 模块

```rust
pub struct ConfigInitializer {
    config_dir: PathBuf,
    cache_dir: PathBuf,
    config_url: Option<String>,
    offline_mode: bool,
}

impl ConfigInitializer {
    /// 初始化配置系统
    pub async fn initialize() -> Result<Config> {
        // 1. 检测配置目录
        // 2. 如果不存在配置，尝试下载或复制默认配置
        // 3. 验证配置完整性
        // 4. 返回加载的配置
    }
    
    /// 从远程下载配置
    pub async fn download_config(&self, url: &str) -> Result<Config> {
        // 下载并验证配置
    }
    
    /// 检查并迁移旧版本配置
    pub fn migrate_config(&self) -> Result<()> {
        // 版本检测和迁移逻辑
    }
}
```

### 5.2 ResourceManager 模块

```rust
pub struct ResourceManager {
    config: Config,
    cache_dir: PathBuf,
    offline_mode: bool,
}

impl ResourceManager {
    /// 获取 OpenGrep 规则
    pub async fn get_rules(&self) -> Result<PathBuf> {
        // 1. 检查本地缓存
        // 2. 如果需要更新，从配置的源下载
        // 3. 返回规则路径
    }
    
    /// 获取 Tree-sitter 语法文件
    pub async fn get_grammars(&self, lang: &str) -> Result<PathBuf> {
        // 类似规则处理
    }
    
    /// 更新所有资源
    pub async fn update_all(&self) -> Result<()> {
        // 批量更新资源
    }
}
```

## 6. 命令行接口增强

### 6.1 新增命令

```bash
# 初始化配置（交互式）
gitai init

# 从指定 URL 下载配置
gitai init --config-url https://company.com/gitai-config.toml

# 离线模式运行
gitai --offline review

# 检查配置状态
gitai config check

# 更新所有资源
gitai config update

# 显示当前配置
gitai config show

# 重置配置到默认值
gitai config reset
```

### 6.2 环境变量支持

```bash
# 覆盖配置源
export GITAI_CONFIG_URL="https://internal.company.com/config.toml"
export GITAI_RULES_URL="https://internal.company.com/rules.git"
export GITAI_TREE_SITTER_URL="https://internal.company.com/grammars.git"

# 设置离线模式
export GITAI_OFFLINE=true

# 设置代理
export GITAI_PROXY="http://proxy.company.com:8080"
```

## 7. 使用场景

### 7.1 首次使用（公网环境）

```bash
# 下载二进制文件
wget https://github.com/nehcuh/gitai/releases/download/v1.0.0/gitai

# 首次运行，自动初始化
./gitai init
# 自动：
# 1. 创建配置目录
# 2. 下载默认配置
# 3. 下载必要的规则和资源

# 正常使用
./gitai review
```

### 7.2 企业内网部署

```bash
# 从内网镜像初始化
./gitai init --config-url https://internal.company.com/gitai/config.toml

# 配置会指向内网资源镜像
# 后续所有资源都从内网下载
```

### 7.3 完全离线环境

```bash
# 准备离线包（包含所有必要资源）
tar xzf gitai-offline-v1.0.0.tar.gz
cd gitai-offline/

# 使用离线模式初始化
./gitai init --offline --resources-dir ./resources

# 正常使用（不会尝试网络请求）
./gitai --offline review
```

### 7.4 开发环境

```bash
# 使用项目内的配置
cargo run -- init --dev

# 直接使用项目 assets 目录的资源
cargo run -- review
```

## 8. 实现步骤

### Phase 1: 基础重构（第1-2个任务）
1. 移动配置文件到 assets/config.example.toml
2. 增强配置文件结构，添加资源源配置
3. 实现基础的 ConfigInitializer

### Phase 2: 资源管理（第3-4个任务）
1. 实现 ResourceManager
2. 添加命令行参数支持
3. 实现环境变量覆盖机制

### Phase 3: 高级特性（第5-6个任务）
1. 实现配置验证和迁移
2. 添加离线模式支持
3. 完善错误处理和日志
4. 更新文档和测试

## 9. 兼容性保证

1. **向后兼容**：检测旧配置文件，自动迁移到新格式
2. **默认行为不变**：不指定参数时，使用默认的 GitHub 源
3. **渐进式升级**：用户可以逐步采用新特性

## 10. 测试计划

1. **单元测试**
   - ConfigInitializer 各种场景测试
   - ResourceManager 下载和缓存测试
   - 配置验证和迁移测试

2. **集成测试**
   - 首次运行初始化流程
   - 离线模式完整流程
   - 配置更新和资源同步

3. **场景测试**
   - 公网环境自动初始化
   - 内网镜像源配置
   - 完全离线运行
   - 配置损坏恢复

## 11. 安全考虑

1. **配置验证**：下载的配置需要格式和内容验证
2. **HTTPS 强制**：默认只允许 HTTPS 源（可配置）
3. **校验和验证**：资源下载后进行完整性校验
4. **权限控制**：配置文件权限设置为 600

## 12. 性能优化

1. **并行下载**：多个资源并行下载
2. **增量更新**：只下载变化的部分
3. **智能缓存**：基于时间戳和版本的缓存策略
4. **延迟加载**：按需下载资源，不是一次性全部下载
