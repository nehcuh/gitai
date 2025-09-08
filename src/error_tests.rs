//! 统一错误处理体系测试
//!
//! 测试新的错误类型整合和转换机制

use crate::error::*;
use crate::{with_error_context, with_error_context_and};
use std::io;

#[test]
fn test_error_type_consolidation() {
    // 测试配置错误转换
    let config_error = ConfigError::FileNotFound("config.toml".to_string());
    let gitai_error: GitAIError = config_error.into();
    assert!(matches!(gitai_error, GitAIError::Config(_)));

    // 测试Git错误转换
    let git_error = GitError::CommandFailed("git command failed".to_string());
    let gitai_error: GitAIError = git_error.into();
    assert!(matches!(gitai_error, GitAIError::Git(_)));

    // 测试网络错误转换
    let network_error = NetworkError::ConnectionFailed("connection failed".to_string());
    let gitai_error: GitAIError = network_error.into();
    assert!(matches!(gitai_error, GitAIError::Network(_)));
}

#[test]
fn test_error_from_std_types() {
    // 测试从std::io::Error转换
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let gitai_error: GitAIError = io_error.into();
    assert!(matches!(
        gitai_error,
        GitAIError::FileSystem(FileSystemError::Io(_))
    ));

    // 测试从serde_json::Error转换
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let gitai_error: GitAIError = json_error.into();
    assert!(matches!(
        gitai_error,
        GitAIError::Parse(ParseError::Json(_))
    ));
}

#[test]
fn test_error_context_enhancement() {
    // 测试错误上下文增强
    let result: std::result::Result<(), io::Error> = Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "access denied",
    ));

    let enhanced_error = result
        .with_file("test.rs")
        .with_function("test_function")
        .with_line(42)
        .with_context("user", "test_user")
        .unwrap_err();

    assert!(matches!(enhanced_error, GitAIError::Unknown(_)));
    let error_msg = enhanced_error.to_string();
    assert!(error_msg.contains("文件: test.rs"));
    assert!(error_msg.contains("函数: test_function"));
    assert!(error_msg.contains("行号: 42"));
    assert!(error_msg.contains("user: test_user"));
}

#[test]
fn test_error_context_info() {
    // 测试ErrorContextInfo结构
    let context = ErrorContextInfo::new()
        .file("src/main.rs")
        .function("main")
        .line(10)
        .add_context("module", "gitai");

    let formatted = context.format();
    assert!(formatted.contains("文件: src/main.rs"));
    assert!(formatted.contains("函数: main"));
    assert!(formatted.contains("行号: 10"));
    assert!(formatted.contains("module: gitai"));
}

#[test]
fn test_error_user_messages() {
    // 测试用户友好的错误消息
    let config_error = GitAIError::Config(ConfigError::FileNotFound("config.toml".to_string()));
    let user_msg = config_error.user_message();
    assert!(user_msg.contains("配置错误"));
    assert!(user_msg.contains("config.toml"));
    assert!(user_msg.contains("~/.config/gitai/config.toml"));

    let git_error = GitAIError::Git(GitError::CommandFailed("git command failed".to_string()));
    let user_msg = git_error.user_message();
    assert!(user_msg.contains("Git 操作失败"));
    assert!(user_msg.contains("git command failed"));

    let network_error = GitAIError::Network(NetworkError::Timeout("request timeout".to_string()));
    let user_msg = network_error.user_message();
    assert!(user_msg.contains("网络连接错误"));
    assert!(user_msg.contains("request timeout"));
}

#[test]
fn test_error_macros() {
    // 测试错误上下文宏
    let result: std::result::Result<(), io::Error> =
        Err(io::Error::new(io::ErrorKind::NotFound, "not found"));

    let enhanced_error = with_error_context!(result, "test.rs", "test_func", 123).unwrap_err();
    let error_msg = enhanced_error.to_string();
    assert!(error_msg.contains("文件: test.rs"));
    assert!(error_msg.contains("函数: test_func"));
    assert!(error_msg.contains("行号: 123"));

    // 测试带自定义上下文的宏
    let result: std::result::Result<(), io::Error> =
        Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
    let enhanced_error = with_error_context_and!(
        result,
        "test.rs",
        "test_func",
        123,
        "module" => "test_module",
        "severity" => "high"
    )
    .unwrap_err();

    let error_msg = enhanced_error.to_string();
    assert!(error_msg.contains("module: test_module"));
    assert!(error_msg.contains("severity: high"));
}

#[test]
fn test_display_implementations() {
    // 测试所有错误类型的Display实现
    let errors = vec![
        GitAIError::Config(ConfigError::Missing("api_key".to_string())),
        GitAIError::Git(GitError::RepositoryNotFound("/path/to/repo".to_string())),
        GitAIError::FileSystem(FileSystemError::FileNotFound("file.txt".to_string())),
        GitAIError::Network(NetworkError::DnsFailed("dns resolution failed".to_string())),
        GitAIError::ScanTool(ScanError::ToolNotFound("semgrep".to_string())),
        GitAIError::AiService(AiError::ModelUnavailable("gpt-4".to_string())),
        GitAIError::Parse(ParseError::Json("invalid json".to_string())),
        GitAIError::Container(ContainerError::ServiceNotRegistered {
            type_name: "TestService".to_string(),
            available_services: vec!["OtherService".to_string()],
            suggestion: Some("register TestService first".to_string()),
        }),
        GitAIError::Update(UpdateError::Network("network error".to_string())),
        GitAIError::Mcp(McpError::InvalidParameters(
            "missing required param".to_string(),
        )),
    ];

    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty());
        assert!(display.len() > 0);
    }
}
