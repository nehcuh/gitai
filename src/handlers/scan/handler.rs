use crate::types::scan::types::*;
use crate::handlers::scan::results::*;
use crate::handlers::scan::tools::{ScanTool as ScanToolTrait, SemgrepScanner, CodeQLScanner};
use crate::handlers::scan::validator::ToolValidator;
use crate::config::AppConfig;
use crate::errors::AppError;
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::sync::mpsc;
use anyhow::Result;
use uuid::Uuid;

/// 扫描处理器
pub struct ScanProcessor {
    config: AppConfig,
}

impl ScanProcessor {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// 验证工具状态
    pub async fn validate_tools(&self) -> Result<crate::handlers::scan::validator::ToolValidationResult> {
        let validator = ToolValidator::new(self.config.clone());
        validator.validate_all_tools().await
    }
    
    /// 自动安装缺失的工具
    pub async fn auto_install_tools(&self) -> Result<crate::handlers::scan::validator::ToolInstallationResult> {
        let validator = ToolValidator::new(self.config.clone());
        validator.auto_install_missing_tools().await
    }
    
    /// 执行扫描
    pub async fn scan(&self, request: ScanRequest) -> Result<ScanResult> {
        let scan_id = Uuid::new_v4().to_string();
        let start_time = chrono::Utc::now();
        
        // 创建进度通道
        let (progress_sender, progress_receiver) = mpsc::channel(100);
        
        // 启动扫描任务
        let scan_task = tokio::spawn({
            let request = request.clone();
            let config = self.config.clone();
            async move {
                Self::execute_scan(scan_id, request, config, progress_sender).await
            }
        });
        
        // 监听进度（可以在这里添加UI更新逻辑）
        let mut progress_receiver = progress_receiver;
        tokio::spawn(async move {
            while let Some(progress) = progress_receiver.recv().await {
                tracing::info!("Scan progress: {:.1}% - {}", progress.progress * 100.0, progress.message);
            }
        });
        
        // 等待扫描完成
        let result = scan_task.await??;
        
        Ok(result)
    }
    
    /// 执行具体的扫描逻辑
    async fn execute_scan(
        scan_id: String,
        request: ScanRequest,
        config: AppConfig,
        progress_sender: mpsc::Sender<ScanProgress>,
    ) -> Result<ScanResult> {
        let mut tool_results = HashMap::new();
        let mut all_findings = Vec::new();
        let mut tools_used = Vec::new();
        
        // 根据配置选择扫描工具
        let tools_to_run = match request.config.tool {
            ScanTool::Semgrep => vec![ScanTool::Semgrep],
            ScanTool::CodeQL => vec![ScanTool::CodeQL],
            ScanTool::Both => vec![ScanTool::Semgrep, ScanTool::CodeQL],
        };
        
        for tool in tools_to_run {
            let tool_result = match tool {
                ScanTool::Semgrep => {
                    let scanner = SemgrepScanner::new(request.config.semgrep_config.clone());
                    Self::run_tool(&scanner, &request, progress_sender.clone()).await?
                }
                ScanTool::CodeQL => {
                    let scanner = CodeQLScanner::new(request.config.codeql_config.clone());
                    Self::run_tool(&scanner, &request, progress_sender.clone()).await?
                }
                ScanTool::Both => unreachable!(), // Both is handled above
            };
            
            // 解析结果
            if tool_result.parse_status == ParseStatus::Success {
                let scanner: Box<dyn ScanToolTrait> = match tool {
                    ScanTool::Semgrep => Box::new(SemgrepScanner::new(request.config.semgrep_config.clone())),
                    ScanTool::CodeQL => Box::new(CodeQLScanner::new(request.config.codeql_config.clone())),
                    ScanTool::Both => unreachable!(),
                };
                
                match scanner.parse_results(&tool_result.raw_output) {
                    Ok(findings) => {
                        all_findings.extend(findings);
                        tools_used.push(tool.clone());
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse {} results: {}", tool, e);
                    }
                }
            }
            
            tool_results.insert(tool.to_string(), tool_result);
        }
        
        // 生成统计信息
        let stats = Self::generate_stats(&all_findings, &tools_used);
        
        // 确定扫描状态
        let status = if tool_results.values().any(|r| r.parse_status == ParseStatus::Failed) {
            ScanStatus::PartiallyCompleted
        } else {
            ScanStatus::Completed
        };
        
        Ok(ScanResult {
            scan_id,
            scan_time: chrono::Utc::now(),
            config_hash: Self::generate_config_hash(&request),
            findings: all_findings,
            stats,
            tool_results,
            status,
        })
    }
    
    /// 运行单个工具
    async fn run_tool<T: ScanToolTrait>(
        scanner: &T,
        request: &ScanRequest,
        progress_sender: mpsc::Sender<ScanProgress>,
    ) -> Result<ToolResult> {
        // 检查工具是否可用
        if !scanner.is_available().await {
            progress_sender.send(ScanProgress {
                stage: ScanStage::Failed(format!("{} not found, attempting to install...", scanner.name())),
                progress: 0.0,
                message: format!("Installing {}...", scanner.name()),
                tool: None,
            }).await?;
            
            // 尝试安装
            scanner.install().await?;
        }
        
        // 运行扫描
        scanner.scan(request, progress_sender).await
    }
    
