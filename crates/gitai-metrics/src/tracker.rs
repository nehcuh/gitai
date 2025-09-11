//! Quality metrics tracking functionality

use gitai_types::common::FindingSeverity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Total number of files processed
    pub total_files: usize,
    /// Number of files that were successfully analyzed
    pub files_analyzed: usize,
    /// Total number of findings/issues discovered
    pub total_findings: usize,
    /// Findings grouped by severity level
    pub findings_by_severity: HashMap<FindingSeverity, usize>,
    /// Time taken for analysis in milliseconds
    pub duration_ms: u64,
    /// Timestamp when metrics were collected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Code quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQuality {
    /// Overall quality score (0-100)
    pub score: f64,
    /// Quality grade (A-F)
    pub grade: QualityGrade,
    /// Quality metrics
    pub metrics: QualityMetrics,
    /// Quality trends over time
    pub trends: Option<QualityTrends>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Quality grade classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QualityGrade {
    /// Excellent quality (90-100)
    A,
    /// Good quality (80-89)
    B,
    /// Fair quality (70-79)
    C,
    /// Poor quality (60-69)
    D,
    /// Failing quality (0-59)
    F,
}

/// Quality trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Historical quality scores
    pub scores: Vec<f64>,
    /// Trend direction
    pub direction: TrendDirection,
    /// Rate of change
    pub rate_of_change: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving
    Improving,
    /// Stable
    Stable,
    /// Declining
    Declining,
}

impl QualityMetrics {
    /// Calculate quality score based on metrics
    pub fn calculate_quality_score(&self) -> f64 {
        if self.total_files == 0 {
            return 0.0;
        }

        let coverage_ratio = self.files_analyzed as f64 / self.total_files as f64;
        let finding_penalty = (self.total_findings as f64 / self.total_files as f64) * 10.0;
        let critical_penalty = *self.findings_by_severity.get(&FindingSeverity::Critical)
            .unwrap_or(&0) as f64 * 5.0;
        let error_penalty = *self.findings_by_severity.get(&FindingSeverity::Error)
            .unwrap_or(&0) as f64 * 2.0;

        let base_score = coverage_ratio * 100.0;
        (base_score - finding_penalty - critical_penalty - error_penalty).max(0.0).min(100.0)
    }

    /// Get quality grade from score
    pub fn get_quality_grade(&self) -> QualityGrade {
        let score = self.calculate_quality_score();
        match score {
            s if s >= 90.0 => QualityGrade::A,
            s if s >= 80.0 => QualityGrade::B,
            s if s >= 70.0 => QualityGrade::C,
            s if s >= 60.0 => QualityGrade::D,
            _ => QualityGrade::F,
        }
    }

    /// Generate recommendations based on metrics
    pub fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.files_analyzed < self.total_files {
            recommendations.push("Consider investigating why some files could not be analyzed".to_string());
        }

        if let Some(&critical_count) = self.findings_by_severity.get(&FindingSeverity::Critical) {
            if critical_count > 0 {
                recommendations.push(format!("Address {} critical security issues immediately", critical_count));
            }
        }

        if let Some(&error_count) = self.findings_by_severity.get(&FindingSeverity::Error) {
            if error_count > 5 {
                recommendations.push(format!("High number of errors ({}): consider improving error handling", error_count));
            }
        }

        if self.duration_ms > 10000 {
            recommendations.push("Analysis time is high: consider optimizing the analysis process".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Code quality is good: maintain current practices".to_string());
        }

        recommendations
    }

