#![allow(missing_docs)]
pub mod analyzer;
// Note: analyzer.rs has been refactored into analyzer/ directory
pub mod cache;
pub mod custom_queries;
pub mod helpers;
pub mod queries;
pub mod unified_analyzer;

pub use cache::CacheStats;
use cache::{CacheKey, TreeSitterCache};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tree_sitter::{Language, Parser};

/// 支持的编程语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SupportedLanguage {
    Java,
    Rust,
    C,
    Cpp,
    Python,
    Go,
    JavaScript,
    TypeScript,
}

impl SupportedLanguage {
    /// 从文件扩展名推断语言
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "java" => Some(Self::Java),
            "rs" => Some(Self::Rust),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            "py" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            _ => None,
        }
    }

    /// 从语言名称推断语言
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "java" => Some(Self::Java),
            "rust" | "rs" => Some(Self::Rust),
            "c" => Some(Self::C),
            "cpp" | "c++" => Some(Self::Cpp),
            "python" | "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "javascript" | "js" => Some(Self::JavaScript),
            "typescript" | "ts" => Some(Self::TypeScript),
            _ => None,
        }
    }

    /// 获取语言名称（用于下载queries）
    pub fn name(&self) -> &'static str {
        match self {
            Self::Java => "java",
            Self::Rust => "rust",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Python => "python",
            Self::Go => "go",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
        }
    }

    /// 获取Tree-sitter语言对象
    pub fn language(&self) -> Option<Language> {
        match self {
            #[cfg(feature = "tree-sitter-java")]
            Self::Java => Some(tree_sitter_java::language()),
            #[cfg(not(feature = "tree-sitter-java"))]
            Self::Java => None,

            #[cfg(feature = "tree-sitter-rust")]
            Self::Rust => Some(tree_sitter_rust::language()),
            #[cfg(not(feature = "tree-sitter-rust"))]
            Self::Rust => None,

            #[cfg(feature = "tree-sitter-c")]
            Self::C => Some(tree_sitter_c::language()),
            #[cfg(not(feature = "tree-sitter-c"))]
            Self::C => None,

            #[cfg(feature = "tree-sitter-cpp")]
            Self::Cpp => Some(tree_sitter_cpp::language()),
            #[cfg(not(feature = "tree-sitter-cpp"))]
            Self::Cpp => None,

            #[cfg(feature = "tree-sitter-python")]
            Self::Python => Some(tree_sitter_python::language()),
            #[cfg(not(feature = "tree-sitter-python"))]
            Self::Python => None,

            #[cfg(feature = "tree-sitter-go")]
            Self::Go => Some(tree_sitter_go::language()),
            #[cfg(not(feature = "tree-sitter-go"))]
            Self::Go => None,

            #[cfg(feature = "tree-sitter-javascript")]
            Self::JavaScript => Some(tree_sitter_javascript::language()),
            #[cfg(not(feature = "tree-sitter-javascript"))]
            Self::JavaScript => None,

            #[cfg(feature = "tree-sitter-typescript")]
            Self::TypeScript => Some(tree_sitter_typescript::language_typescript()),
            #[cfg(not(feature = "tree-sitter-typescript"))]
            Self::TypeScript => None,
        }
    }

    /// 获取所有支持的语言
    pub fn all() -> Vec<Self> {
        vec![
            Self::Java,
            Self::Rust,
            Self::C,
            Self::Cpp,
            Self::Python,
            Self::Go,
            Self::JavaScript,
            Self::TypeScript,
        ]
    }
}

/// Tree-sitter管理器
pub struct TreeSitterManager {
    parsers: HashMap<SupportedLanguage, Parser>,
    queries_manager: queries::QueriesManager,
    cache: Option<TreeSitterCache>,
}

impl TreeSitterManager {
    /// 创建新的管理器
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut parsers = HashMap::new();
        let queries_manager = queries::QueriesManager::new()?;

        // 初始化所有语言的解析器（仅已启用的）
        for lang in SupportedLanguage::all() {
            if let Some(language) = lang.language() {
                let mut parser = Parser::new();
                parser.set_language(language)?;
                parsers.insert(lang, parser);
            }
        }

        // 确保queries已下载
        queries_manager.ensure_queries_downloaded().await?;

