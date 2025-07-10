#!/usr/bin/env rust-script
//! # GitAI MCP客户端简化示例
//! 
//! 这个示例展示了如何与GitAI MCP服务进行基本集成
//! 
//! ## 运行方式
//! 
//! ```bash
//! # 首先启动GitAI MCP服务
//! cargo run --bin gitai-mcp-server
//! 
//! # 然后运行此示例
//! cargo run --example mcp_client_simple
//! ```

use std::collections::HashMap;
use serde_json::{json, Value};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GitAI MCP客户端简化示例");
    println!("================================");
    
    // 1. 模拟MCP服务调用
    println!("\n📡 模拟连接到GitAI MCP服务...");
    
    // 2. 模拟TreeSitter分析服务调用
    println!("\n🌳 示例: TreeSitter代码分析");
    println!("─────────────────────────");
    
    let sample_code = r#"
    fn calculate_fibonacci(n: u32) -> u32 {
        if n <= 1 {
            return n;
        }
        calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
    }
    "#;
    
    let analysis_request = json!({
        "tool": "treesitter_analyze",
        "arguments": {
            "code": sample_code,
            "language": "rust",
            "analysis_type": "full"
        }
    });
    
    println!("📊 TreeSitter分析请求:");
    println!("{}", serde_json::to_string_pretty(&analysis_request)?);
    
    // 3. 模拟AI分析服务调用
    println!("\n🤖 示例: AI代码分析");
    println!("─────────────────────");
    
    let ai_request = json!({
        "tool": "ai_analyze",
        "arguments": {
            "content": sample_code,
            "analysis_type": "quality",
            "language": "rust"
        }
    });
    
    println!("🧠 AI分析请求:");
    println!("{}", serde_json::to_string_pretty(&ai_request)?);
    
    // 4. 模拟DevOps集成服务调用
    println!("\n🔗 示例: DevOps集成");
    println!("─────────────────────");
    
    let devops_request = json!({
        "tool": "devops_query",
        "arguments": {
            "query_type": "work_items",
            "space_id": "12345",
            "item_ids": ["1", "2", "3"]
        }
    });
    
    println!("📋 DevOps查询请求:");
    println!("{}", serde_json::to_string_pretty(&devops_request)?);
    
    // 5. 模拟代码扫描服务调用
    println!("\n🔍 示例: 代码安全扫描");
    println!("─────────────────────");
    
    let scan_request = json!({
        "tool": "code_scan",
        "arguments": {
            "path": "src/",
            "scan_type": "security",
            "rules": ["sql-injection", "xss", "path-traversal"]
        }
    });
    
    println!("🛡️ 代码扫描请求:");
    println!("{}", serde_json::to_string_pretty(&scan_request)?);
    
    // 6. 模拟规则管理服务调用
    println!("\n📝 示例: 规则管理");
    println!("─────────────────────");
    
    let rules_request = json!({
        "tool": "manage_rules",
        "arguments": {
            "action": "list",
            "category": "security"
        }
    });
    
    println!("📋 规则管理请求:");
    println!("{}", serde_json::to_string_pretty(&rules_request)?);
    
    // 7. 创建测试配置
    println!("\n⚙️ 示例: 测试配置");
    println!("─────────────────────");
    
    let test_config = create_test_config();
    println!("🔧 测试配置:");
    println!("{}", serde_json::to_string_pretty(&test_config)?);
    
    println!("\n✅ 所有示例执行完毕！");
    println!("💡 这些示例展示了如何与GitAI MCP服务进行集成的基本结构");
    println!("🚀 要实际连接到MCP服务，请使用更完整的rmcp客户端实现");
    
    Ok(())
}

// 辅助函数：创建测试配置
fn create_test_config() -> HashMap<String, Value> {
    let mut config = HashMap::new();
    config.insert("ai_model".to_string(), json!("gpt-4"));
    config.insert("analysis_depth".to_string(), json!("medium"));
    config.insert("language".to_string(), json!("cn"));
    config.insert("mcp_endpoint".to_string(), json!("stdio"));
    config.insert("transport".to_string(), json!("stdio"));
    config
}

// 辅助函数：处理MCP响应
fn handle_mcp_response(response: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(error) = response.get("error") {
        eprintln!("❌ MCP错误: {}", error);
        return Err("MCP调用失败".into());
    }
    
    if let Some(result) = response.get("result") {
        println!("✅ MCP调用成功");
        println!("📊 结果: {}", serde_json::to_string_pretty(result)?);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert!(config.contains_key("ai_model"));
        assert!(config.contains_key("analysis_depth"));
        assert!(config.contains_key("language"));
        assert!(config.contains_key("mcp_endpoint"));
    }
    
    #[test]
    fn test_handle_mcp_response() {
        let success_response = json!({
            "result": {
                "status": "success",
                "data": "test"
            }
        });
        
        assert!(handle_mcp_response(&success_response).is_ok());
        
        let error_response = json!({
            "error": {
                "code": -1,
                "message": "Test error"
            }
        });
        
        assert!(handle_mcp_response(&error_response).is_err());
    }
}