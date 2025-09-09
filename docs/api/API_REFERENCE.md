# GitAI API 参考文档

## 概述

本文档描述 GitAI 的公共 API 接口，包括 Rust 库接口、MCP 服务接口和 CLI 命令接口。

## 目录

- [Rust 库 API](#rust-库-api)
- [MCP 服务 API](#mcp-服务-api)
- [CLI 命令 API](#cli-命令-api)
- [配置 API](#配置-api)
- [错误处理](#错误处理)

## Rust 库 API

### 核心模块

#### `gitai::analysis`

代码分析模块，提供多维度分析能力。

```rust
pub struct Analyzer {
    pub config: Config,
    pub ai_client: Option<Arc<dyn AiClient>>,
    pub devops_client: Option<Arc<dyn DevOpsClient>>,
}

impl Analyzer {
    /// 创建新的分析器实例
    pub fn new(config: Config) -> Result<Self>
    
    /// 执行代码评审分析
    pub async fn analyze_review(
        &self,
        context: ReviewContext
    ) -> Result<AnalysisResult>
    
    /// 执行安全分析
    pub async fn analyze_security(
        &self,
        path: &Path,
        lang: Option<&str>
    ) -> Result<SecurityResult>
    
    /// 执行偏离度分析
    pub async fn analyze_deviation(
        &self,
        diff: &str,
        issue_ids: &[String]
    ) -> Result<DeviationResult>
}
```

#### `gitai::tree_sitter`

Tree-sitter 结构分析模块。

```rust
pub struct TreeSitterManager {
    parsers: HashMap<Language, Parser>,
    cache: Arc<Mutex<LruCache<String, AnalysisResult>>>,
}

impl TreeSitterManager {
    /// 创建管理器实例
    pub fn new() -> Self
    
    /// 分析单个文件
    pub fn analyze_file(
        &mut self,
        path: &Path,
        content: &str,
        language: Language
    ) -> Result<FileAnalysis>
    
    /// 并发分析多个文件
    pub async fn analyze_files_concurrent(
        &self,
        files: Vec<PathBuf>,
        max_concurrent: usize
    ) -> Result<Vec<FileAnalysis>>
}
```

#### `gitai::mcp`

MCP 服务管理模块。

```rust
pub struct GitAiMcpManager {
    registry: ServiceRegistry,
    config: McpConfig,
}

impl GitAiMcpManager {
    /// 异步创建管理器
    pub async fn new(config: Config) -> Result<Self>
    
    /// 注册服务
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService>
    ) -> Result<()>
    
    /// 调用工具
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Value
    ) -> McpResult<Value>
    
    /// 获取所有工具
    pub fn list_tools(&self) -> Vec<Tool>
}
```

### 服务注册表 API

#### `gitai::mcp::registry`

服务注册和依赖管理。

```rust
pub struct ServiceRegistry {
    services: Arc<DashMap<String, ServiceEntry>>,
    event_bus: Arc<EventBus>,
}

impl ServiceRegistry {
    /// 创建注册表
    pub fn new() -> Self
    
    /// 注册服务
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService>,
        metadata: Value
    ) -> Result<()>
    
    /// 注销服务
    pub async fn deregister_service(
        &self,
        name: &str
    ) -> Result<()>
    
    /// 检查依赖
    pub async fn check_dependencies(
        &self,
        service: &dyn GitAiMcpService
    ) -> Result<()>
    
    /// 获取启动顺序
    pub async fn get_startup_order(&self) -> Result<Vec<String>>
}

/// 服务依赖定义
pub struct ServiceDependency {
    pub service_name: String,
    pub version_req: VersionReq,
    pub optional: bool,
}
```

### DI 容器 API

#### `gitai::di`

依赖注入容器。

```rust
pub struct ServiceContainer {
    // 内部实现
}

impl ServiceContainer {
    /// 创建容器
    pub fn new() -> Self
    
    /// 注册单例服务
    pub async fn register_singleton_simple<T>(
        &self,
        factory: impl Fn() -> Result<T> + 'static
    ) -> Result<()>
    where
        T: Any + Send + Sync + 'static
    
    /// 注册瞬态服务
    pub async fn register_transient_simple<T>(
        &self,
        factory: impl Fn() -> Result<T> + 'static
    ) -> Result<()>
    where
        T: Any + Send + Sync + 'static
    
    /// 解析服务
    pub async fn resolve<T>(&self) -> Result<Arc<T>>
    where
        T: Any + Send + Sync + 'static
    
    /// 开始作用域
    pub async fn begin_scope(&self)
    
    /// 结束作用域
    pub async fn end_scope(&self)
}
```

## MCP 服务 API

### 工具接口

所有 MCP 服务实现以下 trait：

```rust
#[async_trait]
pub trait GitAiMcpService: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;
    
    /// 服务描述
    fn description(&self) -> &str;
    
    /// 服务版本
    fn version(&self) -> Version;
    
    /// 服务依赖
    fn dependencies(&self) -> Vec<ServiceDependency>;
    
    /// 提供的工具列表
    fn tools(&self) -> Vec<Tool>;
    
    /// 处理工具调用
    async fn handle_tool_call(
        &self,
        name: &str,
        arguments: Value
    ) -> McpResult<Value>;
    
    /// 健康检查
    async fn health_check(&self) -> McpResult<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }
}
```

### 可用工具

#### `execute_review`

执行代码评审。

**请求：**
```json
{
  "tree_sitter": true,
  "security_scan": false,
  "issue_ids": ["#123"],
  "format": "text",
  "deviation_analysis": false
}
```

**响应：**
```json
{
  "score": 8.5,
  "summary": "代码质量良好",
  "issues": [],
  "suggestions": []
}
```

#### `execute_commit`

生成智能提交。

**请求：**
```json
{
  "message": null,
  "issue_ids": ["#123"],
  "add_all": false,
  "review": false,
  "dry_run": false
}
```

**响应：**
```json
{
  "message": "feat: add new feature",
  "status": "success"
}
```

#### `execute_scan`

执行安全扫描。

**请求：**
```json
{
  "path": ".",
  "tool": "opengrep",
  "lang": "rust",
  "timeout": 300
}
```

**响应：**
```json
{
  "findings": [],
  "summary": {
    "total": 0,
    "high": 0,
    "medium": 0,
    "low": 0
  }
}
```

#### `execute_analysis`

执行结构分析。

**请求：**
```json
{
  "path": "src/",
  "language": "rust",
  "verbosity": 1
}
```

**响应：**
```json
{
  "files_analyzed": 10,
  "total_time": 1.23,
  "metrics": {}
}
```

## CLI 命令 API

### 基本命令

#### `gitai init`

初始化配置。

```bash
gitai init [OPTIONS]

选项：
  --config-url <URL>     配置文件 URL
  --offline              离线模式
  --force                强制重新初始化
```

#### `gitai review`

执行代码评审。

```bash
gitai review [OPTIONS]

选项：
  --tree-sitter          启用结构分析
  --security-scan        启用安全扫描
  --issue-id <IDS>       关联 Issue
  --format <FORMAT>      输出格式 [text|json|markdown]
  --output <FILE>        输出文件
  --block-on-critical    严重问题时阻断
```

#### `gitai commit`

智能提交。

```bash
gitai commit [OPTIONS]

选项：
  -m, --message <MSG>    提交信息
  --issue-id <IDS>       关联 Issue
  --all                  添加所有文件
  --review               提交前评审
  --dry-run              试运行
```

#### `gitai scan`

安全扫描。

```bash
gitai scan [OPTIONS] [PATH]

选项：
  --lang <LANG>          语言过滤
  --tool <TOOL>          扫描工具
  --timeout <SECS>       超时时间
  --format <FORMAT>      输出格式
  --auto-install         自动安装工具
  --update-rules         更新规则
  --no-history           不保存历史
```

#### `gitai metrics`

质量度量。

```bash
gitai metrics <SUBCOMMAND>

子命令：
  record                 记录快照
  analyze                分析趋势
  report                 生成报告
  export                 导出数据
  compare                比较快照
  list                   列出快照
```

#### `gitai mcp`

MCP 服务器。

```bash
gitai mcp [OPTIONS]

选项：
  --transport <TYPE>     传输类型 [stdio|tcp|sse]
  --port <PORT>          端口号（TCP 模式）
  --config <FILE>        配置文件
```

## 配置 API

### 配置结构

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub ai: AiConfig,
    pub scan: ScanConfig,
    pub devops: Option<DevOpsConfig>,
    pub mcp: Option<McpConfig>,
    pub metrics: Option<MetricsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConfig {
    pub enabled: bool,
    pub server: McpServerConfig,
    pub services: McpServicesConfig,
}
```

### 配置加载

```rust
impl Config {
    /// 从文件加载配置
    pub fn load() -> Result<Self>
    
    /// 从指定路径加载
    pub fn load_from(path: &Path) -> Result<Self>
    
    /// 验证配置
    pub fn validate(&self) -> Result<()>
    
    /// 与环境变量合并
    pub fn merge_with_env(&mut self) -> Result<()>
}
```

## 错误处理

### 错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitAiError {
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("AI 服务错误: {0}")]
    AiService(String),
    
    #[error("扫描错误: {0}")]
    Scan(String),
    
    #[error("Git 操作错误: {0}")]
    Git(String),
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("其他错误: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("无效参数: {0}")]
    InvalidParams(String),
    
    #[error("工具未找到: {0}")]
    ToolNotFound(String),
    
    #[error("执行失败: {0}")]
    ExecutionFailed(String),
    
    #[error("超时: {0}")]
    Timeout(String),
    
    #[error("服务错误: {0}")]
    ServiceError(String),
}
```

### 错误处理示例

```rust
use gitai::{Config, Analyzer, GitAiError};

async fn example() -> Result<(), GitAiError> {
    // 加载配置
    let config = Config::load()?;
    
    // 创建分析器
    let analyzer = Analyzer::new(config)?;
    
    // 执行分析
    let result = analyzer
        .analyze_review(context)
        .await
        .map_err(|e| GitAiError::Other(e.to_string()))?;
    
    Ok(())
}
```

## 使用示例

### Rust 库使用

```rust
use gitai::{Config, mcp::GitAiMcpManager};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = Config::load()?;
    
    // 创建 MCP 管理器
    let manager = GitAiMcpManager::new(config).await?;
    
    // 调用工具
    let result = manager.call_tool(
        "execute_review",
        json!({
            "tree_sitter": true,
            "security_scan": false
        })
    ).await?;
    
    println!("评审结果: {}", result);
    
    Ok(())
}
```

### MCP 客户端集成

```python
import json
import subprocess

def call_gitai_tool(tool_name, arguments):
    """调用 GitAI MCP 工具"""
    request = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        },
        "id": 1
    }
    
    # 启动 MCP 服务器
    process = subprocess.Popen(
        ["gitai-mcp", "serve"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    
    # 发送请求并获取响应
    response_text = process.communicate(
        input=json.dumps(request)
    )[0]
    
    return json.loads(response_text)

# 使用示例
result = call_gitai_tool("execute_review", {
    "tree_sitter": True,
    "security_scan": True
})
print(result)
```

## 版本兼容性

- **稳定 API**：标记为 `1.0` 的接口保证向后兼容
- **实验性 API**：标记为 `experimental` 的接口可能在未来版本中更改
- **废弃 API**：标记为 `deprecated` 的接口将在下个主要版本中移除

## 更多信息

- [架构文档](../architecture/ARCHITECTURE.md)
- [MCP 服务文档](../features/MCP_SERVICE.md)
- [贡献指南](../development/CONTRIBUTING.md)
