//! Scan 命令处理器
//!
//! 处理安全扫描相关的命令，包括代码安全检查、敏感信息检测、漏洞扫描等

use crate::args::Command;
use gitai_core::context::OperationContext;
use gitai_core::config::Config;
use std::path::Path;
use std::time::Instant;
use regex::Regex;

/// 安全扫描结果
#[derive(Debug, serde::Serialize)]
struct ScanResult {
    /// 使用的工具
    tool: String,
    /// 执行时间（秒）
    execution_time: f64,
    /// 扫描的文件数量
    files_scanned: usize,
    /// 发现的问题
    findings: Vec<Finding>,
    /// 扫描统计
    stats: ScanStats,
}

/// 安全发现
#[derive(Debug, serde::Serialize)]
struct Finding {
    /// 问题标题
    title: String,
    /// 问题类型
    severity: Severity,
    /// 规则ID
    rule_id: String,
    /// 文件路径
    file_path: std::path::PathBuf,
    /// 行号
    line: usize,
    /// 列号
    column: Option<usize>,
    /// 问题描述
    message: String,
    /// 修复建议
    recommendation: Option<String>,
}

/// 严重程度
#[derive(Debug, serde::Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
enum Severity {
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重错误级别
    Critical,
}

/// 扫描统计信息
#[derive(Debug, serde::Serialize)]
struct ScanStats {
    /// 总文件数
    total_files: usize,
    /// 按严重程度统计
    by_severity: std::collections::HashMap<String, usize>,
    /// 按类型统计
    by_type: std::collections::HashMap<String, usize>,
}

/// 安全扫描器
struct SecurityScanner {
    /// 敏感信息检测规则
    sensitive_patterns: Vec<SensitivePattern>,
    /// 代码安全检查规则
    security_patterns: Vec<SecurityPattern>,
}

/// 敏感信息检测模式
struct SensitivePattern {
    name: String,
    regex: Regex,
    severity: Severity,
    description: String,
    recommendation: String,
}

/// 代码安全检查模式
struct SecurityPattern {
    name: String,
    regex: Regex,
    severity: Severity,
    description: String,
    recommendation: String,
    file_types: Vec<String>,
}

impl SecurityScanner {
    /// 创建新的安全扫描器
    pub fn new() -> Self {
        let mut scanner = Self {
            sensitive_patterns: Vec::new(),
            security_patterns: Vec::new(),
        };

        scanner.init_sensitive_patterns();
        scanner.init_security_patterns();
        scanner
    }

    /// 初始化敏感信息检测规则
    fn init_sensitive_patterns(&mut self) {
        // API密钥检测
        self.sensitive_patterns.push(SensitivePattern {
            name: "api_key".to_string(),
            regex: Regex::new("(?i)(api[_-]?key|apikey)").unwrap(),
            severity: Severity::Critical,
            description: "检测到API密钥".to_string(),
            recommendation: "请使用环境变量或安全的密钥管理系统".to_string(),
        });

        // 数据库连接字符串检测
        self.sensitive_patterns.push(SensitivePattern {
            name: "database_url".to_string(),
            regex: Regex::new("(?i)(database[_-]?url|db[_-]?url|postgresql://)").unwrap(),
            severity: Severity::Critical,
            description: "检测到数据库连接字符串".to_string(),
            recommendation: "请使用环境变量存储数据库连接信息".to_string(),
        });

        // 密码检测
        self.sensitive_patterns.push(SensitivePattern {
            name: "password".to_string(),
            regex: Regex::new("(?i)(password|passwd|pwd)").unwrap(),
            severity: Severity::Critical,
            description: "检测到可能的密码".to_string(),
            recommendation: "请使用安全的认证机制，不要硬编码密码".to_string(),
        });

        // JWT密钥检测
        self.sensitive_patterns.push(SensitivePattern {
            name: "jwt_secret".to_string(),
            regex: Regex::new("(?i)(jwt[_-]?secret|secret[_-]?key)").unwrap(),
            severity: Severity::Critical,
            description: "检测到JWT密钥".to_string(),
            recommendation: "请使用环境变量存储JWT密钥".to_string(),
        });
    }

    /// 初始化代码安全检查规则
    fn init_security_patterns(&mut self) {
        // SQL注入检测
        self.security_patterns.push(SecurityPattern {
            name: "sql_injection".to_string(),
            regex: Regex::new("(?i)(execute|query)").unwrap(),
            severity: Severity::Error,
            description: "可能的SQL注入漏洞".to_string(),
            recommendation: "使用参数化查询或ORM来防止SQL注入".to_string(),
            file_types: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string()],
        });

