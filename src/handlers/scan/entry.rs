use crate::config::AppConfig;
use crate::errors::{AppError, file_error};
use crate::types::scan::types::*;
use crate::handlers::scan::handler::ScanProcessor;
use crate::handlers::scan::results::*;
use crate::handlers::scan::validator::ToolValidator;
use crate::types::git::ScanArgs;
use std::path::PathBuf;
use anyhow::Result;

/// 处理扫描命令
pub async fn handle_scan(config: &AppConfig, args: ScanArgs) -> Result<(), AppError> {
    let scan_path = args.path.unwrap_or_else(|| ".".to_string());
    let path = PathBuf::from(scan_path);
    
    // 构建扫描配置
    let scan_config = ScanConfig {
        tool: args.tool.unwrap_or(ScanTool::Semgrep),
        path: path.clone(),
        full_scan: args.full,
        remote: args.remote,
        update_rules: args.update_rules,
        output_format: args.format.parse().map_err(|e| {
            crate::errors::AppError::Generic(format!("Invalid output format: {}", e))
        })?,
        enable_ai_translation: args.translate,
        semgrep_config: SemgrepConfig {
            rules_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
                .join("gitai")
                .join("scan-rules")
                .join("semgrep"),
            depth: "medium".to_string(),
            concurrency: 4,
            exclude_patterns: vec![
                "*.test.*".to_string(),
                "*/tests/*".to_string(),
                "*/node_modules/*".to_string(),
                "*/target/*".to_string(),
            ],
            timeout: 300,
        },
        codeql_config: CodeQLConfig {
            standard_library_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cache")
                .join("gitai")
                .join("scan-rules")
                .join("codeql"),
            database_timeout: 30,
            query_timeout: 15,
            security_only: true,
            memory_limit: 2048,
        },
    };
    
    // 构建扫描请求
    let request = ScanRequest {
        config: scan_config,
        language_filter: args.language,
        focus_areas: args.focus.map(|f| f.split(',').map(|s| s.trim().to_string()).collect()),
    };
    
    // 创建扫描处理器
    let processor = ScanProcessor::new(config.clone());
    
    // 验证工具状态
    println!("🔍 检查扫描工具状态...");
    let validation_result = processor.validate_tools().await.map_err(|e| {
        crate::errors::AppError::Generic(format!("工具验证失败: {}", e))
    })?;
    
    // 显示工具状态
    for (tool_name, status) in &validation_result.tool_statuses {
        if status.is_available {
            println!("✅ {}: 已安装 (版本: {})", tool_name, 
                status.version.as_deref().unwrap_or("未知"));
        } else {
            println!("❌ {}: 未安装", tool_name);
        }
    }
    
    // 如果有工具未安装且用户允许自动安装
    if !validation_result.unavailable_tools.is_empty() && args.auto_install {
        println!("📦 自动安装缺失的工具...");
        let install_result = processor.auto_install_tools().await.map_err(|e| {
            crate::errors::AppError::Generic(format!("工具安装失败: {}", e))
        })?;
        
        for (tool, status) in &install_result.installation_results {
            match status {
                crate::handlers::scan::validator::InstallationStatus::Success => {
                    println!("✅ {}: 安装成功", tool);
                }
                crate::handlers::scan::validator::InstallationStatus::Failed(err) => {
                    println!("❌ {}: 安装失败 - {}", tool, err);
                }
                crate::handlers::scan::validator::InstallationStatus::AlreadyInstalled => {
                    println!("✅ {}: 已安装", tool);
                }
            }
        }
    }
    
    // 检查是否有可用的工具
    let available_tools = validation_result.available_tools;
    if available_tools.is_empty() {
        return Err(crate::errors::AppError::Generic(
            "没有可用的扫描工具。请安装 semgrep 或 codeql 后重试。".to_string()
        ));
    }
    
    // 更新请求中的工具配置，只使用可用的工具
    let mut updated_request = request.clone();
    if !available_tools.contains(&updated_request.config.tool) {
        if available_tools.len() == 1 {
            updated_request.config.tool = available_tools[0].clone();
            println!("🔧 使用可用工具: {}", updated_request.config.tool);
        } else {
            updated_request.config.tool = ScanTool::Both;
            println!("🔧 使用所有可用工具: {:?}", available_tools);
        }
    }
    
    println!("🔍 开始扫描...");
    println!("📁 扫描路径: {}", path.display());
    
    // 执行扫描
    let result = processor.scan(updated_request).await.map_err(|e| {
        crate::errors::AppError::Generic(format!("扫描失败: {}", e))
    })?;
    
    // 生成报告
    let report = processor.generate_report(&result);
    
    // 输出结果
    output_scan_result(&report, &args.output, &args.format).await?;
    
    println!("✅ 扫描完成！");
    println!("📊 发现 {} 个问题", result.stats.findings_count);
    println!("🔴 高风险: {} 个", result.stats.high_severity);
    println!("🟡 中风险: {} 个", result.stats.medium_severity);
    println!("🟢 低风险: {} 个", result.stats.low_severity);
    
    Ok(())
}

/// 输出扫描结果
async fn output_scan_result(
    report: &ScanReport,
    output_file: &Option<String>,
    format: &str,
) -> Result<(), AppError> {
    let content = match format {
        "json" => serde_json::to_string_pretty(report).map_err(|e| {
            crate::errors::AppError::Generic(format!("JSON序列化失败: {}", e))
        })?,
        "markdown" | "md" => format_as_markdown(report),
        "text" => format_as_text(report),
        "sarif" => format_as_sarif(report)?,
        _ => return Err(crate::errors::AppError::Generic(format!("不支持的输出格式: {}", format))),
    };
    
    if let Some(output_path) = output_file {
        tokio::fs::write(output_path, content).await.map_err(|e| {
            crate::errors::AppError::Generic(format!("写入输出文件失败: {}", e))
        })?;
        println!("📄 结果已保存到: {}", output_path);
    } else {
        println!("{}", content);
    }
    
    Ok(())
}

