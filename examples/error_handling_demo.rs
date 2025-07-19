#!/usr/bin/env rust-script
//! # GitAI 错误处理演示 (简化版)
//! 
//! 这个示例展示了如何使用 GitAI 的基本错误处理功能
//! 
//! ## 运行方式
//! 
//! ```bash
//! cargo run --example error_handling_demo
//! ```

use gitai::errors::{AppError, ConfigError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GitAI 错误处理功能演示 (简化版)");
    println!("=====================================");
    
    // 1. 演示基本错误类型
    demonstrate_config_error();
    demonstrate_app_error();
    
    println!("✅ 错误处理演示完成");
    Ok(())
}

fn demonstrate_config_error() {
    println!("📁 配置错误示例:");
    
    // 创建一个配置错误
    let config_error = ConfigError::FieldMissing("api_key".to_string());
    println!("   - 字段缺失错误: {:?}", config_error);
    
    let file_error = ConfigError::FileRead(
        "config.toml".to_string(), 
        std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到")
    );
    println!("   - 文件读取错误: {:?}", file_error);
}

fn demonstrate_app_error() {
    println!("🚨 应用错误示例:");
    
    // 创建应用层错误，包装配置错误
    let config_err = ConfigError::FieldMissing("api_url".to_string());
    let app_error = AppError::Config(config_err);
    println!("   - 应用错误包装配置错误: {:?}", app_error);
    
    // 创建通用错误
    let generic_error = AppError::Generic("这是一个通用错误消息".to_string());
    println!("   - 通用应用错误: {:?}", generic_error);
    
    // 创建网络错误
    let network_error = AppError::Network("连接超时".to_string());
    println!("   - 网络错误: {:?}", network_error);
}