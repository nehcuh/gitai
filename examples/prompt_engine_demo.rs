//! 配置化提示词引擎演示程序
//!
//! 这个示例展示了如何使用新的配置化提示词系统：
//! - 从YAML配置文件加载提示词模板
//! - 动态渲染提示词
//! - 支持用户自定义模板

use gitai::config::Config;
use gitai::prompt_engine::PromptEngine;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 GitAI 配置化提示词引擎演示");
    println!("==============================");

    // 加载配置
    let config = Config::load()?;

    // 创建提示词引擎
    let engine = PromptEngine::from_config(&config).await?;

    // 显示可用模板
    println!("\n📋 可用的提示词模板:");
    let templates = engine.list_templates().await;
    for template_name in &templates {
        println!("  - {}", template_name);
    }

    // 演示架构分析模板
    println!("\n🏗️ 演示架构分析模板渲染:");
    if engine.has_template("architectural_analysis").await {
        let mut context = HashMap::new();
        context.insert("language".to_string(), "Rust".to_string());
        context.insert(
            "code".to_string(),
            "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
        );
        context.insert("function_count".to_string(), "1".to_string());
        context.insert("class_count".to_string(), "0".to_string());
        context.insert("function_details".to_string(), "main函数".to_string());
        context.insert("class_details".to_string(), "无类".to_string());
        context.insert("dependencies".to_string(), "无依赖".to_string());

        match engine
            .render_prompt("architectural_analysis", &context)
            .await
        {
            Ok(prompt) => {
                println!("✅ 模板渲染成功:");
                println!("---");
                println!("{}", prompt);
                println!("---");
            }
            Err(e) => {
                println!("❌ 模板渲染失败: {}", e);
            }
        }
    } else {
        println!("❌ 架构分析模板未找到");
    }

    // 演示需求验证模板
    println!("\n📋 演示需求验证模板渲染:");
    if engine.has_template("requirement_validation").await {
        let mut context = HashMap::new();
        context.insert(
            "issue_description".to_string(),
            "添加用户登录功能".to_string(),
        );
        context.insert(
            "acceptance_criteria".to_string(),
            "支持用户名密码登录".to_string(),
        );
        context.insert("language".to_string(), "Rust".to_string());
        context.insert(
            "code".to_string(),
            "struct User { username: String, password: String }".to_string(),
        );
        context.insert(
            "implemented_functions".to_string(),
            "User结构体".to_string(),
        );
        context.insert("class_structure".to_string(), "User类".to_string());
        context.insert("key_features".to_string(), "用户数据结构".to_string());

        match engine
            .render_prompt("requirement_validation", &context)
            .await
        {
            Ok(prompt) => {
                println!("✅ 模板渲染成功:");
                println!("---");
                println!("{}", prompt);
                println!("---");
            }
            Err(e) => {
                println!("❌ 模板渲染失败: {}", e);
            }
        }
    } else {
        println!("❌ 需求验证模板未找到");
    }

    // 显示模板信息
    println!("\n📊 模板信息:");
    for template_name in &templates {
        if let Some(template) = engine.get_template(template_name).await {
            println!(
                "📝 {}: {}",
                template_name,
                template.description.as_deref().unwrap_or("无描述")
            );
            println!("   角色: {}", template.role);
            println!("   变量: {:?}", template.variables);
            println!("   支持语言: {:?}", template.supported_languages);
            println!();
        }
    }

    println!("✅ 配置化提示词引擎演示完成！");
    println!("\\n🎯 主要特性:");
    println!("  • 配置驱动的提示词管理");
    println!("  • YAML格式的模板定义");
    println!("  • 运行时动态加载");
    println!("  • 智能变量替换");
    println!("  • 优雅的错误处理和降级");

    Ok(())
}
