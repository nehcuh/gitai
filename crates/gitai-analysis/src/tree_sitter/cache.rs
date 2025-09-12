// Tree-sitter 分析缓存
// 使用 LRU 缓存策略，避免重复解析相同代码

use crate::tree_sitter::StructuralSummary;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// 缓存键 - 基于内容哈希和语言
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    /// 内容哈希值
    pub content_hash: String,
    /// 语言类型
    pub language: String,
}

impl CacheKey {
    /// 从代码内容创建缓存键
    pub fn from_content(content: &str, language: &str) -> Self {
        let content_hash = format!("{:x}", md5::compute(content.as_bytes()));
        Self {
            content_hash,
            language: language.to_string(),
        }
    }
}

/// 缓存项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 结构分析结果
    pub summary: StructuralSummary,
    /// 创建时间戳
    pub timestamp: u64,
    /// 访问次数
    pub access_count: u32,
}

impl CacheEntry {
    /// 创建新的缓存项
    pub fn new(summary: StructuralSummary) -> Self {
        let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };
        Self {
            summary,
            timestamp: ts,
            access_count: 1,
        }
    }

    /// 检查缓存是否过期
    pub fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };
        now.saturating_sub(self.timestamp) > max_age_seconds
    }

    /// 更新访问计数
    pub fn touch(&mut self) {
        self.access_count += 1;
    }
}

/// Tree-sitter 分析缓存管理器
pub struct TreeSitterCache {
    /// 内存缓存 (LRU)
    memory_cache: Arc<Mutex<LruCache<CacheKey, CacheEntry>>>,
    /// 缓存目录
    cache_dir: std::path::PathBuf,
    /// 最大缓存年龄（秒）
    max_age_seconds: u64,
    /// 统计信息
    stats: Arc<Mutex<CacheStats>>,
}

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub disk_hits: u64,
    pub disk_misses: u64,
}

impl CacheStats {
    /// 计算命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl TreeSitterCache {
    /// 创建新的缓存管理器
    pub fn new(
        capacity: usize,
        max_age_seconds: u64,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 使用统一的路径解析方案
        let base_dir = crate::utils::paths::tree_sitter_cache_dir();

        // 测试环境隔离：为每次测试实例使用独立子目录，避免并发测试干扰
        // 但允许通过环境变量在测试中共享目录以便进行并发磁盘写入测试：
        // - GITAI_TS_CACHE_TEST_SHARED=true    使用共享子目录 shared_test_cache
        // - GITAI_TS_CACHE_TEST_SHARED=<name>  使用指定子目录名
        let cache_dir = if cfg!(test) {
            match std::env::var("GITAI_TS_CACHE_TEST_SHARED") {
                Ok(val) => {
                    let v = val.to_lowercase();
                    if v == "1" || v == "true" {
                        base_dir.join("shared_test_cache")
                    } else if !v.is_empty() {
                        base_dir.join(v)
                    } else {
                        let millis = match std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                        {
                            Ok(d) => d.as_millis(),
                            Err(_) => 0,
                        };
                        let unique = format!("test_{millis}");
                        base_dir.join(unique)
                    }
                }
                Err(_) => {
                    let millis =
                        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                            Ok(d) => d.as_millis(),
                            Err(_) => 0,
                        };
                    let unique = format!("test_{millis}");
                    base_dir.join(unique)
                }
            }
        } else {
            base_dir
        };

        std::fs::create_dir_all(&cache_dir)?;

        let capacity = match NonZeroUsize::new(capacity) {
            Some(nz) => nz,
            None => unsafe { NonZeroUsize::new_unchecked(100) },
        };

