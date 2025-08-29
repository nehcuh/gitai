// 趋势可视化器
// 生成质量趋势报告

use super::{
    QualitySnapshot, TrendAnalysis, Trend,
    Significance
};
use chrono::Utc;
use std::fmt::Write;

/// 趋势可视化器
pub struct TrendVisualizer;

impl TrendVisualizer {
    /// 创建新的可视化器
    pub fn new() -> Self {
        Self
    }
    
    /// 生成 Markdown 格式的趋势报告
    pub fn generate_report(
        &self,
        analysis: &TrendAnalysis,
        snapshots: &[QualitySnapshot],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut report = String::new();
        
        // 报告标题
        writeln!(report, "# 架构质量趋势报告")?;
        writeln!(report)?;
        writeln!(report, "生成时间: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(report)?;
        
        // 执行摘要
        self.write_executive_summary(&mut report, analysis)?;
        
        // 时间范围信息
        self.write_time_range_info(&mut report, analysis)?;
        
        // 整体趋势
        self.write_overall_trend(&mut report, analysis)?;
        
        // 关键指标趋势
        self.write_metric_trends(&mut report, analysis)?;
        
        // 关键发现
        self.write_key_findings(&mut report, analysis)?;
        
        // 历史数据图表
        self.write_trend_chart(&mut report, snapshots)?;
        
        // 改进建议
        self.write_recommendations(&mut report, analysis)?;
        
        // 预测
        if let Some(ref predictions) = analysis.predictions {
            self.write_predictions(&mut report, predictions)?;
        }
        
        // 详细数据表格
        self.write_detailed_data_table(&mut report, snapshots)?;
        
        Ok(report)
    }
    
    /// 写入执行摘要
    fn write_executive_summary(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 执行摘要")?;
        writeln!(report)?;
        
        let trend_emoji = match analysis.overall_trend {
            Trend::Improving => "📈",
            Trend::Stable => "➡️",
            Trend::Degrading => "📉",
            Trend::Mixed => "🔄",
        };
        
        writeln!(report, "**整体趋势**: {} {:?}", trend_emoji, analysis.overall_trend)?;
        writeln!(report)?;
        
        // 统计改善和恶化的指标
        let improving = analysis.metric_trends.values()
            .filter(|t| matches!(t.trend, Trend::Improving))
            .count();
        let degrading = analysis.metric_trends.values()
            .filter(|t| matches!(t.trend, Trend::Degrading))
            .count();
        
        writeln!(report, "- ✅ **改善的指标**: {}", improving)?;
        writeln!(report, "- ⚠️ **恶化的指标**: {}", degrading)?;
        writeln!(report, "- 🔍 **关键发现**: {}", analysis.key_findings.len())?;
        writeln!(report, "- 💡 **改进建议**: {}", analysis.recommendations.len())?;
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入时间范围信息
    fn write_time_range_info(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 分析范围")?;
        writeln!(report)?;
        writeln!(report, "- **起始时间**: {}", analysis.time_range.start.format("%Y-%m-%d"))?;
        writeln!(report, "- **结束时间**: {}", analysis.time_range.end.format("%Y-%m-%d"))?;
        writeln!(report, "- **快照数量**: {}", analysis.time_range.snapshots_count)?;
        
        let duration = analysis.time_range.end - analysis.time_range.start;
        writeln!(report, "- **时间跨度**: {} 天", duration.num_days())?;
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入整体趋势
    fn write_overall_trend(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 整体趋势分析")?;
        writeln!(report)?;
        
        let description = match analysis.overall_trend {
            Trend::Improving => "项目整体质量呈改善趋势，大部分指标都在向好的方向发展。",
            Trend::Stable => "项目质量保持稳定，各项指标变化不大。",
            Trend::Degrading => "项目质量呈下降趋势，需要关注并采取改进措施。",
            Trend::Mixed => "项目质量表现混合，部分指标改善而其他指标恶化。",
        };
        
        writeln!(report, "{}", description)?;
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入指标趋势
    fn write_metric_trends(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 关键指标趋势")?;
        writeln!(report)?;
        
        writeln!(report, "| 指标 | 当前值 | 之前值 | 变化率 | 趋势 | 重要性 |")?;
        writeln!(report, "|------|--------|--------|--------|------|--------|")?;
        
        for (key, trend) in &analysis.metric_trends {
            let trend_symbol = match trend.trend {
                Trend::Improving => "↗️",
                Trend::Stable => "→",
                Trend::Degrading => "↘️",
                Trend::Mixed => "↔️",
            };
            
            let significance_emoji = match trend.significance {
                Significance::Critical => "🔴",
                Significance::High => "🟠",
                Significance::Medium => "🟡",
                Significance::Low => "🟢",
            };
            
            writeln!(
                report,
                "| {} | {:.2} | {:.2} | {:+.1}% | {} | {} |",
                trend.name,
                trend.current_value,
                trend.previous_value,
                trend.change_rate,
                trend_symbol,
                significance_emoji
            )?;
        }
        
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入关键发现
    fn write_key_findings(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        if analysis.key_findings.is_empty() {
            return Ok(());
        }
        
        writeln!(report, "## 关键发现")?;
        writeln!(report)?;
        
        for finding in &analysis.key_findings {
            let emoji = match finding.significance {
                Significance::Critical => "🚨",
                Significance::High => "⚠️",
                Significance::Medium => "📌",
                Significance::Low => "ℹ️",
            };
            
            writeln!(report, "### {} {}", emoji, finding.title)?;
            writeln!(report)?;
            writeln!(report, "{}", finding.description)?;
            writeln!(report)?;
            writeln!(report, "- **指标**: {}", finding.metric)?;
            writeln!(report, "- **重要性**: {:?}", finding.significance)?;
            writeln!(report, "- **检测时间**: {}", finding.detected_at.format("%Y-%m-%d %H:%M"))?;
            writeln!(report)?;
        }
        
        Ok(())
    }
    
    /// 写入趋势图表（ASCII）
    fn write_trend_chart(&self, report: &mut String, snapshots: &[QualitySnapshot]) -> Result<(), std::fmt::Error> {
        if snapshots.len() < 2 {
            return Ok(());
        }
        
        writeln!(report, "## 趋势图表")?;
        writeln!(report)?;
        writeln!(report, "### 技术债务趋势")?;
        writeln!(report)?;
        writeln!(report, "```")?;
        
        // 简化的ASCII图表
        let max_debt = snapshots.iter()
            .map(|s| s.technical_debt.debt_score)
            .fold(0.0_f64, f64::max);
        
        let chart_height = 10;
        let chart_width = snapshots.len().min(50);
        
        for i in (0..=chart_height).rev() {
            let threshold = (i as f64 / chart_height as f64) * max_debt;
            write!(report, "{:6.1} | ", threshold)?;
            
            for j in 0..chart_width {
                let snapshot_idx = j * snapshots.len() / chart_width;
                let value = snapshots[snapshot_idx].technical_debt.debt_score;
                
                if value >= threshold {
                    write!(report, "█")?;
                } else {
                    write!(report, " ")?;
                }
            }
            
            writeln!(report)?;
        }
        
        write!(report, "        +")?;
        for _ in 0..chart_width {
            write!(report, "-")?;
        }
        writeln!(report)?;
        
        writeln!(report, "```")?;
        writeln!(report)?;
        
        writeln!(report, "### 复杂度趋势")?;
        writeln!(report)?;
        writeln!(report, "```")?;
        
        // 复杂度图表
        let max_complexity = snapshots.iter()
            .map(|s| s.complexity_metrics.avg_cyclomatic_complexity)
            .fold(0.0_f64, f64::max);
        
        for i in (0..=chart_height).rev() {
            let threshold = (i as f64 / chart_height as f64) * max_complexity;
            write!(report, "{:6.1} | ", threshold)?;
            
            for j in 0..chart_width {
                let snapshot_idx = j * snapshots.len() / chart_width;
                let value = snapshots[snapshot_idx].complexity_metrics.avg_cyclomatic_complexity;
                
                if value >= threshold {
                    write!(report, "▓")?;
                } else {
                    write!(report, " ")?;
                }
            }
            
            writeln!(report)?;
        }
        
        write!(report, "        +")?;
        for _ in 0..chart_width {
            write!(report, "-")?;
        }
        writeln!(report)?;
        
        writeln!(report, "```")?;
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入改进建议
    fn write_recommendations(&self, report: &mut String, analysis: &TrendAnalysis) -> Result<(), std::fmt::Error> {
        if analysis.recommendations.is_empty() {
            return Ok(());
        }
        
        writeln!(report, "## 改进建议")?;
        writeln!(report)?;
        
        for (i, recommendation) in analysis.recommendations.iter().enumerate() {
            writeln!(report, "{}. {}", i + 1, recommendation)?;
        }
        
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入预测
    fn write_predictions(&self, report: &mut String, predictions: &super::Predictions) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 预测分析")?;
        writeln!(report)?;
        writeln!(report, "**置信度**: {:.0}%", predictions.confidence)?;
        writeln!(report)?;
        
        if let Some(ref debt_date) = predictions.debt_critical_date {
            writeln!(report, "- **技术债务临界点**: 预计在 {} 达到临界水平", 
                    debt_date.format("%Y-%m-%d"))?;
        }
        
        if predictions.predicted_complexity_next_month > 0.0 {
            writeln!(report, "- **下月复杂度预测**: {:.2}", 
                    predictions.predicted_complexity_next_month)?;
        }
        
        if let Some(ref refactor_date) = predictions.recommended_refactor_date {
            writeln!(report, "- **建议重构时间**: {}", 
                    refactor_date.format("%Y-%m-%d"))?;
        }
        
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 写入详细数据表格
    fn write_detailed_data_table(&self, report: &mut String, snapshots: &[QualitySnapshot]) -> Result<(), std::fmt::Error> {
        writeln!(report, "## 详细数据")?;
        writeln!(report)?;
        
        writeln!(report, "<details>")?;
        writeln!(report, "<summary>点击展开详细历史数据</summary>")?;
        writeln!(report)?;
        
        writeln!(report, "| 日期 | Commit | LOC | 模块数 | 循环依赖 | 债务分 | 复杂度 | API稳定性 |")?;
        writeln!(report, "|------|--------|-----|--------|----------|--------|--------|-----------|")?;
        
        for snapshot in snapshots.iter().rev().take(20) {
            writeln!(
                report,
                "| {} | {} | {} | {} | {} | {:.1} | {:.1} | {:.1}% |",
                snapshot.timestamp.format("%m-%d"),
                &snapshot.commit_hash[..7],
                snapshot.lines_of_code,
                snapshot.architecture_metrics.module_count,
                snapshot.architecture_metrics.circular_dependencies,
                snapshot.technical_debt.debt_score,
                snapshot.complexity_metrics.avg_cyclomatic_complexity,
                snapshot.api_metrics.stability_score,
            )?;
        }
        
        writeln!(report)?;
        writeln!(report, "</details>")?;
        writeln!(report)?;
        
        Ok(())
    }
    
    /// 生成 HTML 报告
    pub fn generate_html_report(
        &self,
        analysis: &TrendAnalysis,
        snapshots: &[QualitySnapshot],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let markdown = self.generate_report(analysis, snapshots)?;
        
        // 简单的 Markdown 到 HTML 转换
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>架构质量趋势报告</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }}
        h2 {{
            color: #34495e;
            margin-top: 30px;
        }}
        h3 {{
            color: #7f8c8d;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
            background: white;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        th {{
            background: #3498db;
            color: white;
            padding: 12px;
            text-align: left;
        }}
        td {{
            padding: 10px;
            border-bottom: 1px solid #ecf0f1;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        pre {{
            background: #2c3e50;
            color: #ecf0f1;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
        }}
        code {{
            background: #ecf0f1;
            padding: 2px 5px;
            border-radius: 3px;
        }}
        .trend-up {{ color: #27ae60; }}
        .trend-down {{ color: #e74c3c; }}
        .trend-stable {{ color: #95a5a6; }}
        details {{
            background: white;
            padding: 10px;
            margin: 20px 0;
            border-radius: 5px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        summary {{
            cursor: pointer;
            font-weight: bold;
            padding: 5px;
        }}
        summary:hover {{
            color: #3498db;
        }}
    </style>
</head>
<body>
    <div id="content">
        <!-- Markdown 内容将在这里通过 JS 库转换 -->
        {}
    </div>
</body>
</html>
        "#, markdown.replace('\n', "\\n").replace('"', "\\\""));
        
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::*;
    use chrono::Duration;
    use std::collections::HashMap;
    
    fn create_test_analysis() -> TrendAnalysis {
        let mut metric_trends = HashMap::new();
        
        metric_trends.insert(
            "technical_debt".to_string(),
            MetricTrend {
                name: "技术债务".to_string(),
                current_value: 55.0,
                previous_value: 45.0,
                change_rate: 22.2,
                trend: Trend::Degrading,
                significance: Significance::High,
            },
        );
        
        metric_trends.insert(
            "complexity".to_string(),
            MetricTrend {
                name: "复杂度".to_string(),
                current_value: 6.5,
                previous_value: 5.8,
                change_rate: 12.1,
                trend: Trend::Degrading,
                significance: Significance::Medium,
            },
        );
        
        TrendAnalysis {
            time_range: TimeRange {
                start: Utc::now() - Duration::days(30),
                end: Utc::now(),
                snapshots_count: 10,
            },
            overall_trend: Trend::Mixed,
            metric_trends,
            key_findings: vec![
                Finding {
                    title: "技术债务快速增长".to_string(),
                    description: "技术债务在过去30天增长了22%".to_string(),
                    metric: "technical_debt".to_string(),
                    significance: Significance::High,
                    detected_at: Utc::now(),
                },
            ],
            recommendations: vec![
                "建议进行代码重构".to_string(),
                "安排技术债务清理周期".to_string(),
            ],
            predictions: Some(Predictions {
                debt_critical_date: Some(Utc::now() + Duration::days(60)),
                predicted_complexity_next_month: 7.0,
                recommended_refactor_date: Some(Utc::now() + Duration::days(14)),
                confidence: 75.0,
            }),
        }
    }
    
    #[test]
    fn test_generate_report() {
        let visualizer = TrendVisualizer::new();
        let analysis = create_test_analysis();
        let snapshots = vec![];
        
        let result = visualizer.generate_report(&analysis, &snapshots);
        assert!(result.is_ok());
        
        let report = result.unwrap();
        assert!(report.contains("架构质量趋势报告"));
        assert!(report.contains("执行摘要"));
        assert!(report.contains("关键指标趋势"));
        assert!(report.contains("改进建议"));
    }
    
    #[test]
    fn test_html_report() {
        let visualizer = TrendVisualizer::new();
        let analysis = create_test_analysis();
        let snapshots = vec![];
        
        let result = visualizer.generate_html_report(&analysis, &snapshots);
        assert!(result.is_ok());
        
        let html = result.unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("架构质量趋势报告"));
    }
}