        // 初始化缓存（可通过环境变量覆盖默认值）
        // GITAI_TS_CACHE_CAPACITY: usize (默认 100)
        // GITAI_TS_CACHE_MAX_AGE: u64 秒 (默认 3600)
        let default_capacity = 100usize;
        let default_max_age = 3600u64;
        let capacity = std::env::var("GITAI_TS_CACHE_CAPACITY")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(default_capacity);
        let max_age = std::env::var("GITAI_TS_CACHE_MAX_AGE")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(default_max_age);
        let cache = TreeSitterCache::new(capacity, max_age).ok();

        Ok(Self {
            parsers,
            queries_manager,
            cache,
        })
    }

    /// 获取缓存统计（如果启用了缓存）
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    /// 清空缓存（内存+磁盘），若未启用缓存则为 no-op
    pub fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref cache) = self.cache {
            cache.clear()?;
        }
        Ok(())
    }

    /// 获取缓存配置（capacity, max_age_seconds）
    pub fn cache_settings(&self) -> Option<(usize, u64)> {
        self.cache.as_ref().map(|c| c.settings())
    }

    /// 并发分析多个文件
    ///
    /// # Arguments
    /// * `file_paths` - 要分析的文件路径列表
    /// * `max_concurrent` - 最大并发数（默认为 4）
    ///
    /// # Returns
    /// 返回每个文件的分析结果，包含文件路径和分析摘要
    pub async fn analyze_files_concurrent(
        &self,
        file_paths: Vec<PathBuf>,
        max_concurrent: Option<usize>,
    ) -> Result<Vec<FileAnalysisResult>, Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::stream::{self, StreamExt};

        let pool_size = max_concurrent.unwrap_or(4).max(1);
        let start_time = std::time::Instant::now();

        log::info!(
            "开始并发分析 {} 个文件，工作线程数: {}",
            file_paths.len(),
            pool_size
        );

        // 构建管理器池（限制并发）
        let (man_tx, man_rx) = tokio::sync::mpsc::channel::<TreeSitterManager>(pool_size);
        for _ in 0..pool_size {
            let mgr = TreeSitterManager::new().await?;
            // 忽略发送失败（不会发生，因为接收端在下面）
            let _ = man_tx.send(mgr).await;
        }
        let man_rx = Arc::new(tokio::sync::Mutex::new(man_rx));
        let man_tx = Arc::new(man_tx);

        // 使用 stream 以池大小并发处理文件
        let results: Vec<FileAnalysisResult> = stream::iter(file_paths)
            .map(|path| {
                let man_rx = man_rx.clone();
                let man_tx = man_tx.clone();
                async move {
                    // 获取一个可用的管理器
                    let mut mgr = {
                        let mut rx = man_rx.lock().await;
                        match rx.recv().await {
                            Some(m) => m,
                            None => return None, // 池不可用
                        }
                    };

                    // 读取文件内容
                    let content = match tokio::fs::read_to_string(&path).await {
                        Ok(c) => c,
                        Err(e) => {
                            log::debug!("读取文件失败 {}: {}", path.display(), e);
                            // 归还管理器
                            let _ = man_tx.send(mgr).await;
                            return None;
                        }
                    };

                    // 推断语言
                    let language = match path
                        .extension()
                        .and_then(|s| s.to_str())
                        .and_then(SupportedLanguage::from_extension)
                    {
                        Some(lang) => lang,
                        None => {
                            log::debug!("无法推断语言，跳过: {}", path.display());
                            let _ = man_tx.send(mgr).await;
                            return None;
                        }
                    };

                    // 分析
                    let file_start = std::time::Instant::now();
                    let result = mgr.analyze_structure(&content, language);
                    let analysis_time = file_start.elapsed().as_secs_f64();

                    // 归还管理器
                    let _ = man_tx.send(mgr).await;

                    match result {
                        Ok(summary) => {
                            log::debug!(
                                "文件 {} 分析完成，耗时: {:.2}秒",
                                path.display(),
                                analysis_time
                            );
                            Some(FileAnalysisResult {
                                file_path: path,
                                language,
                                summary,
                                analysis_time,
                            })
                        }
                        Err(e) => {
                            log::debug!("分析失败 {}: {}", path.display(), e);
                            None
                        }
                    }
                }
            })
            .buffer_unordered(pool_size)
            .filter_map(|r| async move { r })
            .collect()
            .await;

        let total_time = start_time.elapsed().as_secs_f64();
        let files_analyzed = results.len();
        let throughput = if total_time > 0.0 {
            files_analyzed as f64 / total_time
        } else {
            0.0
        };

        log::info!(
            "并发分析完成: {files_analyzed} 个文件成功分析，总耗时: {total_time:.2}秒，吞吐量: {throughput:.2} 文件/秒"
        );

        if files_analyzed > 0 {
            let avg_time = total_time / files_analyzed as f64;
            let max_time = results.iter().map(|r| r.analysis_time).fold(0.0, f64::max);
            let min_time = results
                .iter()
                .map(|r| r.analysis_time)
                .fold(f64::MAX, f64::min);
            log::info!(
                "性能统计 - 平均: {avg_time:.3}秒, 最快: {min_time:.3}秒, 最慢: {max_time:.3}秒"
            );
        }

        Ok(results)
    }

    /// 分析目录中的所有代码文件（并发）
    pub async fn analyze_directory_concurrent(
        &self,
        dir_path: &Path,
        language_filter: Option<SupportedLanguage>,
        max_concurrent: Option<usize>,
    ) -> Result<DirectoryAnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        use walkdir::WalkDir;

        let mut file_paths = Vec::new();

        // 收集所有需要分析的文件
        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // 跳过隐藏文件和常见的排除目录
            if path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .map(|s| s.starts_with('.') || s == "target" || s == "node_modules")
                    .unwrap_or(false)
            }) {
                continue;
            }

            // 检查文件扩展名
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if let Some(lang) = SupportedLanguage::from_extension(ext) {
                    // 应用语言过滤器
                    if language_filter.is_none() || language_filter == Some(lang) {
                        file_paths.push(path.to_path_buf());
                    }
                }
            }
        }

        log::info!(
            "在目录 {} 中找到 {} 个代码文件",
            dir_path.display(),
            file_paths.len()
        );

        // 并发分析所有文件
        let file_results = self
            .analyze_files_concurrent(file_paths, max_concurrent)
            .await?;

        // 按语言分组结果
        let mut language_statistics: HashMap<String, LanguageStatistics> = HashMap::new();
        let mut total_functions = 0;
        let mut total_classes = 0;
        let mut total_lines = 0;

        for result in &file_results {
            let lang_stats = language_statistics
                .entry(result.language.name().to_string())
                .or_insert_with(|| LanguageStatistics {
                    file_count: 0,
                    function_count: 0,
                    class_count: 0,
                    line_count: 0,
                    avg_analysis_time: 0.0,
                });

            lang_stats.file_count += 1;
            lang_stats.function_count += result.summary.functions.len();
            lang_stats.class_count += result.summary.classes.len();
            // 估算行数（通过函数和类的行范围）
            let mut max_line = 0;
            for func in &result.summary.functions {
                if func.line_end > max_line {
                    max_line = func.line_end;
                }
            }
            for class in &result.summary.classes {
                if class.line_end > max_line {
                    max_line = class.line_end;
                }
            }
            lang_stats.line_count += max_line;

            total_functions += result.summary.functions.len();
            total_classes += result.summary.classes.len();
            total_lines += max_line;
        }

        // 计算每种语言的平均分析时间
        for result in &file_results {
            if let Some(lang_stats) = language_statistics.get_mut(result.language.name()) {
                lang_stats.avg_analysis_time = (lang_stats.avg_analysis_time
                    * (lang_stats.file_count - 1) as f64
                    + result.analysis_time)
                    / lang_stats.file_count as f64;
            }
        }

        Ok(DirectoryAnalysisResult {
            directory: dir_path.to_path_buf(),
            language_statistics,
            total_files: file_results.len(),
            total_functions,
            total_classes,
            total_lines,
        })
    }

    /// 分析目录中的所有代码文件（并发，带 include/exclude 过滤）
    pub async fn analyze_directory_concurrent_with_filters(
        &self,
        dir_path: &Path,
        language_filter: Option<SupportedLanguage>,
        max_concurrent: Option<usize>,
        include_globs: Option<&[&str]>,
        exclude_globs: Option<&[&str]>,
    ) -> Result<DirectoryAnalysisResult, Box<dyn std::error::Error + Send + Sync>> {
        use walkdir::WalkDir;

        fn matches_pattern(pat: &str, text: &str) -> bool {
            // 简单通配符匹配：'*' 任意序列，'?' 单字符
            fn inner(p: &[u8], t: &[u8]) -> bool {
                if p.is_empty() {
                    return t.is_empty();
                }
                match p[0] {
                    b'*' => {
                        // 尝试匹配任意长度
                        for i in 0..=t.len() {
                            if inner(&p[1..], &t[i..]) {
                                return true;
                            }
                        }
                        false
                    }
                    b'?' => {
                        if t.is_empty() {
                            false
                        } else {
                            inner(&p[1..], &t[1..])
                        }
                    }
                    c => {
                        if !t.is_empty() && c == t[0] {
                            inner(&p[1..], &t[1..])
                        } else {
                            false
                        }
                    }
                }
            }
            inner(pat.as_bytes(), text.as_bytes())
        }

        fn match_any(patterns: &[&str], text: &str) -> bool {
            patterns.iter().any(|p| matches_pattern(p, text))
        }

        let mut file_paths = Vec::new();

        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // 跳过隐藏文件和常见的排除目录
            if path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .map(|s| s.starts_with('.') || s == "target" || s == "node_modules")
                    .unwrap_or(false)
            }) {
                continue;
            }

            // 计算相对路径用于匹配
            let rel = path
                .strip_prefix(dir_path)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if let Some(ex_patterns) = exclude_globs {
                if match_any(ex_patterns, &rel) {
                    continue;
                }
            }
            if let Some(in_patterns) = include_globs {
                if !in_patterns.is_empty() && !match_any(in_patterns, &rel) {
                    continue;
                }
            }

            // 检查文件扩展名
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if let Some(lang) = SupportedLanguage::from_extension(ext) {
                    if language_filter.is_none() || language_filter == Some(lang) {
                        file_paths.push(path.to_path_buf());
                    }
                }
            }
        }

        log::info!(
            "在目录 {} 中找到 {} 个代码文件 (包含过滤)",
            dir_path.display(),
            file_paths.len()
        );

        let file_results = self
            .analyze_files_concurrent(file_paths, max_concurrent)
            .await?;

        // 复用与基础方法相同的汇总逻辑
        let mut language_statistics: HashMap<String, LanguageStatistics> = HashMap::new();
        let mut total_functions = 0;
        let mut total_classes = 0;
        let mut total_lines = 0;

        for result in &file_results {
            let lang_stats = language_statistics
                .entry(result.language.name().to_string())
                .or_insert_with(|| LanguageStatistics {
                    file_count: 0,
                    function_count: 0,
                    class_count: 0,
                    line_count: 0,
                    avg_analysis_time: 0.0,
                });

            lang_stats.file_count += 1;
            lang_stats.function_count += result.summary.functions.len();
            lang_stats.class_count += result.summary.classes.len();
            let mut max_line = 0;
            for func in &result.summary.functions {
                if func.line_end > max_line {
                    max_line = func.line_end;
                }
            }
            for class in &result.summary.classes {
                if class.line_end > max_line {
                    max_line = class.line_end;
                }
            }
            lang_stats.line_count += max_line;

            total_functions += result.summary.functions.len();
            total_classes += result.summary.classes.len();
            total_lines += max_line;
        }

        for result in &file_results {
            if let Some(lang_stats) = language_statistics.get_mut(result.language.name()) {
                lang_stats.avg_analysis_time = (lang_stats.avg_analysis_time
                    * (lang_stats.file_count - 1) as f64
                    + result.analysis_time)
                    / lang_stats.file_count as f64;
            }
        }

        Ok(DirectoryAnalysisResult {
            directory: dir_path.to_path_buf(),
            language_statistics,
            total_files: file_results.len(),
            total_functions,
            total_classes,
            total_lines,
        })
    }

    /// 获取指定语言的解析器
    pub fn get_parser(&mut self, language: SupportedLanguage) -> Option<&mut Parser> {
        self.parsers.get_mut(&language)
    }

    /// 获取查询管理器
    pub fn queries(&self) -> &queries::QueriesManager {
        &self.queries_manager
    }

    /// 分析代码结构（支持缓存）
    pub fn analyze_structure(
        &mut self,
        code: &str,
        language: SupportedLanguage,
    ) -> Result<StructuralSummary, Box<dyn std::error::Error + Send + Sync>> {
        log::debug!(
            "开始分析 {:?} 语言代码，代码长度: {} 字符",
            language,
            code.len()
        );

        // 检查缓存
        if let Some(ref cache) = self.cache {
            let cache_key = CacheKey::from_content(code, language.name());
            if let Some(cached_summary) = cache.get(&cache_key) {
                log::info!("使用缓存的分析结果 - {language:?} 语言");
                return Ok(cached_summary);
            }
        }

        let parser = self.get_parser(language).ok_or_else(|| {
            let error = format!("Parser not found for language {language:?}");
            log::error!("{error}");
            error
        })?;

        let tree = parser.parse(code, None).ok_or_else(|| {
            let error = format!("Failed to parse {language:?} code");
            log::error!("{error}");
            error
        })?;

        log::debug!("Tree 解析成功，根节点: {}", tree.root_node().kind());

        // 使用新的统一分析器
        let analyzer = unified_analyzer::UnifiedAnalyzer::new(language).map_err(|e| {
            log::error!("Failed to create UnifiedAnalyzer for {language:?}: {e}");
            e
        })?;

        let result = analyzer.analyze(&tree, code.as_bytes()).map_err(|e| {
            log::error!("Failed to analyze structure for {language:?}: {e}");
            e
        })?;

        log::info!(
            "结构分析成功：{:?} 语言，函数: {}, 类: {}, 注释: {}",
            language,
            result.functions.len(),
            result.classes.len(),
            result.comments.len()
        );

        // 保存到缓存
        if let Some(ref cache) = self.cache {
            let cache_key = CacheKey::from_content(code, language.name());
            if let Err(e) = cache.set(cache_key, result.clone()) {
                log::warn!("缓存保存失败: {e}");
            }
        }

        Ok(result)
    }
}

