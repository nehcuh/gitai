use std::process::Command;
use std::path::Path;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use log::debug;

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
    pub severity: Severity,
    pub rule_id: String,
    pub code_snippet: Option<String>,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 运行OpenGrep扫描
pub fn run_opengrep_scan(config: &Config, path: &Path, lang: Option<&str>, timeout_override: Option<u64>, include_version: bool) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let start_time = std::time::Instant::now();
    
    // 构建命令（不要把可执行名放入 args）
    let mut args = vec![
        "--json".to_string(),
        "--quiet".to_string(),
        format!("--timeout={}", timeout_override.unwrap_or(config.scan.timeout)),
    ];
    if config.scan.jobs > 0 {
        args.push(format!("--jobs={}", config.scan.jobs));
    }
    
    // 规则目录
    let rules_dir = config.scan.rules_dir.clone().map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".cache").join("gitai").join("rules")
        });
    let mut rules_info: Option<RulesInfo> = None;
    if rules_dir.exists() {
        if let Ok(mut iter) = std::fs::read_dir(&rules_dir) {
            if iter.next().is_some() {
                // 若指定了语言，直接使用对应子目录；否则再尝试自动选择
                let rules_root = if let Some(l) = lang { rules_dir.join(l) } else {
                    select_language_rules(&rules_dir, path).unwrap_or_else(|| pick_rules_path(&rules_dir))
                };
                args.push(format!("--config={}", rules_root.display()));
                // 读取元信息
                rules_info = read_rules_info(&rules_root).or_else(|| read_rules_info(&rules_dir));
            }
        }
    }
    
    // 执行命令
    let output = Command::new("opengrep")
        .args(&args)
        .arg(path)
        .output()?;
    
    let execution_time = start_time.elapsed().as_secs_f64();
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(ScanResult {
            tool: "opengrep".to_string(),
            version: if include_version { get_opengrep_version()? } else { "unknown".to_string() },
            execution_time,
            findings: vec![],
            error: Some(stderr.to_string()),
            rules_info,
        });
    }
    
    // 解析结果
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("📄 OpenGrep stdout: {}", stdout);
    
    let findings = match parse_opengrep_output(&stdout) {
        Ok(f) => f,
        Err(e) => {
            debug!("❌ JSON 解析失败: {}", e);
            return Ok(ScanResult {
                tool: "opengrep".to_string(),
                version: if include_version { get_opengrep_version().unwrap_or_else(|_| "unknown".to_string()) } else { "unknown".to_string() },
                execution_time,
                findings: vec![],
                error: Some(format!("JSON 解析失败: {}", e)),
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

/// 获取OpenGrep版本
fn get_opengrep_version() -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let output = Command::new("opengrep")
        .arg("--version")
        .output()?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// 解析OpenGrep输出（整块 JSON，遍历 results 数组）
fn parse_opengrep_output(output: &str) -> Result<Vec<Finding>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    debug!("🔍 解析OpenGrep输出: {}", output);
    
    if output.trim().is_empty() {
        debug!("⚠️ OpenGrep 输出为空");
        return Ok(Vec::new());
    }
    
    let v: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| format!("JSON 解析失败: {}, 输入: {}", e, output))?;
    
    debug!("📄 JSON 结构: {:?}", v);
    
    let mut findings = Vec::new();
    if let Some(results) = v.get("results").and_then(|r| r.as_array()) {
        debug!("📋 找到 {} 个结果", results.len());
        for (i, item) in results.iter().enumerate() {
            match create_finding_from_result(item) {
                Ok(finding) => {
                    findings.push(finding);
                }
                Err(e) => {
                    debug!("❌ 解析第 {} 个结果失败: {}", i, e);
                }
            }
        }
    } else {
        debug!("⚠️ 未找到 results 数组");
        // 检查是否有错误信息
        if let Some(errors) = v.get("errors").and_then(|e| e.as_array()) {
            debug!("❌ OpenGrep 报告错误: {:?}", errors);
        }
        // 检查扫描的路径
        if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
            debug!("📂 扫描的路径: {:?}", paths);
        }
    }
    
    Ok(findings)
}

/// 从 results[i] 构建 Finding
fn create_finding_from_result(item: &serde_json::Value) -> Result<Finding, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let title = item["extra"]["message"].as_str().unwrap_or("Unknown issue").to_string();
    let file_path = item["path"].as_str().unwrap_or("").to_string();
    let line = item["start"]["line"].as_u64().unwrap_or(0) as usize;
    let rule_id = item["check_id"].as_str().unwrap_or("unknown").to_string();

    let severity_str = item["severity"].as_str().unwrap_or("WARNING");
    let severity = match severity_str {
        "ERROR" => Severity::Error,
        "WARNING" => Severity::Warning,
        _ => Severity::Info,
    };

    let code_snippet = item["lines"].as_str().map(|s| s.to_string());

    Ok(Finding {
        title,
        file_path: std::path::PathBuf::from(file_path),
        line,
        severity,
        rule_id,
        code_snippet,
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
    let meta_path = rules_dir.join(".rules.meta");
    if let Ok(content) = fs::read_to_string(&meta_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let sources = v["sources"].as_array()
                .map(|a| a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect())
                .unwrap_or_else(|| Vec::new());
            let total = v["total_rules"].as_u64().unwrap_or(0) as usize;
            let updated_at = v["updated_at"].as_str().map(|s| s.to_string());
            return Some(RulesInfo {
                dir: rules_dir.display().to_string(),
                sources,
                total_rules: total,
                updated_at,
            });
        }
    }
    // 回退：仅提供目录
    Some(RulesInfo { dir: rules_dir.display().to_string(), sources: Vec::new(), total_rules: 0, updated_at: None })
}

/// 根据扫描目录中的主要语言，优先选择对应的规则子目录
fn select_language_rules(rules_dir: &std::path::Path, scan_path: &std::path::Path) -> Option<std::path::PathBuf> {
    // 统计常见语言扩展出现次数（最多查看前 500 个文件）
    let mut counts: std::collections::HashMap<&'static str, usize> = Default::default();
    let mut seen = 0usize;
    for entry in WalkDir::new(scan_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) {
                let key = match ext.as_str() {
                    "java" => "java",
                    "py" => "python",
                    "js" => "javascript",
                    "ts" => "typescript",
                    "go" => "go",
                    "rs" => "rust",
                    "rb" => "ruby",
                    "php" => "php",
                    "kt" => "kotlin",
                    "scala" => "scala",
                    "swift" => "swift",
                    "c" | "h" => "c",
                    "cpp" | "cxx" | "hpp" => "c",
                    _ => "",
                };
                if !key.is_empty() {
                    *counts.entry(key).or_insert(0) += 1;
                }
            }
            seen += 1;
            if seen >= 500 { break; }
        }
    }
    if counts.is_empty() { return None; }
    let (lang, _) = counts.into_iter().max_by_key(|(_, c)| *c).unwrap();
    let candidate = rules_dir.join(lang);
    if candidate.exists() { Some(candidate) } else { None }
}

