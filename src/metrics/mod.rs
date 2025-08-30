// 架构质量趋势追踪
// 记录和分析项目架构质量指标的历史变化

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod storage;
pub mod trend_analyzer;
pub mod visualizer;

use crate::project_insights::ProjectInsights;
use crate::tree_sitter::StructuralSummary;

/// 质量指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySnapshot {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// Git commit hash
    pub commit_hash: String,
    /// 分支名称
    pub branch: String,
    /// 代码行数
    pub lines_of_code: usize,
    /// 架构指标
    pub architecture_metrics: ArchitectureMetrics,
    /// 复杂度指标
    pub complexity_metrics: ComplexityMetrics,
    /// API 稳定性指标
    pub api_metrics: ApiMetrics,
    /// 技术债务指标
    pub technical_debt: TechnicalDebtMetrics,
    /// 自定义标签
    pub tags: Vec<String>,
}

/// 架构指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureMetrics {
    /// 模块数量
    pub module_count: usize,
    /// 平均模块大小
    pub avg_module_size: f64,
    /// 循环依赖数量
    pub circular_dependencies: usize,
    /// 架构违规数量
    pub pattern_violations: usize,
    /// 耦合度评分 (0-100, 越低越好)
    pub coupling_score: f64,
    /// 内聚度评分 (0-100, 越高越好)
    pub cohesion_score: f64,
}

/// 复杂度指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// 平均圈复杂度
    pub avg_cyclomatic_complexity: f64,
    /// 最大圈复杂度
    pub max_cyclomatic_complexity: u32,
    /// 平均函数长度
    pub avg_function_length: f64,
    /// 最大函数长度
    pub max_function_length: usize,
    /// 高复杂度函数数量
    pub high_complexity_functions: usize,
    /// 需要重构的函数数量
    pub functions_needing_refactor: usize,
}

/// API 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    /// 公开 API 数量
    pub public_api_count: usize,
    /// 已弃用 API 数量
    pub deprecated_api_count: usize,
    /// API 稳定性评分 (0-100)
    pub stability_score: f64,
    /// 破坏性变更数量（相对上一个快照）
    pub breaking_changes: usize,
    /// 新增 API 数量
    pub new_apis: usize,
    /// 移除 API 数量
    pub removed_apis: usize,
}

/// 技术债务指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtMetrics {
    /// 总体债务评分 (0-100, 越低越好)
    pub debt_score: f64,
    /// 代码重复率
    pub duplication_rate: f64,
    /// 注释覆盖率
    pub comment_coverage: f64,
    /// 测试覆盖率估计
    pub test_coverage_estimate: f64,
    /// TODO/FIXME 注释数量
    pub todo_count: usize,
    /// 估计修复时间（小时）
    pub estimated_remediation_hours: f64,
}

/// 趋势分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 分析时间范围
    pub time_range: TimeRange,
    /// 整体趋势
    pub overall_trend: Trend,
    /// 各指标趋势
    pub metric_trends: HashMap<String, MetricTrend>,
    /// 关键发现
    pub key_findings: Vec<Finding>,
    /// 改进建议
    pub recommendations: Vec<String>,
    /// 预测
    pub predictions: Option<Predictions>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub snapshots_count: usize,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
    Mixed,
}

/// 单个指标的趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    pub name: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_rate: f64,
    pub trend: Trend,
    pub significance: Significance,
}

/// 重要性级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Significance {
    Critical,
    High,
    Medium,
    Low,
}

/// 关键发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub title: String,
    pub description: String,
    pub metric: String,
    pub significance: Significance,
    pub detected_at: DateTime<Utc>,
}

/// 预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predictions {
    /// 预测的技术债务达到临界点的时间
    pub debt_critical_date: Option<DateTime<Utc>>,
    /// 预测的下个月复杂度
    pub predicted_complexity_next_month: f64,
    /// 建议的重构时机
    pub recommended_refactor_date: Option<DateTime<Utc>>,
    /// 置信度 (0-100)
    pub confidence: f64,
}

/// 质量趋势追踪器
pub struct QualityTracker {
    /// 存储路径
    storage_path: PathBuf,
    /// 历史快照
    snapshots: Vec<QualitySnapshot>,
    /// 当前分支
    current_branch: String,
}

impl QualityTracker {
    /// 创建新的质量追踪器
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gitai")
            .join("metrics");

        std::fs::create_dir_all(&storage_path)?;