/// 单个文件的分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileAnalysisResult {
    /// 文件路径
    pub file_path: PathBuf,
    /// 编程语言
    pub language: SupportedLanguage,
    /// 分析摘要
    pub summary: StructuralSummary,
    /// 分析耗时（秒）
    pub analysis_time: f64,
}

/// 语言统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LanguageStatistics {
    /// 该语言的文件数
    pub file_count: usize,
    /// 该语言的函数数
    pub function_count: usize,
    /// 该语言的类数
    pub class_count: usize,
    /// 该语言的代码行数
    pub line_count: usize,
    /// 该语言的平均分析时间（秒）
    pub avg_analysis_time: f64,
}

/// 目录分析结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DirectoryAnalysisResult {
    /// 目录路径
    pub directory: PathBuf,
    /// 按语言分组的统计结果
    pub language_statistics: HashMap<String, LanguageStatistics>,
    /// 总文件数
    pub total_files: usize,
    /// 总函数数
    pub total_functions: usize,
    /// 总类数
    pub total_classes: usize,
    /// 总代码行数
    pub total_lines: usize,
}

/// 代码结构摘要
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StructuralSummary {
    /// 单语言模式的语言标识（保持向后兼容）
    pub language: String,
    /// 多语言模式的分析结果
    pub language_summaries: std::collections::HashMap<String, LanguageSummary>,
    /// 单语言模式的分析结果（保持向后兼容）
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub complexity_hints: Vec<String>,
    pub calls: Vec<FunctionCallInfo>,
}

