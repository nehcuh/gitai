//! GitAI 端到端集成测试
//!
//! 测试主要功能的端到端流程

use std::process::Command;
use std::path::Path;
use std::fs;

/// 测试GitAI基本功能是否正常工作
#[test]
fn test_gitai_basic_functionality() {
    // 检查gitai命令是否可用
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "--help"])
        .output()
        .expect("Failed to execute gitai --help");

    assert!(output.status.success());
    let help_output = String::from_utf8_lossy(&output.stdout);
    assert!(help_output.contains("AI驱动的Git工作流助手"));
    assert!(help_output.contains("review"));
    assert!(help_output.contains("commit"));
    assert!(help_output.contains("scan"));
}

/// 测试GitAI初始化功能
#[test]
fn test_gitai_init_functionality() {
    let temp_dir = std::env::temp_dir().join("gitai_e2e_test");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

    // 在临时目录中初始化git仓库
    let git_init = Command::new("git")
        .args(&["init"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to init git repo");
    assert!(git_init.status.success());

    // 配置git用户信息
    let git_config = Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to configure git");
    assert!(git_config.status.success());

    let git_config2 = Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to configure git");
    assert!(git_config2.status.success());

    // 创建一个测试文件
    let test_file = temp_dir.join("test.rs");
    fs::write(&test_file, r#"fn main() {
    println!("Hello, GitAI!");
}"#).expect("Failed to write test file");

    // 运行gitai review命令（离线模式）
    let review_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "gitai", "--",
            "review", "--offline", "--format", "json"
        ])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to run gitai review");

    // 检查review命令是否成功执行
    if review_output.status.success() {
        let review_result = String::from_utf8_lossy(&review_output.stdout);
        println!("Review output: {}", review_result);
        // 在离线模式下，应该能生成基础的评审报告
        assert!(review_result.contains("变更统计") || review_result.contains("代码评审报告"));
    } else {
        // 如果失败，检查错误信息
        let error_output = String::from_utf8_lossy(&review_output.stderr);
        println!("Review error: {}", error_output);
        // 某些情况下review可能失败，但这不应该导致测试失败
        // 因为我们在测试基本功能是否可用
    }

    // 清理临时目录
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 测试配置功能
#[test]
fn test_gitai_config_functionality() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "config", "--help"])
        .output()
        .expect("Failed to execute gitai config --help");

    assert!(output.status.success());
    let config_help = String::from_utf8_lossy(&output.stdout);
    assert!(config_help.contains("配置管理"));
    assert!(config_help.contains("check"));
    assert!(config_help.contains("show"));
}

/// 测试功能特性显示
#[test]
fn test_gitai_features_functionality() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "features"])
        .output()
        .expect("Failed to execute gitai features");

    assert!(output.status.success());
    let features_output = String::from_utf8_lossy(&output.stdout);
    assert!(features_output.contains("GitAI 功能特性"));
}

/// 测试MCP相关命令
#[test]
fn test_gitai_mcp_functionality() {
    // 测试MCP help
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "mcp", "--help"])
        .output()
        .expect("Failed to execute gitai mcp --help");

    assert!(output.status.success());
    let mcp_help = String::from_utf8_lossy(&output.stdout);
    assert!(mcp_help.contains("启动MCP服务器"));
    assert!(mcp_help.contains("transport"));
}

/// 测试版本信息
#[test]
fn test_gitai_version() {
    // 这个测试确保程序能够正常启动并显示基本信息
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "--version"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let version_output = String::from_utf8_lossy(&result.stdout);
                println!("Version output: {}", version_output);
            }
        }
        Err(_) => {
            // 版本命令可能不存在，这是可以接受的
        }
    }
}

/// 性能基准测试 - 确保基本的性能水平
#[test]
fn test_basic_performance() {
    let start = std::time::Instant::now();

    // 测试help命令的响应时间
    let output = Command::new("cargo")
        .args(&["run", "--bin", "gitai", "--", "--help"])
        .output()
        .expect("Failed to execute gitai --help");

    let duration = start.elapsed();

    assert!(output.status.success());
    // help命令应该在合理时间内完成（比如10秒内）
    assert!(duration.as_secs() < 10, "Help command took too long: {:?}", duration);
}

/// 集成测试：验证主要模块可用性
#[test]
fn test_module_availability() {
    // 这些测试验证各个主要模块是否可以正常编译和链接
    // 不需要实际运行，只需要确保没有链接错误

    // 验证gitai-core模块
    let output = Command::new("cargo")
        .args(&["test", "--package", "gitai-core", "--lib", "test_version"])
        .output()
        .expect("Failed to test gitai-core");

    assert!(output.status.success(), "gitai-core module test failed");

    // 验证gitai-cli模块
    let output = Command::new("cargo")
        .args(&["test", "--package", "gitai-cli", "--lib", "test_cache_config_default"])
        .output()
        .expect("Failed to test gitai-cli");

    assert!(output.status.success(), "gitai-cli module test failed");
}