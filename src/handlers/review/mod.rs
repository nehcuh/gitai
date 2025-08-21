pub mod analysis;
pub mod ai;
pub mod types;

use crate::{
    config::AppConfig,
    errors::AppError,
    handlers::git::extract_diff_for_review_in_dir,
    types::git::ReviewArgs,
    utils::common,
};
use analysis::DiffAnalyzer;
use ai::AIReviewEngine;
use types::StandardReviewRequest;
use std::sync::Arc;
use std::path::Path;
use chrono::Local;

/// 核心review逻辑，返回分析结果
async fn perform_review(config: &AppConfig, args: ReviewArgs) -> Result<String, AppError> {
    let diff_text = extract_diff_for_review_in_dir(&args, args.path.as_deref()).await?;
    if diff_text.trim().is_empty() {
        return Err(AppError::Generic("没有找到需要审查的代码变更".to_string()));
    }
    
    let config_arc = Arc::new(config.clone());
    let ai_analysis_engine = Arc::new(crate::handlers::analysis::AIAnalysisEngine::new(config_arc.clone()));
    
    let diff_analyzer = DiffAnalyzer::new(config.tree_sitter.clone(), ai_analysis_engine.clone());
    let analysis_result = diff_analyzer.analyze_diff(&diff_text, true).await?;
    
    let ai_engine = AIReviewEngine::new(config_arc, ai_analysis_engine);
    let request = StandardReviewRequest {
        diff_text: diff_text.clone(),
        analysis_text: analysis_result.analysis_text.clone(),
        language_info: analysis_result.language_info.clone(),
    };
    
    ai_engine.perform_standard_review(request).await.map(|result| result.content)
}

/// 执行review并打印结果
pub async fn handle_review(config: &AppConfig, args: ReviewArgs) -> Result<(), AppError> {
    let result = perform_review(config, args.clone()).await?;
    
    // 处理输出到文件
    if let Some(output_path) = &args.output {
        println!("🔍 调试: 检测到output参数: {}", output_path);
        match save_review_to_file(&result, output_path, &args).await {
            Ok(()) => println!("✅ Review结果已保存到: {}", output_path),
            Err(e) => println!("❌ 保存失败: {}", e),
        }
    } else {
        // 默认保存到缓存目录
        match save_review_to_cache(&result, &args).await {
            Ok(cache_path) => {
                println!("✅ Review结果已保存到缓存: {}", cache_path);
                println!("{}", result);
            }
            Err(e) => {
                println!("⚠️ 缓存保存失败，但仍显示结果: {}", e);
                println!("{}", result);
            }
        }
    }
    Ok(())
}

/// 执行review并返回结果
pub async fn handle_review_with_output(config: &AppConfig, args: ReviewArgs) -> Result<String, AppError> {
    perform_review(config, args).await
}

pub async fn handle_review_with_output_in_dir(
    config: &mut AppConfig,
    args: ReviewArgs,
    _dir: Option<&str>,
) -> Result<String, AppError> {
    handle_review_with_output(config, args).await
}

