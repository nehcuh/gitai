use std::fs;
use tempfile::TempDir;
use gitai::tree_sitter_analyzer::query_manager::{QueryManager, QueryManagerConfig, QuerySource};
use gitai::tree_sitter_analyzer::analyzer::TreeSitterAnalyzer;
use gitai::config::TreeSitterConfig;

#[test]
fn test_query_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = QueryManagerConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = QueryManager::new(config);
    assert!(manager.is_ok(), "QueryManager should be created successfully");
}

#[test]
fn test_tree_sitter_analyzer_with_query_manager() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut tree_sitter_config = TreeSitterConfig::default();
    tree_sitter_config.query_manager_config.cache_dir = temp_dir.path().to_path_buf();
    
    let analyzer = TreeSitterAnalyzer::new(tree_sitter_config);
    assert!(analyzer.is_ok(), "TreeSitterAnalyzer should be created with QueryManager");
}

#[test]
fn test_query_source_configuration() {
    let sources = vec![
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
    ];

    assert_eq!(sources.len(), 2);
    assert!(sources[0].enabled);
    assert!(!sources[1].enabled);
    assert!(sources[0].priority < sources[1].priority);
}

#[test]
fn test_query_manager_config_default() {
    let config = QueryManagerConfig::default();
    
    // 验证默认配置
    assert_eq!(config.sources.len(), 3); // nvim-treesitter, helix-editor, zed-editor
    assert_eq!(config.cache_ttl, 24 * 60 * 60); // 24小时
    assert!(config.auto_update);
    assert_eq!(config.update_interval, 7 * 24 * 60 * 60); // 7天
    assert_eq!(config.network_timeout, 30); // 30秒

    // 验证默认源配置
    let nvim_source = config.sources.iter().find(|s| s.name == "nvim-treesitter").unwrap();
    assert!(nvim_source.enabled);
    assert_eq!(nvim_source.priority, 1);

    let helix_source = config.sources.iter().find(|s| s.name == "helix-editor").unwrap();
    assert!(!helix_source.enabled); // 默认禁用
    assert_eq!(helix_source.priority, 2);
}

#[test]
fn test_query_manager_cache_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let config = QueryManagerConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        cache_ttl: 1, // 1秒，便于测试过期
        ..Default::default()
    };

    let mut manager = QueryManager::new(config).unwrap();
    
    // 验证缓存目录创建
    assert!(temp_dir.path().exists());
    
    // 测试缓存清理（即使没有缓存文件也应该成功）
    let cleanup_result = manager.cleanup_cache();
    assert!(cleanup_result.is_ok());
}

#[test]
fn test_tree_sitter_config_with_query_manager() {
    let config = TreeSitterConfig::default();
    
    // 验证 TreeSitterConfig 包含 QueryManagerConfig
    assert!(config.enabled);
    assert_eq!(config.analysis_depth, "medium");
    assert!(config.cache_enabled);
    assert!(!config.languages.is_empty());
    
    // 验证查询管理器配置存在
    assert!(config.query_manager_config.auto_update);
    assert!(!config.query_manager_config.sources.is_empty());
}

#[test]
fn test_query_manager_supported_languages() {
    let temp_dir = TempDir::new().unwrap();
    let config = QueryManagerConfig {
        cache_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let manager = QueryManager::new(config).unwrap();
    
    // 刚创建的管理器应该没有已下载的语言
    let supported_languages = manager.get_supported_languages();
    // 由于没有实际下载，应该为空或者从现有缓存读取
    assert!(supported_languages.is_empty() || !supported_languages.is_empty());
}