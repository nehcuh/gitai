use crate::types::scan::types::*;
use crate::handlers::scan::results::*;
use crate::config::AppConfig;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::sync::mpsc;
use anyhow::{Result, anyhow};
use serde_json;

/// 扫描工具特征
#[async_trait::async_trait]
pub trait ScanTool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;
    
    /// 检查工具是否可用
    async fn is_available(&self) -> bool;
    
    /// 获取工具版本
    async fn get_version(&self) -> Result<String>;
    
    /// 安装工具
    async fn install(&self) -> Result<()>;
    
    /// 运行扫描
    async fn scan(&self, request: &ScanRequest, progress_sender: mpsc::Sender<ScanProgress>) -> Result<ToolResult>;
    
    /// 解析结果
    fn parse_results(&self, raw_output: &str) -> Result<Vec<Finding>>;
    
    /// 映射规则类型
    fn map_rule_type(&self, metadata: &serde_json::Value) -> RuleType;
}

/// Semgrep扫描器
pub struct SemgrepScanner {
    config: SemgrepConfig,
}

impl SemgrepScanner {
    pub fn new(config: SemgrepConfig) -> Self {
        Self { config }
    }
    
    fn build_command(&self, scan_path: &Path, language_filter: Option<&str>) -> Vec<String> {
        let mut args = vec![
            "semgrep".to_string(),
            "--json".to_string(),
            "--quiet".to_string(),
            format!("--config={}", self.config.rules_path.display()),
            format!("--timeout={}", self.config.timeout),
            format!("--jobs={}", self.config.concurrency),
        ];
        
        // 添加排除模式
        for pattern in &self.config.exclude_patterns {
            args.push(format!("--exclude={}", pattern));
        }
        
        // 添加语言过滤
        if let Some(lang) = language_filter {
            args.push(format!("--lang={}", lang));
        }
        
        // 添加扫描路径
        args.push(scan_path.display().to_string());
        
        args
    }
}

#[async_trait::async_trait]
impl ScanTool for SemgrepScanner {
    fn name(&self) -> &str {
        "semgrep"
    }
    
