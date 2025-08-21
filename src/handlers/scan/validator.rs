use crate::handlers::scan::tools::{ScanTool, SemgrepScanner, CodeQLScanner};
use crate::types::scan::types::*;
use crate::config::AppConfig;
use std::collections::HashMap;
use anyhow::Result;

/// 工具验证器
pub struct ToolValidator {
    config: AppConfig,
}

impl ToolValidator {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// 验证所有工具的状态
    pub async fn validate_all_tools(&self) -> Result<ToolValidationResult> {
        let mut tool_statuses = HashMap::new();
        let mut available_tools = Vec::new();
        let mut unavailable_tools = Vec::new();
        
        // 验证Semgrep
        let semgrep_scanner = SemgrepScanner::new(self.config.scan.semgrep_config.clone());
        let semgrep_status = self.validate_tool(&semgrep_scanner).await;
        tool_statuses.insert("semgrep".to_string(), semgrep_status.clone());
        
        if semgrep_status.is_available {
            available_tools.push(ScanTool::Semgrep);
        } else {
            unavailable_tools.push(ScanTool::Semgrep);
        }
        
        // 验证CodeQL
        let codeql_scanner = CodeQLScanner::new(self.config.scan.codeql_config.clone());
        let codeql_status = self.validate_tool(&codeql_scanner).await;
        tool_statuses.insert("codeql".to_string(), codeql_status.clone());
        
        if codeql_status.is_available {
            available_tools.push(ScanTool::CodeQL);
        } else {
            unavailable_tools.push(ScanTool::CodeQL);
        }
        
        Ok(ToolValidationResult {
            tool_statuses,
            available_tools,
            unavailable_tools,
        })
    }
    
    /// 验证单个工具
    async fn validate_tool<T: ScanTool>(&self, tool: &T) -> ToolStatus {
        let is_available = tool.is_available().await;
        let version = if is_available {
            tool.get_version().await.ok()
        } else {
            None
        };
        
        ToolStatus {
            name: tool.name().to_string(),
            is_available,
            version,
            install_methods: self.get_install_methods(tool.name()),
        }
    }
    
    /// 获取工具的安装方法
    fn get_install_methods(&self, tool_name: &str) -> Vec<String> {
        match tool_name {
            "semgrep" => vec![
                "pip3 install --user semgrep".to_string(),
                "pip install --user semgrep".to_string(),
                "curl -fsSL https://semgrep.dev/install | sh".to_string(),
            ],
            "codeql" => vec![
                "gh extension install github/gh-codeql".to_string(),
                "brew install gh && gh extension install github/gh-codeql".to_string(),
                "Manual download from https://github.com/github/codeql-cli-binaries".to_string(),
            ],
            _ => vec![],
        }
    }
    
    /// 自动安装所有不可用的工具
    pub async fn auto_install_missing_tools(&self) -> Result<ToolInstallationResult> {
        let validation_result = self.validate_all_tools().await?;
        let mut installation_results = HashMap::new();
        let mut successfully_installed = Vec::new();
        let mut failed_installations = Vec::new();
        
        // 安装Semgrep
        if !validation_result.tool_statuses["semgrep"].is_available {
            let semgrep_scanner = SemgrepScanner::new(self.config.scan.semgrep_config.clone());
            match semgrep_scanner.install().await {
                Ok(_) => {
                    installation_results.insert("semgrep".to_string(), InstallationStatus::Success);
                    successfully_installed.push(ScanTool::Semgrep);
                }
                Err(e) => {
                    installation_results.insert("semgrep".to_string(), InstallationStatus::Failed(e.to_string()));
                    failed_installations.push((ScanTool::Semgrep, e.to_string()));
                }
            }
        }
        
        // 安装CodeQL
        if !validation_result.tool_statuses["codeql"].is_available {
            let codeql_scanner = CodeQLScanner::new(self.config.scan.codeql_config.clone());
            match codeql_scanner.install().await {
                Ok(_) => {
                    installation_results.insert("codeql".to_string(), InstallationStatus::Success);
                    successfully_installed.push(ScanTool::CodeQL);
                }
                Err(e) => {
                    installation_results.insert("codeql".to_string(), InstallationStatus::Failed(e.to_string()));
                    failed_installations.push((ScanTool::CodeQL, e.to_string()));
                }
            }
        }
        
        Ok(ToolInstallationResult {
            installation_results,
            successfully_installed,
            failed_installations,
        })
    }
}

/// 工具验证结果
#[derive(Debug, Clone)]
pub struct ToolValidationResult {
    pub tool_statuses: HashMap<String, ToolStatus>,
    pub available_tools: Vec<ScanTool>,
    pub unavailable_tools: Vec<ScanTool>,
}

/// 工具状态
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub is_available: bool,
    pub version: Option<String>,
    pub install_methods: Vec<String>,
}

/// 工具安装结果
#[derive(Debug, Clone)]
pub struct ToolInstallationResult {
    pub installation_results: HashMap<String, InstallationStatus>,
    pub successfully_installed: Vec<ScanTool>,
    pub failed_installations: Vec<(ScanTool, String)>,
}

/// 安装状态
#[derive(Debug, Clone)]
pub enum InstallationStatus {
    Success,
    Failed(String),
    AlreadyInstalled,
}