fn pick_rules_path(dir: &std::path::Path) -> std::path::PathBuf {
    use std::fs;
    // 如果根目录有文件，直接使用根目录
    if let Ok(mut entries) = fs::read_dir(dir) {
        for e in entries.by_ref().flatten() {
            if e.file_type().map(|t| t.is_file()).unwrap_or(false) {
                return dir.to_path_buf();
            }
        }
    }
    // 否则如果只有一个子目录，返回该子目录
    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                subdirs.push(e.path());
            }
        }
    }
    if subdirs.len() == 1 { return subdirs.remove(0); }
    // 回退到根目录
    dir.to_path_buf()
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
                "通过 cargo 安装 OpenGrep 失败: {}\n建议：\n1) 确认已安装 Rust 工具链 (https://rustup.rs) 并已将 ~/.cargo/bin 加入 PATH\n2) 手动执行: cargo install opengrep",
                stderr
            ).into());
        }
    }

    // cargo 不可用：给出明确的安装指引
    let guide = "未检测到 cargo。请先安装 Rust 工具链，然后使用 cargo 安装 OpenGrep:\n\n1) 安装 Rust（推荐 rustup）: https://rustup.rs\n2) 安装 OpenGrep: cargo install opengrep\n3) 将 cargo 的 bin 目录加入 PATH: export PATH=\"$HOME/.cargo/bin:$PATH\"";
    Err(guide.into())
}
