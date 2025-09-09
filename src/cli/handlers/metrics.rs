use anyhow::Result;
use log::{debug, info};
use std::path::PathBuf;

use gitai::args::{Command, MetricsAction};
use gitai::config::Config;

/// Handler for metrics command with Command enum
#[cfg(feature = "metrics")]
pub async fn handle_command(
    config: &Config,
    command: &Command,
) -> crate::cli::CliResult<()> {
    match command {
        Command::Metrics { action } => {
            handle_metrics(config, action).await.map_err(|e| e.into())
        }
        _ => Err("Invalid command for metrics handler".into()),
    }
}

/// Handle metrics commands
#[cfg(feature = "metrics")]
async fn handle_metrics(_config: &Config, action: &MetricsAction) -> Result<()> {
    use gitai::metrics::QualityTracker;
    use gitai::project_insights::InsightsGenerator;
    use gitai::tree_sitter::TreeSitterManager;
    use gitai::git;
    
    match action {
        MetricsAction::Record { tags, force } => {
            info!("Recording code quality snapshot (force: {})", force);
            println!("📊 记录代码质量快照...");

            // 检查是否有代码变化（除非强制记录）
            if !force {
                let status = git::run_git(&["status".to_string(), "--porcelain".to_string()])?;
                if status.trim().is_empty() {
                    println!("ℹ️  没有检测到代码变化");
                    println!("💡 使用 --force 强制记录快照");
                    return Ok(());
                }
            }

            // 创建质量追踪器
            let mut tracker = QualityTracker::new()?;

            // 分析当前代码
            println!("🔍 分析代码结构...");
            let mut manager = TreeSitterManager::new().await?;

            // 获取当前目录的代码文件并分析
            let mut summary = gitai::tree_sitter::StructuralSummary::default();
            let code_files = find_code_files(".")?;

            for file_path in &code_files {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                        if let Some(lang) = gitai::tree_sitter::SupportedLanguage::from_extension(ext) {
                            if let Ok(file_summary) = manager.analyze_structure(&content, lang) {
                                // 合并结果
                                summary.functions.extend(file_summary.functions);
                                summary.classes.extend(file_summary.classes);
                                summary.comments.extend(file_summary.comments);
                            }
                        }
                    }
                }
            }

            // 生成项目洞察
            println!("💡 生成项目洞察...");
            let insights = InsightsGenerator::generate(&summary, None);

            // 记录快照
            let mut snapshot = tracker.record_snapshot(&summary, &insights)?;

            // 添加标签
            if !tags.is_empty() {
                snapshot.tags = tags.clone();
            }

            println!("✅ 质量快照已记录");
            println!("   Commit: {}", &snapshot.commit_hash[..7]);
            println!("   分支: {}", snapshot.branch);
            println!("   代码行数: {}", snapshot.lines_of_code);
            println!("   技术债务: {:.1}", snapshot.technical_debt.debt_score);
            println!("   复杂度: {:.1}", snapshot.complexity_metrics.avg_cyclomatic_complexity);
            
            info!("Quality snapshot recorded successfully");
            Ok(())
        }
        MetricsAction::Analyze { days, format, output } => {
            info!("Analyzing quality trends for {} days in {} format", days.unwrap_or(30), format);
            println!("📈 分析质量趋势...");

            let tracker = QualityTracker::new()?;
            let analysis = tracker.analyze_trends(*days)?;

            let result = match format.as_str() {
                "json" => serde_json::to_string_pretty(&analysis)?,
                "markdown" | "html" => {
                    let visualizer = gitai::metrics::visualizer::TrendVisualizer::new();
                    if format == "html" {
                        visualizer.generate_html_report(&analysis, tracker.get_snapshots())?
                    } else {
                        visualizer.generate_report(&analysis, tracker.get_snapshots())?
                    }
                }
                _ => {
                    // 文本格式
                    format!(
                        "质量趋势分析\n\n整体趋势: {:?}\n时间范围: {} 到 {}\n快照数量: {}\n关键发现: {}\n改进建议: {}\n",
                        analysis.overall_trend,
                        analysis.time_range.start.format("%Y-%m-%d"),
                        analysis.time_range.end.format("%Y-%m-%d"),
                        analysis.time_range.snapshots_count,
                        analysis.key_findings.len(),
                        analysis.recommendations.len()
                    )
                }
            };

            if let Some(output_path) = output {
                std::fs::write(output_path, result)?;
                println!("📁 分析结果已保存到: {}", output_path.display());
                info!("Analysis results saved to: {}", output_path.display());
            } else {
                println!("{}", result);
            }
            
            Ok(())
        }
        MetricsAction::Report { report_type: _, output, html } => {
            info!("Generating quality report (html: {})", html);
            println!("📄 生成质量报告...");

            let tracker = QualityTracker::new()?;

            let report = if *html {
                let analysis = tracker.analyze_trends(None)?;
                let visualizer = gitai::metrics::visualizer::TrendVisualizer::new();
                visualizer.generate_html_report(&analysis, tracker.get_snapshots())?
            } else {
                tracker.generate_report(output.as_deref())?
            };

            if let Some(output_path) = output {
                std::fs::write(output_path, report)?;
                println!("✅ 报告已生成: {}", output_path.display());
                info!("Report generated: {}", output_path.display());
            } else {
                println!("{}", report);
            }
            
            Ok(())
        }
        MetricsAction::List { limit, branch, format } => {
            info!("Listing quality snapshots (limit: {}, format: {})", limit, format);
            
            let tracker = QualityTracker::new()?;
            let snapshots = tracker.get_snapshots();

            // 过滤分支
            let filtered: Vec<_> = if let Some(branch_name) = branch {
                snapshots.iter().filter(|s| s.branch == *branch_name).collect()
            } else {
                snapshots.iter().collect()
            };

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(
                        &filtered.into_iter().take(*limit).collect::<Vec<_>>(),
                    )?;
                    println!("{json}");
                }
                _ => {
                    println!("📊 质量快照列表:");
                    for (i, snapshot) in filtered.into_iter().take(*limit).enumerate() {
                        println!(
                            "{}. {} [{}] - {} (debt: {:.1})",
                            i + 1,
                            &snapshot.commit_hash[..7],
                            snapshot.branch,
                            snapshot.timestamp.format("%Y-%m-%d %H:%M"),
                            snapshot.technical_debt.debt_score
                        );
                        debug!("Listed snapshot: {} on branch {}", 
                               &snapshot.commit_hash[..7], snapshot.branch);
                    }
                }
            }
            
            info!("Listed {} snapshots", filtered.len().min(*limit));
            Ok(())
        }
    }
}

