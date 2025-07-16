#![allow(dead_code)]
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use crate::errors::TreeSitterError;

/// 查询源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySource {
    /// 源名称
    pub name: String,
    /// 基础URL
    pub base_url: String,
    /// 版本或commit hash
    pub version: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 优先级 (数字越小优先级越高)
    pub priority: u8,
}

/// 查询文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// 语言名称
    pub language: String,
    /// 查询类型 (highlights, injections, locals, etc.)
    pub query_type: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 源名称
    pub source: String,
    /// 下载时间
    pub downloaded_at: SystemTime,
    /// 文件哈希
    pub file_hash: String,
    /// 版本
    pub version: Option<String>,
}

/// 查询管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryManagerConfig {
    /// 查询源列表
    pub sources: Vec<QuerySource>,
    /// 本地缓存目录
    pub cache_dir: PathBuf,
    /// 缓存过期时间(秒)
    pub cache_ttl: u64,
    /// 是否启用自动更新
    pub auto_update: bool,
    /// 更新检查间隔(秒)
    pub update_interval: u64,
    /// 网络超时时间(秒)
    pub network_timeout: u64,
}

impl Default for QueryManagerConfig {
    fn default() -> Self {
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cache")
            .join("gitai")
            .join("queries");

        Self {
            sources: vec![
                QuerySource {
                    name: "nvim-treesitter".to_string(),
                    base_url: "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/master/queries".to_string(),
                    version: None,
                    enabled: true,
                    priority: 1,
                },
                QuerySource {
                    name: "helix-editor".to_string(),
                    base_url: "https://raw.githubusercontent.com/helix-editor/helix/master/runtime/queries".to_string(),
                    version: None,
                    enabled: false,
                    priority: 2,
                },
                QuerySource {
                    name: "zed-editor".to_string(),
                    base_url: "https://raw.githubusercontent.com/zed-industries/zed/main/assets/queries".to_string(),
                    version: None,
                    enabled: false,
                    priority: 3,
                },
            ],
            cache_dir,
            cache_ttl: 24 * 60 * 60, // 24小时
            auto_update: true,
            update_interval: 7 * 24 * 60 * 60, // 7天
            network_timeout: 30, // 30秒
        }
    }
}

/// 查询管理器
pub struct QueryManager {
    config: QueryManagerConfig,
    metadata: HashMap<String, QueryMetadata>,
    http_client: reqwest::Client,
}

