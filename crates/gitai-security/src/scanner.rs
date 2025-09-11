use gitai_core::config::Config;
use lazy_static::lazy_static;
use log::debug;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

lazy_static! {
    static ref VERSION_CACHE: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref RULES_CACHE: Arc<RwLock<HashMap<PathBuf, RulesInfo>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Security scan result
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

/// Security finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub title: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: String,
    pub rule_id: Option<String>,
    pub code_snippet: Option<String>,
    pub message: String,
    pub remediation: Option<String>,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Run OpenGrep security scan
pub fn run_opengrep_scan(
    config: &Config,
    path: &Path,
    lang: Option<&str>,
    timeout_override: Option<u64>,
    include_version: bool,
) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let start_time = std::time::Instant::now();

    if !path.exists() {
        log::error!("扫描路径不存在: {}", path.display());
        return Err(format!("扫描路径不存在: {}", path.display()).into());
    }

    log::info!("开始扫描: {}", path.display());

    let mut args = vec![
        "--json".to_string(),
        "--quiet".to_string(),
        format!("--timeout={}", timeout_override.unwrap_or(config.scan.timeout)),
    ];
    if config.scan.jobs > 0 {
        args.push(format!("--jobs={}", config.scan.jobs));
    }

    // honor .gitignore
    args.push("--use-git-ignore".to_string());

    // Rules directory
    let rules_dir = config
        .scan
        .rules_dir
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
                .join("gitai")
                .join("rules")
        });
    let mut rules_info: Option<RulesInfo> = None;
    let mut used_config_paths: Vec<PathBuf> = Vec::new();
    if rules_dir.exists() {
        if let Ok(mut iter) = std::fs::read_dir(&rules_dir) {
            if iter.next().is_some() {
                let known_langs = [
                    "java", "python", "javascript", "typescript", "go", "rust", "c", "cpp", "ruby", "php", "kotlin", "scala", "swift",
                ];

                fn dir_contains_valid_rules(dir: &Path) -> bool {
                    use std::fs;
                    let mut stack = vec![dir.to_path_buf()];
                    while let Some(d) = stack.pop() {
                        if let Ok(entries) = fs::read_dir(&d) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_dir() {
                                    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                                        if name.starts_with('.') {
                                            continue;
                                        }
                                    }
                                    stack.push(p);
                                } else if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                                    if ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml") {
                                        if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                                            if fname.starts_with('.') {
                                                continue;
                                            }
                                            if fname.contains("pre-commit") {
                                                continue;
                                            }
                                        }
                                        if let Ok(content) = fs::read_to_string(&p) {
                                            for line in content.lines().take(200) {
                                                let t = line.trim_start();
                                                if t.starts_with("rules:") || t.starts_with("rules :") {
                                                    return true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    false
                }

                let mut candidate_roots: Vec<PathBuf> = vec![rules_dir.clone()];
                if let Ok(entries) = std::fs::read_dir(&rules_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            candidate_roots.push(p);
                        }
                    }
                }

                if let Some(l) = lang {
                    let mut found_any = false;
                    for root in &candidate_roots {
                        let candidate = root.join(l);
                        if candidate.exists() && candidate.is_dir() {
                            if dir_contains_valid_rules(&candidate) {
                                if !used_config_paths.iter().any(|x| x == &candidate) {
                                    used_config_paths.push(candidate.clone());
                                }
                                found_any = true;
                            } else {
                                log::warn!("指定语言 '{}' 的规则目录存在但未检测到有效规则: {}", l, candidate.display());
                            }
                        }
                    }
                    if !found_any {
                        log::warn!("未找到指定语言 '{}' 的有效规则目录（已检查候选根目录下的子目录）: {}", l, rules_dir.display());
                    }
                } else {
                    for root in &candidate_roots {
                        for l in known_langs {
                            let p = root.join(l);
                            if p.exists() && p.is_dir() && dir_contains_valid_rules(&p) && !used_config_paths.iter().any(|x| x == &p) {
                                used_config_paths.push(p);
                            }
                        }
                    }
                    if used_config_paths.is_empty() {
                        log::warn!("未在规则目录及其一级子目录下找到任何包含有效规则的语言子目录: {}", rules_dir.display());
                    }
                }

                for p in &used_config_paths {
                    args.push(format!("--config={}", p.display()));
                }

                if let Some(first) = used_config_paths.first() {
                    rules_info = read_rules_info(first).or_else(|| read_rules_info(&rules_dir));
                }
            }
        }
    }

    log::debug!("执行命令: opengrep {} {}", args.join(" "), path.display());
    let output = Command::new("opengrep")
        .args(&args)
        .arg(path)
        .output()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            log::error!("执行 OpenGrep 失败: {e}");
            Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "执行 OpenGrep 失败: {e}\n💡 请确保 OpenGrep 已安装并在 PATH 中"
            ))
        })?;

    let execution_time = start_time.elapsed().as_secs_f64();
    let exit_code = output.status.code().unwrap_or(-1);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr_trim = stderr.trim();

        if exit_code == 2 {
            if stderr_trim.is_empty() || stderr_trim.contains("No rules") || stderr_trim.contains("No files") {
                log::info!("OpenGrep 退出码 2：无匹配规则或文件，视为成功扫描");
            } else {
                let err_msg = stderr_trim.to_string();
                log::warn!("OpenGrep 返回错误状态码 2: {}", err_msg);
                return Ok(ScanResult {
                    tool: "opengrep".to_string(),
                    version: if include_version { get_opengrep_version()? } else { "unknown".to_string() },
                    execution_time,
                    findings: vec![],
                    error: Some(err_msg),
                    rules_info,
                });
            }
        } else {
            let err_msg = if !stderr_trim.is_empty() {
                stderr_trim.to_string()
            } else {
                let head = stdout.lines().take(5).collect::<Vec<_>>().join(" | ");
                if head.is_empty() {
                    format!("OpenGrep exited with status {} (no stderr)", exit_code)
                } else {
                    let mut s = format!("OpenGrep exited with status {} (no stderr). stdout: {}", exit_code, head);
                    if s.len() > 500 { s.truncate(500); }
                    s
                }
            };
            log::warn!("OpenGrep 返回非零状态码 ({}): {}", exit_code, err_msg);
            return Ok(ScanResult {
                tool: "opengrep".to_string(),
                version: if include_version { get_opengrep_version()? } else { "unknown".to_string() },
                execution_time,
                findings: vec![],
                error: Some(err_msg),
                rules_info,
            });
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("📄 OpenGrep stdout: {stdout}");
    if !used_config_paths.is_empty() {
        let joined = used_config_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ");
        debug!("📦 使用规则目录: {}", joined);
    }

    let findings = match parse_opengrep_output(&stdout) {
        Ok(f) => f,
        Err(e) => {
            debug!("❌ JSON 解析失败: {e}");
            return Ok(ScanResult {
                tool: "opengrep".to_string(),
                version: if include_version { get_opengrep_version().unwrap_or_else(|_| "unknown".to_string()) } else { "unknown".to_string() },
                execution_time,
                findings: vec![],
                error: Some(format!("JSON 解析失败: {e}")),
                rules_info,
            });
        }
    };

    Ok(ScanResult {
        tool: "opengrep".to_string(),
        version: if include_version { get_opengrep_version()? } else { "unknown".to_string() },
        execution_time,
        findings,
        error: None,
        rules_info,
    })
}

