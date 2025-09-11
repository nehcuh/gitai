//! 核心功能集成测试
//!
//! 测试 GitAI 核心功能在 workspace 结构中是否正常工作

use gitai_core::config::Config;
use gitai_core::context::{GitInfo, OperationOptions};
use gitai_types::{FindingSeverity, ProjectInfo};

/// 测试配置加载功能
#[test]
fn test_config_loading() {
    // 应该能够创建默认配置
    let config = Config::default();
    assert!(!config.ai.api_url.is_empty());
    assert!(config.scan.default_path.exists());
    
    // 应该能够从环境变量加载配置（如果存在）
    if let Ok(loaded_config) = Config::load() {
        assert!(!loaded_config.ai.api_url.is_empty());
    }
}

/// 测试上下文创建功能
#[test]
fn test_context_creation() {
    let git_info = GitInfo {
        branch: "main".to_string(),
        commit: "abc123".to_string(),
        author: "Test User".to_string(),
        message: "Test commit".to_string(),
        changed_files: vec!["src/main.rs".to_string()],
    };
    
    let options = OperationOptions {
        dry_run: true,
        verbose: false,
        language: Some("rust".to_string()),
        output: None,
        format: None,
        issue_ids: vec![],
        deviation_analysis: false,
        tree_sitter: false,
        security_scan: false,
        scan_tool: None,
        architectural_analysis: false,
        dependency_analysis: false,
        block_on_critical: false,
        include_suggestions: false,
        message: None,
        add_all: false,
        review_before_commit: false,
        cache_enabled: true,
        timeout: Some(300),
        max_depth: Some(5),
    };
    
    assert_eq!(git_info.branch, "main");
    assert_eq!(git_info.commit, "abc123");
    assert!(options.dry_run);
    assert!(options.cache_enabled);
}

/// 测试类型系统
#[test]
fn test_type_system() {
    let project_info = ProjectInfo {
        name: "test-project".to_string(),
        path: "/tmp/test".into(),
        language: gitai_types::ProgrammingLanguage::Rust,
        total_files: 10,
        total_lines: 1000,
        dependencies: vec![],
        created_at: chrono::Utc::now(),
        last_modified: chrono::Utc::now(),
    };
    
    assert_eq!(project_info.name, "test-project");
    assert_eq!(project_info.total_files, 10);
    assert_eq!(project_info.total_lines, 1000);
    
    let severity = FindingSeverity::Critical;
    assert!(matches!(severity, FindingSeverity::Critical));
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    use gitai_types::GitAIError;
    
    let error = GitAIError::ConfigurationError("Test error".to_string());
    assert_eq!(error.to_string(), "Configuration error: Test error");
    
    let error = GitAIError::GitError("Git operation failed".to_string());
    assert_eq!(error.to_string(), "Git error: Git operation failed");
}

/// 测试序列化/反序列化
#[test]
fn test_serialization() {
    use serde_json;
    
    let project_info = ProjectInfo {
        name: "test-project".to_string(),
        path: "/tmp/test".into(),
        language: gitai_types::ProgrammingLanguage::Rust,
        total_files: 10,
        total_lines: 1000,
        dependencies: vec![],
        created_at: chrono::Utc::now(),
        last_modified: chrono::Utc::now(),
    };
    
    let serialized = serde_json::to_string(&project_info)
        .expect("Failed to serialize project info");
    
    let deserialized: ProjectInfo = serde_json::from_str(&serialized)
        .expect("Failed to deserialize project info");
    
    assert_eq!(project_info.name, deserialized.name);
    assert_eq!(project_info.total_files, deserialized.total_files);
    assert_eq!(project_info.language, deserialized.language);
}