impl QueryManager {
    /// 创建新的查询管理器
    pub fn new(config: QueryManagerConfig) -> Result<Self, TreeSitterError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.network_timeout))
            .build()
            .map_err(|e| TreeSitterError::QueryError(format!("Failed to create HTTP client: {}", e)))?;

        // 确保缓存目录存在
        fs::create_dir_all(&config.cache_dir)
            .map_err(|e| TreeSitterError::IOError(e))?;

        let mut manager = Self {
            config,
            metadata: HashMap::new(),
            http_client,
        };

        // 加载现有的元数据
        manager.load_metadata()?;

        Ok(manager)
    }

    /// 获取查询文件内容
    pub async fn get_query(&mut self, language: &str, query_type: &str) -> Result<String, TreeSitterError> {
        let key = format!("{}:{}", language, query_type);

        // 检查缓存
        if let Some(metadata) = self.metadata.get(&key) {
            if self.is_cache_valid(&metadata) {
                return self.load_cached_query(&metadata);
            }
        }

        // 克隆sources以避免借用检查问题
        let sources = self.config.sources.clone();
        
        // 尝试从各个源下载
        for source in &sources {
            if !source.enabled {
                continue;
            }

            match self.download_query_from_source(source, language, query_type).await {
                Ok(content) => {
                    // 缓存成功下载的查询
                    self.cache_query(source, language, query_type, &content)?;
                    return Ok(content);
                }
                Err(e) => {
                    tracing::warn!("Failed to download query from {}: {}", source.name, e);
                    continue;
                }
            }
        }

        // 如果所有源都失败，尝试使用本地缓存
        if let Some(metadata) = self.metadata.get(&key) {
            tracing::warn!("Using expired cache for {}:{}", language, query_type);
            return self.load_cached_query(&metadata);
        }

        Err(TreeSitterError::QueryError(format!(
            "Failed to get query for {}:{} from any source",
            language, query_type
        )))
    }

    /// 从源下载查询
    async fn download_query_from_source(
        &self,
        source: &QuerySource,
        language: &str,
        query_type: &str,
    ) -> Result<String, TreeSitterError> {
        let url = format!(
            "{}/{}/{}.scm",
            source.base_url, language, query_type
        );

        tracing::info!("Downloading query from: {}", url);

        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| TreeSitterError::QueryError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TreeSitterError::QueryError(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| TreeSitterError::QueryError(format!("Failed to read response: {}", e)))?;

        Ok(content)
    }

    /// 缓存查询文件
    fn cache_query(
        &mut self,
        source: &QuerySource,
        language: &str,
        query_type: &str,
        content: &str,
    ) -> Result<(), TreeSitterError> {
        let key = format!("{}:{}", language, query_type);
        
        // 创建语言目录
        let lang_dir = self.config.cache_dir.join(language);
        fs::create_dir_all(&lang_dir)
            .map_err(|e| TreeSitterError::IOError(e))?;

        // 写入查询文件
        let file_path = lang_dir.join(format!("{}.scm", query_type));
        fs::write(&file_path, content)
            .map_err(|e| TreeSitterError::IOError(e))?;

        // 计算文件哈希
        let file_hash = self.calculate_hash(content);

        // 更新元数据
        let metadata = QueryMetadata {
            language: language.to_string(),
            query_type: query_type.to_string(),
            file_path,
            source: source.name.clone(),
            downloaded_at: SystemTime::now(),
            file_hash,
            version: source.version.clone(),
        };

        self.metadata.insert(key, metadata);

        // 保存元数据
        self.save_metadata()?;

        Ok(())
    }

    /// 加载缓存的查询文件
    fn load_cached_query(&self, metadata: &QueryMetadata) -> Result<String, TreeSitterError> {
        fs::read_to_string(&metadata.file_path)
            .map_err(|e| TreeSitterError::IOError(e))
    }

    /// 检查缓存是否有效
    fn is_cache_valid(&self, metadata: &QueryMetadata) -> bool {
        if let Ok(elapsed) = metadata.downloaded_at.elapsed() {
            elapsed.as_secs() < self.config.cache_ttl
        } else {
            false
        }
    }

    /// 计算内容哈希
    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 加载元数据
    fn load_metadata(&mut self) -> Result<(), TreeSitterError> {
        let metadata_file = self.config.cache_dir.join("metadata.json");
        
        if metadata_file.exists() {
            let content = fs::read_to_string(&metadata_file)
                .map_err(|e| TreeSitterError::IOError(e))?;
            
            self.metadata = serde_json::from_str(&content)
                .map_err(|e| TreeSitterError::QueryError(format!("Failed to parse metadata: {}", e)))?;
        }

        Ok(())
    }

    /// 保存元数据
    fn save_metadata(&self) -> Result<(), TreeSitterError> {
        let metadata_file = self.config.cache_dir.join("metadata.json");
        
        let content = serde_json::to_string_pretty(&self.metadata)
            .map_err(|e| TreeSitterError::QueryError(format!("Failed to serialize metadata: {}", e)))?;
        
        fs::write(&metadata_file, content)
            .map_err(|e| TreeSitterError::IOError(e))?;

        Ok(())
    }

    /// 清理过期缓存
    pub fn cleanup_cache(&mut self) -> Result<(), TreeSitterError> {
        let now = SystemTime::now();
        let mut to_remove = Vec::new();

        for (key, metadata) in &self.metadata {
            if let Ok(elapsed) = now.duration_since(metadata.downloaded_at) {
                if elapsed.as_secs() > self.config.cache_ttl * 2 {
                    // 删除超过两倍TTL的缓存
                    to_remove.push(key.clone());
                    
                    // 删除文件
                    if metadata.file_path.exists() {
                        fs::remove_file(&metadata.file_path)
                            .map_err(|e| TreeSitterError::IOError(e))?;
                    }
                }
            }
        }

        for key in to_remove {
            self.metadata.remove(&key);
        }

        self.save_metadata()?;
        Ok(())
    }

    /// 强制更新所有查询
    pub async fn force_update_all(&mut self) -> Result<(), TreeSitterError> {
        let keys: Vec<String> = self.metadata.keys().cloned().collect();
        
        for key in keys {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                let language = parts[0];
                let query_type = parts[1];
                
                // 强制重新下载
                self.metadata.remove(&key);
                if let Err(e) = self.get_query(language, query_type).await {
                    tracing::warn!("Failed to update query {}:{}: {}", language, query_type, e);
                }
            }
        }

        Ok(())
    }

    /// 获取支持的语言列表
    pub fn get_supported_languages(&self) -> Vec<String> {
        self.metadata
            .values()
            .map(|m| m.language.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// 获取语言的可用查询类型
    pub fn get_query_types_for_language(&self, language: &str) -> Vec<String> {
        self.metadata
            .values()
            .filter(|m| m.language == language)
            .map(|m| m.query_type.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_query_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = QueryManagerConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = QueryManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_calculate_hash() {
        let temp_dir = TempDir::new().unwrap();
        let config = QueryManagerConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = QueryManager::new(config).unwrap();
        let hash1 = manager.calculate_hash("test content");
        let hash2 = manager.calculate_hash("test content");
        let hash3 = manager.calculate_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}