fn get_opengrep_version() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    {
        let cache = VERSION_CACHE.read();
        if let Some(version) = cache.get("opengrep") {
            return Ok(version.clone());
        }
    }

    let output = Command::new("opengrep").arg("--version").output()?;
    let version = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    };

    {
        let mut cache = VERSION_CACHE.write();
        cache.insert("opengrep".to_string(), version.clone());
    }

    Ok(version)
}

fn parse_opengrep_output(
    output: &str,
) -> Result<Vec<Finding>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    debug!("🔍 解析OpenGrep输出: {output}");

    if output.trim().is_empty() {
        debug!("⚠️ OpenGrep 输出为空");
        return Ok(Vec::new());
    }

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
                Ok(finding) => findings.push(finding),
                Err(e) => debug!("❌ 解析第 {i} 个结果失败: {e}"),
            }
        }
    } else {
        debug!("⚠️ 未找到 results 数组");
        if let Some(errors) = v.get("errors").and_then(|e| e.as_array()) {
            debug!("❌ OpenGrep 报告错误: {errors:?}");
        }
        if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
            debug!("📂 扫描的路径: {paths:?}");
        }
    }

    Ok(findings)
}

fn create_finding_from_result(
    item: &serde_json::Value,
) -> Result<Finding, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let title = item["extra"]["message"].as_str().unwrap_or("Unknown issue").to_string();
    let file_path = item["path"].as_str().unwrap_or("").to_string();
    let line = item["start"]["line"].as_u64().unwrap_or(0) as usize;
    let column = item["start"]["col"].as_u64().unwrap_or(0) as usize;
    let rule_id = item["check_id"].as_str().map(|s| s.to_string());
    let severity_str = item["severity"].as_str().unwrap_or("WARNING");
    let code_snippet = item["lines"].as_str().map(|s| s.to_string());
    let message = item["extra"]["message"].as_str().unwrap_or(title.as_str()).to_string();
    let remediation = item["extra"]["fix"].as_str().map(|s| s.to_string());

    Ok(Finding {
        title,
        file_path: PathBuf::from(file_path),
        line,
        column,
        severity: severity_str.to_string(),
        rule_id,
        code_snippet,
        message,
        remediation,
    })
}

