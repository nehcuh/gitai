// 缓存模块
// TODO: 实现统一的缓存策略

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// 简单的内存缓存管理器（开发期占位实现）
pub struct CacheManager {
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取键的值（若存在，返回克隆）
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.read().get(key).cloned()
    }

    /// 设置键的值（覆盖）
    pub fn set(&self, key: String, value: Vec<u8>) {
        self.cache.write().insert(key, value);
    }

    /// 清空全部缓存
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}
