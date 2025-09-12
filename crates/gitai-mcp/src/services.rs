//! GitAI MCP 服务实现 - 简化版本
//!
//! 提供所有 GitAI 功能的 MCP 服务接口：
//! - 代码评审
//! - 安全扫描
//! - 智能提交
//! - 代码分析
//! - 依赖图生成
//! - 偏差分析

use crate::error::{McpError, McpResult};
use async_trait::async_trait;
use gitai_core::config::Config;
use log::{debug, info, warn};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

// Real subsystems
use gitai_analysis::analysis::Analyzer as CodeAnalyzer;
use gitai_analysis::architectural_impact::graph_export;
use gitai_analysis::architectural_impact::graph_export::build_global_dependency_graph;
use gitai_analysis::{OperationContext as AnalysisCtx, OperationOptions as AnalysisOpts};
use gitai_core::git;
use gitai_security::scanner as security;
use std::process::{Command, Stdio};

/// MCP 服务接口 - 简化版本
#[async_trait]
pub trait McpService: Send + Sync {
    /// 获取服务名称
    fn name(&self) -> &str;

    /// 获取服务描述
    fn description(&self) -> &str;

    /// 检查服务是否可用
    async fn is_available(&self) -> bool;

    /// 执行服务
    async fn execute(&self, params: Value) -> Result<Value, McpError>;
}

/// 代码评审服务
pub struct ReviewService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl ReviewService {
    /// 创建新的代码评审服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行代码评审
    pub async fn execute_review(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行代码评审，参数: {params:?}");

        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let issue_ids = params
            .get("issue_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let tree_sitter = params
            .get("tree_sitter")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let security_scan = params
            .get("security_scan")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!("📝 开始代码评审: {path}");

        // 构建分析上下文
        let mut opts = AnalysisOpts::default();
        opts.tree_sitter = tree_sitter;
        opts.security_scan = security_scan;
        opts.issue_ids = issue_ids.clone();
        opts.deviation_analysis = false;
        opts.format = format;

        let mut ctx = AnalysisCtx::new((*self.config).clone());
        // 获取全量 diff（若没有变更则尝试最后一次提交）
        match git::get_all_diff() {
            Ok(diff) => {
                ctx = ctx.with_options(opts).with_diff(diff);
            }
            Err(e) => {
                warn!("获取 diff 失败: {e}");
                ctx = ctx.with_options(opts);
            }
        }

        let analysis = CodeAnalyzer::analyze(&ctx)
            .await
            .map_err(|e| McpError::ExecutionFailed(format!("Analyzer failed: {e}")))?;

        let out = json!({
            "status": "success",
            "path": path,
            "review_report": analysis.review_result,
            "security_findings": analysis.security_findings,
            "deviation_analysis": analysis.deviation_analysis,
            "duration_ms": start_time.elapsed().as_millis(),
            "issue_ids": issue_ids,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        info!(
            "✅ 代码评审完成，耗时: {}ms",
            start_time.elapsed().as_millis()
        );
        Ok(out)
    }
}

#[async_trait]
impl McpService for ReviewService {
    fn name(&self) -> &str {
        "代码评审"
    }

    fn description(&self) -> &str {
        "执行代码评审和质量分析"
    }

    async fn is_available(&self) -> bool {
        true // 简化版本总是可用
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_review(&params).await
    }
}

/// 安全扫描服务
pub struct ScanService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl ScanService {
    /// 创建新的安全扫描服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行安全扫描
    pub async fn execute_scan(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行安全扫描，参数: {params:?}");

        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let language = params.get("lang").and_then(|v| v.as_str());
        let timeout = params.get("timeout").and_then(|v| v.as_u64());
        info!(
            "🔒 开始安全扫描: {path} (语言: {})",
            language.unwrap_or("auto")
        );

        let res =
            security::run_opengrep_scan(&self.config, Path::new(path), language, timeout, true)
                .map_err(|e| McpError::ExecutionFailed(format!("opengrep failed: {e}")))?;

        let mut v = serde_json::to_value(&res)
            .map_err(|e| McpError::ExecutionFailed(format!("serialize scan result failed: {e}")))?;
        // 附带元信息
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "duration_ms".to_string(),
                json!(start_time.elapsed().as_millis()),
            );
            obj.insert("path".to_string(), json!(path));
        }

        info!(
            "✅ 安全扫描完成，耗时: {}ms",
            start_time.elapsed().as_millis()
        );
        Ok(v)
    }
}

#[async_trait]
impl McpService for ScanService {
    fn name(&self) -> &str {
        "安全扫描"
    }

