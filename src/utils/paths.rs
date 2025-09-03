//! 跨平台路径解析工具模块
//!
//! 提供统一的配置和缓存目录解析，支持：
//! - Windows、macOS、Linux 跨平台兼容
//! - 环境变量覆盖：GITAI_CONFIG_DIR、GITAI_CACHE_DIR
//! - 向后兼容：优先使用已存在的 ~/.config/gitai、~/.cache/gitai
//! - 平台标准：使用各平台标准目录作为默认位置

use std::path::{Path, PathBuf};
use std::{env, fs};

/// 展开用户目录 ~ 到实际路径
///
/// 支持：
/// - Unix 风格：~/path、~  
/// - Windows 风格：~\path
///
/// # Examples
/// ```
/// use gitai::utils::paths::expand_user;
///
/// let path = expand_user("~/Documents");
/// assert!(path.is_absolute());
/// ```
pub fn expand_user(input: &str) -> PathBuf {
    if let Some(stripped) = input.strip_prefix('~') {
        if let Some(home) = home::home_dir() {
            // 支持 Unix (/) 和 Windows (\) 路径分隔符
            let stripped = stripped
                .strip_prefix('/')
                .or_else(|| stripped.strip_prefix('\\'))
                .unwrap_or(stripped);

            return if stripped.is_empty() {
                home
            } else {
                home.join(stripped)
            };
        }
    }
    PathBuf::from(input)
}

/// 获取 GitAI 配置目录
///
/// 优先级：
/// 1. 环境变量 GITAI_CONFIG_DIR
/// 2. 向后兼容：~/.config/gitai（如果存在）
/// 3. 平台标准配置目录/gitai
pub fn config_dir() -> PathBuf {
    // 1. 环境变量覆盖
    if let Ok(path) = env::var("GITAI_CONFIG_DIR") {
        return expand_user(&path);
    }

    // 2. 向后兼容检查
    let legacy = expand_user("~/.config/gitai");
    if legacy.exists() {
        return legacy;
    }

    // 3. 平台标准目录
    directories::BaseDirs::new()
        .expect("Failed to determine user directories")
        .config_dir()
        .join("gitai")
}

/// 获取 GitAI 缓存目录
///
/// 优先级：
/// 1. 环境变量 GITAI_CACHE_DIR
/// 2. 向后兼容：~/.cache/gitai（如果存在）
/// 3. 平台标准缓存目录/gitai
pub fn cache_dir() -> PathBuf {
    // 1. 环境变量覆盖
    if let Ok(path) = env::var("GITAI_CACHE_DIR") {
        return expand_user(&path);
    }

    // 2. 向后兼容检查
    let legacy = expand_user("~/.cache/gitai");
    if legacy.exists() {
        return legacy;
    }

    // 3. 平台标准目录
    directories::BaseDirs::new()
        .expect("Failed to determine user directories")
        .cache_dir()
        .join("gitai")
}

/// 获取提示词模板目录
pub fn prompts_dir() -> PathBuf {
    config_dir().join("prompts")
}

/// 获取 OpenGrep 规则目录
pub fn rules_dir() -> PathBuf {
    cache_dir().join("rules")
}

/// 获取代码评审缓存目录
pub fn review_cache_dir() -> PathBuf {
    cache_dir().join("review_cache")
}

/// 获取扫描历史目录
pub fn scan_history_dir() -> PathBuf {
    cache_dir().join("scan_history")
}

/// 获取 Tree-sitter 相关目录
pub fn tree_sitter_dir() -> PathBuf {
    cache_dir().join("tree-sitter")
}

/// 获取 Tree-sitter 查询文件目录
pub fn tree_sitter_queries_dir() -> PathBuf {
    cache_dir().join("tree-sitter-queries")
}

/// 获取 Tree-sitter 缓存目录
pub fn tree_sitter_cache_dir() -> PathBuf {
    cache_dir().join("tree_sitter_cache")
}

/// 获取默认配置文件路径
pub fn default_config_file() -> PathBuf {
    config_dir().join("config.toml")
}

/// 确保目录存在，如果不存在则创建
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

/// 解析配置文件中的路径字符串，处理 ~ 展开
pub fn resolve_config_path(path: &str) -> PathBuf {
    if path.trim().is_empty() {
        return PathBuf::new();
    }
    expand_user(path)
}

