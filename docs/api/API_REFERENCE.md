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

Tree-sitter 结构分析模块（多语言，支持并发）。

```rust
pub struct TreeSitterManager {
    // 内部：parsers、queries_manager、可选缓存
}

impl TreeSitterManager {
    /// 创建新的管理器（异步，确保查询规则可用）
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>>;

    /// 分析内存中的代码（单文件、单语言）
    pub fn analyze_structure(
        &mut self,
        code: &str,
        language: SupportedLanguage,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>>;

    /// 并发分析多个文件
    pub async fn analyze_files_concurrent(
        &self,
        file_paths: Vec<PathBuf>,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<FileAnalysisResult>, Box<dyn std::error::Error + Send + Sync>>;

    /// 并发分析目录中的所有代码文件
    pub async fn analyze_directory_concurrent(
        &self,
        dir_path: &Path,
        language_filter: Option<SupportedLanguage>,
        max_concurrent: Option<usize>,
    ) -> Result<DirectoryAnalysisResult, Box<dyn std::error::Error + Send + Sync>>;
}

/// 关键类型（节选）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StructuralSummary { /* 单/多语言的函数、类、导入、复杂度等 */ }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LanguageSummary { /* 单语言摘要 + file_count */ }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileAnalysisResult { /* file_path, language, summary, analysis_time */ }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectoryAnalysisResult { /* 按语言统计、总计等 */ }
```

性能说明：
- 并发分析使用固定大小的工作池（默认 4 个）。每个 worker 复用一个 TreeSitterManager 实例，避免为每个文件重复构建解析器与查询管理器。
- I/O 与解析并行执行，并在分析完成后记录吞吐量与单文件耗时统计日志。
- 使用 `max_concurrent` 控制并发度，建议根据 CPU 核心数与 I/O 能力进行调整。

#### `gitai::mcp`

MCP 服务与注册表模块。

```rust
// 服务管理器（带性能统计）
pub struct GitAiMcpManager {
    // managed_registry, performance_collector
}

impl GitAiMcpManager {
    /// 创建管理器（异步，根据配置自动注册启用的服务）
    pub async fn new(config: Config) -> McpResult<Self>;

    /// 列出所有工具（聚合各服务）
    pub async fn get_all_tools(&self) -> Vec<Tool>;

    /// 处理工具调用（按工具名路由到对应服务）
    pub async fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<serde_json::Value>;

    /// 可选：动态注册/注销服务、列出/筛选服务、获取性能统计
}

// 服务注册表（依赖管理、事件、拓扑排序）
pub struct ServiceRegistry { /* 内部: 元数据表、实例表、监听器 */ }

impl ServiceRegistry {
    pub fn new() -> Self;

    /// 注册服务（带元数据）
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<()>;

    /// 注销服务（按 service_id）
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()>;

    /// 列出服务（包含状态、依赖、工具等元数据）
    pub async fn list_services(&self) -> Vec<ServiceMetadata>;

    /// 获取启动顺序（拓扑排序，返回 service_id 列表）
    pub async fn get_startup_order(&self) -> McpResult<Vec<String>>;
}

#[async_trait]
pub trait GitAiMcpService: Send + Sync { /* name/description/version/dependencies/tools/handle_tool_call */ }
```

### 服务注册表 API

#### `gitai::mcp::registry`

服务注册与依赖管理（当前实现）。

```rust
use semver::{Version, VersionReq};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceMetadata>>>,
    service_instances: Arc<RwLock<HashMap<String, Arc<dyn GitAiMcpService + Send + Sync>>>>,
    event_listeners: Arc<RwLock<Vec<Arc<dyn ServiceEventListener + Send + Sync>>>>,
}

impl ServiceRegistry {
    /// 创建注册表
    pub fn new() -> Self;

    /// 注册服务（带依赖校验与循环检测）
    pub async fn register_service(
        &self,
        service: Arc<dyn GitAiMcpService + Send + Sync>,
        config: serde_json::Value,
    ) -> McpResult<()>;

    /// 注销服务
    pub async fn unregister_service(&self, service_id: &str, reason: String) -> McpResult<()>;

    /// 列出已注册服务（含状态/依赖/工具列表）
    pub async fn list_services(&self) -> Vec<ServiceMetadata>;

    /// 根据工具名查询所属服务
    pub async fn find_service_by_tool(&self, tool_name: &str)
        -> Option<Arc<dyn GitAiMcpService + Send + Sync>>;

    /// 获取启动顺序（拓扑排序）
    pub async fn get_startup_order(&self) -> McpResult<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub tools: Vec<String>,
    pub dependencies: Vec<ServiceDependency>,
    pub status: ServiceStatus,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone)]
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

### MCP Quickstart (stdio)

1) Set environment variables (example uses local Ollama; replace secrets with your own):
```bash
export GITAI_AI_API_URL="http://localhost:11434/v1/chat/completions"
export GITAI_AI_MODEL="qwen2.5:32b"
# Optional external API key
export GITAI_AI_API_KEY="{{OPENAI_OR_OTHER_API_KEY}}"
```

2) Start the MCP server (stdio transport):
```bash
gitai mcp --transport stdio
```

3) Minimal MCP request (tools/list):
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}
```

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

### 可用工具（工具名 → 所属服务）
- execute_review → review
- execute_commit → commit
- execute_scan → scan
- execute_analysis → analysis
- analyze_deviation → deviation
- execute_dependency_graph → dependency
- convert_graph_to_image → dependency
- summarize_graph → analysis
- query_call_chain → analysis

