// review 缓存模块
// 负责评审结果的缓存管理

use super::types::{ReviewCache, ReviewConfig};

/// 生成缓存键
pub fn build_cache_key(diff: &str, cfg: &ReviewConfig) -> String {
    let diff_hash = format!("{:x}", md5::compute(diff.as_bytes()));
    let mut ids = cfg.issue_ids.clone();
    ids.sort();
    let payload = serde_json::json!({
        "diff": diff_hash,
        "language": cfg.language,
        "security_scan": cfg.security_scan,
        "deviation_analysis": cfg.deviation_analysis,
        "issue_ids": ids,
    });
    format!("{:x}", md5::compute(payload.to_string().as_bytes()))
}

/// 检查缓存
pub fn check_cache(
    cache_key: &str,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("review_cache");

    let cache_file = cache_dir.join(format!("review_{cache_key}.json"));

    if !cache_file.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cache_file)?;
    let cache: ReviewCache = serde_json::from_str(&content)?;

    if cache.is_expired(3600) {
        return Ok(None);
    }

    Ok(Some(cache.review_result))
}

/// 保存缓存
pub fn save_cache(
    cache_key: &str,
    result: &str,
    language: &Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("gitai")
        .join("review_cache");

    std::fs::create_dir_all(&cache_dir)?;

    let cache = ReviewCache::new(cache_key, result.to_string(), language.clone());
    let cache_file = cache_dir.join(format!("review_{cache_key}.json"));

    let content = serde_json::to_string_pretty(&cache)?;
    std::fs::write(&cache_file, content)?;

    // 清理旧缓存（保留最近 100 个）
    cleanup_old_caches(&cache_dir, 100)?;

    Ok(())
}

/// 清理旧缓存
fn cleanup_old_caches(
    cache_dir: &std::path::PathBuf,
    max_count: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut entries: Vec<_> = std::fs::read_dir(cache_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();

    if entries.len() <= max_count {
        return Ok(());
    }

    // 按修改时间排序（最旧的在前）
    entries.sort_by(|a, b| {
        let a_time = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_time = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        a_time.cmp(&b_time)
    });

    // 删除最旧的缓存
    let to_remove = entries.len() - max_count;
    for entry in entries.iter().take(to_remove) {
        let _ = std::fs::remove_file(entry.path());
    }

    Ok(())
}