/// 确定review类型并生成相应的内容
fn determine_review_type_and_content(
    args: &ReviewArgs,
    original_content: &str,
) -> Result<(String, String), AppError> {
    let (review_type, header) = match (&args.commit1, &args.commit2) {
        (Some(commit1), Some(commit2)) => {
            let header = format!(
                "# 代码评审报告 - Commit 比较\n\n**比较范围**: {}..{}\n**评审时间**: {}\n\n---\n\n",
                commit1,
                commit2,
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            ("commit".to_string(), header)
        }
        (Some(commit), None) => {
            let header = format!(
                "# 代码评审报告 - 单个提交分析\n\n**提交**: {}\n**评审时间**: {}\n\n---\n\n",
                commit,
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            ("commit".to_string(), header)
        }
        (None, None) => {
            // 对于没有指定commit的情况，我们通过diff内容中的标识来判断类型
            let review_type = if original_content.contains("<!-- REVIEW_TYPE: staged -->") {
                "staged"
            } else if original_content.contains("<!-- REVIEW_TYPE: working -->") {
                "working"
            } else {
                // 如果没有标识，默认认为是staged
                "staged"
            };
            
            // 移除标识行，避免在AI分析中出现
            let clean_content = original_content
                .lines()
                .filter(|line| !line.contains("<!-- REVIEW_TYPE:"))
                .collect::<Vec<_>>()
                .join("\n");
            
            let header = match review_type {
                "staged" => format!(
                    "# 🚧 PRE-COMMIT 代码评审报告\n\n**状态**: 已暂存但未提交的变更\n**评审时间**: {}\n**⚠️ 注意**: 这些更改尚未提交，请记得提交代码\n\n---\n\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
                "working" => format!(
                    "# 📝 WORKING COPY 代码评审报告\n\n**状态**: 工作区变更（未暂存）\n**评审时间**: {}\n**💡 提示**: 使用 `git add` 暂存这些变更后再提交\n\n---\n\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
                _ => format!(
                    "# 代码评审报告\n\n**评审时间**: {}\n\n---\n\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
            };
            
            let content = format!("{}{}", header, clean_content);
            return Ok((review_type.to_string(), content));
        }
        (None, Some(_)) => {
            return Err(AppError::Generic(
                "如果指定了第二个提交，则必须同时指定第一个提交。".to_string(),
            ))
        }
    };
    
    // 对于commit类型，直接返回原始内容
    let content = format!("{}{}", header, original_content);
    Ok((review_type, content))
}

/// 保存review结果到缓存目录
async fn save_review_to_cache(content: &str, args: &ReviewArgs) -> Result<String, AppError> {
    use std::path::PathBuf;
    use chrono::Local;
    
    // 获取缓存目录路径
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("gitai");
    
    // 确保缓存目录存在
    tokio::fs::create_dir_all(&cache_dir).await
        .map_err(|e| AppError::Generic(format!("无法创建缓存目录: {}", e)))?;
    
    // 生成缓存文件名
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let (review_type, _) = determine_review_type_and_content(args, content)?;
    
    let filename = match review_type.as_str() {
        "staged" => format!("review_STAGED_{}.md", timestamp),
        "working" => format!("review_WORKING_{}.md", timestamp),
        "commit" => {
            if let Some(commit1) = &args.commit1 {
                if let Some(commit2) = &args.commit2 {
                    format!("review_{}_{}.md", commit1, commit2)
                } else {
                    format!("review_{}.md", commit1)
                }
            } else {
                format!("review_{}.md", timestamp)
            }
        }
        _ => format!("review_{}.md", timestamp),
    };
    
    let cache_path = cache_dir.join(filename);
    
    // 使用 save_review_to_file 函数来保存
    save_review_to_file(content, cache_path.to_str().unwrap(), args).await?;
    
    Ok(cache_path.to_string_lossy().to_string())
}

/// 保存review结果到文件
async fn save_review_to_file(
    content: &str,
    output_path: &str,
    args: &ReviewArgs,
) -> Result<(), AppError> {
    let (review_type, formatted_content) = determine_review_type_and_content(args, content)?;
    
    let path = Path::new(output_path);
    
    // 如果路径是目录，生成文件名
    let final_path = if path.is_dir() || path.to_string_lossy().ends_with('/') {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = match review_type.as_str() {
            "staged" => format!("review_STAGED_{}.md", timestamp),
            "working" => format!("review_WORKING_{}.md", timestamp),
            "commit" => {
                if let Some(commit1) = &args.commit1 {
                    if let Some(commit2) = &args.commit2 {
                        format!("review_{}_{}.md", commit1, commit2)
                    } else {
                        format!("review_{}.md", commit1)
                    }
                } else {
                    format!("review_{}.md", timestamp)
                }
            }
            _ => format!("review_{}.md", timestamp),
        };
        
        path.join(filename)
    } else {
        path.to_path_buf()
    };
    
    // 确保目录存在
    if let Some(parent) = final_path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| AppError::Generic(format!("无法创建目录: {}", e)))?;
    }
    
    // 写入文件
    tokio::fs::write(&final_path, formatted_content).await
        .map_err(|e| AppError::Generic(format!("无法写入文件: {}", e)))?;
    
    Ok(())
}