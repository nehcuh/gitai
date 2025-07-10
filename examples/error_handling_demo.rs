#!/usr/bin/env rust-script
//! # GitAI 错误处理和日志记录演示
//! 
//! 这个示例展示了如何使用 GitAI 的增强错误处理和日志记录功能
//! 
//! ## 运行方式
//! 
//! ```bash
//! cargo run --example error_handling_demo
//! ```

use gitai::{
    errors::{
        AppError, ErrorMessage, ErrorSeverity, ErrorCategory, ErrorContext,
        error_codes,
    },
    logging::{LoggingConfig, LoggingEnvironment, LogFormat, init_logging, OperationTimer},
    utils::{ResultExt, OptionExt},
    log_operation, log_error, measure_performance, error_context,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    let logging_config = LoggingConfig::development();
    init_logging(logging_config)?;
    
    println!("🚀 GitAI 错误处理和日志记录功能演示");
    println!("=====================================");
    
    // 1. 演示结构化错误处理
    println!("\n📊 1. 结构化错误处理演示");
    demonstrate_structured_errors()?;
    
    // 2. 演示日志记录功能
    println!("\n📝 2. 日志记录功能演示");
    demonstrate_logging();
    
    // 3. 演示性能监控
    println!("\n⚡ 3. 性能监控演示");
    demonstrate_performance_monitoring();
    
    // 4. 演示错误处理工具
    println!("\n🛠️ 4. 错误处理工具演示");
    demonstrate_error_handling_tools()?;
    
    println!("\n✅ 所有演示完成！");
    Ok(())
}

fn demonstrate_structured_errors() -> Result<(), AppError> {
    println!("创建结构化错误消息...");
    
    // 创建配置错误
    let config_error = ErrorMessage::new(
        error_codes::CONFIG_FILE_NOT_FOUND,
        "Configuration file not found at expected location",
        ErrorSeverity::High,
        ErrorCategory::Configuration,
    )
    .with_details("The system looked for config.toml in the current directory")
    .with_context(error_context!("load_config", "config_file" => "config.toml"));
    
    println!("✨ 配置错误: {}", config_error.message);
    println!("   代码: {}", config_error.code);
    println!("   严重程度: {:?}", config_error.severity);
    println!("   类别: {:?}", config_error.category);
    
    // 创建网络错误
    let network_error = ErrorMessage::new(
        error_codes::SYSTEM_NETWORK_ERROR,
        "Failed to connect to AI service",
        ErrorSeverity::Medium,
        ErrorCategory::Network,
    )
    .with_details("Connection timeout after 30 seconds")
    .with_context(error_context!("ai_request", "endpoint" => "http://localhost:11434"));
    
    println!("🌐 网络错误: {}", network_error.message);
    println!("   代码: {}", network_error.code);
    
    // 转换为 AppError
    let app_error: AppError = config_error.into();
    println!("🔄 转换为 AppError: {}", app_error);
    
    Ok(())
}

fn demonstrate_logging() {
    println!("演示各种日志级别...");
    
    // 结构化日志记录
    log_operation!(info, "system_startup", 
        component = "demo",
        version = "1.0.0"
    );
    
    // 错误日志
    let mock_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    log_error!(mock_error, "file_read",
        file_path = "/tmp/test.txt",
        operation_id = "read_001"
    );
    
    // 普通日志
    tracing::info!("普通信息日志");
    tracing::warn!("警告信息");
    tracing::debug!("调试信息");
    
    println!("📋 日志记录已输出到终端");
}

fn demonstrate_performance_monitoring() {
    println!("演示性能监控...");
    
    // 使用 OperationTimer
    let timer = OperationTimer::new("file_processing")
        .with_metadata("file_count", "100")
        .with_metadata("processing_type", "batch");
    
    // 模拟一些工作
    simulate_work(50);
    
    // 完成计时
    timer.finish();
    
    // 使用宏进行性能测量
    let result = measure_performance!("data_calculation", {
        simulate_work(100);
        "计算完成"
    });
    
    println!("⏱️ 性能监控结果: {}", result);
    
    // 带元数据的性能测量
    let _result = measure_performance_with_metadata!("complex_operation", {
        "input_size" => "1000",
        "algorithm" => "quicksort"
    }, {
        simulate_work(30);
        "排序完成"
    });
}

fn demonstrate_error_handling_tools() -> Result<(), AppError> {
    println!("演示错误处理工具...");
    
    // 演示 ResultExt
    let result: Result<i32, &str> = Err("模拟错误");
    let handled_result = result.with_error_code(
        error_codes::SYSTEM_IO_ERROR,
        "I/O operation failed during demo"
    );
    
    match handled_result {
        Ok(_) => println!("✅ 操作成功"),
        Err(e) => println!("❌ 操作失败: {}", e),
    }
    
    // 演示 OptionExt
    let option: Option<String> = None;
    let handled_option = option.ok_or_error(
        error_codes::CONFIG_VALIDATION_ERROR,
        "Required configuration value is missing"
    );
    
    match handled_option {
        Ok(value) => println!("✅ 值: {}", value),
        Err(e) => println!("❌ 缺少值: {}", e),
    }
    
    // 演示日志记录
    let success_result: Result<String, &str> = Ok("成功".to_string());
    let logged_result = success_result.log_on_error("test_operation");
    println!("📝 记录成功操作: {:?}", logged_result);
    
    let error_result: Result<String, &str> = Err("失败");
    let logged_error = error_result.log_on_error("test_operation");
    println!("📝 记录失败操作: {:?}", logged_error);
    
    Ok(())
}

fn simulate_work(duration_ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = ErrorMessage::new(
            "TEST_001",
            "Test error message",
            ErrorSeverity::High,
            ErrorCategory::Business,
        );
        
        assert_eq!(error.code, "TEST_001");
        assert_eq!(error.message, "Test error message");
        assert_eq!(error.severity, ErrorSeverity::High);
        assert_eq!(error.category, ErrorCategory::Business);
    }

    #[test]
    fn test_performance_timer() {
        let timer = OperationTimer::new("test_operation")
            .with_metadata("test_key", "test_value");
        
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 1);
        
        timer.finish();
    }
}