/// 单个语言的分析结果
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LanguageSummary {
    pub language: String,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub comments: Vec<CommentInfo>,
    pub complexity_hints: Vec<String>,
    pub calls: Vec<FunctionCallInfo>,
    /// 该语言涉及的文件数量
    pub file_count: usize,
}

impl StructuralSummary {
    /// 创建单语言模式的结构摘要
    pub fn single_language(language: String, summary: LanguageSummary) -> Self {
        let mut result = Self {
            language: language.clone(),
            language_summaries: std::collections::HashMap::new(),
            functions: summary.functions.clone(),
            classes: summary.classes.clone(),
            imports: summary.imports.clone(),
            exports: summary.exports.clone(),
            comments: summary.comments.clone(),
            complexity_hints: summary.complexity_hints.clone(),
            calls: summary.calls.clone(),
        };
        result.language_summaries.insert(language, summary);
        result
    }

    /// 创建多语言模式的结构摘要
    pub fn multi_language(
        language_summaries: std::collections::HashMap<String, LanguageSummary>,
    ) -> Self {
        let mut result = Self {
            language: "multi-language".to_string(),
            language_summaries,
            ..Default::default()
        };

        // 合并所有语言的结果以保持向后兼容
        for summary in result.language_summaries.values() {
            result.functions.extend(summary.functions.clone());
            result.classes.extend(summary.classes.clone());
            result.imports.extend(summary.imports.clone());
            result.exports.extend(summary.exports.clone());
            result.comments.extend(summary.comments.clone());
            result
                .complexity_hints
                .extend(summary.complexity_hints.clone());
            result.calls.extend(summary.calls.clone());
        }

        result
    }

