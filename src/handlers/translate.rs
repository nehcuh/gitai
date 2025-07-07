use std::fs;
use std::path::{Path, PathBuf};
use colored::Colorize;
use crate::config::AppConfig;
use crate::errors::AppError;
use crate::rule_manager::RuleManager;
use crate::handlers::ai;
use crate::types::ai::ChatMessage;
use crate::types::git::TranslateArgs;

/// 处理翻译命令
pub async fn handle_translate(config: &AppConfig, args: TranslateArgs) -> Result<(), AppError> {
    match args.target.as_str() {
        "rules" => {
            handle_translate_rules(config, args).await
        }
        _ => {
            eprintln!("{}", format!("❌ 不支持的翻译目标: {}", args.target).red());
            eprintln!("{}", "支持的翻译目标: rules".yellow());
            Err(AppError::Generic("不支持的翻译目标".to_string()))
        }
    }
}

/// 处理规则翻译
async fn handle_translate_rules(config: &AppConfig, args: TranslateArgs) -> Result<(), AppError> {
    println!("{}", format!("🌐 开始翻译代码扫描规则到{}语言...", args.to_language).blue());
    
    // 初始化规则管理器
    let mut rule_manager = RuleManager::new(config.scan.rule_manager.clone())
        .map_err(|e| AppError::Generic(format!("规则管理器初始化失败: {}", e)))?;
    
    // 获取规则文件路径，如果需要会自动更新
    println!("{}", "📥 检查规则更新...".yellow());
    let rule_paths = rule_manager.get_rule_paths(false).await
        .map_err(|e| AppError::Generic(format!("获取规则路径失败: {}", e)))?;
    
    if rule_paths.is_empty() {
        return Err(AppError::Generic("未发现任何规则文件".to_string()));
    }
    
    // 获取规则目录（从第一个规则文件推导）
    let rules_dir = rule_paths[0].parent()
        .ok_or_else(|| AppError::Generic("无法确定规则目录".to_string()))?
        .to_path_buf();
    
    // 向上查找到包含rules目录的根目录
    let mut rules_dir = rules_dir;
    while rules_dir.file_name() != Some(std::ffi::OsStr::new("rules")) && rules_dir.parent().is_some() {
        rules_dir = rules_dir.parent().unwrap().to_path_buf();
    }
    if !rules_dir.exists() {
        return Err(AppError::Generic("规则目录不存在".to_string()));
    }
    
    // 设置翻译输出目录 - 使用目标语言目录
    let translated_dir = args.output.unwrap_or_else(|| {
        rules_dir.parent().unwrap().join(&args.to_language)
    });
    
    if !translated_dir.exists() {
        fs::create_dir_all(&translated_dir)
            .map_err(|e| AppError::Generic(format!("创建翻译目录失败: {}", e)))?;
    }
    
    println!("{}", format!("📂 规则目录: {}", rules_dir.display()).cyan());
    println!("{}", format!("📂 {}语言翻译输出目录: {}", args.to_language, translated_dir.display()).cyan());
    
    // 加载translator prompt
    let translator_prompt = load_translator_prompt(config)?;
    
    // 扫描规则文件
    let rule_files = scan_rule_files(&rules_dir)?;
    println!("{}", format!("🔍 发现 {} 个规则文件", rule_files.len()).green());
    
    if rule_files.is_empty() {
        println!("{}", "⚠️ 未发现任何规则文件".yellow());
        return Ok(());
    }
    
    // 翻译规则文件
    let mut translated_count = 0;
    let mut skipped_count = 0;
    
    for rule_file in rule_files {
        let relative_path = rule_file.strip_prefix(&rules_dir)
            .map_err(|e| AppError::Generic(format!("计算相对路径失败: {}", e)))?;
        
        let output_file = translated_dir.join(relative_path);
        
        // 检查是否需要翻译
        if !args.force && output_file.exists() {
            let rule_modified = get_file_modified_time(&rule_file)?;
            let translated_modified = get_file_modified_time(&output_file)?;
            
            if translated_modified >= rule_modified {
                skipped_count += 1;
                continue;
            }
        }
        
        // 创建输出目录
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Generic(format!("创建输出目录失败: {}", e)))?;
        }
        
        // 翻译单个文件
        println!("{}", format!("🌐 翻译: {}", relative_path.display()).cyan());
        
        match translate_rule_file(config, &rule_file, &output_file, &translator_prompt).await {
            Ok(_) => {
                translated_count += 1;
                println!("{}", format!("  ✅ 完成: {}", relative_path.display()).green());
            }
            Err(e) => {
                eprintln!("{}", format!("  ❌ 失败: {}: {}", relative_path.display(), e).red());
            }
        }
    }
    
    // 输出统计信息
    println!("\n{}", "📊 翻译统计:".blue());
    println!("{}", format!("  ✅ 翻译完成: {} 个文件", translated_count).green());
    println!("{}", format!("  ⏭️  跳过: {} 个文件", skipped_count).yellow());
    println!("{}", format!("  📁 输出目录: {}", translated_dir.display()).cyan());
    
    Ok(())
}