/// Check whether OpenGrep is installed
pub fn is_opengrep_installed() -> bool {
    Command::new("opengrep")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Read rules metadata (with cache)
pub fn read_rules_info(rules_dir: &Path) -> Option<RulesInfo> {
    use std::fs;

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
                .map(|a| a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
                .unwrap_or_else(Vec::new);
            let total = v["total_rules"].as_u64().unwrap_or(0) as usize;
            let updated_at = v["updated_at"].as_str().map(|s| s.to_string());
            Some(RulesInfo { dir: rules_dir.display().to_string(), sources, total_rules: total, updated_at })
        } else {
            Some(RulesInfo { dir: rules_dir.display().to_string(), sources: Vec::new(), total_rules: 0, updated_at: None })
        }
    } else {
        Some(RulesInfo { dir: rules_dir.display().to_string(), sources: Vec::new(), total_rules: 0, updated_at: None })
    };

    if let Some(ref info) = rules_info {
        let mut cache = RULES_CACHE.write();
        cache.insert(rules_dir.to_path_buf(), info.clone());
    }

    rules_info
}

/// Install OpenGrep via cargo (with helpful guidance)
pub fn install_opengrep() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("🔧 正在安装OpenGrep...");

    let cargo_available = Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if cargo_available {
        let output = Command::new("cargo").args(["install", "opengrep"]).output()?;

        if output.status.success() {
            if !is_opengrep_installed() {
                println!("ℹ️ 已通过 cargo 安装，但未检测到 opengrep 在 PATH。若使用 rustup 默认目录，请添加到 PATH:");
                println!("   export PATH=\"$HOME/.cargo/bin:$PATH\"");
            }
            println!("✅ OpenGrep 安装完成");
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("通过 cargo 安装 OpenGrep 失败: {stderr}\n建议：\n1) 确认已安装 Rust 工具链 (https://rustup.rs) 并已将 ~/.cargo/bin 加入 PATH\n2) 手动执行: cargo install opengrep").into());
        }
    }

    let guide = "未检测到 cargo。请先安装 Rust 工具链，然后使用 cargo 安装 OpenGrep:\n\n1) 安装 Rust（推荐 rustup）: https://rustup.rs\n2) 安装 OpenGrep: cargo install opengrep\n3) 将 cargo 的 bin 目录加入 PATH: export PATH=\"$HOME/.cargo/bin:$PATH\"";
    Err(guide.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitai_core::config::Config;
    use std::fs;

    fn sample_json() -> String {
        serde_json::json!({
            "results": [
                {
                    "path": "src/main.rs",
                    "start": { "line": 10, "col": 5 },
                    "check_id": "OG001",
                    "severity": "ERROR",
                    "lines": "let x = 42;",
                    "extra": { "message": "Hardcoded value", "fix": "Use const" }
                },
                {
                    "path": "lib/mod.rs",
                    "start": { "line": 1, "col": 1 },
                    "check_id": "OG002",
                    "severity": "WARNING",
                    "lines": "unsafe { /* ... */ }",
                    "extra": { "message": "Unsafe block", "fix": null }
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn test_parse_opengrep_output_valid() {
        let out = sample_json();
        let findings = parse_opengrep_output(&out).expect("should parse valid json");
        assert_eq!(findings.len(), 2);
        let f0 = &findings[0];
        assert_eq!(f0.title, "Hardcoded value");
        assert_eq!(f0.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(f0.line, 10);
        assert_eq!(f0.column, 5);
        assert_eq!(f0.severity, "ERROR");
        assert_eq!(f0.rule_id.as_deref(), Some("OG001"));
        assert_eq!(f0.code_snippet.as_deref(), Some("let x = 42;"));
        assert!(f0.remediation.as_deref() == Some("Use const"));

        let f1 = &findings[1];
        assert_eq!(f1.severity, "WARNING");
    }

    #[test]
    fn test_parse_opengrep_output_empty_and_noise() {
        let findings = parse_opengrep_output("").expect("empty is ok");
        assert!(findings.is_empty());

        let findings2 = parse_opengrep_output("INFO: scanning... no json here").expect("no json -> empty");
        assert!(findings2.is_empty());

        let noisy = format!("some logs... {}", sample_json());
        let findings3 = parse_opengrep_output(&noisy).expect("noisy json should parse");
        assert_eq!(findings3.len(), 2);
    }

    #[test]
    fn test_read_rules_info_cache_behavior() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let rules_dir = tmp.path().join("rules");
        fs::create_dir_all(&rules_dir).unwrap();

        let meta_path = rules_dir.join(".rules.meta");
        fs::write(
            &meta_path,
            serde_json::json!({
                "sources": ["https://example.com/rules.tar.gz"],
                "total_rules": 5,
                "updated_at": "2025-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .unwrap();

        let info1 = read_rules_info(&rules_dir).expect("some rules info");
        assert_eq!(info1.total_rules, 5);
        assert_eq!(info1.sources.len(), 1);

        // mutate file to see cache keeps old value
        fs::write(
            &meta_path,
            serde_json::json!({
                "sources": ["https://example.com/other.tar.gz"],
                "total_rules": 9,
                "updated_at": "2026-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .unwrap();

        let info2 = read_rules_info(&rules_dir).expect("still cached");
        assert_eq!(info2.total_rules, 5, "cache should return first result");
        assert_eq!(info2.sources.len(), 1);
        assert_eq!(info1.dir, info2.dir);
    }

    #[test]
    fn test_run_opengrep_scan_missing_path_errors_early() {
        let mut cfg = Config::default();
        cfg.scan.timeout = 1;
        cfg.scan.jobs = 0;

        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("does_not_exist");
        let res = run_opengrep_scan(&cfg, &missing, None, None, false);
        assert!(res.is_err());
        let msg = format!("{}", res.err().unwrap());
        assert!(msg.contains("扫描路径不存在"));
    }
}