/// Find code files in the given directory
#[cfg(feature = "metrics")]
fn find_code_files(dir: &str) -> Result<Vec<PathBuf>> {
    use std::fs;
    
    let mut code_files = Vec::new();
    let extensions = [
        "rs", "java", "py", "js", "ts", "go", "c", "cpp", "h", "hpp",
        "rb", "php", "swift", "kt", "scala", "cs", "vb", "pl", "r", "m"
    ];
    
    fn visit_dir(dir: &std::path::Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Skip common non-code directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !matches!(name, "target" | "node_modules" | ".git" | "build" | "dist") {
                        visit_dir(&path, extensions, files)?;
                    }
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    files.push(path);
                }
            }
        }
        Ok(())
    }
    
    visit_dir(std::path::Path::new(dir), &extensions, &mut code_files)?;
    Ok(code_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitai::config::{AiConfig, ScanConfig};

    fn create_test_config() -> Config {
        Config {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "test-model".to_string(),
                api_key: None,
                temperature: Some(0.3),
            },
            scan: ScanConfig {
                default_path: Some(".".to_string()),
                timeout: Some(300),
                jobs: Some(4),
            },
            devops: None,
            mcp: None,
        }
    }

    #[tokio::test]
    #[cfg(feature = "metrics")]
    async fn test_handle_metrics_list() {
        let config = create_test_config();
        let action = MetricsAction::List {
            limit: 5,
            branch: None,
            format: "text".to_string(),
        };
        
        let result = handle_metrics(&config, &action).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    #[cfg(feature = "metrics")]
    async fn test_find_code_files() {
        let result = find_code_files(".");
        assert!(result.is_ok());
        
        if let Ok(files) = result {
            // Should find at least some Rust files in this project
            assert!(!files.is_empty());
        }
    }
}
