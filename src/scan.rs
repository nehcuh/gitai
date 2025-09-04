use crate::config::Config;
use log::debug;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

// 全局版本缓存，避免重复调用
lazy_static::lazy_static! {
    static ref VERSION_CACHE: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref RULES_CACHE: Arc<RwLock<HashMap<std::path::PathBuf, RulesInfo>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub tool: String,
    pub version: String,
    pub execution_time: f64,
    pub findings: Vec<Finding>,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rules_info: Option<RulesInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesInfo {
    pub dir: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sources: Vec<String>,
    pub total_rules: usize,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub updated_at: Option<String>,
}

/// 安全问题发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub title: String,
    pub file_path: std::path::PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: String,
    pub rule_id: Option<String>,
    pub code_snippet: Option<String>,
    pub message: String,
    pub remediation: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 运行OpenGrep扫描
pub fn run_opengrep_scan(
    config: &Config,
    path: &Path,
    lang: Option<&str>,
    timeout_override: Option<u64>,
    include_version: bool,
) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let start_time = std::time::Instant::now();

    // 检查路径是否存在
    if !path.exists() {
        log::error!("扫描路径不存在: {}", path.display());
        return Err(format!("扫描路径不存在: {}", path.display()).into());
    }

    log::info!("开始扫描: {}", path.display());

    // 构建命令（不要把可执行名放入 args）
    let mut args = vec![
        "--json".to_string(),
        "--quiet".to_string(),
        format!(
            "--timeout={}",
            timeout_override.unwrap_or(config.scan.timeout)
        ),
    ];
    if config.scan.jobs > 0 {
        args.push(format!("--jobs={}", config.scan.jobs));
    }

    // 规则目录
    let rules_dir = config
        .scan
        .rules_dir
        .clone()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cache")
                .join("gitai")
                .join("rules")
        });
    let mut rules_info: Option<RulesInfo> = None;
    let mut used_config_paths: Vec<std::path::PathBuf> = Vec::new();
    if rules_dir.exists() {
        if let Ok(mut iter) = std::fs::read_dir(&rules_dir) {
            if iter.next().is_some() {
                // 语言已指定：仅使用该子目录；未指定：包含所有存在的语言子目录，避免根目录中的非规则 YAML 被解析
                let known_langs = [
                    "java",
                    "python",
                    "javascript",
                    "typescript",
                    "go",
                    "rust",
                    "c",
                    "cpp",
                    "ruby",
                    "php",
                    "kotlin",
                    "scala",
                    "swift",
                ];
                if let Some(l) = lang {
                    let candidate = rules_dir.join(l);
                    let rules_root = if candidate.exists() {
                        candidate
                    } else {
                        rules_dir.clone()
                    };
                    used_config_paths.push(rules_root.clone());
                } else {
                    for l in known_langs {
                        let p = rules_dir.join(l);
                        if p.exists() && p.is_dir() {
                            used_config_paths.push(p);
                        }
                    }
                    // 回退：若没有任何语言子目录存在，则退回根目录
                    if used_config_paths.is_empty() {
                        used_config_paths.push(rules_dir.clone());
                    }
                }

                // 添加所有配置目录
                for p in &used_config_paths {
                    args.push(format!("--config={}", p.display()));
                }

                // 读取元信息：优先使用第一个有效目录；如果没有，则尝试根目录
                if let Some(first) = used_config_paths.first() {
                    rules_info = read_rules_info(first).or_else(|| read_rules_info(&rules_dir));
                }
            }
        }
    }

    // 执行命令
    log::debug!("执行命令: opengrep {} {}", args.join(" "), path.display());
    let output = Command::new("opengrep")
        .args(&args)
        .arg(path)
        .output()
        .map_err(|e| {
            log::error!("执行 OpenGrep 失败: {e}");
            format!("执行 OpenGrep 失败: {e}\n💡 请确保 OpenGrep 已安装并在 PATH 中")
        })?;

    let execution_time = start_time.elapsed().as_secs_f64();

    // 处理退出码
    // OpenGrep/Semgrep 退出码说明：
    // 0 = 成功，有或没有发现
    // 1 = 未捕获的错误
    // 2 = 命令无效或找不到规则/文件
    // 对于退出码 2，我们需要检查是否真的是错误还是只是没有发现
    let exit_code = output.status.code().unwrap_or(-1);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr_trim = stderr.trim();

        // 退出码 2 可能只是没有匹配的文件或规则，需要进一步判断
        if exit_code == 2 {
            // 检查是否有实际的错误信息
            if stderr_trim.is_empty()
                || stderr_trim.contains("No rules")
                || stderr_trim.contains("No files")
            {
                // 这是一个"无发现"的情况，不是真正的错误
                log::info!("OpenGrep 退出码 2：无匹配规则或文件，视为成功扫描");
                // 继续处理，将其视为成功但无发现
            } else {
                // 有实际的错误信息
                let err_msg = stderr_trim.to_string();
                log::warn!("OpenGrep 返回错误状态码 2: {}", err_msg);
                return Ok(ScanResult {
                    tool: "opengrep".to_string(),
                    version: if include_version {
                        get_opengrep_version()?
                    } else {
                        "unknown".to_string()
                    },
                    execution_time,
                    findings: vec![],
                    error: Some(err_msg),
                    rules_info,
                });
            }
        } else {
            // 其他非零退出码，视为错误
            let err_msg = if !stderr_trim.is_empty() {
                stderr_trim.to_string()
            } else {
                // 附带 stdout 的前几行，帮助定位（截断到 500 字符）
                let head = stdout.lines().take(5).collect::<Vec<_>>().join(" | ");
                if head.is_empty() {
                    format!("OpenGrep exited with status {} (no stderr)", exit_code)
                } else {
                    let mut s = format!(
                        "OpenGrep exited with status {} (no stderr). stdout: {}",
                        exit_code, head
                    );
                    if s.len() > 500 {
                        s.truncate(500);
                    }
                    s
                }
            };
            log::warn!("OpenGrep 返回非零状态码 ({}): {}", exit_code, err_msg);
            return Ok(ScanResult {
                tool: "opengrep".to_string(),
                version: if include_version {
                    get_opengrep_version()?
                } else {
                    "unknown".to_string()
                },
                execution_time,
                findings: vec![],
                error: Some(err_msg),
                rules_info,
            });
        }
    }

    // 解析结果
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("📄 OpenGrep stdout: {stdout}");
    if !used_config_paths.is_empty() {
        let joined = used_config_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        debug!("📦 使用规则目录: {}", joined);
    }

    let findings = match parse_opengrep_output(&stdout) {
        Ok(f) => f,
        Err(e) => {
            debug!("❌ JSON 解析失败: {e}");
            return Ok(ScanResult {
                tool: "opengrep".to_string(),
                version: if include_version {
                    get_opengrep_version().unwrap_or_else(|_| "unknown".to_string())
                } else {
                    "unknown".to_string()
                },
                execution_time,
                findings: vec![],
                error: Some(format!("JSON 解析失败: {e}")),
                rules_info,
            });
        }
    };

    Ok(ScanResult {
        tool: "opengrep".to_string(),
        version: if include_version {
            get_opengrep_version()?
        } else {
            "unknown".to_string()
        },
        execution_time,
        findings,
        error: None,
        rules_info,
    })
}