        let current_branch = Self::get_current_branch()?;
        let snapshots = storage::load_snapshots(&storage_path, &current_branch)?;

        Ok(Self {
            storage_path,
            snapshots,
            current_branch,
        })
    }

    /// 记录新的质量快照
    pub fn record_snapshot(
        &mut self,
        summary: &StructuralSummary,
        insights: &ProjectInsights,
    ) -> Result<QualitySnapshot, Box<dyn std::error::Error + Send + Sync>> {
        let commit_hash = Self::get_current_commit()?;
        let lines_of_code = Self::count_lines_of_code()?;

        let snapshot = QualitySnapshot {
            timestamp: Utc::now(),
            commit_hash,
            branch: self.current_branch.clone(),
            lines_of_code,
            architecture_metrics: Self::calculate_architecture_metrics(insights),
            complexity_metrics: Self::calculate_complexity_metrics(summary, insights),
            api_metrics: Self::calculate_api_metrics(insights, self.snapshots.last()),
            technical_debt: Self::calculate_technical_debt(insights),
            tags: Vec::new(),
        };

        // 保存快照
        self.snapshots.push(snapshot.clone());
        storage::save_snapshot(&self.storage_path, &snapshot)?;

        log::info!("记录质量快照: commit {}", snapshot.commit_hash);

        Ok(snapshot)
    }

    /// 分析趋势
    pub fn analyze_trends(
        &self,
        days_back: Option<i64>,
    ) -> Result<TrendAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let analyzer = trend_analyzer::TrendAnalyzer::new(&self.snapshots);
        analyzer.analyze(days_back)
    }

    /// 生成报告
    pub fn generate_report(
        &self,
        output_path: Option<&Path>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let analysis = self.analyze_trends(None)?;
        let visualizer = visualizer::TrendVisualizer::new();

        let report = visualizer.generate_report(&analysis, &self.snapshots)?;

        if let Some(path) = output_path {
            std::fs::write(path, &report)?;
            log::info!("质量趋势报告已保存到: {:?}", path);
        }

        Ok(report)
    }

    /// 获取当前分支
    fn get_current_branch() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = std::process::Command::new("git")
            .args(&["branch", "--show-current"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 获取当前 commit hash
    fn get_current_commit() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// 统计代码行数
    fn count_lines_of_code() -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // 简化实现，实际可以使用 tokei 或类似工具
        let output = std::process::Command::new("find")
            .args(&[
                ".", "-type", "f", "-name", "*.rs", "-o", "-name", "*.java", "-o", "-name", "*.py",
                "-o", "-name", "*.js", "-o", "-name", "*.ts",
            ])
            .arg("-exec")
            .arg("wc")
            .arg("-l")
            .arg("{}")
            .arg("+")
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines = output_str
            .lines()
            .last()
            .and_then(|l| l.split_whitespace().next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        Ok(lines)
    }

    /// 计算架构指标
    fn calculate_architecture_metrics(insights: &ProjectInsights) -> ArchitectureMetrics {
        ArchitectureMetrics {
            module_count: insights.architecture.module_dependencies.nodes.len(),
            avg_module_size: 0.0, // 需要更详细的实现
            circular_dependencies: insights
                .architecture
                .module_dependencies
                .circular_dependencies
                .len(),
            pattern_violations: insights.architecture.pattern_violations.len(),
            coupling_score: insights.architecture.coupling_analysis.average_coupling * 100.0,
            cohesion_score: 75.0, // 简化实现
        }
    }

    /// 计算复杂度指标
    fn calculate_complexity_metrics(
        summary: &StructuralSummary,
        insights: &ProjectInsights,
    ) -> ComplexityMetrics {
        let function_lengths: Vec<usize> = summary
            .functions
            .iter()
            .map(|f| f.line_end - f.line_start)
            .collect();

        let avg_length = if function_lengths.is_empty() {
            0.0
        } else {
            function_lengths.iter().sum::<usize>() as f64 / function_lengths.len() as f64
        };

        let max_length = function_lengths.iter().max().copied().unwrap_or(0);

        ComplexityMetrics {
            avg_cyclomatic_complexity: 5.0, // 简化实现
            max_cyclomatic_complexity: 15,  // 简化实现
            avg_function_length: avg_length,
            max_function_length: max_length,
            high_complexity_functions: insights.quality_hotspots.complexity_hotspots.len(),
            functions_needing_refactor: insights
                .quality_hotspots
                .complexity_hotspots
                .iter()
                .filter(|h| h.complexity_score > 30)
                .count(),
        }
    }

    /// 计算 API 指标
    fn calculate_api_metrics(
        insights: &ProjectInsights,
        previous: Option<&QualitySnapshot>,
    ) -> ApiMetrics {
        let public_api_count = insights.api_surface.public_apis.len();
        let deprecated_api_count = insights.api_surface.deprecated_apis.len();

        let (breaking_changes, new_apis, removed_apis) = if let Some(prev) = previous {
            (
                insights.impact_analysis.breaking_changes.len(),
                public_api_count.saturating_sub(prev.api_metrics.public_api_count),
                prev.api_metrics
                    .public_api_count
                    .saturating_sub(public_api_count),
            )
        } else {
            (0, public_api_count, 0)
        };

        ApiMetrics {
            public_api_count,
            deprecated_api_count,
            stability_score: insights.api_surface.api_stability.stability_score * 100.0,
            breaking_changes,
            new_apis,
            removed_apis,
        }
    }

    /// 计算技术债务
    fn calculate_technical_debt(insights: &ProjectInsights) -> TechnicalDebtMetrics {
        TechnicalDebtMetrics {
            debt_score: insights
                .quality_hotspots
                .maintenance_burden
                .technical_debt_score,
            duplication_rate: 0.0,       // 需要实现
            comment_coverage: 0.0,       // 需要从 summary 计算
            test_coverage_estimate: 0.0, // 需要实现
            todo_count: 0,               // 需要扫描注释
            estimated_remediation_hours: insights
                .quality_hotspots
                .maintenance_burden
                .estimated_refactoring_hours,
        }
    }

    /// 比较两个快照
    pub fn compare_snapshots(
        &self,
        snapshot1: &QualitySnapshot,
        snapshot2: &QualitySnapshot,
    ) -> HashMap<String, f64> {
        let mut changes = HashMap::new();

        changes.insert(
            "complexity_change".to_string(),
            snapshot2.complexity_metrics.avg_cyclomatic_complexity
                - snapshot1.complexity_metrics.avg_cyclomatic_complexity,
        );

        changes.insert(
            "debt_change".to_string(),
            snapshot2.technical_debt.debt_score - snapshot1.technical_debt.debt_score,
        );

        changes.insert(
            "api_stability_change".to_string(),
            snapshot2.api_metrics.stability_score - snapshot1.api_metrics.stability_score,
        );

        changes
    }

    /// 获取历史快照
    pub fn get_snapshots(&self) -> &[QualitySnapshot] {
        &self.snapshots
    }

    /// 清理旧快照
    pub fn cleanup_old_snapshots(
        &mut self,
        days_to_keep: i64,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);

        let initial_count = self.snapshots.len();
        self.snapshots.retain(|s| s.timestamp > cutoff_date);
        let removed_count = initial_count - self.snapshots.len();

        if removed_count > 0 {
            storage::save_all_snapshots(&self.storage_path, &self.current_branch, &self.snapshots)?;
            log::info!("清理了 {} 个旧快照", removed_count);
        }

        Ok(removed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_tracker_creation() {
        // 注意：这个测试需要 git 仓库环境
        if std::process::Command::new("git")
            .args(&["status"])
            .output()
            .is_ok()
        {
            let tracker = QualityTracker::new();
            assert!(tracker.is_ok(), "应该能创建质量追踪器");
        }
    }

    #[test]
    fn test_trend_calculation() {
        let snapshot1 = create_test_snapshot(50.0, 30.0);
        let snapshot2 = create_test_snapshot(45.0, 35.0);

        let tracker = QualityTracker {
            storage_path: PathBuf::from("."),
            snapshots: vec![snapshot1.clone(), snapshot2.clone()],
            current_branch: "main".to_string(),
        };

        let changes = tracker.compare_snapshots(&snapshot1, &snapshot2);

        assert!(changes.get("debt_change").unwrap() < &0.0); // debt decreased from 50 to 45
        assert!(changes.get("complexity_change").unwrap() > &0.0); // complexity increased from 30 to 35
    }

    fn create_test_snapshot(debt_score: f64, complexity: f64) -> QualitySnapshot {
        QualitySnapshot {
            timestamp: Utc::now(),
            commit_hash: "test_hash".to_string(),
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
                debt_score,
                duplication_rate: 5.0,
                comment_coverage: 70.0,
                test_coverage_estimate: 60.0,
                todo_count: 10,
                estimated_remediation_hours: 20.0,
            },
            tags: vec![],
        }
    }
}
