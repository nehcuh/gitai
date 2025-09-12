//! Metrics storage functionality

use crate::tracker::QualityMetrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage backend for metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorage {
    /// Stored metrics data
    data: HashMap<String, ProjectMetrics>,
    /// Storage configuration
    config: StorageConfig,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    backend: StorageBackend,
    /// Storage path for file-based backends
    path: Option<String>,
    /// Connection string for database backends
    connection_string: Option<String>,
}

/// Storage backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    /// In-memory storage
    Memory,
    /// File-based storage (JSON)
    File,
    /// SQLite database
    SQLite,
    /// PostgreSQL database
    PostgreSQL,
}

/// Metrics data for a specific project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Project identifier
    project_id: String,
    /// Collection of metrics snapshots
    metrics: Vec<QualityMetrics>,
    /// Project metadata
    metadata: HashMap<String, String>,
}

impl MetricsStorage {
    /// Create new metrics storage with default configuration
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            config: StorageConfig {
                backend: StorageBackend::Memory,
                path: None,
                connection_string: None,
            },
        }
    }

    /// Create metrics storage with custom configuration
    pub fn with_config(config: StorageConfig) -> Self {
        Self {
            data: HashMap::new(),
            config,
        }
    }

    /// Store metrics for a project
    pub fn store_metrics(
        &mut self,
        project_id: &str,
        metrics: QualityMetrics,
    ) -> Result<(), StorageError> {
        let project_metrics = self
            .data
            .entry(project_id.to_string())
            .or_insert(ProjectMetrics {
                project_id: project_id.to_string(),
                metrics: Vec::new(),
                metadata: HashMap::new(),
            });

        project_metrics.metrics.push(metrics);

        match self.config.backend {
            StorageBackend::Memory => Ok(()),
            StorageBackend::File => self.persist_to_file(),
            StorageBackend::SQLite => self.persist_to_sqlite(),
            StorageBackend::PostgreSQL => self.persist_to_postgres(),
        }
    }

    /// Retrieve metrics for a project
    pub fn get_metrics(&self, project_id: &str) -> Option<&Vec<QualityMetrics>> {
        self.data.get(project_id).map(|pm| &pm.metrics)
    }

    /// Get all project metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, ProjectMetrics> {
        &self.data
    }

    /// Delete metrics for a project
    pub fn delete_metrics(&mut self, project_id: &str) -> Result<(), StorageError> {
        self.data.remove(project_id);
        Ok(())
    }

    /// Persist data to file
    fn persist_to_file(&self) -> Result<(), StorageError> {
        if let Some(path) = &self.config.path {
            let json = serde_json::to_string_pretty(&self.data)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;

            std::fs::write(path, json).map_err(|e| StorageError::Io(e.to_string()))?;
        }
        Ok(())
    }

    /// Load data from file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), StorageError> {
        let content = std::fs::read_to_string(path).map_err(|e| StorageError::Io(e.to_string()))?;

        let data: HashMap<String, ProjectMetrics> = serde_json::from_str(&content)
            .map_err(|e| StorageError::Deserialization(e.to_string()))?;

        self.data = data;
        Ok(())
    }

    /// Persist to SQLite (placeholder)
    fn persist_to_sqlite(&self) -> Result<(), StorageError> {
        // TODO: Implement SQLite storage
        Ok(())
    }

    /// Persist to PostgreSQL (placeholder)
    fn persist_to_postgres(&self) -> Result<(), StorageError> {
        // TODO: Implement PostgreSQL storage
        Ok(())
    }
}

impl Default for MetricsStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// I/O error wrapper for storage operations
    #[error("I/O error: {0}")]
    Io(String),
    /// Serialization error when converting data to persisted formats
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Deserialization error when reading persisted data
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    /// Database backend reported an error
    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_storage_creation() {
        let storage = MetricsStorage::new();
        assert!(storage.get_all_metrics().is_empty());
    }

    #[test]
    fn test_store_and_retrieve_metrics() {
        let mut storage = MetricsStorage::new();
        let metrics = QualityMetrics {
            total_files: 10,
            files_analyzed: 8,
            total_findings: 2,
            findings_by_severity: HashMap::new(),
            duration_ms: 1500,
            timestamp: Utc::now(),
        };

        storage
            .store_metrics("test-project", metrics.clone())
            .unwrap();
        let retrieved = storage.get_metrics("test-project").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].total_files, 10);
    }

    #[test]
    fn test_file_storage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_metrics.json");
        let path_str = file_path.to_string_lossy().to_string();

        let config = StorageConfig {
            backend: StorageBackend::File,
            path: Some(path_str.clone()),
            connection_string: None,
        };

        let mut storage = MetricsStorage::with_config(config.clone());
        let metrics = QualityMetrics {
            total_files: 5,
            files_analyzed: 3,
            total_findings: 1,
            findings_by_severity: HashMap::new(),
            duration_ms: 800,
            timestamp: Utc::now(),
        };

        storage.store_metrics("test-project", metrics).unwrap();

        // Load into new storage
        let mut storage2 = MetricsStorage::with_config(config.clone());
        storage2.load_from_file(&path_str).unwrap();

        let retrieved = storage2.get_metrics("test-project").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].total_files, 5);
    }
}