    async fn is_available(&self) -> bool {
        match self.get_version().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    async fn get_version(&self) -> Result<String> {
        let output = Command::new("semgrep")
            .arg("--version")
            .output()
            .map_err(|e| anyhow!("Failed to run semgrep --version: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("Semgrep --version failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let version = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        Ok(version)
    }
    
    async fn install(&self) -> Result<()> {
        // 检查是否已安装
        if self.is_available().await {
            return Ok(());
        }
        
        // 尝试多种安装方法
        let mut last_error = None;
        
        // 方法1: 使用pip3安装
        if let Err(e) = self.install_with_pip3().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 方法2: 使用pip安装
        if let Err(e) = self.install_with_pip().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 方法3: 使用curl安装脚本
        if let Err(e) = self.install_with_curl().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 所有方法都失败了
        Err(anyhow!("Failed to install Semgrep using all methods. Last error: {}", 
            last_error.unwrap_or_else(|| anyhow!("Unknown error"))))
    }
    
    async fn scan(&self, request: &ScanRequest, mut progress_sender: mpsc::Sender<ScanProgress>) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // 发送进度更新
        progress_sender.send(ScanProgress {
            stage: ScanStage::RunningScan,
            progress: 0.0,
            message: "Starting Semgrep scan...".to_string(),
            tool: Some("semgrep".to_string()),
        }).await?;
        
        let args = self.build_command(&request.config.path, request.language_filter.as_deref());
        
        let output = Command::new(&args[0])
            .args(&args[1..])
            .output()
            .map_err(|e| anyhow!("Failed to run semgrep: {}", e))?;
        
        let execution_time = start_time.elapsed().as_secs_f64();
        
        let raw_output = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let parse_status = if output.status.success() {
            ParseStatus::Success
        } else {
            ParseStatus::Failed
        };
        
        let error = if !output.status.success() {
            Some(format!("Semgrep failed: {}", stderr))
        } else {
            None
        };
        
        // 获取版本信息
        let tool_version = self.get_version().await.unwrap_or_else(|_| "unknown".to_string());
        
        Ok(ToolResult {
            tool_name: "semgrep".to_string(),
            tool_version,
            execution_time,
            raw_output,
            parse_status,
            error,
        })
    }
    
    fn parse_results(&self, raw_output: &str) -> Result<Vec<Finding>> {
        let semgrep_results: Vec<SemgrepResult> = serde_json::from_str(raw_output)
            .map_err(|e| anyhow!("Failed to parse semgrep JSON: {}", e))?;
        
        let mut findings = Vec::new();
        
        for (index, result) in semgrep_results.into_iter().enumerate() {
            let finding = Finding {
                id: format!("semgrep-{}", index),
                title: result.message.clone(),
                description: result.message,
                severity: result.extra.severity.parse().unwrap_or(Severity::Warning),
                rule_type: self.map_rule_type(&result.extra.metadata),
                rule_id: result.check_id,
                source_tool: "semgrep".to_string(),
                file_path: PathBuf::from(&result.path),
                location: Location {
                    start_line: result.start.line,
                    end_line: result.end.line,
                    start_column: Some(result.start.col),
                    end_column: Some(result.end.col),
                },
                code_snippet: Some(CodeSnippet {
                    content: result.extra.lines.join("\n"),
                    highlight_range: Some(Location {
                        start_line: result.start.line,
                        end_line: result.end.line,
                        start_column: Some(result.start.col),
                        end_column: Some(result.end.col),
                    }),
                    context_lines: 3,
                }),
                fix_suggestions: vec![],
                tags: result.extra.metadata.get("tags")
                    .and_then(|tags| tags.as_array())
                    .map(|tags| tags.iter()
                        .filter_map(|tag| tag.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_default(),
                metadata: HashMap::new(),
            };
            
            findings.push(finding);
        }
        
        Ok(findings)
    }
    
    fn map_rule_type(&self, metadata: &serde_json::Value) -> RuleType {
        if let Some(category) = metadata.get("category").and_then(|c| c.as_str()) {
            match category {
                "security" => RuleType::Security,
                "correctness" => RuleType::Correctness,
                "performance" => RuleType::Performance,
                "maintainability" => RuleType::Maintainability,
                "best-practice" => RuleType::BestPractice,
                _ => RuleType::Custom(category.to_string()),
            }
        } else {
            RuleType::BestPractice
        }
    }
}

impl SemgrepScanner {
    /// 使用pip3安装
    async fn install_with_pip3(&self) -> Result<()> {
        let output = Command::new("pip3")
            .args(&["install", "--user", "semgrep"])
            .output()
            .map_err(|e| anyhow!("Failed to run pip3 install: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("pip3 install failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    /// 使用pip安装
    async fn install_with_pip(&self) -> Result<()> {
        let output = Command::new("pip")
            .args(&["install", "--user", "semgrep"])
            .output()
            .map_err(|e| anyhow!("Failed to run pip install: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("pip install failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    /// 使用curl安装脚本
    async fn install_with_curl(&self) -> Result<()> {
        let output = Command::new("curl")
            .args(&["-fsSL", "https://semgrep.dev/install"])
            .output()
            .map_err(|e| anyhow!("Failed to download semgrep install script: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("Failed to download install script: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // 执行安装脚本
        let script = String::from_utf8_lossy(&output.stdout);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
            .map_err(|e| anyhow!("Failed to execute install script: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("Install script execution failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn scan(&self, request: &ScanRequest, mut progress_sender: mpsc::Sender<ScanProgress>) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // 发送进度更新
        progress_sender.send(ScanProgress {
            stage: ScanStage::RunningScan,
            progress: 0.0,
            message: "Starting Semgrep scan...".to_string(),
            tool: Some("semgrep".to_string()),
        }).await?;
        
        let args = self.build_command(&request.config.path, request.language_filter.as_deref());
        
        let output = Command::new(&args[0])
            .args(&args[1..])
            .output()
            .map_err(|e| anyhow!("Failed to run semgrep: {}", e))?;
        
        let execution_time = start_time.elapsed().as_secs_f64();
        
        let raw_output = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let parse_status = if output.status.success() {
            ParseStatus::Success
        } else {
            ParseStatus::Failed
        };
        
        let error = if !output.status.success() {
            Some(format!("Semgrep failed: {}", stderr))
        } else {
            None
        };
        
        // 获取版本信息
        let tool_version = self.get_version().await.unwrap_or_else(|_| "unknown".to_string());
        
        Ok(ToolResult {
            tool_name: "semgrep".to_string(),
            tool_version,
            execution_time,
            raw_output,
            parse_status,
            error,
        })
    }
    
    fn parse_results(&self, raw_output: &str) -> Result<Vec<Finding>> {
        let semgrep_results: Vec<SemgrepResult> = serde_json::from_str(raw_output)
            .map_err(|e| anyhow!("Failed to parse semgrep JSON: {}", e))?;
        
        let mut findings = Vec::new();
        
        for (index, result) in semgrep_results.into_iter().enumerate() {
            let finding = Finding {
                id: format!("semgrep-{}", index),
                title: result.message.clone(),
                description: result.message,
                severity: result.extra.severity.parse().unwrap_or(Severity::Warning),
                rule_type: self.map_rule_type(&result.extra.metadata),
                rule_id: result.check_id,
                source_tool: "semgrep".to_string(),
                file_path: PathBuf::from(&result.path),
                location: Location {
                    start_line: result.start.line,
                    end_line: result.end.line,
                    start_column: Some(result.start.col),
                    end_column: Some(result.end.col),
                },
                code_snippet: Some(CodeSnippet {
                    content: result.extra.lines.join("\n"),
                    highlight_range: Some(Location {
                        start_line: result.start.line,
                        end_line: result.end.line,
                        start_column: Some(result.start.col),
                        end_column: Some(result.end.col),
                    }),
                    context_lines: 3,
                }),
                fix_suggestions: vec![],
                tags: result.extra.metadata.get("tags")
                    .and_then(|tags| tags.as_array())
                    .map(|tags| tags.iter()
                        .filter_map(|tag| tag.as_str())
                        .map(|s| s.to_string())
                        .collect())
                    .unwrap_or_default(),
                metadata: HashMap::new(),
            };
            
            findings.push(finding);
        }
        
        Ok(findings)
    }
    
    fn map_rule_type(&self, metadata: &serde_json::Value) -> RuleType {
        if let Some(category) = metadata.get("category").and_then(|c| c.as_str()) {
            match category {
                "security" => RuleType::Security,
                "correctness" => RuleType::Correctness,
                "performance" => RuleType::Performance,
                "maintainability" => RuleType::Maintainability,
                "best-practice" => RuleType::BestPractice,
                _ => RuleType::Custom(category.to_string()),
            }
        } else {
            RuleType::BestPractice
        }
    }
}

/// CodeQL扫描器
pub struct CodeQLScanner {
    config: CodeQLConfig,
}

impl CodeQLScanner {
    pub fn new(config: CodeQLConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl ScanTool for CodeQLScanner {
    fn name(&self) -> &str {
        "codeql"
    }
    
    async fn is_available(&self) -> bool {
        match self.get_version().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    /// 获取工具版本
    async fn get_version(&self) -> Result<String> {
        let output = Command::new("codeql")
            .arg("--version")
            .output()
            .map_err(|e| anyhow!("Failed to run codeql --version: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("CodeQL --version failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        let version = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        Ok(version)
    }
    
    async fn install(&self) -> Result<()> {
        if self.is_available().await {
            return Ok(());
        }
        
        // 尝试多种安装方法
        let mut last_error = None;
        
        // 方法1: 使用npm安装（如果通过GitHub CLI）
        if let Err(e) = self.install_with_github_cli().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 方法2: 使用brew安装（macOS）
        if let Err(e) = self.install_with_brew().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 方法3: 手动下载安装
        if let Err(e) = self.install_manual().await {
            last_error = Some(e);
        } else {
            return Ok(());
        }
        
        // 所有方法都失败了
        Err(anyhow!("Failed to install CodeQL using all methods. Last error: {}", 
            last_error.unwrap_or_else(|| anyhow!("Unknown error"))))
    }
    
    /// 使用GitHub CLI安装
    async fn install_with_github_cli(&self) -> Result<()> {
        let output = Command::new("gh")
            .args(&["extension", "install", "github/gh-codeql"])
            .output()
            .map_err(|e| anyhow!("Failed to run gh extension install: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("gh extension install failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    /// 使用brew安装（macOS）
    async fn install_with_brew(&self) -> Result<()> {
        let output = Command::new("brew")
            .args(&["install", "gh"])
            .output()
            .map_err(|e| anyhow!("Failed to run brew install: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("brew install failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // 然后安装CodeQL扩展
        let output = Command::new("gh")
            .args(&["extension", "install", "github/gh-codeql"])
            .output()
            .map_err(|e| anyhow!("Failed to run gh extension install: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow!("gh extension install failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    /// 手动下载安装
    async fn install_manual(&self) -> Result<()> {
        // CodeQL CLI需要手动下载和设置
        // 这里提供安装指导
        Err(anyhow!("CodeQL requires manual installation. Please visit: https://github.com/github/codeql-cli-binaries"))
    }
    
    async fn scan(&self, request: &ScanRequest, mut progress_sender: mpsc::Sender<ScanProgress>) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // 发送进度更新
        progress_sender.send(ScanProgress {
            stage: ScanStage::RunningScan,
            progress: 0.0,
            message: "Starting CodeQL scan...".to_string(),
            tool: Some("codeql".to_string()),
        }).await?;
        
        // 获取版本信息
        let tool_version = self.get_version().await.unwrap_or_else(|_| "unknown".to_string());
        
        // CodeQL扫描实现较复杂，这里简化处理
        // 实际实现需要：创建数据库、运行查询、解析结果等
        let execution_time = start_time.elapsed().as_secs_f64();
        
        Ok(ToolResult {
            tool_name: "codeql".to_string(),
            tool_version,
            execution_time,
            raw_output: "CodeQL scanning not fully implemented".to_string(),
            parse_status: ParseStatus::Failed,
            error: Some("CodeQL scanning not fully implemented".to_string()),
        })
    }
    
    fn parse_results(&self, raw_output: &str) -> Result<Vec<Finding>> {
        // CodeQL结果解析实现
        Ok(vec![])
    }
    
    fn map_rule_type(&self, metadata: &serde_json::Value) -> RuleType {
        // CodeQL规则类型映射
        if let Some(kind) = metadata.get("kind").and_then(|k| k.as_str()) {
            match kind {
                "security" => RuleType::Security,
                "correctness" => RuleType::Correctness,
                "performance" => RuleType::Performance,
                "maintainability" => RuleType::Maintainability,
                _ => RuleType::Custom(kind.to_string()),
            }
        } else {
            RuleType::Security
        }
    }
}

/// Semgrep结果结构
#[derive(Debug, Clone, serde::Deserialize)]
struct SemgrepResult {
    path: String,
    start: SemgrepPosition,
    end: SemgrepPosition,
    extra: SemgrepExtra,
    check_id: String,
    message: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SemgrepPosition {
    line: usize,
    col: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SemgrepExtra {
    lines: Vec<String>,
    metadata: serde_json::Value,
    severity: String,
}