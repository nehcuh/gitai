use crate::config::Config;
use crate::prompts::{PromptManager, PromptContext};
use crate::project_insights::InsightsGenerator;
use serde::{Deserialize, Serialize};

/// AI请求
#[derive(Serialize)]
struct AiRequest {
    model: String,
    messages: Vec<AiMessage>,
    temperature: f32,
}

/// AI消息
#[derive(Serialize, Deserialize)]
struct AiMessage {
    role: String,
    content: String,
}

/// AI响应
#[derive(Deserialize)]
struct AiResponse {
    choices: Vec<AiChoice>,
}

#[derive(Deserialize)]
struct AiChoice {
    message: AiMessage,
}

pub async fn call_ai(config: &Config, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let request = AiRequest {
        model: config.ai.model.clone(),
        messages: vec![
            AiMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant for Git operations.".to_string(),
            },
            AiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: config.ai.temperature,
    };
    
    let mut req = client
        .post(&config.ai.api_url)
        .json(&request);
    if let Some(ref key) = config.ai.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let response = req
        .send()
        .await?;
    
    let ai_response: AiResponse = response.json().await?;
    let content = ai_response.choices.first()
        .ok_or("No response from AI")?
        .message
        .content
        .clone();
    
    Ok(content)
}

pub async fn generate_commit_message(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Generate a concise commit message for the following changes:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}

pub async fn review_code(config: &Config, diff: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Review the following code changes and provide feedback:\n\n{diff}"
    );
    
    call_ai(config, &prompt).await
}

pub async fn call_ai_with_template(
    config: &Config, 
    template_name: &str, 
    context: &PromptContext
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt_manager = PromptManager::new(config.clone());
    let language = prompt_manager.get_language();
    
    // 加载并渲染提示词模板
    let rendered_prompt = prompt_manager.load_and_render(template_name, context, language)?;
    
    // 添加调试信息
    log::debug!("模板名称: {}", template_name);
    log::debug!("渲染后的提示词长度: {}", rendered_prompt.len());
    log::debug!("上下文变量: {:?}", context.variables.keys().collect::<Vec<_>>());
    log::debug!("渲染后的提示词预览: {}", &rendered_prompt[..rendered_prompt.char_indices().nth(500).map(|(i, _)| i).unwrap_or(rendered_prompt.len())]);
    
    let client = reqwest::Client::new();
    
    let request = AiRequest {
        model: config.ai.model.clone(),
        messages: vec![
            AiMessage {
                role: "user".to_string(),
                content: rendered_prompt,
            },
        ],
        temperature: config.ai.temperature,
    };
    
    let mut req = client
        .post(&config.ai.api_url)
        .json(&request);
    if let Some(ref key) = config.ai.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    let response = req
        .send()
        .await?;
    
    let ai_response: AiResponse = response.json().await?;
    let content = ai_response.choices.first()
        .ok_or("No response from AI")?
        .message
        .content
        .clone();
    
    Ok(content)
}

pub async fn review_code_with_template(
    config: &Config, 
    diff: &str, 
    tree_sitter_summary: Option<&str>,
    security_scan_results: &str,
    devops_issue_context: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut context = PromptContext::new()
        .with_variable("diff", diff)
        .with_variable("security_scan_results", security_scan_results)
        .with_variable("devops_issue_context", devops_issue_context);
    
    // 使用增强的架构洞察替代简单的统计
    if let Some(summary) = tree_sitter_summary {
        // 尝试解析为 StructuralSummary 并生成架构洞察
        match serde_json::from_str::<crate::tree_sitter::StructuralSummary>(summary) {
            Ok(structural_summary) => {
                let insights = InsightsGenerator::generate(&structural_summary, None);
                
                // 使用 ProjectInsights 的 to_ai_context 方法
                let ai_context = insights.to_ai_context();
                
                // 构建详细的架构上下文
                let architecture_context = format!(
                    "## 架构洞察分析\n\n{}\
                    \n### 代码结构统计\n\
                    - 函数数量: {}\n\
                    - 类/结构体数量: {}\n\
                    - 复杂度热点: {} 个\n\
                    - 架构层次: {} 层\n\
                    - 架构违规: {} 处\n\
                    - 循环依赖: {} 个\n\
                    - 公开API: {} 个\n\
                    - 技术债务评分: {:.1}\n\n\
                    ### 原始统计信息\n{}",
                    ai_context,
                    structural_summary.functions.len(),
                    structural_summary.classes.len(),
                    insights.quality_hotspots.complexity_hotspots.len(),
                    insights.architecture.architectural_layers.len(),
                    insights.architecture.pattern_violations.len(),
                    insights.architecture.module_dependencies.circular_dependencies.len(),
                    insights.api_surface.public_apis.len(),
                    insights.quality_hotspots.maintenance_burden.technical_debt_score,
                    summary
                );
                
                context = context.with_variable("tree_sitter_summary", &architecture_context);
            },
            Err(_) => {
                // 如果解析失败，使用原始摘要
                context = context.with_variable("tree_sitter_summary", summary);
            }
        }
    } else {
        context = context.with_variable("tree_sitter_summary", "无结构分析信息");
    }
    
    call_ai_with_template(config, "review", &context).await
}

pub async fn generate_commit_message_with_template(
    config: &Config, 
    diff: &str,
    tree_sitter_summary: Option<&str>
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut context = PromptContext::new()
        .with_variable("diff", diff);
    
    // 如果有结构分析，添加架构影响信息
    if let Some(summary) = tree_sitter_summary {
        match serde_json::from_str::<crate::tree_sitter::StructuralSummary>(summary) {
            Ok(structural_summary) => {
                let insights = InsightsGenerator::generate(&structural_summary, None);
                
                // 为提交信息提供关键架构影响信息
                let impact_summary = format!(
                    "架构影响: 函数变更{}个, 类变更{}个, 复杂度热点{}个, API影响{}",
                    structural_summary.functions.len(),
                    structural_summary.classes.len(),
                    insights.quality_hotspots.complexity_hotspots.len(),
                    if !insights.impact_analysis.breaking_changes.is_empty() { 
                        format!("有{}个破坏性变更", insights.impact_analysis.breaking_changes.len())
                    } else { 
                        "兼容".to_string()
                    }
                );
                
                context = context.with_variable("architecture_impact", &impact_summary);
            },
            Err(_) => {}
        }
    }
    
    call_ai_with_template(config, "commit-generator", &context).await
}
