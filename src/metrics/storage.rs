// 质量指标存储模块
// 使用 JSON Lines 格式按分支存储快照数据

use super::QualitySnapshot;
use chrono::{DateTime, Utc};
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// 加载指定分支的历史快照
pub fn load_snapshots(
    storage_path: &Path,
    branch: &str,
) -> Result<Vec<QualitySnapshot>, Box<dyn std::error::Error + Send + Sync>> {
    let file_path = get_snapshot_file_path(storage_path, branch);

    if !file_path.exists() {
        log::debug!("快照文件不存在: {file_path:?}");
        return Ok(Vec::new());
    }

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    let mut snapshots = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<QualitySnapshot>(&line) {
            Ok(snapshot) => snapshots.push(snapshot),
            Err(e) => {
                log::warn!("无法解析快照行: {e}");
                // 继续处理其他行，不中断
            }
        }
    }

    // 按时间戳排序
    snapshots.sort_by_key(|s| s.timestamp);

    log::info!("从 {} 分支加载了 {} 个历史快照", branch, snapshots.len());

    Ok(snapshots)
}

/// 保存单个快照（追加到文件）
pub fn save_snapshot(
    storage_path: &Path,
    snapshot: &QualitySnapshot,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_path = get_snapshot_file_path(storage_path, &snapshot.branch);

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)?;

    let mut writer = BufWriter::new(file);

    // 写入 JSON Line
    let json_line = serde_json::to_string(snapshot)?;
    writeln!(writer, "{json_line}")?;
    writer.flush()?;

    log::debug!("保存快照到: {file_path:?}");

    Ok(())
}

/// 保存所有快照（覆盖文件）
pub fn save_all_snapshots(
    storage_path: &Path,
    branch: &str,
    snapshots: &[QualitySnapshot],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_path = get_snapshot_file_path(storage_path, branch);

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(&file_path)?;
    let mut writer = BufWriter::new(file);

    for snapshot in snapshots {
        let json_line = serde_json::to_string(snapshot)?;
        writeln!(writer, "{json_line}")?;
    }

    writer.flush()?;

    log::info!("保存 {} 个快照到: {:?}", snapshots.len(), file_path);

    Ok(())
}

/// 加载所有分支的快照
pub fn load_all_branches_snapshots(
    storage_path: &Path,
) -> Result<Vec<(String, Vec<QualitySnapshot>)>, Box<dyn std::error::Error + Send + Sync>> {
    let mut all_snapshots = Vec::new();

    if !storage_path.exists() {
        return Ok(all_snapshots);
    }

    // 遍历所有 .jsonl 文件
    for entry in std::fs::read_dir(storage_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Some(branch_name) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix("snapshots_"))
            {
                let snapshots = load_snapshots(storage_path, branch_name)?;
                if !snapshots.is_empty() {
                    all_snapshots.push((branch_name.to_string(), snapshots));
                }
            }
        }
    }

    Ok(all_snapshots)
}

/// 导出快照为 CSV 格式
pub fn export_to_csv(
    snapshots: &[QualitySnapshot],
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut writer = csv::Writer::from_path(output_path)?;

    // 写入标题行
    writer.write_record([
        "timestamp",
        "commit_hash",
        "branch",
        "lines_of_code",
        "module_count",
        "circular_dependencies",
        "coupling_score",
        "avg_complexity",
        "max_complexity",
        "public_api_count",
        "api_stability_score",
        "debt_score",
        "remediation_hours",
    ])?;

    // 写入数据行
    for snapshot in snapshots {
        writer.write_record([
            snapshot.timestamp.to_rfc3339(),
            snapshot.commit_hash.clone(),
            snapshot.branch.clone(),
            snapshot.lines_of_code.to_string(),
            snapshot.architecture_metrics.module_count.to_string(),
            snapshot
                .architecture_metrics
                .circular_dependencies
                .to_string(),
            format!("{:.2}", snapshot.architecture_metrics.coupling_score),
            format!(
                "{:.2}",
                snapshot.complexity_metrics.avg_cyclomatic_complexity
            ),
            snapshot
                .complexity_metrics
                .max_cyclomatic_complexity
                .to_string(),
            snapshot.api_metrics.public_api_count.to_string(),
            format!("{:.2}", snapshot.api_metrics.stability_score),
            format!("{:.2}", snapshot.technical_debt.debt_score),
            format!("{:.2}", snapshot.technical_debt.estimated_remediation_hours),
        ])?;
    }

    writer.flush()?;
    log::info!("导出 {} 个快照到 CSV: {:?}", snapshots.len(), output_path);

    Ok(())
}

/// 合并多个分支的快照
pub fn merge_branch_snapshots(
    storage_path: &Path,
    target_branch: &str,
    source_branches: &[&str],
) -> Result<Vec<QualitySnapshot>, Box<dyn std::error::Error + Send + Sync>> {
    let mut all_snapshots = Vec::new();

    // 加载目标分支
    all_snapshots.extend(load_snapshots(storage_path, target_branch)?);

    // 加载源分支
    for branch in source_branches {
        let snapshots = load_snapshots(storage_path, branch)?;
        all_snapshots.extend(snapshots);
    }

    // 按时间戳排序并去重（基于 commit_hash）
    all_snapshots.sort_by_key(|s| s.timestamp);
    all_snapshots.dedup_by_key(|s| s.commit_hash.clone());

    Ok(all_snapshots)
}