/// 获取OpenGrep版本（使用缓存）
fn get_opengrep_version() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    // 先检查缓存
    {
        let cache = VERSION_CACHE.read();
        if let Some(version) = cache.get("opengrep") {
            return Ok(version.clone());
        }
    }

    // 缓存未命中，执行命令
    let output = Command::new("opengrep").arg("--version").output()?;

    let version = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    };

    // 写入缓存
    {
        let mut cache = VERSION_CACHE.write();
        cache.insert("opengrep".to_string(), version.clone());
    }

    Ok(version)
}

/// 解析OpenGrep输出（整块 JSON，遍历 results 数组）
fn parse_opengrep_output(
    output: &str,
) -> Result<Vec<Finding>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    debug!("🔍 解析OpenGrep输出: {output}");

    if output.trim().is_empty() {
        debug!("⚠️ OpenGrep 输出为空");
        return Ok(Vec::new());
    }

    // 查找 JSON 部分（可能有标题信息在前面）
    let json_part = if let Some(pos) = output.find('{') {
        &output[pos..]
    } else {
        debug!("⚠️ 未找到 JSON 开始标志");
        return Ok(Vec::new());
    };

    let v: serde_json::Value = serde_json::from_str(json_part)
        .map_err(|e| format!("JSON 解析失败: {e}, JSON部分: {json_part}"))?;

    debug!("📄 JSON 结构: {v:?}");

    let mut findings = Vec::new();
    if let Some(results) = v.get("results").and_then(|r| r.as_array()) {
        debug!("📋 找到 {} 个结果", results.len());
        for (i, item) in results.iter().enumerate() {
            match create_finding_from_result(item) {
                Ok(finding) => {
                    findings.push(finding);
                }
                Err(e) => {
                    debug!("❌ 解析第 {i} 个结果失败: {e}");
                }
            }
        }
    } else {
        debug!("⚠️ 未找到 results 数组");
        // 检查是否有错误信息
        if let Some(errors) = v.get("errors").and_then(|e| e.as_array()) {
            debug!("❌ OpenGrep 报告错误: {errors:?}");
        }
        // 检查扫描的路径
        if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
            debug!("📂 扫描的路径: {paths:?}");
        }
    }

    Ok(findings)
}

