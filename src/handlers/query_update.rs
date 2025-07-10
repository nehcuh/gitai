use std::process;
use colored::Colorize;
use crate::config::AppConfig;
use crate::errors::AppError;
use crate::tree_sitter_analyzer::analyzer::TreeSitterAnalyzer;

/// 更新查询命令的处理器
pub async fn handle_query_update() -> Result<(), AppError> {
    println!("{}", "正在更新 Tree-sitter 查询文件...".blue());

    // 加载配置
    let config = AppConfig::load().map_err(AppError::Config)?;
    
    if !config.tree_sitter.enabled {
        println!("{}", "Tree-sitter 分析未启用，跳过查询更新".yellow());
        return Ok(());
    }

    // 创建分析器
    let mut analyzer = TreeSitterAnalyzer::new(config.tree_sitter)
        .map_err(AppError::TreeSitter)?;

    println!("{}", "正在强制更新所有查询文件...".cyan());
    
    // 强制更新所有查询
    match analyzer.update_queries().await {
        Ok(_) => {
            println!("{}", "✓ 查询文件更新成功".green());
            
            // 显示支持的语言
            let supported_languages = analyzer.get_query_supported_languages();
            if !supported_languages.is_empty() {
                println!("\n{}", "已下载查询的语言:".blue());
                for language in supported_languages {
                    println!("  • {}", language.green());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ 查询文件更新失败: {}", e).red());
            process::exit(1);
        }
    }

    Ok(())
}

/// 清理查询缓存的处理器
pub fn handle_query_cleanup() -> Result<(), AppError> {
    println!("{}", "正在清理查询缓存...".blue());

    // 加载配置
    let config = AppConfig::load().map_err(AppError::Config)?;
    
    if !config.tree_sitter.enabled {
        println!("{}", "Tree-sitter 分析未启用，跳过缓存清理".yellow());
        return Ok(());
    }

    // 创建分析器
    let mut analyzer = TreeSitterAnalyzer::new(config.tree_sitter)
        .map_err(AppError::TreeSitter)?;

    // 清理缓存
    match analyzer.cleanup_query_cache() {
        Ok(_) => {
            println!("{}", "✓ 查询缓存清理成功".green());
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
    let config = AppConfig::load().map_err(AppError::Config)?;
    
    if !config.tree_sitter.enabled {
        println!("{}", "Tree-sitter 分析: 未启用".yellow());
        return Ok(());
    }

    println!("{}", "Tree-sitter 分析: 已启用".green());
    println!("分析深度: {}", config.tree_sitter.analysis_depth.cyan());
    println!("缓存启用: {}", if config.tree_sitter.cache_enabled { "是".green() } else { "否".red() });
    
    // 在创建分析器之前先保存需要的配置信息
    let supported_languages_builtin = config.tree_sitter.languages.clone();
    let query_config = config.tree_sitter.query_manager_config.clone();
    
    // 创建分析器
    let analyzer = TreeSitterAnalyzer::new(config.tree_sitter)
        .map_err(AppError::TreeSitter)?;

    // 显示支持的语言
    let supported_languages = analyzer.get_query_supported_languages();
    
    println!("\n{}", "内置支持的语言:".blue());
    for language in &supported_languages_builtin {
        println!("  • {}", language.green());
    }

    if !supported_languages.is_empty() {
        println!("\n{}", "已下载查询的语言:".blue());
        for language in supported_languages {
            println!("  • {}", language.cyan());
        }
    } else {
        println!("\n{}", "暂无已下载的查询文件".yellow());
        println!("{}", "提示: 运行 'gitai update-queries' 来下载最新的查询文件".dimmed());
    }

    // 显示查询管理器配置
    println!("\n{}", "查询管理器配置:".blue());
    println!("缓存目录: {}", query_config.cache_dir.display().to_string().cyan());
    println!("缓存TTL: {} 小时", (query_config.cache_ttl / 3600).to_string().cyan());
    println!("自动更新: {}", if query_config.auto_update { "是".green() } else { "否".red() });
    println!("网络超时: {} 秒", query_config.network_timeout.to_string().cyan());

    println!("\n{}", "可用的查询源:".blue());
    for source in &query_config.sources {
        let status = if source.enabled { "启用".green() } else { "禁用".red() };
        println!("  • {} [{}] (优先级: {})", source.name.cyan(), status, source.priority);
        println!("    URL: {}", source.base_url.dimmed());
    }

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