    /// Create a new quality metrics instance
    pub fn new() -> Self {
        Self {
            total_files: 0,
            files_analyzed: 0,
            total_findings: 0,
            findings_by_severity: HashMap::new(),
            duration_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Merge with another metrics instance
    pub fn merge(&mut self, other: &QualityMetrics) {
        self.total_files += other.total_files;
        self.files_analyzed += other.files_analyzed;
        self.total_findings += other.total_findings;
        self.duration_ms += other.duration_ms;

        for (severity, count) in &other.findings_by_severity {
            *self.findings_by_severity.entry(*severity).or_insert(0) += count;
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeQuality {
    /// Create code quality assessment from metrics
    pub fn from_metrics(metrics: QualityMetrics) -> Self {
        let score = metrics.calculate_quality_score();
        let grade = metrics.get_quality_grade();
        let recommendations = metrics.generate_recommendations();

        Self {
            score,
            grade,
            metrics,
            trends: None,
            recommendations,
        }
    }

    /// Create with historical trends
    pub fn with_trends(mut self, historical_data: &[QualityMetrics]) -> Self {
        if historical_data.len() >= 2 {
            let scores: Vec<f64> = historical_data
                .iter()
                .map(|m| m.calculate_quality_score())
                .collect();

            let direction = if scores.len() >= 2 {
                let recent = scores[scores.len() - 1];
                let previous = scores[scores.len() - 2];
                if recent > previous + 1.0 {
                    TrendDirection::Improving
                } else if previous > recent + 1.0 {
                    TrendDirection::Declining
                } else {
                    TrendDirection::Stable
                }
            } else {
                TrendDirection::Stable
            };

            let rate_of_change = if scores.len() >= 2 {
                scores[scores.len() - 1] - scores[0]
            } else {
                0.0
            };

            self.trends = Some(QualityTrends {
                scores,
                direction,
                rate_of_change,
            });
        }

        self
    }
}

impl QualityGrade {
    /// Get letter grade as string
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityGrade::A => "A",
            QualityGrade::B => "B",
            QualityGrade::C => "C",
            QualityGrade::D => "D",
            QualityGrade::F => "F",
        }
    }

    /// Get color for the grade
    pub fn color(&self) -> &'static str {
        match self {
            QualityGrade::A => "green",
            QualityGrade::B => "blue",
            QualityGrade::C => "yellow",
            QualityGrade::D => "orange",
            QualityGrade::F => "red",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_quality_metrics_creation() {
        let metrics = QualityMetrics::new();
        assert_eq!(metrics.total_files, 0);
        assert_eq!(metrics.files_analyzed, 0);
        assert_eq!(metrics.total_findings, 0);
    }

    #[test]
    fn test_quality_score_calculation() {
        let mut metrics = QualityMetrics::new();
        metrics.total_files = 10;
        metrics.files_analyzed = 10;
        metrics.total_findings = 0;

        let score = metrics.calculate_quality_score();
        assert_eq!(score, 100.0);

        metrics.total_findings = 5;
        let score_with_findings = metrics.calculate_quality_score();
        assert!(score_with_findings < 100.0);
    }

    #[test]
    fn test_quality_grade() {
        let mut metrics = QualityMetrics::new();
        metrics.total_files = 10;
        metrics.files_analyzed = 10;
        metrics.total_findings = 0;

        assert_eq!(metrics.get_quality_grade(), QualityGrade::A);

        metrics.total_findings = 20;
        let grade = metrics.get_quality_grade();
        assert_ne!(grade, QualityGrade::A);
    }

    #[test]
    fn test_code_quality_creation() {
        let metrics = QualityMetrics::new();
        let quality = CodeQuality::from_metrics(metrics);
        assert!(quality.score >= 0.0 && quality.score <= 100.0);
        assert!(!quality.recommendations.is_empty());
    }

    #[test]
    fn test_metrics_merge() {
        let mut metrics1 = QualityMetrics::new();
        metrics1.total_files = 5;
        metrics1.files_analyzed = 4;
        metrics1.total_findings = 1;
        metrics1.findings_by_severity.insert(FindingSeverity::Warning, 1);

        let mut metrics2 = QualityMetrics::new();
        metrics2.total_files = 3;
        metrics2.files_analyzed = 2;
        metrics2.total_findings = 1;
        metrics2.findings_by_severity.insert(FindingSeverity::Error, 1);

        metrics1.merge(&metrics2);
        assert_eq!(metrics1.total_files, 8);
        assert_eq!(metrics1.files_analyzed, 6);
        assert_eq!(metrics1.total_findings, 2);
        assert_eq!(metrics1.findings_by_severity.get(&FindingSeverity::Warning), Some(&1));
        assert_eq!(metrics1.findings_by_severity.get(&FindingSeverity::Error), Some(&1));
    }
}