        // 硬编码IP地址检测
        self.security_patterns.push(SecurityPattern {
            name: "hardcoded_ip".to_string(),
            regex: Regex::new("\\b\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\b").unwrap(),
            severity: Severity::Warning,
            description: "检测到硬编码IP地址".to_string(),
            recommendation: "使用配置文件或环境变量管理网络地址".to_string(),
            file_types: vec!["rs".to_string(), "js".to_string(), "ts".to_string(), "py".to_string(), "yaml".to_string(), "yml".to_string()],
        });

        // 不安全的随机数生成检测
        self.security_patterns.push(SecurityPattern {
            name: "weak_random".to_string(),
            regex: Regex::new("(?i)(math\\.random|rand)").unwrap(),
            severity: Severity::Warning,
            description: "使用了不安全的随机数生成器".to_string(),
            recommendation: "使用密码学安全的随机数生成器".to_string(),
            file_types: vec!["js".to_string(), "ts".to_string(), "rs".to_string(), "py".to_string()],
        });
    }

    /// 扫描目录
    pub async fn scan_directory(
        &self,
        path: &Path,
        lang: Option<&str>,
        _context: Option<OperationContext>,
    ) -> std::result::Result<ScanResult, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let start_time = Instant::now();
        let mut findings = Vec::new();
        let mut files_scanned = 0;

        println!("🔍 开始扫描目录: {}", path.display());

        // 使用栈来避免递归
        let mut dirs_to_scan = vec![path.to_path_buf()];

        while let Some(current_dir) = dirs_to_scan.pop() {
            // 遍历目录中的文件
            let mut entries = tokio::fs::read_dir(&current_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    // 跳过隐藏目录和常见的忽略目录
                    if let Some(file_name) = entry_path.file_name() {
                        let name_str = file_name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "node_modules" || name_str == "target" {
                            continue;
                        }
                    }

                    // 将子目录添加到扫描栈
                    dirs_to_scan.push(entry_path);
                } else if entry_path.is_file() {
                    // 检查文件类型过滤
                    if let Some(file_name) = entry_path.file_name() {
                        let name_str = file_name.to_string_lossy();

                        // 跳过隐藏文件和二进制文件
                        if name_str.starts_with('.') || self.is_binary_file(&name_str) {
                            continue;
                        }

                        // 语言过滤
                        if let Some(lang_filter) = lang {
                            if !self.matches_language(&name_str, lang_filter) {
                                continue;
                            }
                        }
                    }

                    // 扫描文件
                    match self.scan_file(&entry_path).await {
                        Ok(mut file_findings) => {
                            findings.append(&mut file_findings);
                            files_scanned += 1;
                        }
                        Err(_) => {
                            // 忽略无法读取的文件
                            continue;
                        }
                    }
                }
            }
        }

        let execution_time = start_time.elapsed().as_secs_f64();
        let stats = self.generate_stats(&findings, files_scanned);

        Ok(ScanResult {
            tool: "gitai-security-scanner".to_string(),
            execution_time,
            files_scanned,
            findings,
            stats,
        })
    }

    /// 扫描单个文件
    async fn scan_file(&self, file_path: &Path) -> std::result::Result<Vec<Finding>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut findings = Vec::new();

        let file_content = tokio::fs::read_to_string(file_path).await?;
        let file_name = file_path.file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy();

        // 检查敏感信息模式
        for pattern in &self.sensitive_patterns {
            for (line_num, line) in file_content.lines().enumerate() {
                if let Some(match_) = pattern.regex.find(line) {
                    findings.push(Finding {
                        title: pattern.name.clone(),
                        severity: pattern.severity.clone(),
                        rule_id: format!("sensitive-{}", pattern.name),
                        file_path: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: Some(match_.start()),
                        message: pattern.description.clone(),
                        recommendation: Some(pattern.recommendation.clone()),
                    });
                }
            }
        }

        // 检查代码安全模式（根据文件类型）
        for pattern in &self.security_patterns {
            if pattern.file_types.is_empty() || pattern.file_types.iter().any(|ext| file_name.ends_with(&format!(".{}", ext))) {
                for (line_num, line) in file_content.lines().enumerate() {
                    if let Some(match_) = pattern.regex.find(line) {
                        findings.push(Finding {
                            title: pattern.name.clone(),
                            severity: pattern.severity.clone(),
                            rule_id: format!("security-{}", pattern.name),
                            file_path: file_path.to_path_buf(),
                            line: line_num + 1,
                            column: Some(match_.start()),
                            message: pattern.description.clone(),
                            recommendation: Some(pattern.recommendation.clone()),
                        });
                    }
                }
            }
        }

        Ok(findings)
    }

    /// 判断是否为二进制文件
    fn is_binary_file(&self, file_name: &str) -> bool {
        let binary_extensions = vec![
            "exe", "dll", "so", "dylib", "bin", "img", "iso",
            "zip", "tar", "gz", "rar", "7z", "pdf", "doc", "docx",
            "png", "jpg", "jpeg", "gif", "bmp", "mp3", "mp4"
        ];

        binary_extensions.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)))
    }

    /// 检查文件是否匹配指定语言
    fn matches_language(&self, file_name: &str, lang: &str) -> bool {
        let language_extensions = std::collections::HashMap::from([
            ("rust", vec!["rs"]),
            ("javascript", vec!["js", "jsx"]),
            ("typescript", vec!["ts", "tsx"]),
            ("python", vec!["py"]),
            ("go", vec!["go"]),
            ("java", vec!["java"]),
            ("cpp", vec!["cpp", "cc", "cxx", "c++"]),
            ("c", vec!["c", "h"]),
            ("yaml", vec!["yaml", "yml"]),
            ("json", vec!["json"]),
        ]);

        if let Some(extensions) = language_extensions.get(lang) {
            extensions.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)))
        } else {
            false
        }
    }

    /// 生成统计信息
    fn generate_stats(&self, findings: &[Finding], files_scanned: usize) -> ScanStats {
        let mut by_severity = std::collections::HashMap::new();
        let mut by_type = std::collections::HashMap::new();

        for finding in findings {
            let severity_key = format!("{:?}", finding.severity);
            *by_severity.entry(severity_key).or_insert(0) += 1;
            *by_type.entry(finding.title.clone()).or_insert(0) += 1;
        }

        ScanStats {
            total_files: files_scanned,
            by_severity,
            by_type,
        }
    }
}

type HandlerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// 处理 scan 命令
pub async fn handle_command(
    _config: &Config,
    command: &Command,
) -> HandlerResult<()> {
    match command {
        Command::Scan {
            path,
            tool: _,
            full: _,
            remote: _,
            update_rules: _,
            format,
            output,
            translate: _,
            auto_install: _,
            lang,
            no_history: _,
            timeout: _,
            benchmark: _,
        } => {
            let show_progress = format != "json";

            if show_progress {
                println!("🚀 GitAI 安全扫描器 v1.0");
                println!("🔍 扫描目标: {}", path.display());
                if let Some(ref lang) = lang {
                    println!("🌐 语言过滤: {}", lang);
                }
                println!();
            }

            // 创建扫描器
            let scanner = SecurityScanner::new();

            // 创建操作上下文
            let context = OperationContext::new();

            // 执行扫描
            let result = scanner
                .scan_directory(path, lang.as_deref(), Some(context))
                .await?;

            // 输出结果
            if format == "json" {
                let json = serde_json::to_string_pretty(&result)?;
                if let Some(output_path) = output {
                    tokio::fs::write(output_path, json).await?;
                    println!("📄 结果已保存到: {}", output_path.display());
                } else {
                    println!("{}", json);
                }
            } else {
                display_console_output(&result, show_progress)?;
            }

            Ok(())
        }
        Command::ScanHistory { limit, format: _ } => {
            // TODO: 实现扫描历史逻辑
            println!("📋 扫描历史 (最近{}次):", limit);
            println!("💡 扫描历史功能正在开发中...");
            Ok(())
        }
        _ => Err("Invalid command for scan handler".into()),
    }
}

