use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tree_sitter::SupportedLanguage;

// Default Tree-sitter queries URL (fallback when config is not available)
const DEFAULT_NVIM_TREESITTER_BASE: &str =
    "https://raw.githubusercontent.com/nvim-treesitter/nvim-treesitter/master/queries";
const QUERY_FILES: &[&str] = &[
    "highlights.scm",
    "locals.scm",
    "injections.scm",
    "folds.scm",
    "indents.scm",
];

/// Queries管理器，负责下载和缓存Tree-sitter查询文件
pub struct QueriesManager {
    cache_dir: PathBuf,
    queries: HashMap<SupportedLanguage, LanguageQueries>,
    tree_sitter_base_url: String,
}

/// 单个语言的查询集合
#[derive(Debug, Clone, Default)]
pub struct LanguageQueries {
    pub highlights: Option<String>,
    pub locals: Option<String>,
    pub injections: Option<String>,
    pub folds: Option<String>,
    pub indents: Option<String>,
}

impl QueriesManager {
    /// 创建新的查询管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_base_url(None)
    }

    /// 创建带自定义基础URL的查询管理器
    pub fn with_base_url(base_url: Option<String>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".cache"))
            .join("gitai")
            .join("tree-sitter-queries");

        // 从配置文件中读取 tree_sitter_url，如果无法读取则使用默认值或传入的 base_url
        let tree_sitter_base_url = base_url
            .or_else(|| Self::load_tree_sitter_url_from_config())
            .unwrap_or_else(|| DEFAULT_NVIM_TREESITTER_BASE.to_string());

        // 如果来自配置的是git仓库URL，转换为raw URL
        let tree_sitter_base_url = if tree_sitter_base_url.contains("github.com") && tree_sitter_base_url.ends_with(".git") {
            tree_sitter_base_url
                .replace(".git", "")
                .replace("github.com/", "raw.githubusercontent.com/")
                + "/master/queries"
        } else if tree_sitter_base_url.contains("github.com") && !tree_sitter_base_url.contains("raw.githubusercontent.com") {
            tree_sitter_base_url.replace("github.com/", "raw.githubusercontent.com/") + "/master/queries"
        } else {
            tree_sitter_base_url
        };

        log::debug!(
            "创建 Tree-sitter 查询管理器，缓存目录: {}，基础URL: {}",
            cache_dir.display(),
            tree_sitter_base_url
        );

        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            let error = format!(
                "Failed to create cache directory {}: {e}",
                cache_dir.display()
            );
            log::error!("{error}");
            Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error + Send + Sync>
        })?;

        log::info!("Tree-sitter 查询管理器初始化成功");

        Ok(Self {
            cache_dir,
            queries: HashMap::new(),
            tree_sitter_base_url,
        })
    }

    /// 从配置文件中加载 tree_sitter_url
    fn load_tree_sitter_url_from_config() -> Option<String> {
        let config_path = dirs::home_dir()?
            .join(".config")
            .join("gitai")
            .join("config.toml");

        if !config_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&config_path).ok()?;
        let config: toml::Value = toml::from_str(&content).ok()?;
        
        config
            .get("sources")?
            .get("tree_sitter_url")?
            .as_str()
            .map(|s| s.to_string())
    }

    /// 确保所有支持的语言的queries已下载
    pub async fn ensure_queries_downloaded(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for lang in SupportedLanguage::all() {
            self.ensure_language_queries(lang).await?;
        }
        Ok(())
    }

    /// 确保特定语言的queries已下载
    pub async fn ensure_language_queries(
        &self,
        language: SupportedLanguage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let lang_dir = self.cache_dir.join(language.name());

        // 检查是否已存在
        if lang_dir.exists() && self.is_queries_complete(&lang_dir) {
            log::debug!("Queries for {} already cached", language.name());
            return Ok(());
        }

        // 创建语言目录
        fs::create_dir_all(&lang_dir)?;

        log::info!("Downloading queries for {}", language.name());

        // 下载每个查询文件
        for query_file in QUERY_FILES {
            let url = format!(
                "{}/{}/{}",
                self.tree_sitter_base_url,
                language.name(),
                query_file
            );
            let file_path = lang_dir.join(query_file);

            match self.download_query_file(&url).await {
                Ok(content) => {
                    fs::write(&file_path, content)?;
                    log::debug!("Downloaded {} for {}", query_file, language.name());
                }
                Err(e) => {
                    // 某些语言可能没有所有的查询文件，这是正常的
                    log::debug!(
                        "Could not download {} for {}: {}",
                        query_file,
                        language.name(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// 下载单个查询文件
    async fn download_query_file(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(format!("Failed to download from {}: {}", url, response.status()).into())
        }
    }

    /// 检查查询文件是否完整（至少有highlights.scm）
    fn is_queries_complete(&self, lang_dir: &Path) -> bool {
        lang_dir.join("highlights.scm").exists()
    }

    /// 加载特定语言的查询
    pub fn load_language_queries(
        &mut self,
        language: SupportedLanguage,
    ) -> Result<&LanguageQueries, Box<dyn std::error::Error + Send + Sync>> {
        // 如果已加载，直接返回
        if self.queries.contains_key(&language) {
            return Ok(&self.queries[&language]);
        }

        let lang_dir = self.cache_dir.join(language.name());
        let mut queries = LanguageQueries::default();

        // 加载各个查询文件
        if let Ok(content) = fs::read_to_string(lang_dir.join("highlights.scm")) {
            queries.highlights = Some(content);
        }
        if let Ok(content) = fs::read_to_string(lang_dir.join("locals.scm")) {
            queries.locals = Some(content);
        }
        if let Ok(content) = fs::read_to_string(lang_dir.join("injections.scm")) {
            queries.injections = Some(content);
        }
        if let Ok(content) = fs::read_to_string(lang_dir.join("folds.scm")) {
            queries.folds = Some(content);
        }
        if let Ok(content) = fs::read_to_string(lang_dir.join("indents.scm")) {
            queries.indents = Some(content);
        }

        self.queries.insert(language, queries);
        Ok(&self.queries[&language])
    }

    /// 获取查询内容
    pub fn get_query(
        &mut self,
        language: SupportedLanguage,
        query_type: QueryType,
    ) -> Option<String> {
        self.load_language_queries(language).ok()?;

        let queries = self.queries.get(&language)?;
        match query_type {
            QueryType::Highlights => queries.highlights.clone(),
            QueryType::Locals => queries.locals.clone(),
            QueryType::Injections => queries.injections.clone(),
            QueryType::Folds => queries.folds.clone(),
            QueryType::Indents => queries.indents.clone(),
        }
    }

    /// 清理缓存
    pub fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// 获取缓存目录
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

/// 查询类型
#[derive(Debug, Clone, Copy)]
pub enum QueryType {
    Highlights,
    Locals,
    Injections,
    Folds,
    Indents,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queries_download() {
        let manager = QueriesManager::new().unwrap();

        // 测试下载Java queries
        manager
            .ensure_language_queries(SupportedLanguage::Java)
            .await
            .unwrap();

        // 验证文件存在
        let java_dir = manager.cache_dir.join("java");
        assert!(java_dir.exists());
        assert!(java_dir.join("highlights.scm").exists());
    }
}
