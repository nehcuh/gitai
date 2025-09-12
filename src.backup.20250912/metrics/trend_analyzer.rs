// 趋势分析器
// 分析架构质量指标的历史趋势

use super::{
    Finding, MetricTrend, Predictions, QualitySnapshot, Significance, TimeRange, Trend,
    TrendAnalysis,
};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// 趋势分析器
pub struct TrendAnalyzer<'a> {
    snapshots: &'a [QualitySnapshot],
}

impl<'a> TrendAnalyzer<'a> {
    /// 创建新的趋势分析器
    pub fn new(snapshots: &'a [QualitySnapshot]) -> Self {
        Self { snapshots }
    }

    /// 执行趋势分析
    pub fn analyze(
        &self,
        days_back: Option<i64>,
    ) -> Result<TrendAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        // 过滤时间范围内的快照
        let filtered_snapshots = self.filter_by_time_range(days_back);

        if filtered_snapshots.len() < 2 {
            return Err("需要至少2个快照才能进行趋势分析".into());
        }

        // 计算时间范围
        let time_range = TimeRange {
            start: filtered_snapshots[0].timestamp,
            end: filtered_snapshots[filtered_snapshots.len() - 1].timestamp,
            snapshots_count: filtered_snapshots.len(),
        };

        // 分析各指标趋势
        let metric_trends = self.analyze_metric_trends(&filtered_snapshots)?;

        // 计算整体趋势
        let overall_trend = self.calculate_overall_trend(&metric_trends);

        // 识别关键发现
        let key_findings = self.identify_key_findings(&filtered_snapshots, &metric_trends);

        // 生成建议
        let recommendations = self.generate_recommendations(&metric_trends, &key_findings);

        // 生成预测（如果有足够数据）
        let predictions = if filtered_snapshots.len() >= 5 {
            Some(self.generate_predictions(&filtered_snapshots, &metric_trends))
        } else {
            None
        };

