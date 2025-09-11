//! Workspace 集成测试
//!
//! 测试 GitAI workspace 结构是否正常工作

use std::process::Command;

/// 测试所有 crate 是否能正确编译
#[test]
fn test_workspace_compilation() {
    // 测试 gitai-types
    let output = Command::new("cargo")
        .args(&["check", "-p", "gitai-types"])
        .output()
        .expect("Failed to execute cargo check for gitai-types");
    
    assert!(output.status.success(), 
        "gitai-types compilation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    // 测试 gitai-core
    let output = Command::new("cargo")
        .args(&["check", "-p", "gitai-core"])
        .output()
        .expect("Failed to execute cargo check for gitai-core");
    
    assert!(output.status.success(), 
        "gitai-core compilation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    // 测试 gitai-cli
    let output = Command::new("cargo")
        .args(&["check", "-p", "gitai-cli"])
        .output()
        .expect("Failed to execute cargo check for gitai-cli");
    
    assert!(output.status.success(), 
        "gitai-cli compilation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    // 测试 gitai-mcp
    let output = Command::new("cargo")
        .args(&["check", "-p", "gitai-mcp"])
        .output()
        .expect("Failed to execute cargo check for gitai-mcp");
    
    assert!(output.status.success(), 
        "gitai-mcp compilation failed: {}", 
        String::from_utf8_lossy(&output.stderr));
}

/// 测试 CLI 基本功能
#[test]
fn test_cli_basic_functionality() {
    let output = Command::new("cargo")
        .args(&["run", "-p", "gitai-cli", "--", "--help"])
        .output()
        .expect("Failed to execute gitai-cli --help");
    
    assert!(output.status.success(), 
        "gitai-cli --help failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    let help_output = String::from_utf8_lossy(&output.stdout);
    assert!(help_output.contains("GitAI"), 
        "Help output should contain 'GitAI'");
    assert!(help_output.contains("Usage:"), 
        "Help output should contain usage information");
}

/// 测试 MCP 服务器基本功能
#[test]
fn test_mcp_server_basic_functionality() {
    let output = Command::new("cargo")
        .args(&["run", "-p", "gitai-mcp", "--", "--help"])
        .output()
        .expect("Failed to execute gitai-mcp --help");
    
    assert!(output.status.success(), 
        "gitai-mcp --help failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    let help_output = String::from_utf8_lossy(&output.stdout);
    assert!(help_output.contains("GitAI MCP"), 
        "MCP help output should contain 'GitAI MCP'");
}

/// 测试 workspace 成员配置
#[test]
fn test_workspace_members() {
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1"])
        .output()
        .expect("Failed to execute cargo metadata");
    
    assert!(output.status.success(), 
        "cargo metadata failed: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    let metadata = String::from_utf8_lossy(&output.stdout);
    assert!(metadata.contains("gitai-types"), 
        "Workspace should contain gitai-types");
    assert!(metadata.contains("gitai-core"), 
        "Workspace should contain gitai-core");
    assert!(metadata.contains("gitai-cli"), 
        "Workspace should contain gitai-cli");
    assert!(metadata.contains("gitai-mcp"), 
        "Workspace should contain gitai-mcp");
}