/// 获取快照文件路径
fn get_snapshot_file_path(storage_path: &Path, branch: &str) -> PathBuf {
    // 清理分支名称中的特殊字符
    let safe_branch_name = branch.replace(['/', '\\', ':'], "_");

    storage_path.join(format!("snapshots_{safe_branch_name}.jsonl"))
}

/// 清理过期的快照文件
pub fn cleanup_expired_files(
    storage_path: &Path,
    days_to_keep: i64,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);
    let mut removed_count = 0;

    if !storage_path.exists() {
        return Ok(0);
    }

    for entry in std::fs::read_dir(storage_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // 检查文件修改时间
            let metadata = std::fs::metadata(&path)?;
            if let Ok(modified) = metadata.modified() {
                let modified_time: DateTime<Utc> = modified.into();
                if modified_time < cutoff_date {
                    std::fs::remove_file(&path)?;
                    removed_count += 1;
                    log::info!("删除过期快照文件: {path:?}");
                }
            }
        }
    }

    Ok(removed_count)
}

/// 获取存储统计信息
pub struct StorageStats {
    pub total_snapshots: usize,
    pub total_branches: usize,
    pub storage_size_bytes: u64,
    pub oldest_snapshot: Option<DateTime<Utc>>,
    pub newest_snapshot: Option<DateTime<Utc>>,
}

pub fn get_storage_stats(
    storage_path: &Path,
) -> Result<StorageStats, Box<dyn std::error::Error + Send + Sync>> {
    let mut stats = StorageStats {
        total_snapshots: 0,
        total_branches: 0,
        storage_size_bytes: 0,
        oldest_snapshot: None,
        newest_snapshot: None,
    };

    if !storage_path.exists() {
        return Ok(stats);
    }

    let all_branches = load_all_branches_snapshots(storage_path)?;

    stats.total_branches = all_branches.len();

    for (_, snapshots) in all_branches {
        stats.total_snapshots += snapshots.len();

        for snapshot in snapshots {
            // 更新最旧和最新时间戳
            match stats.oldest_snapshot {
                None => stats.oldest_snapshot = Some(snapshot.timestamp),
                Some(oldest) if snapshot.timestamp < oldest => {
                    stats.oldest_snapshot = Some(snapshot.timestamp)
                }
                _ => {}
            }

            match stats.newest_snapshot {
                None => stats.newest_snapshot = Some(snapshot.timestamp),
                Some(newest) if snapshot.timestamp > newest => {
                    stats.newest_snapshot = Some(snapshot.timestamp)
                }
                _ => {}
            }
        }
    }

    // 计算存储大小
    for entry in std::fs::read_dir(storage_path)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("jsonl") {
            let metadata = entry.metadata()?;
            stats.storage_size_bytes += metadata.len();
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_snapshot(commit: &str, branch: &str) -> QualitySnapshot {
        QualitySnapshot {
            timestamp: Utc::now(),
            commit_hash: commit.to_string(),
            branch: branch.to_string(),
            lines_of_code: 1000,
            architecture_metrics: super::super::ArchitectureMetrics {
                module_count: 10,
                avg_module_size: 100.0,
                circular_dependencies: 0,
                pattern_violations: 0,
                coupling_score: 30.0,
                cohesion_score: 70.0,
            },
            complexity_metrics: super::super::ComplexityMetrics {
                avg_cyclomatic_complexity: 5.0,
                max_cyclomatic_complexity: 15,
                avg_function_length: 25.0,
                max_function_length: 100,
                high_complexity_functions: 5,
                functions_needing_refactor: 2,
            },
            api_metrics: super::super::ApiMetrics {
                public_api_count: 50,
                deprecated_api_count: 5,
                stability_score: 85.0,
                breaking_changes: 0,
                new_apis: 5,
                removed_apis: 0,
            },
            technical_debt: super::super::TechnicalDebtMetrics {
                debt_score: 45.0,
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
    fn test_save_and_load_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let snapshot1 = create_test_snapshot("commit1", "main");
        let snapshot2 = create_test_snapshot("commit2", "main");

        // 保存快照
        save_snapshot(storage_path, &snapshot1).unwrap();
        save_snapshot(storage_path, &snapshot2).unwrap();

        // 加载快照
        let loaded = load_snapshots(storage_path, "main").unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].commit_hash, "commit1");
        assert_eq!(loaded[1].commit_hash, "commit2");
    }

    #[test]
    fn test_multiple_branches() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let main_snapshot = create_test_snapshot("main_commit", "main");
        let dev_snapshot = create_test_snapshot("dev_commit", "dev");

        save_snapshot(storage_path, &main_snapshot).unwrap();
        save_snapshot(storage_path, &dev_snapshot).unwrap();

        let main_loaded = load_snapshots(storage_path, "main").unwrap();
        let dev_loaded = load_snapshots(storage_path, "dev").unwrap();

        assert_eq!(main_loaded.len(), 1);
        assert_eq!(dev_loaded.len(), 1);
        assert_eq!(main_loaded[0].branch, "main");
        assert_eq!(dev_loaded[0].branch, "dev");
    }

    #[test]
    fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let snapshot1 = create_test_snapshot("commit1", "main");
        let snapshot2 = create_test_snapshot("commit2", "dev");

        save_snapshot(storage_path, &snapshot1).unwrap();
        save_snapshot(storage_path, &snapshot2).unwrap();

        let stats = get_storage_stats(storage_path).unwrap();
        assert_eq!(stats.total_snapshots, 2);
        assert_eq!(stats.total_branches, 2);
        assert!(stats.storage_size_bytes > 0);
        assert!(stats.oldest_snapshot.is_some());
        assert!(stats.newest_snapshot.is_some());
    }
}