/// 从 results[i] 构建 Finding
fn create_finding_from_result(
    item: &serde_json::Value,
) -> Result<Finding, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let title = item["extra"]["message"]
        .as_str()
        .unwrap_or("Unknown issue")
        .to_string();
    let file_path = item["path"].as_str().unwrap_or("").to_string();
    let line = item["start"]["line"].as_u64().unwrap_or(0) as usize;
    let column = item["start"]["col"].as_u64().unwrap_or(0) as usize;
    let rule_id = item["check_id"].as_str().map(|s| s.to_string());

    let severity_str = item["severity"].as_str().unwrap_or("WARNING");

    let code_snippet = item["lines"].as_str().map(|s| s.to_string());

    let message = item["extra"]["message"]
        .as_str()
        .unwrap_or(title.as_str())
        .to_string();

    let remediation = item["extra"]["fix"].as_str().map(|s| s.to_string());

    Ok(Finding {
        title,
        file_path: std::path::PathBuf::from(file_path),
        line,
        column,
        severity: severity_str.to_string(),
        rule_id,
        code_snippet,
        message,
        remediation,
    })
}

/// 检查OpenGrep是否已安装
pub fn is_opengrep_installed() -> bool {
    Command::new("opengrep")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn read_rules_info(rules_dir: &std::path::Path) -> Option<RulesInfo> {
    use std::fs;

    // 先检查缓存
    {
        let cache = RULES_CACHE.read();
        if let Some(info) = cache.get(rules_dir) {
            return Some(info.clone());
        }
    }

    let meta_path = rules_dir.join(".rules.meta");
    let rules_info = if let Ok(content) = fs::read_to_string(&meta_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let sources = v["sources"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(|x| x.to_string()))
                        .collect()
                })
                .unwrap_or_else(Vec::new);
            let total = v["total_rules"].as_u64().unwrap_or(0) as usize;
            let updated_at = v["updated_at"].as_str().map(|s| s.to_string());
            Some(RulesInfo {
                dir: rules_dir.display().to_string(),
                sources,
                total_rules: total,
                updated_at,
            })
        } else {
            // 回退：仅提供目录
            Some(RulesInfo {
                dir: rules_dir.display().to_string(),
                sources: Vec::new(),
                total_rules: 0,
                updated_at: None,
            })
        }
    } else {
        // 回退：仅提供目录
        Some(RulesInfo {
            dir: rules_dir.display().to_string(),
            sources: Vec::new(),
            total_rules: 0,
            updated_at: None,
        })
    };

    // 写入缓存
    if let Some(ref info) = rules_info {
        let mut cache = RULES_CACHE.write();
        cache.insert(rules_dir.to_path_buf(), info.clone());
    }

    rules_info
}


/// 安装OpenGrep（优先使用 cargo；若不可用则给出明确指引）
pub fn install_opengrep() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("🔧 正在安装OpenGrep...");

    // 先检测 cargo 是否可用
    let cargo_available = Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if cargo_available {
        let output = Command::new("cargo")
            .args(["install", "opengrep"])
            .output()?;

        if output.status.success() {
            // 提示 PATH 配置（如未生效）
            if !is_opengrep_installed() {
                println!("ℹ️ 已通过 cargo 安装，但未检测到 opengrep 在 PATH。若使用 rustup 默认目录，请添加到 PATH:");
                println!("   export PATH=\"$HOME/.cargo/bin:$PATH\"");
            }
            println!("✅ OpenGrep 安装完成");
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "通过 cargo 安装 OpenGrep 失败: {stderr}\n建议：\n1) 确认已安装 Rust 工具链 (https://rustup.rs) 并已将 ~/.cargo/bin 加入 PATH\n2) 手动执行: cargo install opengrep"
            ).into());
        }
    }

    // cargo 不可用：给出明确的安装指引
    let guide = "未检测到 cargo。请先安装 Rust 工具链，然后使用 cargo 安装 OpenGrep:\n\n1) 安装 Rust（推荐 rustup）: https://rustup.rs\n2) 安装 OpenGrep: cargo install opengrep\n3) 将 cargo 的 bin 目录加入 PATH: export PATH=\"$HOME/.cargo/bin:$PATH\"";
    Err(guide.into())
}