        Ok(TrendAnalysis {
            time_range,
            overall_trend,
            metric_trends,
            key_findings,
            recommendations,
            predictions,
        })
    }

    /// 过滤时间范围内的快照
    fn filter_by_time_range(&self, days_back: Option<i64>) -> Vec<&QualitySnapshot> {
        if let Some(days) = days_back {
            let cutoff = Utc::now() - Duration::days(days);
            self.snapshots
                .iter()
                .filter(|s| s.timestamp >= cutoff)
                .collect()
        } else {
            self.snapshots.iter().collect()
        }
    }

    /// 分析指标趋势
    fn analyze_metric_trends(
        &self,
        snapshots: &[&QualitySnapshot],
    ) -> Result<HashMap<String, MetricTrend>, Box<dyn std::error::Error + Send + Sync>> {
        let mut trends = HashMap::new();

        if snapshots.len() < 2 {
            return Ok(trends);
        }

        let first = snapshots[0];
        let last = snapshots[snapshots.len() - 1];

        // 复杂度趋势
        trends.insert(
            "complexity".to_string(),
            self.calculate_metric_trend(
                "平均复杂度",
                first.complexity_metrics.avg_cyclomatic_complexity,
                last.complexity_metrics.avg_cyclomatic_complexity,
                false, // 越低越好
            ),
        );

        // 技术债务趋势
        trends.insert(
            "technical_debt".to_string(),
            self.calculate_metric_trend(
                "技术债务",
                first.technical_debt.debt_score,
                last.technical_debt.debt_score,
                false, // 越低越好
            ),
        );

        // API稳定性趋势
        trends.insert(
            "api_stability".to_string(),
            self.calculate_metric_trend(
                "API稳定性",
                first.api_metrics.stability_score,
                last.api_metrics.stability_score,
                true, // 越高越好
            ),
        );

        // 耦合度趋势
        trends.insert(
            "coupling".to_string(),
            self.calculate_metric_trend(
                "耦合度",
                first.architecture_metrics.coupling_score,
                last.architecture_metrics.coupling_score,
                false, // 越低越好
            ),
        );

        // 代码规模趋势
        trends.insert(
            "code_size".to_string(),
            self.calculate_metric_trend(
                "代码规模",
                first.lines_of_code as f64,
                last.lines_of_code as f64,
                true, // 中性，只记录变化
            ),
        );

        Ok(trends)
    }

    /// 计算单个指标的趋势
    fn calculate_metric_trend(
        &self,
        name: &str,
        previous_value: f64,
        current_value: f64,
        higher_is_better: bool,
    ) -> MetricTrend {
        let change_rate = if previous_value != 0.0 {
            ((current_value - previous_value) / previous_value) * 100.0
        } else {
            0.0
        };

        let trend = if change_rate.abs() < 1.0 {
            Trend::Stable
        } else if higher_is_better {
            if change_rate > 0.0 {
                Trend::Improving
            } else {
                Trend::Degrading
            }
        } else if change_rate < 0.0 {
            Trend::Improving
        } else {
            Trend::Degrading
        };

        let significance = if change_rate.abs() > 50.0 {
            Significance::Critical
        } else if change_rate.abs() > 20.0 {
            Significance::High
        } else if change_rate.abs() > 10.0 {
            Significance::Medium
        } else {
            Significance::Low
        };

        MetricTrend {
            name: name.to_string(),
            current_value,
            previous_value,
            change_rate,
            trend,
            significance,
        }
    }

    /// 计算整体趋势
    fn calculate_overall_trend(&self, metric_trends: &HashMap<String, MetricTrend>) -> Trend {
        let mut improving = 0;
        let mut degrading = 0;
        let mut _stable = 0;

        for trend in metric_trends.values() {
            match trend.trend {
                Trend::Improving => improving += 1,
                Trend::Degrading => degrading += 1,
                Trend::Stable => _stable += 1,
                Trend::Mixed => {}
            }
        }

        if improving > 0 && degrading > 0 {
            Trend::Mixed
        } else if improving > degrading {
            Trend::Improving
        } else if degrading > improving {
            Trend::Degrading
        } else {
            Trend::Stable
        }
    }

    /// 识别关键发现
    fn identify_key_findings(
        &self,
        snapshots: &[&QualitySnapshot],
        metric_trends: &HashMap<String, MetricTrend>,
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // 检查技术债务急剧增加
        if let Some(debt_trend) = metric_trends.get("technical_debt") {
            if debt_trend.change_rate > 30.0 {
                findings.push(Finding {
                    title: "技术债务快速增长".to_string(),
                    description: format!(
                        "技术债务增长了 {:.1}%，从 {:.1} 增加到 {:.1}",
                        debt_trend.change_rate, debt_trend.previous_value, debt_trend.current_value
                    ),
                    metric: "technical_debt".to_string(),
                    significance: Significance::High,
                    detected_at: Utc::now(),
                });
            }
        }

        // 检查复杂度增加
        if let Some(complexity_trend) = metric_trends.get("complexity") {
            if complexity_trend.change_rate > 20.0 {
                findings.push(Finding {
                    title: "代码复杂度显著增加".to_string(),
                    description: format!("平均圈复杂度增长了 {:.1}%", complexity_trend.change_rate),
                    metric: "complexity".to_string(),
                    significance: Significance::Medium,
                    detected_at: Utc::now(),
                });
            }
        }

        // 检查循环依赖
        if let Some(last) = snapshots.last() {
            if last.architecture_metrics.circular_dependencies > 0 {
                findings.push(Finding {
                    title: "存在循环依赖".to_string(),
                    description: format!(
                        "检测到 {} 个循环依赖",
                        last.architecture_metrics.circular_dependencies
                    ),
                    metric: "circular_dependencies".to_string(),
                    significance: Significance::High,
                    detected_at: last.timestamp,
                });
            }
        }

        // 检查API稳定性下降
        if let Some(api_trend) = metric_trends.get("api_stability") {
            if api_trend.change_rate < -15.0 {
                findings.push(Finding {
                    title: "API稳定性下降".to_string(),
                    description: format!("API稳定性评分下降了 {:.1}%", api_trend.change_rate.abs()),
                    metric: "api_stability".to_string(),
                    significance: Significance::Medium,
                    detected_at: Utc::now(),
                });
            }
        }

        findings
    }

    /// 生成改进建议
    fn generate_recommendations(
        &self,
        metric_trends: &HashMap<String, MetricTrend>,
        findings: &[Finding],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 基于趋势的建议
        for (metric_name, trend) in metric_trends {
            match trend.trend {
                Trend::Degrading if trend.significance as i32 >= Significance::Medium as i32 => {
                    match metric_name.as_str() {
                        "complexity" => {
                            recommendations.push(
                                "建议进行代码重构，降低复杂度。考虑拆分大函数和复杂逻辑"
                                    .to_string(),
                            );
                        }
                        "technical_debt" => {
                            recommendations.push(
                                "建议安排技术债务清理周期，优先处理高影响的技术债务".to_string(),
                            );
                        }
                        "coupling" => {
                            recommendations.push(
                                "建议重新审视模块边界，减少模块间依赖，提高内聚性".to_string(),
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // 基于发现的建议
        for finding in findings {
            if finding.significance as i32 >= Significance::High as i32
                && finding.metric.as_str() == "circular_dependencies"
            {
                recommendations
                    .push("紧急：需要解决循环依赖问题，这会影响系统的可维护性和测试性".to_string());
            }
        }

        // 通用建议
        if recommendations.is_empty() {
            recommendations.push("继续保持良好的代码质量实践".to_string());
        }

        recommendations
    }

    /// 生成预测
    fn generate_predictions(
        &self,
        snapshots: &[&QualitySnapshot],
        metric_trends: &HashMap<String, MetricTrend>,
    ) -> Predictions {
        // 简化的线性预测
        let debt_trend = metric_trends.get("technical_debt");
        let complexity_trend = metric_trends.get("complexity");

        let mut predictions = Predictions {
            debt_critical_date: None,
            predicted_complexity_next_month: 0.0,
            recommended_refactor_date: None,
            confidence: 0.0,
        };

        // 预测技术债务临界点
        if let Some(trend) = debt_trend {
            if trend.change_rate > 10.0 {
                // 简单线性预测：假设按当前速率增长
                let days_to_critical =
                    ((100.0 - trend.current_value) / (trend.change_rate / 30.0)) as i64;
                if days_to_critical > 0 && days_to_critical < 365 {
                    predictions.debt_critical_date =
                        Some(Utc::now() + Duration::days(days_to_critical));
                }
            }
        }

        // 预测下个月的复杂度
        if let Some(trend) = complexity_trend {
            predictions.predicted_complexity_next_month =
                trend.current_value * (1.0 + trend.change_rate / 100.0 / 12.0);
        }

        // 建议重构时机
        if snapshots.len() >= 3 {
            let recent_debt_growth = self.calculate_recent_growth_rate(snapshots, "debt");
            if recent_debt_growth > 5.0 {
                predictions.recommended_refactor_date = Some(Utc::now() + Duration::days(14));
            }
        }

        // 计算置信度
        predictions.confidence = match snapshots.len() {
            n if n >= 10 => 85.0,
            n if n >= 5 => 70.0,
            _ => 50.0,
        };

        predictions
    }

    /// 计算近期增长率
    fn calculate_recent_growth_rate(&self, snapshots: &[&QualitySnapshot], metric: &str) -> f64 {
        if snapshots.len() < 2 {
            return 0.0;
        }

        let recent = &snapshots[snapshots.len() - 1];
        let previous = &snapshots[snapshots.len() - 2];

        match metric {
            "debt" => {
                let diff = recent.technical_debt.debt_score - previous.technical_debt.debt_score;
                if previous.technical_debt.debt_score != 0.0 {
                    (diff / previous.technical_debt.debt_score) * 100.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    /// 检测异常值
    pub fn detect_anomalies(&self) -> Vec<(DateTime<Utc>, String)> {
        let mut anomalies = Vec::new();

        if self.snapshots.len() < 3 {
            return anomalies;
        }

        // 计算移动平均和标准差
        for i in 2..self.snapshots.len() {
            let current = &self.snapshots[i];
            let window = &self.snapshots[i.saturating_sub(2)..i];

            // 检查技术债务异常
            let avg_debt: f64 = window
                .iter()
                .map(|s| s.technical_debt.debt_score)
                .sum::<f64>()
                / window.len() as f64;

            if (current.technical_debt.debt_score - avg_debt).abs() > avg_debt * 0.5 {
                anomalies.push((
                    current.timestamp,
                    format!(
                        "技术债务异常: {:.1} (平均: {:.1})",
                        current.technical_debt.debt_score, avg_debt
                    ),
                ));
            }

            // 检查复杂度异常
            let avg_complexity: f64 = window
                .iter()
                .map(|s| s.complexity_metrics.avg_cyclomatic_complexity)
                .sum::<f64>()
                / window.len() as f64;

            if (current.complexity_metrics.avg_cyclomatic_complexity - avg_complexity).abs()
                > avg_complexity * 0.4
            {
                anomalies.push((
                    current.timestamp,
                    format!(
                        "复杂度异常: {:.1} (平均: {:.1})",
                        current.complexity_metrics.avg_cyclomatic_complexity, avg_complexity
                    ),
                ));
            }
        }

        anomalies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::*;

    fn create_test_snapshot(
        debt: f64,
        complexity: f64,
        timestamp: DateTime<Utc>,
    ) -> QualitySnapshot {
        QualitySnapshot {
            timestamp,
            commit_hash: "test".to_string(),
            branch: "main".to_string(),
            lines_of_code: 1000,
            architecture_metrics: ArchitectureMetrics {
                module_count: 10,
                avg_module_size: 100.0,
                circular_dependencies: 0,
                pattern_violations: 0,
                coupling_score: 30.0,
                cohesion_score: 70.0,
            },
            complexity_metrics: ComplexityMetrics {
                avg_cyclomatic_complexity: complexity,
                max_cyclomatic_complexity: 20,
                avg_function_length: 25.0,
                max_function_length: 100,
                high_complexity_functions: 5,
                functions_needing_refactor: 2,
            },
            api_metrics: ApiMetrics {
                public_api_count: 50,
                deprecated_api_count: 5,
                stability_score: 85.0,
                breaking_changes: 0,
                new_apis: 5,
                removed_apis: 0,
            },
            technical_debt: TechnicalDebtMetrics {
                debt_score: debt,
                duplication_rate: 5.0,
                comment_coverage: 70.0,
                test_coverage_estimate: 60.0,
                todo_count: 10,
                estimated_remediation_hours: 20.0,
            },
            tags: vec![],
        }
    }

    #[test]
    fn test_trend_analysis() {
        let now = Utc::now();
        let snapshots = vec![
            create_test_snapshot(30.0, 5.0, now - Duration::days(30)),
            create_test_snapshot(35.0, 5.5, now - Duration::days(20)),
            create_test_snapshot(40.0, 6.0, now - Duration::days(10)),
            create_test_snapshot(45.0, 6.5, now),
        ];

        let analyzer = TrendAnalyzer::new(&snapshots);
        let result = analyzer.analyze(None);

        assert!(result.is_ok());
        let analysis = result.unwrap();

        // 验证趋势分析结果
        assert_eq!(analysis.time_range.snapshots_count, 4);
        assert!(analysis.metric_trends.contains_key("technical_debt"));
        assert!(analysis.metric_trends.contains_key("complexity"));

        // 技术债务应该显示为恶化趋势
        let debt_trend = &analysis.metric_trends["technical_debt"];
        assert!(matches!(debt_trend.trend, Trend::Degrading));
    }

    #[test]
    fn test_anomaly_detection() {
        let now = Utc::now();
        let snapshots = vec![
            create_test_snapshot(30.0, 5.0, now - Duration::days(30)),
            create_test_snapshot(32.0, 5.2, now - Duration::days(20)),
            create_test_snapshot(80.0, 5.3, now - Duration::days(10)), // 异常值
            create_test_snapshot(35.0, 5.5, now),
        ];

        let analyzer = TrendAnalyzer::new(&snapshots);
        let anomalies = analyzer.detect_anomalies();

        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|(_, desc)| desc.contains("技术债务异常")));
    }
}