        Ok(Self {
            memory_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            cache_dir,
            max_age_seconds,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        })
    }

    /// 获取缓存项
    pub fn get(&self, key: &CacheKey) -> Option<StructuralSummary> {
        // 首先检查内存缓存
        if let Ok(mut cache) = self.memory_cache.lock() {
            if let Some(entry) = cache.get_mut(key) {
                if !entry.is_expired(self.max_age_seconds) {
                    entry.touch();
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.hits += 1;
                    }
                    log::debug!("缓存命中 (内存): {}", key.content_hash);
                    return Some(entry.summary.clone());
                } else {
                    // 过期的缓存项，移除
                    cache.pop(key);
                }
            }
        }

        // 尝试从磁盘加载
        if let Some(entry) = self.load_from_disk(key) {
            if !entry.is_expired(self.max_age_seconds) {
                // 加载到内存缓存
                if let Ok(mut cache) = self.memory_cache.lock() {
                    cache.put(key.clone(), entry.clone());
                }
                if let Ok(mut stats) = self.stats.lock() {
                    stats.disk_hits += 1;
                }
                log::debug!("缓存命中 (磁盘): {}", key.content_hash);
                return Some(entry.summary);
            } else {
                // 删除过期的磁盘缓存
                let _ = self.remove_from_disk(key);
            }
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.misses += 1;
        }
        log::debug!("缓存未命中: {}", key.content_hash);
        None
    }

    /// 设置缓存项
    pub fn set(
        &self,
        key: CacheKey,
        summary: StructuralSummary,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let entry = CacheEntry::new(summary);

        // 保存到内存缓存
        if let Ok(mut cache) = self.memory_cache.lock() {
            if let Some(_evicted) = cache.push(key.clone(), entry.clone()) {
                if let Ok(mut stats) = self.stats.lock() {
                    stats.evictions += 1;
                }
            }
        }

        // 异步保存到磁盘
        self.save_to_disk(&key, &entry)?;

        log::debug!("缓存保存: {}", key.content_hash);
        Ok(())
    }

    /// 清除所有缓存
    pub fn clear(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 清除内存缓存
        if let Ok(mut cache) = self.memory_cache.lock() {
            cache.clear();
        }

        // 清除磁盘缓存：删除目录并重建，避免并发测试下的残留
        if self.cache_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.cache_dir) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(Box::new(e));
                }
            }
        }
        std::fs::create_dir_all(&self.cache_dir)?;

        // 重置统计
        if let Ok(mut stats) = self.stats.lock() {
            stats.reset();
        }

        log::info!("缓存已清除");
        Ok(())
    }

    /// 获取统计信息
    pub fn stats(&self) -> CacheStats {
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| CacheStats::default())
    }

    /// 返回当前缓存配置（capacity, max_age_seconds）
    pub fn settings(&self) -> (usize, u64) {
        let cap = self.memory_cache.lock().map(|c| c.cap().get()).unwrap_or(0);
        (cap, self.max_age_seconds)
    }

    /// 预热缓存
    pub fn warm_up(
        &self,
        items: Vec<(CacheKey, StructuralSummary)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (key, summary) in items {
            self.set(key, summary)?;
        }
        log::info!("缓存预热完成");
        Ok(())
    }

    /// 从磁盘加载缓存
    fn load_from_disk(&self, key: &CacheKey) -> Option<CacheEntry> {
        let file_path = self.cache_file_path(key);

        if !file_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                match serde_json::from_str::<CacheEntry>(&content) {
                    Ok(entry) => Some(entry),
                    Err(e) => {
                        log::warn!("缓存文件解析失败: {e}");
                        // 删除损坏的缓存文件
                        let _ = std::fs::remove_file(&file_path);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("缓存文件读取失败: {e}");
                None
            }
        }
    }

    /// 保存到磁盘
    fn save_to_disk(
        &self,
        key: &CacheKey,
        entry: &CacheEntry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.cache_file_path(key);

        let content = serde_json::to_string_pretty(entry)?;
        Self::write_atomic(&file_path, &content)?;

        Ok(())
    }

    /// 原子写入文件：写入到同目录的临时文件并原子替换目标文件
    fn write_atomic_target(file_path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;
        let dir = file_path
            .parent()
            .ok_or_else(|| std::io::Error::other("no parent dir"))?;
        let fname = file_path
            .file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| "cache".into());
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        let tmp_name = format!(".{fname}.{now}.tmp");
        let tmp_path = dir.join(tmp_name);

        {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;
            f.write_all(bytes)?;
            // 尽力刷新到磁盘
            let _ = f.sync_all();
        }

        // 在类 Unix 系统上，rename 是原子的并会替换目标文件
        std::fs::rename(&tmp_path, file_path)?;
        Ok(())
    }

    /// 便捷方法：将字符串内容原子写入文件
    fn write_atomic(file_path: &std::path::Path, content: &str) -> std::io::Result<()> {
        Self::write_atomic_target(file_path, content.as_bytes())
    }

    /// 从磁盘删除缓存
    fn remove_from_disk(
        &self,
        key: &CacheKey,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.cache_file_path(key);

        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
        }

        Ok(())
    }

    /// 获取缓存文件路径
    fn cache_file_path(&self, key: &CacheKey) -> std::path::PathBuf {
        let filename = format!("{}_{}.json", key.language, key.content_hash);
        self.cache_dir.join(filename)
    }

    /// 清理过期缓存
    pub fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut cleaned = 0;

        // 清理内存缓存中的过期项
        if let Ok(mut cache) = self.memory_cache.lock() {
            let keys_to_remove: Vec<_> = cache
                .iter()
                .filter(|(_, entry)| entry.is_expired(self.max_age_seconds))
                .map(|(key, _)| key.clone())
                .collect();

            for key in keys_to_remove {
                cache.pop(&key);
                cleaned += 1;
            }
        }

        // 清理磁盘缓存中的过期项
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&content) {
                            if cache_entry.is_expired(self.max_age_seconds) {
                                std::fs::remove_file(&path)?;
                                cleaned += 1;
                            }
                        }
                    }
                }
            }
        }

        log::info!("清理了 {cleaned} 个过期缓存项");
        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_creation() {
        let key1 = CacheKey::from_content("test code", "rust");
        let key2 = CacheKey::from_content("test code", "rust");
        let key3 = CacheKey::from_content("other code", "rust");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let summary = StructuralSummary {
            language: "rust".to_string(),
            language_summaries: std::collections::HashMap::new(),
            functions: vec![],
            classes: vec![],
            imports: vec![],
            exports: vec![],
            comments: vec![],
            complexity_hints: vec![],
            calls: vec![],
        };

        let mut entry = CacheEntry::new(summary);
        entry.timestamp = 0; // 设置为很久以前

        assert!(entry.is_expired(10));
        assert!(!CacheEntry::new(StructuralSummary::default()).is_expired(3600));
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats {
            hits: 75,
            misses: 25,
            ..Default::default()
        };

        assert_eq!(stats.hit_rate(), 0.75);

        stats.reset();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = TreeSitterCache::new(10, 3600).unwrap();

        let key = CacheKey::from_content("test code", "rust");
        let summary = StructuralSummary {
            language: "rust".to_string(),
            language_summaries: std::collections::HashMap::new(),
            functions: vec![],
            classes: vec![],
            imports: vec![],
            exports: vec![],
            comments: vec![],
            complexity_hints: vec![],
            calls: vec![],
        };

        // 测试缓存未命中
        assert!(cache.get(&key).is_none());

        // 测试设置缓存
        cache.set(key.clone(), summary.clone()).unwrap();

        // 测试缓存命中
        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().language, "rust");

        // 测试统计
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);

        // 测试清除缓存
        cache.clear().unwrap();
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_settings_returns_config() {
        let cap = 123;
        let max_age = 4567u64;
        let cache = TreeSitterCache::new(cap, max_age).expect("cache new");
        let (actual_cap, actual_age) = cache.settings();
        assert_eq!(actual_cap, cap);
        assert_eq!(actual_age, max_age);
    }
}