/// 获取路径来源描述（用于日志调试）
pub fn get_path_source(path: &Path) -> &'static str {
    let path_str = path.to_string_lossy();

    if env::var("GITAI_CONFIG_DIR").is_ok() || env::var("GITAI_CACHE_DIR").is_ok() {
        "environment variable"
    } else if path_str.contains(".config/gitai") || path_str.contains(".cache/gitai") {
        "legacy path"
    } else {
        "platform default"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_expand_user() {
        // 测试 ~ 单独展开
        let home_path = expand_user("~");
        assert!(home_path.is_absolute());

        // 测试 ~/subdir Unix 风格
        let unix_path = expand_user("~/Documents");
        assert!(unix_path.ends_with("Documents"));
        assert!(unix_path.is_absolute());

        // 测试 ~\subdir Windows 风格
        let windows_path = expand_user("~\\Documents");
        assert!(windows_path.ends_with("Documents"));
        assert!(windows_path.is_absolute());

        // 测试非 ~ 开头的路径保持不变
        let regular_path = expand_user("/usr/local/bin");
        assert_eq!(regular_path, PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn test_env_override() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy();

        // 测试配置目录环境变量覆盖
        env::set_var("GITAI_CONFIG_DIR", temp_path.as_ref());
        let config_path = config_dir();
        assert_eq!(config_path, temp_dir.path());
        env::remove_var("GITAI_CONFIG_DIR");

        // 测试缓存目录环境变量覆盖
        env::set_var("GITAI_CACHE_DIR", temp_path.as_ref());
        let cache_path = cache_dir();
        assert_eq!(cache_path, temp_dir.path());
        env::remove_var("GITAI_CACHE_DIR");
    }

    #[test]
    fn test_subdirectories() {
        let temp_config_dir = TempDir::new().unwrap();
        let temp_cache_dir = TempDir::new().unwrap();

        // 使用环境变量覆盖默认路径
        env::set_var("GITAI_CONFIG_DIR", temp_config_dir.path());
        env::set_var("GITAI_CACHE_DIR", temp_cache_dir.path());

        let config = config_dir();
        let cache = cache_dir();

        // 测试所有子目录都基于正确的父目录
        assert_eq!(prompts_dir(), config.join("prompts"));
        assert_eq!(rules_dir(), cache.join("rules"));
        assert_eq!(review_cache_dir(), cache.join("review_cache"));
        assert_eq!(scan_history_dir(), cache.join("scan_history"));
        assert_eq!(tree_sitter_dir(), cache.join("tree-sitter"));
        assert_eq!(tree_sitter_queries_dir(), cache.join("tree-sitter-queries"));
        assert_eq!(tree_sitter_cache_dir(), cache.join("tree_sitter_cache"));
        assert_eq!(default_config_file(), config.join("config.toml"));

        // 清理环境变量
        env::remove_var("GITAI_CONFIG_DIR");
        env::remove_var("GITAI_CACHE_DIR");
    }

    #[test]
    fn test_ensure_dir() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("nested").join("directory");

        // 目录应该不存在
        assert!(!test_path.exists());

        // 创建目录
        ensure_dir(&test_path).unwrap();

        // 现在应该存在
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_resolve_config_path() {
        // 测试 ~ 展开
        let path = resolve_config_path("~/test");
        assert!(path.is_absolute());
        assert!(path.ends_with("test"));

        // 测试空字符串
        let empty_path = resolve_config_path("");
        assert_eq!(empty_path, PathBuf::new());

        // 测试普通路径
        let normal_path = resolve_config_path("/usr/local");
        assert_eq!(normal_path, PathBuf::from("/usr/local"));
    }

    #[test]
    fn test_path_source() {
        // 测试环境变量来源
        env::set_var("GITAI_CONFIG_DIR", "/tmp/test");
        let config_path = config_dir();
        assert_eq!(get_path_source(&config_path), "environment variable");
        env::remove_var("GITAI_CONFIG_DIR");

        // 测试 legacy 路径识别
        let legacy_path = PathBuf::from("/home/user/.config/gitai");
        assert_eq!(get_path_source(&legacy_path), "legacy path");

        let legacy_cache = PathBuf::from("/home/user/.cache/gitai");
        assert_eq!(get_path_source(&legacy_cache), "legacy path");
    }
}
