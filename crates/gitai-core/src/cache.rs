// 缓存模块
// TODO: 实现统一的缓存策略

use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct CacheManager {
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.read().get(key).cloned()
    }
    
    pub fn set(&self, key: String, value: Vec<u8>) {
        self.cache.write().insert(key, value);
    }
    
    pub fn clear(&self) {
        self.cache.write().clear();
    }
}

impl Default for CacheManager {
    fn default() -> Self { Self::new() }
}