    /// 生成统计信息
    fn generate_stats(findings: &[Finding], tools_used: &[ScanTool]) -> ScanStats {
        let mut high_severity = 0;
        let mut medium_severity = 0;
        let mut low_severity = 0;
        
        for finding in findings {
            match finding.severity {
                Severity::Error => high_severity += 1,
                Severity::Warning => medium_severity += 1,
                Severity::Info | Severity::Style => low_severity += 1,
            }
        }
        
        ScanStats {
            total_files: findings.iter()
                .map(|f| f.file_path.clone())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            scanned_files: findings.iter()
                .map(|f| f.file_path.clone())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            findings_count: findings.len(),
            high_severity,
            medium_severity,
            low_severity,
            scan_duration_seconds: 0.0, // 在实际执行时计算
            tools_used: tools_used.to_vec(),
        }
    }
    
    /// 生成配置哈希
    fn generate_config_hash(request: &ScanRequest) -> String {
        use sha2::{Sha256, Digest};
        let config_str = format!("{:?}", request.config);
        let mut hasher = Sha256::new();
        hasher.update(config_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// 生成报告
    pub fn generate_report(&self, result: &ScanResult) -> ScanReport {
        let mut file_results = HashMap::new();
        let mut severity_breakdown = HashMap::new();
        let mut type_breakdown = HashMap::new();
        
        // 按文件分组
        for finding in &result.findings {
            let file_path = &finding.file_path;
            let file_result = file_results.entry(file_path.clone()).or_insert_with(|| FileScanResult {
                file_path: file_path.clone(),
                file_type: Self::get_file_type(file_path),
                findings: Vec::new(),
                stats: FileStats {
                    lines_of_code: 0, // 需要实际计算
                    finding_density: 0.0,
                    max_severity: None,
                },
            });
            
            file_result.findings.push(finding.clone());
        }
        
        // 按严重程度分组
        for finding in &result.findings {
            severity_breakdown
                .entry(finding.severity.clone())
                .or_insert_with(Vec::new)
                .push(finding.clone());
        }
        
        // 按类型分组
        for finding in &result.findings {
            type_breakdown
                .entry(finding.rule_type.clone())
                .or_insert_with(Vec::new)
                .push(finding.clone());
        }
        
        // 生成建议
        let recommendations = Self::generate_recommendations(&result.findings);
        
        ScanReport {
            summary: ScanSummary {
                scan_id: result.scan_id.clone(),
                scan_time: result.scan_time,
                project_name: "Unknown".to_string(), // 需要从配置获取
                scan_path: PathBuf::new(), // 需要从请求获取
                tools_used: result.stats.tools_used.iter().map(|t| t.to_string()).collect(),
                total_files: result.stats.total_files,
                total_findings: result.stats.findings_count,
                severity_counts: HashMap::new(), // 需要计算
                status: result.status.clone(),
            },
            file_results,
            severity_breakdown,
            type_breakdown,
            recommendations,
        }
    }
    
    /// 获取文件类型
    fn get_file_type(path: &PathBuf) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    /// 生成建议
    fn generate_recommendations(findings: &[Finding]) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();
        
        // 统计不同类型的问题
        let mut security_count = 0;
        let mut performance_count = 0;
        let mut error_count = 0;
        
        for finding in findings {
            match finding.severity {
                Severity::Error => error_count += 1,
                _ => {}
            }
            
            match finding.rule_type {
                RuleType::Security => security_count += 1,
                RuleType::Performance => performance_count += 1,
                _ => {}
            }
        }
        
        // 根据统计生成建议
        if security_count > 0 {
            recommendations.push(Recommendation {
                priority: Priority::Critical,
                title: "修复安全问题".to_string(),
                description: format!("发现 {} 个安全问题，建议立即修复", security_count),
                related_findings: findings
                    .iter()
                    .filter(|f| f.rule_type == RuleType::Security)
                    .map(|f| f.id.clone())
                    .collect(),
                estimated_effort: Some(security_count as f64 * 0.5),
            });
        }
        
        if error_count > 0 {
            recommendations.push(Recommendation {
                priority: Priority::High,
                title: "修复错误级别问题".to_string(),
                description: format!("发现 {} 个错误级别问题，建议尽快修复", error_count),
                related_findings: findings
                    .iter()
                    .filter(|f| f.severity == Severity::Error)
                    .map(|f| f.id.clone())
                    .collect(),
                estimated_effort: Some(error_count as f64 * 0.3),
            });
        }
        
        if performance_count > 0 {
            recommendations.push(Recommendation {
                priority: Priority::Medium,
                title: "优化性能问题".to_string(),
                description: format!("发现 {} 个性能问题，建议优化", performance_count),
                related_findings: findings
                    .iter()
                    .filter(|f| f.rule_type == RuleType::Performance)
                    .map(|f| f.id.clone())
                    .collect(),
                estimated_effort: Some(performance_count as f64 * 0.8),
            });
        }
        
        recommendations
    }
}