/// 显示控制台输出
fn display_console_output(result: &ScanResult, show_progress: bool) -> HandlerResult<()> {
    if show_progress {
        println!("📊 扫描完成!");
        println!("⏱️  执行时间: {:.2}s", result.execution_time);
        println!("📁 扫描文件: {} 个", result.files_scanned);
        println!();

        // 显示统计信息
        if !result.stats.by_severity.is_empty() {
            println!("📈 严重程度分布:");
            let mut severities: Vec<_> = result.stats.by_severity.iter().collect();
            severities.sort_by(|a, b| {
                let severity_order = ["Critical", "Error", "Warning", "Info"];
                let a_index = severity_order.iter().position(|&s| s == a.0).unwrap_or(999);
                let b_index = severity_order.iter().position(|&s| s == b.0).unwrap_or(999);
                a_index.cmp(&b_index)
            });

            for (severity, count) in severities {
                let icon = match severity.as_str() {
                    "Critical" => "🚨",
                    "Error" => "❌",
                    "Warning" => "⚠️",
                    "Info" => "ℹ️",
                    _ => "📝",
                };
                println!("  {} {}: {} 个", icon, severity, count);
            }
            println!();
        }

        if result.findings.is_empty() {
            println!("✅ 恭喜! 未发现安全问题");
        } else {
            println!("🔍 发现 {} 个安全问题:", result.findings.len());
            println!();

            // 按严重程度分组显示
            let mut grouped_findings = std::collections::HashMap::new();
            for finding in &result.findings {
                grouped_findings
                    .entry(&finding.severity)
                    .or_insert_with(Vec::new)
                    .push(finding);
            }

            let severity_order = [Severity::Critical, Severity::Error, Severity::Warning, Severity::Info];
            for severity in &severity_order {
                if let Some(findings) = grouped_findings.get(severity) {
                    let severity_name = match severity {
                        Severity::Critical => "严重",
                        Severity::Error => "错误",
                        Severity::Warning => "警告",
                        Severity::Info => "信息",
                    };

                    let icon = match severity {
                        Severity::Critical => "🚨",
                        Severity::Error => "❌",
                        Severity::Warning => "⚠️",
                        Severity::Info => "ℹ️",
                    };

                    println!("{} {} ({} 个)", icon, severity_name, findings.len());

                    for (i, finding) in findings.iter().enumerate() {
                        println!("  {}. {} [{}]", i + 1, finding.title, finding.rule_id);
                        println!("     📍 {}:{}", finding.file_path.display(), finding.line);
                        println!("     📝 {}", finding.message);
                        if let Some(ref recommendation) = finding.recommendation {
                            println!("     💡 {}", recommendation);
                        }
                        println!();
                    }

                    if findings.len() >= 3 {
                        println!();
                    }
                }
            }

            // 显示类型统计
            if !result.stats.by_type.is_empty() {
                println!("📊 问题类型分布:");
                let mut types: Vec<_> = result.stats.by_type.iter().collect();
                types.sort_by(|a, b| b.1.cmp(a.1));

                for (problem_type, count) in types.iter().take(5) {
                    println!("  • {}: {} 个", problem_type, count);
                }
                if types.len() > 5 {
                    println!("  • ... 还有 {} 种其他类型", types.len() - 5);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_security_scanner_creation() {
        let scanner = SecurityScanner::new();
        assert!(!scanner.sensitive_patterns.is_empty());
        assert!(!scanner.security_patterns.is_empty());
    }

    #[test]
    fn test_binary_file_detection() {
        let scanner = SecurityScanner::new();

        assert!(scanner.is_binary_file("test.exe"));
        assert!(scanner.is_binary_file("document.pdf"));
        assert!(scanner.is_binary_file("image.png"));
        assert!(!scanner.is_binary_file("source.rs"));
        assert!(!scanner.is_binary_file("config.yaml"));
    }

    #[test]
    fn test_language_matching() {
        let scanner = SecurityScanner::new();

        assert!(scanner.matches_language("main.rs", "rust"));
        assert!(scanner.matches_language("script.js", "javascript"));
        assert!(scanner.matches_language("app.ts", "typescript"));
        assert!(scanner.matches_language("module.py", "python"));
        assert!(!scanner.matches_language("main.cpp", "rust"));
        assert!(!scanner.matches_language("index.html", "javascript"));
    }

    #[test]
    fn test_scan_stats_generation() {
        let scanner = SecurityScanner::new();

        let findings = vec![
            Finding {
                title: "api_key".to_string(),
                severity: Severity::Critical,
                rule_id: "sensitive-api_key".to_string(),
                file_path: std::path::PathBuf::from("test.rs"),
                line: 1,
                column: None,
                message: "检测到API密钥".to_string(),
                recommendation: Some("请使用环境变量".to_string()),
            },
            Finding {
                title: "hardcoded_ip".to_string(),
                severity: Severity::Warning,
                rule_id: "security-hardcoded_ip".to_string(),
                file_path: std::path::PathBuf::from("config.yaml"),
                line: 5,
                column: None,
                message: "检测到硬编码IP地址".to_string(),
                recommendation: Some("使用配置文件".to_string()),
            },
        ];

        let stats = scanner.generate_stats(&findings, 2);

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.by_severity.get("Critical"), Some(&1));
        assert_eq!(stats.by_severity.get("Warning"), Some(&1));
        assert_eq!(stats.by_type.get("api_key"), Some(&1));
        assert_eq!(stats.by_type.get("hardcoded_ip"), Some(&1));
    }

    #[tokio::test]
    async fn test_scan_file_with_sensitive_info() {
        let scanner = SecurityScanner::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        // 写入包含敏感信息的测试代码
        tokio::fs::write(&file_path, r#"
use std::env;

const API_KEY: &str = "sk-1234567890abcdef1234567890abcdef1234567890";
const DB_URL: &str = "postgresql://user:password@localhost:5432/db";
const PASSWORD: &str = "secretpassword123";
const JWT_SECRET: &str = "my-super-secret-jwt-key-1234567890abcdef";

fn main() {
    println!("Hello, world!");
}
"#).await.unwrap();

        let findings = scanner.scan_file(&file_path).await.unwrap();

        // 应该检测到API密钥和数据库连接字符串
        assert!(!findings.is_empty());

        // 验证检测到的敏感信息
        let api_key_finding: Vec<_> = findings.iter()
            .filter(|f| f.title == "api_key")
            .collect();
        assert!(!api_key_finding.is_empty());

        let db_url_finding: Vec<_> = findings.iter()
            .filter(|f| f.title == "database_url")
            .collect();
        assert!(!db_url_finding.is_empty());
    }

    #[tokio::test]
    async fn test_scan_file_with_security_issues() {
        let scanner = SecurityScanner::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("script.js");

        // 写入包含安全问题的JavaScript代码
        tokio::fs::write(&file_path, r#"
const server = "192.168.1.100";
const random = Math.random();

function queryUser(id) {
    const sql = "SELECT * FROM users WHERE id = " + id;
    return execute(sql);
}
"#).await.unwrap();

        let findings = scanner.scan_file(&file_path).await.unwrap();

        // 应该检测到硬编码IP地址和不安全的随机数生成
        assert!(!findings.is_empty());

        // 验证检测到的安全问题
        let ip_finding: Vec<_> = findings.iter()
            .filter(|f| f.title == "hardcoded_ip")
            .collect();
        assert!(!ip_finding.is_empty());

        let random_finding: Vec<_> = findings.iter()
            .filter(|f| f.title == "weak_random")
            .collect();
        assert!(!random_finding.is_empty());
    }

    #[test]
    fn test_finding_creation() {
        let finding = Finding {
            title: "test_finding".to_string(),
            severity: Severity::Error,
            rule_id: "test-001".to_string(),
            file_path: std::path::PathBuf::from("/test/file.rs"),
            line: 42,
            column: Some(10),
            message: "Test finding message".to_string(),
            recommendation: Some("Test recommendation".to_string()),
        };

        assert_eq!(finding.title, "test_finding");
        assert_eq!(finding.severity, Severity::Error);
        assert_eq!(finding.line, 42);
        assert_eq!(finding.column, Some(10));
        assert_eq!(finding.message, "Test finding message");
        assert_eq!(finding.recommendation, Some("Test recommendation".to_string()));
    }

    #[test]
    fn test_scan_result_creation() {
        let mut by_severity = HashMap::new();
        by_severity.insert("Critical".to_string(), 1);
        by_severity.insert("Warning".to_string(), 2);

        let mut by_type = HashMap::new();
        by_type.insert("api_key".to_string(), 1);
        by_type.insert("hardcoded_ip".to_string(), 2);

        let stats = ScanStats {
            total_files: 5,
            by_severity,
            by_type,
        };

        let result = ScanResult {
            tool: "gitai-security-scanner".to_string(),
            execution_time: 1.23,
            files_scanned: 5,
            findings: vec![],
            stats,
        };

        assert_eq!(result.tool, "gitai-security-scanner");
        assert_eq!(result.execution_time, 1.23);
        assert_eq!(result.files_scanned, 5);
        assert!(result.findings.is_empty());
        assert_eq!(result.stats.total_files, 5);
    }
}