/// 格式化为Markdown
fn format_as_markdown(report: &crate::handlers::scan::results::ScanReport) -> String {
    let mut output = String::new();
    
    output.push_str("# 🔍 代码扫描报告\n\n");
    output.push_str(&format!("**扫描时间**: {}\n", report.summary.scan_time.format("%Y-%m-%d %H:%M:%S")));
    output.push_str(&format!("**项目名称**: {}\n", report.summary.project_name));
    output.push_str(&format!("**扫描路径**: {}\n", report.summary.scan_path.display()));
    output.push_str(&format!("**使用工具**: {}\n", report.summary.tools_used.join(", ")));
    output.push_str(&format!("**扫描文件数**: {}\n", report.summary.total_files));
    output.push_str(&format!("**发现问题数**: {}\n\n", report.summary.total_findings));
    
    // 严重程度分布
    output.push_str("## 📊 严重程度分布\n\n");
    for (severity, findings) in &report.severity_breakdown {
        let emoji = match severity {
            crate::handlers::scan::results::Severity::Error => "🔴",
            crate::handlers::scan::results::Severity::Warning => "🟡",
            crate::handlers::scan::results::Severity::Info => "🟢",
            crate::handlers::scan::results::Severity::Style => "🔵",
        };
        output.push_str(&format!("{} **{}**: {} 个\n", emoji, severity, findings.len()));
    }
    output.push('\n');
    
    // 按文件显示结果
    output.push_str("## 📁 文件扫描结果\n\n");
    for (file_path, file_result) in &report.file_results {
        if !file_result.findings.is_empty() {
            output.push_str(&format!("### {}\n\n", file_path.display()));
            for finding in &file_result.findings {
                let severity_emoji = match finding.severity {
                    crate::handlers::scan::results::Severity::Error => "🔴",
                    crate::handlers::scan::results::Severity::Warning => "🟡",
                    crate::handlers::scan::results::Severity::Info => "🟢",
                    crate::handlers::scan::results::Severity::Style => "🔵",
                };
                
                output.push_str(&format!("{} **{}**\n", severity_emoji, finding.title));
                output.push_str(&format!("- **规则**: `{}`\n", finding.rule_id));
                output.push_str(&format!("- **位置**: 第{}行\n", finding.location.start_line));
                output.push_str(&format!("- **描述**: {}\n", finding.description));
                output.push('\n');
            }
        }
    }
    
    // 建议
    if !report.recommendations.is_empty() {
        output.push_str("## 💡 修复建议\n\n");
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            let priority_emoji = match recommendation.priority {
                crate::handlers::scan::results::Priority::Critical => "🚨",
                crate::handlers::scan::results::Priority::High => "⚠️",
                crate::handlers::scan::results::Priority::Medium => "ℹ️",
                crate::handlers::scan::results::Priority::Low => "💭",
            };
            
            output.push_str(&format!("{}. {} {}\n", i + 1, priority_emoji, recommendation.title));
            output.push_str(&format!("   {}\n", recommendation.description));
            if let Some(effort) = recommendation.estimated_effort {
                output.push_str(&format!("   **预估工作量**: {:.1} 人天\n", effort));
            }
            output.push('\n');
        }
    }
    
    output
}

/// 格式化为文本
fn format_as_text(report: &crate::handlers::scan::results::ScanReport) -> String {
    let mut output = String::new();
    
    output.push_str("代码扫描报告\n");
    output.push_str("============\n\n");
    output.push_str(&format!("扫描时间: {}\n", report.summary.scan_time.format("%Y-%m-%d %H:%M:%S")));
    output.push_str(&format!("项目名称: {}\n", report.summary.project_name));
    output.push_str(&format!("扫描路径: {}\n", report.summary.scan_path.display()));
    output.push_str(&format!("使用工具: {}\n", report.summary.tools_used.join(", ")));
    output.push_str(&format!("扫描文件数: {}\n", report.summary.total_files));
    output.push_str(&format!("发现问题数: {}\n\n", report.summary.total_findings));
    
    // 严重程度分布
    output.push_str("严重程度分布:\n");
    for (severity, findings) in &report.severity_breakdown {
        output.push_str(&format!("- {}: {} 个\n", severity, findings.len()));
    }
    output.push('\n');
    
    // 按文件显示结果
    output.push_str("文件扫描结果:\n");
    for (file_path, file_result) in &report.file_results {
        if !file_result.findings.is_empty() {
            output.push_str(&format!("\n文件: {}\n", file_path.display()));
            for finding in &file_result.findings {
                output.push_str(&format!("- {}: {} (第{}行)\n", 
                    finding.severity, finding.title, finding.location.start_line));
            }
        }
    }
    
    output
}

/// 格式化为SARIF
fn format_as_sarif(report: &crate::handlers::scan::results::ScanReport) -> Result<String, AppError> {
    // SARIF格式实现
    let sarif_report = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "gitai-scan",
                    "version": "1.0.0",
                    "informationUri": "https://github.com/nehcuh/gitai"
                }
            },
            "results": []
        }]
    });
    
    serde_json::to_string_pretty(&sarif_report).map_err(|e| {
        crate::errors::AppError::Generic(format!("SARIF序列化失败: {}", e))
    })
}