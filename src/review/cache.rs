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

    Ok(())
}
