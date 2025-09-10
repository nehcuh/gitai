#[cfg(not(feature = "mcp"))]
fn main() {
    eprintln!(
        "This example requires the 'mcp' feature.\nRun with: cargo run --example mcp_registry_demo --features mcp"
    );
}

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gitai::config::Config;
    use gitai::mcp::registry::ServiceRegistryBuilder;
    use gitai::mcp::services::{AnalysisService, CommitService, ReviewService, ScanService};
    use std::sync::Arc;
    // 初始化日志
    env_logger::init();

    println!("🚀 GitAI MCP Service Registry Demo");
    println!("==================================\n");

    // 创建服务注册表
    let registry = ServiceRegistryBuilder::new().build();

    // 创建配置
    let config = Config::default();
    let service_config = serde_json::json!({});

    println!("📦 注册服务演示\n");

    // 1. 注册基础服务
    println!("1️⃣ 注册 Analysis 服务（无依赖）...");
    let analysis_service = Arc::new(AnalysisService::new(config.clone())?);
    match registry
        .register_service(analysis_service, service_config.clone())
        .await
    {
        Ok(_) => println!("   ✅ Analysis 服务注册成功"),
        Err(e) => println!("   ❌ Analysis 服务注册失败: {e}"),
    }

    println!("\n2️⃣ 注册 Scan 服务（无依赖）...");
    let scan_service = Arc::new(ScanService::new(config.clone())?);
    match registry
        .register_service(scan_service, service_config.clone())
        .await
    {
        Ok(_) => println!("   ✅ Scan 服务注册成功"),
        Err(e) => println!("   ❌ Scan 服务注册失败: {e}"),
    }

    println!("\n3️⃣ 注册 Review 服务（无依赖）...");
    let review_service = Arc::new(ReviewService::new(config.clone())?);
    match registry
        .register_service(review_service, service_config.clone())
        .await
    {
        Ok(_) => println!("   ✅ Review 服务注册成功"),
        Err(e) => println!("   ❌ Review 服务注册失败: {e}"),
    }

    println!("\n4️⃣ 注册 Commit 服务（可选依赖 Review 服务）...");
    let commit_service = Arc::new(CommitService::new(config.clone())?);
    match registry
        .register_service(commit_service, service_config.clone())
        .await
    {
        Ok(_) => println!("   ✅ Commit 服务注册成功（Review 服务依赖为可选）"),
        Err(e) => println!("   ❌ Commit 服务注册失败: {e}"),
    }

    // 显示当前服务状态
    println!("\n📊 服务注册状态：");
    let services = registry.list_services().await;
    for service in &services {
        println!(
            "   - {} (v{}) - 状态: {:?}",
            service.name, service.version, service.status
        );
        if !service.dependencies.is_empty() {
            for dep in &service.dependencies {
                println!(
                    "     └─ 依赖: {} ({}{})",
                    dep.service_name,
                    dep.version_req,
                    if dep.optional { ", 可选" } else { "" }
                );
            }
        }
    }

    // 获取并显示服务启动顺序
    println!("\n🔄 服务启动顺序（拓扑排序）：");
    match registry.get_startup_order().await {
        Ok(order) => {
            let ordered_services: Vec<String> = order
                .iter()
                .filter_map(|id| services.iter().find(|s| s.id == *id))
                .map(|s| s.name.clone())
                .collect();

            for (i, name) in ordered_services.iter().enumerate() {
                println!("   {}. {}", i + 1, name);
            }
        }
        Err(e) => println!("   ❌ 获取启动顺序失败: {e}"),
    }

    // 演示依赖关系对注销的影响
    println!("\n🗑️ 服务注销演示：");

    // 尝试注销被依赖的服务
    let review_service_id = services
        .iter()
        .find(|s| s.name == "review")
        .map(|s| s.id.clone())
        .unwrap();

    println!("\n5️⃣ 尝试注销 Review 服务（被 Commit 服务依赖）...");
    match registry
        .unregister_service(&review_service_id, "演示注销".to_string())
        .await
    {
        Ok(_) => println!("   ✅ Review 服务注销成功"),
        Err(e) => println!("   ⚠️  Review 服务注销失败: {e}"),
    }

    // 先注销依赖服务
    let commit_service_id = services
        .iter()
        .find(|s| s.name == "commit")
        .map(|s| s.id.clone())
        .unwrap();

    println!("\n6️⃣ 先注销 Commit 服务...");
    match registry
        .unregister_service(&commit_service_id, "演示注销".to_string())
        .await
    {
        Ok(_) => println!("   ✅ Commit 服务注销成功"),
        Err(e) => println!("   ❌ Commit 服务注销失败: {e}"),
    }

    println!("\n7️⃣ 再次尝试注销 Review 服务...");
    match registry
        .unregister_service(&review_service_id, "演示注销".to_string())
        .await
    {
        Ok(_) => println!("   ✅ Review 服务注销成功"),
        Err(e) => println!("   ❌ Review 服务注销失败: {e}"),
    }

    // 最终状态
    println!("\n📊 最终服务状态：");
    let final_services = registry.list_services().await;
    if final_services.is_empty() {
        println!("   （无服务注册）");
    } else {
        for service in &final_services {
            println!(
                "   - {} (v{}) - 状态: {:?}",
                service.name, service.version, service.status
            );
        }
    }

    println!("\n✨ 演示完成！");

    Ok(())
}