    /// 获取所有检测到的语言
    pub fn detected_languages(&self) -> Vec<&str> {
        if self.language_summaries.is_empty() {
            vec![&self.language]
        } else {
            self.language_summaries.keys().map(|s| s.as_str()).collect()
        }
    }

    /// 检查是否为多语言模式
    pub fn is_multi_language(&self) -> bool {
        self.language_summaries.len() > 1
    }
}

impl LanguageSummary {
    /// 从旧的 StructuralSummary 转换
    pub fn from_structural_summary(summary: &StructuralSummary) -> Self {
        Self {
            language: summary.language.clone(),
            functions: summary.functions.clone(),
            classes: summary.classes.clone(),
            imports: summary.imports.clone(),
            exports: summary.exports.clone(),
            comments: summary.comments.clone(),
            complexity_hints: summary.complexity_hints.clone(),
            calls: summary.calls.clone(),
            file_count: 1,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub is_async: bool,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub fields: Vec<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub is_abstract: bool,
    pub extends: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionCallInfo {
    pub callee: String,
    pub line: usize,
    pub is_method: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommentInfo {
    pub text: String,
    pub line: usize,
    pub is_doc_comment: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_language_from_extension() {
        assert_eq!(
            SupportedLanguage::from_extension("java"),
            Some(SupportedLanguage::Java)
        );
        assert_eq!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            SupportedLanguage::from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("go"),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(
            SupportedLanguage::from_extension("c"),
            Some(SupportedLanguage::C)
        );
        assert_eq!(
            SupportedLanguage::from_extension("cpp"),
            Some(SupportedLanguage::Cpp)
        );
        assert_eq!(SupportedLanguage::from_extension("unknown"), None);
    }

    #[test]
    fn test_supported_language_name() {
        assert_eq!(SupportedLanguage::Java.name(), "java");
        assert_eq!(SupportedLanguage::Rust.name(), "rust");
        assert_eq!(SupportedLanguage::Python.name(), "python");
        assert_eq!(SupportedLanguage::JavaScript.name(), "javascript");
        assert_eq!(SupportedLanguage::TypeScript.name(), "typescript");
        assert_eq!(SupportedLanguage::Go.name(), "go");
        assert_eq!(SupportedLanguage::C.name(), "c");
        assert_eq!(SupportedLanguage::Cpp.name(), "cpp");
    }

    #[test]
    fn test_supported_language_all() {
        let all_langs = SupportedLanguage::all();
        assert_eq!(all_langs.len(), 8);
        assert!(all_langs.contains(&SupportedLanguage::Java));
        assert!(all_langs.contains(&SupportedLanguage::Rust));
        assert!(all_langs.contains(&SupportedLanguage::Python));
        assert!(all_langs.contains(&SupportedLanguage::JavaScript));
        assert!(all_langs.contains(&SupportedLanguage::TypeScript));
        assert!(all_langs.contains(&SupportedLanguage::Go));
        assert!(all_langs.contains(&SupportedLanguage::C));
        assert!(all_langs.contains(&SupportedLanguage::Cpp));
    }

    #[tokio::test]
    async fn test_tree_sitter_manager_creation() {
        let result = TreeSitterManager::new().await;
        assert!(result.is_ok(), "Failed to create TreeSitterManager");

        let mut manager = result.unwrap();

        // 测试是否可以获取已启用语言的解析器
        for lang in SupportedLanguage::all() {
            let parser = manager.get_parser(lang);
            // 只有当语言启用时才期望获取到解析器
            if lang.language().is_some() {
                assert!(
                    parser.is_some(),
                    "Should be able to get parser for enabled language {lang:?}"
                );
            } else {
                assert!(
                    parser.is_none(),
                    "Should not have parser for disabled language {lang:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_analyze_empty_code() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        // 使用第一个已启用的语言进行测试
        let enabled_languages: Vec<_> = SupportedLanguage::all()
            .into_iter()
            .filter(|lang| lang.language().is_some())
            .collect();

        if !enabled_languages.is_empty() {
            let lang = enabled_languages[0];
            let result = manager.analyze_structure("", lang);
            assert!(
                result.is_ok(),
                "Should handle empty code gracefully for {lang:?}"
            );

            let summary = result.unwrap();
            assert_eq!(summary.language, lang.name());
            assert_eq!(summary.functions.len(), 0);
            assert_eq!(summary.classes.len(), 0);
        } else {
            println!("跳过空代码测试 - 没有启用的语言");
        }
    }

    #[tokio::test]
    async fn test_analyze_simple_java_code() {
        // 只有在 Java 支持启用时才测试
        if SupportedLanguage::Java.language().is_none() {
            println!("跳过 Java 代码分析测试 - Java 支持未启用");
            return;
        }

        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let java_code = r#"
        public class Test {
            public void hello() {
                System.out.println("Hello");
            }
        }
        "#;

        let result = manager.analyze_structure(java_code, SupportedLanguage::Java);
        assert!(result.is_ok(), "Should successfully analyze Java code");

        let summary = result.unwrap();
        assert_eq!(summary.language, "java");
        // 简单验证解析结果存在（但不强制要求数量）
        // Length is always >= 0, no need to check
    }

    #[tokio::test]
    async fn test_analyze_simple_rust_code() {
        // Only test when Rust support is enabled
        if SupportedLanguage::Rust.language().is_none() {
            println!("跳过 Rust 代码分析测试 - Rust 支持未启用");
            return;
        }
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        let rust_code = r#"
        pub struct TestStruct {
            field: String,
        }
        
        impl TestStruct {
            pub fn new() -> Self {
                Self { field: String::new() }
            }
        }
        "#;

        let result = manager.analyze_structure(rust_code, SupportedLanguage::Rust);
        assert!(result.is_ok(), "Should successfully analyze Rust code");

        let summary = result.unwrap();
        assert_eq!(summary.language, "rust");
        // 简单验证解析结果存在
        // Length is always >= 0, no need to check
    }

    #[tokio::test]
    async fn test_analyze_multiple_languages() {
        let mut manager = TreeSitterManager::new()
            .await
            .expect("Failed to create manager");

        // 只测试已启用的语言
        let test_codes = vec![
            (
                SupportedLanguage::Java,
                "public class Test { void hello() {} }",
            ),
            (SupportedLanguage::Rust, "pub fn hello() {}"),
            (SupportedLanguage::Python, "def hello(): pass"),
            (SupportedLanguage::JavaScript, "function hello() {}"),
            (SupportedLanguage::Go, "func hello() {}"),
        ];

        for (lang, code) in test_codes {
            // 只测试已启用的语言
            if lang.language().is_some() {
                let result = manager.analyze_structure(code, lang);
                assert!(
                    result.is_ok(),
                    "Should successfully analyze enabled language {lang:?} code"
                );

                let summary = result.unwrap();
                assert_eq!(summary.language, lang.name());
            } else {
                // 对于未启用的语言，应该返回错误
                let result = manager.analyze_structure(code, lang);
                assert!(
                    result.is_err(),
                    "Should fail to analyze disabled language {lang:?} code"
                );
            }
        }
    }
}