    fn description(&self) -> &str {
        "执行安全漏洞扫描"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_scan(&params).await
    }
}

/// 智能提交服务
pub struct CommitService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl CommitService {
    /// 创建新的智能提交服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行智能提交
    pub async fn execute_commit(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行智能提交，参数: {params:?}");

        let add_all = params
            .get("add_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let dry_run = params
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("feat: 自动生成的提交信息");

        info!("💾 开始智能提交 (add_all: {add_all}, dry_run: {dry_run})");

        if dry_run {
            // 只返回将要执行的信息
            return Ok(json!({
                "status": "dry_run",
                "planned": {
                    "add_all": add_all,
                    "commit_message": message,
                },
                "duration_ms": start_time.elapsed().as_millis(),
            }));
        }

        if add_all {
            git::git_add_all()
                .map_err(|e| McpError::ExecutionFailed(format!("git add . failed: {e}")))?;
        }
        let _ = git::git_commit(message)
            .map_err(|e| McpError::ExecutionFailed(format!("git commit failed: {e}")))?;
        let hash = git::get_current_commit()
            .map_err(|e| McpError::ExecutionFailed(format!("get commit hash failed: {e}")))?;

        let result = json!({
            "status": "success",
            "commit": {
                "hash": hash.trim(),
                "message": message,
            },
            "duration_ms": start_time.elapsed().as_millis(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        info!(
            "✅ 智能提交完成，耗时: {}ms",
            start_time.elapsed().as_millis()
        );
        Ok(result)
    }
}

#[async_trait]
impl McpService for CommitService {
    fn name(&self) -> &str {
        "智能提交"
    }

    fn description(&self) -> &str {
        "执行智能代码提交"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_commit(&params).await
    }
}

/// 代码分析服务
pub struct AnalysisService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl AnalysisService {
    /// 创建新的代码分析服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行代码分析
    pub async fn execute_analysis(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行代码分析，参数: {params:?}");

        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let language = params
            .get("language")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let verbosity = params
            .get("verbosity")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        info!(
            "🔬 开始代码分析: {path} (语言: {}, 详细程度: {verbosity})",
            language.as_deref().unwrap_or("auto")
        );

        let mut opts = AnalysisOpts::default();
        opts.tree_sitter = true;
        opts.language = language;
        // verbosity 暂未直接使用，保留供内部日志参考

        let mut ctx = AnalysisCtx::new((*self.config).clone());
        match git::get_all_diff() {
            Ok(diff) => ctx = ctx.with_options(opts).with_diff(diff),
            Err(_) => ctx = ctx.with_options(opts),
        }

        let analysis = CodeAnalyzer::analyze(&ctx)
            .await
            .map_err(|e| McpError::ExecutionFailed(format!("Analyzer failed: {e}")))?;

        let out = serde_json::to_value(&analysis)
            .unwrap_or_else(|_| json!({"review_result": analysis.review_result}));

        info!(
            "✅ 代码分析完成，耗时: {}ms",
            start_time.elapsed().as_millis()
        );
        Ok(json!({
            "status": "success",
            "path": path,
            "analysis": out,
            "duration_ms": start_time.elapsed().as_millis(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[async_trait]
impl McpService for AnalysisService {
    fn name(&self) -> &str {
        "代码分析"
    }

    fn description(&self) -> &str {
        "执行多语言代码结构分析"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_analysis(&params).await
    }
}

/// 依赖图服务
pub struct DependencyService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl DependencyService {
    /// 创建新的依赖图服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行依赖图生成
    pub async fn execute_dependency_graph(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行依赖图生成，参数: {params:?}");

        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");
        let _include_calls = params
            .get("include_calls")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let _include_imports = params
            .get("include_imports")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        info!("🔗 开始生成依赖图: {path} (格式: {format})");

        match format {
            "mermaid" => {
                let graph = build_global_dependency_graph(Path::new(path))
                    .await
                    .map_err(|e| McpError::ExecutionFailed(format!("build graph failed: {e}")))?;
                let mut out = String::from("graph LR\n");
                for e in &graph.edges {
                    out.push_str(&format!("  \"{}\" --> \"{}\"\n", e.from, e.to));
                }
                Ok(json!({
                    "status": "success",
                    "path": path,
                    "format": "mermaid",
                    "graph": out,
                    "duration_ms": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            "svg" => {
                let dot = graph_export::export_dot_string(Path::new(path), 0.15)
                    .await
                    .map_err(|e| McpError::ExecutionFailed(format!("graph export failed: {e}")))?;
                // 尝试使用 graphviz dot 转换为 SVG
                let svg = match try_dot_to_svg(&dot) {
                    Ok(svg) => svg,
                    Err(e) => {
                        return Ok(json!({
                            "status": "degraded",
                            "path": path,
                            "format": "dot",
                            "graph": dot,
                            "warning": format!("dot->svg 转换失败: {} (可能未安装 graphviz 'dot')", e),
                            "duration_ms": start_time.elapsed().as_millis(),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    }
                };
                Ok(json!({
                    "status": "success",
                    "path": path,
                    "format": "svg",
                    "graph": svg,
                    "duration_ms": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            "json" => {
                // 使用摘要 JSON（更易消费）
                let summary = graph_export::export_summary_string(
                    Path::new(path),
                    1,     // radius
                    200,   // top_k
                    false, // seeds_from_diff
                    "json",
                    3000,  // budget_tokens
                    false, // with_communities
                    "labelprop",
                    50,
                    10,
                    false,
                    5,
                    5,
                )
                .await
                .map_err(|e| McpError::ExecutionFailed(format!("graph summary failed: {e}")))?;
                let json_v: Value =
                    serde_json::from_str(&summary).unwrap_or_else(|_| json!({"summary": summary}));
                Ok(json!({
                    "status": "success",
                    "path": path,
                    "format": "json",
                    "graph": json_v,
                    "duration_ms": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
            "dot" | "ascii" | _ => {
                let dot = graph_export::export_dot_string(Path::new(path), 0.15)
                    .await
                    .map_err(|e| McpError::ExecutionFailed(format!("graph export failed: {e}")))?;
                Ok(json!({
                    "status": "success",
                    "path": path,
                    "format": "dot",
                    "graph": dot,
                    "duration_ms": start_time.elapsed().as_millis(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            }
        }
    }
}

fn try_dot_to_svg(dot: &str) -> Result<String, String> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn dot failed: {}", e))?;
    use std::io::Write as _;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(dot.as_bytes())
            .map_err(|e| format!("write dot stdin failed: {}", e))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait dot failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("dot exited with status {:?}", output.status.code()));
    }
    let svg = String::from_utf8(output.stdout).map_err(|e| format!("invalid utf8 svg: {}", e))?;
    Ok(svg)
}

#[async_trait]
impl McpService for DependencyService {
    fn name(&self) -> &str {
        "依赖图"
    }

    fn description(&self) -> &str {
        "生成代码依赖图"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_dependency_graph(&params).await
    }
}

/// 偏差分析服务
pub struct DeviationService {
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl DeviationService {
    /// 创建新的偏差分析服务
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    /// 执行偏差分析
    pub async fn execute_deviation(&self, params: &Value) -> McpResult<Value> {
        let start_time = Instant::now();
        debug!("🔍 执行偏差分析，参数: {params:?}");

        let issue_ids = params
            .get("issue_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        info!("📊 开始偏差分析 (Issue 数量: {len})", len = issue_ids.len());

        let mut opts = AnalysisOpts::default();
        opts.deviation_analysis = true;
        opts.issue_ids = issue_ids.clone();

        let mut ctx = AnalysisCtx::new((*self.config).clone());
        match git::get_all_diff() {
            Ok(diff) => ctx = ctx.with_options(opts).with_diff(diff),
            Err(_) => ctx = ctx.with_options(opts),
        }

        let analysis = CodeAnalyzer::analyze(&ctx)
            .await
            .map_err(|e| McpError::ExecutionFailed(format!("Analyzer failed: {e}")))?;

        let out = json!({
            "status": "success",
            "issue_ids": issue_ids,
            "deviation_analysis": analysis.deviation_analysis,
            "duration_ms": start_time.elapsed().as_millis(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!(
            "✅ 偏差分析完成，耗时: {}ms",
            start_time.elapsed().as_millis()
        );
        Ok(out)
    }
}

#[async_trait]
impl McpService for DeviationService {
    fn name(&self) -> &str {
        "偏差分析"
    }

    fn description(&self) -> &str {
        "分析代码变更与 Issue 的偏离度"
    }

    async fn is_available(&self) -> bool {
        true
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        self.execute_deviation(&params).await
    }
}

/// 服务工厂 - 创建所有服务实例
pub struct ServiceFactory;

impl ServiceFactory {
    /// 创建所有服务
    pub fn create_services(config: Arc<Config>) -> Vec<Box<dyn McpService>> {
        vec![
            Box::new(ReviewService::new(config.clone())),
            Box::new(ScanService::new(config.clone())),
            Box::new(CommitService::new(config.clone())),
            Box::new(AnalysisService::new(config.clone())),
            Box::new(DependencyService::new(config.clone())),
            Box::new(DeviationService::new(config.clone())),
        ]
    }

    /// 根据名称获取服务
    pub fn get_service_by_name<'a>(
        services: &'a [Box<dyn McpService>],
        name: &'a str,
    ) -> Option<&'a dyn McpService> {
        services
            .iter()
            .find(|s| s.name() == name)
            .map(|s| s.as_ref())
    }
}
