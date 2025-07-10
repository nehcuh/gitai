use gitai::{config::AppConfig, mcp_bridge::GitAiMcpBridge};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing MCP Bridge directly...");
    
    // Load config
    let config = AppConfig::load()?;
    let bridge = GitAiMcpBridge::new(config);
    
    // Test gitai_status
    println!("\n📊 Testing gitai_status...");
    let status_result = bridge.gitai_status(Some(true)).await;
    match status_result {
        Ok(result) => {
            println!("✅ Status result: {:?}", result);
        }
        Err(e) => {
            println!("❌ Status error: {:?}", e);
        }
    }
    
    // Test gitai_diff
    println!("\n📝 Testing gitai_diff...");
    let diff_result = bridge.gitai_diff(Some(true), None).await;
    match diff_result {
        Ok(result) => {
            println!("✅ Diff result: {:?}", result);
        }
        Err(e) => {
            println!("❌ Diff error: {:?}", e);
        }
    }
    
    // Test gitai_review
    println!("\n🔍 Testing gitai_review...");
    let review_result = bridge.gitai_review(None, None, None, None).await;
    match review_result {
        Ok(result) => {
            println!("✅ Review result: {:?}", result);
        }
        Err(e) => {
            println!("❌ Review error: {:?}", e);
        }
    }
    
    Ok(())
}