#### `execute_review`

执行代码评审。

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| path | string | 否 | 当前工作目录 | MCP 服务运行目录非仓库根时可指定仓库根路径 |
| tree_sitter | bool | 否 | false | 启用多语言结构分析 |
| security_scan | bool | 否 | false | 启用安全扫描（需 security 特性） |
| issue_ids | string[] | 否 | [] | 关联 Issue 列表，提供后隐式启用偏离度分析 |
| space_id | integer | 否 | 配置 devops.space_id | Coding 空间（项目）ID，覆盖配置 |
| scan_tool | string | 否 | opengrep | 扫描工具（如 opengrep） |
| deviation_analysis | bool | 否 | false | 显式开启偏离度分析（未提供时在有 issue_ids 时自动启用） |
| format | enum(text,json,markdown) | 否 | text | 输出格式 |

**请求：**
```json
{
  "path": ".",            // 可选：服务运行目录非仓库根时指定
  "tree_sitter": true,
  "security_scan": false,
  "issue_ids": ["#123"],
  "space_id": 123456,      // 可选
  "scan_tool": "opengrep", // 可选
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

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| message | string | 否 | null | 提交信息，未提供时将由 AI 生成 |
| issue_ids | string[] | 否 | [] | 关联 Issue 列表 |
| add_all | bool | 否 | false | 添加所有修改的文件 |
| review | bool | 否 | false | 提交前执行评审摘要 |
| tree_sitter | bool | 否 | false | 评审时启用结构分析（在 review=true 时有效） |
| dry_run | bool | 否 | false | 试运行，不实际提交 |

**请求：**
```json
{
  "message": null,
  "issue_ids": ["#123"],
  "add_all": false,
  "review": false,
  "tree_sitter": false,
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

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| path | string | 是 | - | 扫描路径 |
| tool | enum(opengrep,security) | 否 | opengrep | security 等同于 opengrep |
| timeout | integer | 否 | 300 | 超时时间（秒） |
| lang | string | 否 | 自动检测 | 语言过滤（默认多语言规则） |

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

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| path | string | 是 | - | 文件或目录路径 |
| language | string | 否 | 自动推断 | 编程语言（如 rust/java/go 等） |
| verbosity | integer(0-2) | 否 | 1 | 详细程度 |

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

#### `analyze_deviation`

分析代码变更与 DevOps Issue 的偏离度。

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| issue_ids | string[] | 是 | - | 需要评估的 Issue 列表 |
| diff | string | 否 | 从 git 获取 | 默认为当前变更 diff；可显式传入 |

**请求：**
```json
{
  "issue_ids": ["#123", "#456"],
  "diff": "...optional git diff content..."
}
```

**响应（示例）：**
```json
{
  "match_score": 85,
  "deviation_reasons": ["有部分 Issue 要求未完全满足"],
  "matched_issues": ["#123", "#456"],
  "unmatched_issues": [],
  "details": {"note": "..."},
  "needs_attention": false
}
```

#### `query_call_chain`

查询函数调用链（上游/下游）。

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| path | string | 否 | . | 扫描目录 |
| start | string | 是 | - | 起始函数名 |
| end | string | 否 | - | 结束函数名 |
| direction | enum(downstream,upstream) | 否 | downstream | 方向 |
| max_depth | integer(1-32) | 否 | 8 | 最大深度 |
| max_paths | integer(1-100) | 否 | 20 | 返回路径上限 |

#### `execute_dependency_graph`

生成代码依赖图（默认 ASCII）。

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| path | string | 是 | - | 文件或目录路径 |
| format | enum(json,dot,svg,mermaid,ascii) | 否 | ascii | 输出格式 |
| output | string | 否 | - | 输出文件路径（可选） |
| depth | integer | 否 | 无限 | 分析深度 |
| include_calls | bool | 否 | true | 是否包含调用关系 |
| include_imports | bool | 否 | true | 是否包含导入关系 |
| verbosity | integer(0-3) | 否 | 1 | 详细程度 |
|| confirm | bool | 否 | false | 大型项目导出完整图需确认 |

**请求：**
```json
{
  "name": "execute_dependency_graph",
  "arguments": {
    "path": ".",
    "format": "ascii",
    "verbosity": 1
  }
}
```

#### `convert_graph_to_image`

将 DOT 或 Mermaid 内容转换为 PNG/SVG/PDF 图像文件。

参数表：

| 名称 | 类型 | 必填 | 默认 | 说明 |
|-----|------|------|------|------|
| input_format | enum(dot,mermaid) | 是 | - | 输入格式 |
| input_content | string | 是 | - | 图内容（DOT/Mermaid） |
| output_format | enum(png,svg,pdf) | 是 | - | 输出格式 |
| output_path | string | 是 | - | 输出文件路径 |
| engine | enum(dot,neato,circo,fdp,sfdp,twopi) | 否 | dot | Graphviz 布局引擎 |

**请求：**
```json
{
  "name": "convert_graph_to_image",
  "arguments": {
    "input_format": "dot",
    "input_content": "digraph G { A -> B }",
    "output_format": "svg",
    "output_path": "graph.svg",
    "engine": "dot"
  }
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
    let result = manager
        .handle_tool_call(
            "execute_review",
            json!({
                "tree_sitter": true,
                "security_scan": false
            })
        )
        .await?;
    
    println!("评审结果: {}", serde_json::to_string_pretty(&result)?);
    
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
