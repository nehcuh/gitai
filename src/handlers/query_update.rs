use std::process;
use colored::Colorize;
use crate::config::AppConfig;
use crate::errors::AppError;
use crate::tree_sitter_analyzer::analyzer::TreeSitterAnalyzer;

/// 更新查询命令的处理器
pub async fn handle_query_update() -> Result<(), AppError> {
    println!("{}", "正在更新 Tree-sitter 查询文件...".blue());

    // 加载配置
    let config = AppConfig::load()?;
    
    if config.tree_sitter.enabled != Some(true) {
        println!("{}", "Tree-sitter 分析未启用，跳过查询更新".yellow());
        return Ok(());
    }

    // 创建分析器
    let mut analyzer = TreeSitterAnalyzer::new(config.tree_sitter)?;

    println!("{}", "正在重新初始化查询文件...".cyan());
    
    // 重新初始化查询（现在是内置的，不需要下载）
    match analyzer.update_queries().await {
        Ok(_) => {
            println!("{}", "✓ 查询文件重新初始化成功".green());
            
            // 显示支持的语言
            let supported_languages = analyzer.get_query_supported_languages();
            if !supported_languages.is_empty() {
                println!("\n{}", "支持的语言:".blue());
                for language in supported_languages {
                    println!("  • {}", language);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ 查询文件重新初始化失败: {}", e).red());
            process::exit(1);
        }
    }

    Ok(())
}

/// 清理查询缓存的处理器
pub fn handle_query_cleanup() -> Result<(), AppError> {
    println!("{}", "正在清理查询缓存...".blue());

    // 加载配置
    let config = AppConfig::load()?;
    
    if config.tree_sitter.enabled != Some(true) {
        println!("{}", "Tree-sitter 分析未启用，跳过缓存清理".yellow());
        return Ok(());
    }

    // 创建分析器
    let mut analyzer = TreeSitterAnalyzer::new(config.tree_sitter)?;

    // 清理缓存（现在是no-op）
    match analyzer.cleanup_query_cache() {
        Ok(_) => {
            println!("{}", "✓ 查询缓存清理成功（内置查询，无需清理）".green());
        }
        Err(e) => {
            eprintln!("{}", format!("✗ 查询缓存清理失败: {}", e).red());
            process::exit(1);
        }
    }

    Ok(())
}

/// 列出查询状态的处理器
pub fn handle_query_status() -> Result<(), AppError> {
    println!("{}", "查询系统状态:".blue());

    // 加载配置
    let config = AppConfig::load()?;
    
    if config.tree_sitter.enabled != Some(true) {
        println!("{}", "Tree-sitter 分析: 未启用".yellow());
        return Ok(());
    }

    println!("{}", "Tree-sitter 分析: 已启用".green());
    println!("缓存启用: {}", if config.tree_sitter.cache_enabled == Some(true) { "是".green() } else { "否".red() });
    
    // 创建分析器
    let analyzer = TreeSitterAnalyzer::new(config.tree_sitter)?;

    // 显示支持的语言
    let supported_languages = analyzer.get_query_supported_languages();
    
    println!("\n{}", "内置支持的语言:".blue());
    for language in supported_languages {
        println!("  • {}", language);
    }

    // 显示查询系统信息
    println!("\n{}", "查询系统信息:".blue());
    println!("{}", "查询类型: 内置静态查询".green());
    println!("{}", "网络依赖: 无".green());
    println!("{}", "缓存机制: 无需缓存".green());
    println!("{}", "更新方式: 重新编译程序".yellow());

    Ok(())
}

#[cfg(test)]
mod tests {
    // Unused imports removed - these can be re-added when more comprehensive tests are implemented

    #[test]
    fn test_query_handlers() {
        // 这里可以添加更多的单元测试
        // 由于这些函数依赖于配置文件，在测试环境中可能需要模拟
    }
}