/// 加载translator prompt
fn load_translator_prompt(config: &AppConfig) -> Result<String, AppError> {
    let translator_path = config.get_prompt_path("translator")?;
    
    if !translator_path.exists() {
        return Err(AppError::Generic(format!(
            "翻译器prompt文件不存在: {}",
            translator_path.display()
        )));
    }
    
    fs::read_to_string(&translator_path)
        .map_err(|e| AppError::Generic(format!("读取翻译器prompt失败: {}", e)))
}

/// 扫描规则文件
fn scan_rule_files(rules_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut rule_files = Vec::new();
    
    fn scan_directory(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
        let entries = fs::read_dir(dir)
            .map_err(|e| AppError::Generic(format!("读取目录失败: {}: {}", dir.display(), e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| AppError::Generic(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                scan_directory(&path, files)?;
            } else if let Some(extension) = path.extension() {
                if extension == "yml" || extension == "yaml" {
                    files.push(path);
                }
            }
        }
        
        Ok(())
    }
    
    scan_directory(rules_dir, &mut rule_files)?;
    rule_files.sort();
    
    Ok(rule_files)
}

/// 翻译单个规则文件
async fn translate_rule_file(
    config: &AppConfig,
    input_file: &Path,
    output_file: &Path,
    translator_prompt: &str,
) -> Result<(), AppError> {
    // 读取原始规则文件
    let rule_content = fs::read_to_string(input_file)
        .map_err(|e| AppError::Generic(format!("读取规则文件失败: {}", e)))?;
    
    // 构建翻译请求
    let translation_request = format!(
        "{}\n\n# 要翻译的规则文件内容：\n\n```yaml\n{}\n```",
        translator_prompt,
        rule_content
    );
    
    // 调用AI进行翻译
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: translation_request,
        }
    ];
    
    let translated_content = ai::execute_ai_request_generic(config, messages, "翻译", false).await
        .map_err(|e| AppError::AI(e))?;
    
    // 提取翻译后的YAML内容
    let final_content = extract_yaml_from_translation(&translated_content);
    
    // 写入翻译后的文件
    fs::write(output_file, final_content)
        .map_err(|e| AppError::Generic(format!("写入翻译文件失败: {}", e)))?;
    
    Ok(())
}

/// 从翻译结果中提取YAML内容
fn extract_yaml_from_translation(translation: &str) -> String {
    // 查找自由翻译部分
    if let Some(free_translation_start) = translation.find("自由翻译") {
        let free_translation_part = &translation[free_translation_start..];
        
        // 在自由翻译部分查找YAML代码块
        if let Some(yaml_start) = free_translation_part.find("```yaml") {
            let yaml_content_start = yaml_start + 7; // 跳过 "```yaml"
            if let Some(yaml_end) = free_translation_part[yaml_content_start..].find("```") {
                let yaml_content = &free_translation_part[yaml_content_start..yaml_content_start + yaml_end];
                return yaml_content.trim().to_string();
            }
        }
    }
    
    // 如果没有找到自由翻译部分，查找任何YAML代码块
    if let Some(yaml_start) = translation.find("```yaml") {
        let yaml_content_start = yaml_start + 7;
        if let Some(yaml_end) = translation[yaml_content_start..].find("```") {
            let yaml_content = &translation[yaml_content_start..yaml_content_start + yaml_end];
            return yaml_content.trim().to_string();
        }
    }
    
    // 如果都没找到，返回原始内容（作为fallback）
    translation.to_string()
}

/// 获取文件修改时间
fn get_file_modified_time(file_path: &Path) -> Result<std::time::SystemTime, AppError> {
    let metadata = fs::metadata(file_path)
        .map_err(|e| AppError::Generic(format!("获取文件元数据失败: {}", e)))?;
    
    metadata.modified()
        .map_err(|e| AppError::Generic(format!("获取文件修改时间失败: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_yaml_from_translation() {
        let translation = r#"
逐字翻译

这是一些逐字翻译的内容...

评估和反思

这是评估内容...

自由翻译

```yaml
id: test-rule
language: rust
message: 这是翻译后的消息
```
"#;
        
        let result = extract_yaml_from_translation(translation);
        assert!(result.contains("id: test-rule"));
        assert!(result.contains("language: rust"));
        assert!(result.contains("这是翻译后的消息"));
    }
    
    #[test]
    fn test_extract_yaml_fallback() {
        let translation = r#"
这是一些普通内容

```yaml
id: fallback-rule
language: python
```

更多内容...
"#;
        
        let result = extract_yaml_from_translation(translation);
        assert!(result.contains("id: fallback-rule"));
        assert!(result.contains("language: python"));
    }
}