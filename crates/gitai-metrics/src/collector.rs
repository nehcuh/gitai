//! Metrics collection functionality

use crate::tracker::QualityMetrics;
use gitai_types::{AnalysisResult, Project};
use std::collections::HashMap;
use std::path::Path;

/// Metrics collector for gathering code quality metrics
pub struct MetricsCollector {
    /// Collection of metrics data
    metrics: HashMap<String, Vec<QualityMetrics>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Collect metrics from analysis results
    pub fn collect_from_analysis(&mut self, analysis: &AnalysisResult) -> QualityMetrics {
        let metrics = QualityMetrics {
            total_files: analysis.summary.total_files,
            files_analyzed: analysis.summary.files_analyzed,
            total_findings: analysis.summary.total_findings,
            findings_by_severity: analysis.summary.findings_by_severity.clone(),
            duration_ms: analysis.summary.duration_ms,
            timestamp: chrono::Utc::now(),
        };

        self.metrics
            .insert(analysis.project_id.clone(), vec![metrics.clone()]);
        metrics
    }

    /// Collect metrics from project files
    pub fn collect_from_project(&mut self, project: &Project, _path: &Path) -> QualityMetrics {
        // This would typically involve analyzing files in the project
        // For now, we'll return default metrics
        let metrics = QualityMetrics {
            total_files: 0,
            files_analyzed: 0,
            total_findings: 0,
            findings_by_severity: HashMap::new(),
            duration_ms: 0,
            timestamp: chrono::Utc::now(),
        };

        self.metrics
            .insert(project.id.clone(), vec![metrics.clone()]);
        metrics
    }

    /// Get all collected metrics
    pub fn get_metrics(&self) -> &HashMap<String, Vec<QualityMetrics>> {
        &self.metrics
    }

    /// Get metrics for a specific project
    pub fn get_project_metrics(&self, project_id: &str) -> Option<&Vec<QualityMetrics>> {
        self.metrics.get(project_id)
    }

    /// Clear all collected metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitai_types::{AnalysisStatus, AnalysisSummary, FindingSeverity};

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.get_metrics().is_empty());
    }

    #[test]
    fn test_collect_from_analysis() {
        let mut collector = MetricsCollector::new();
        let analysis = AnalysisResult {
            id: "test-analysis".to_string(),
            project_id: "test-project".to_string(),
            analysis_type: "security".to_string(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
            status: AnalysisStatus::Completed,
            findings: vec![],
            summary: AnalysisSummary {
                total_files: 10,
                files_analyzed: 8,
                total_findings: 2,
                findings_by_severity: {
                    let mut map = HashMap::new();
                    map.insert(FindingSeverity::Warning, 2);
                    map
                },
                duration_ms: 1500,
            },
        };

        let metrics = collector.collect_from_analysis(&analysis);
        assert_eq!(metrics.total_files, 10);
        assert_eq!(metrics.files_analyzed, 8);
        assert_eq!(metrics.total_findings, 2);

        let project_metrics = collector.get_project_metrics("test-project");
        assert!(project_metrics.is_some());
        assert_eq!(project_metrics.unwrap().len(), 1);
    }
}
