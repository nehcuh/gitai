// AI 上下文格式化模块
// 将架构影响分析结果格式化为 AI 友好的文本

use super::{ArchitecturalImpactAnalysis, BreakingChange, RiskLevel};

/// 将架构影响分析转换为 AI 友好的上下文字符串
pub fn format_for_ai_context(analysis: &ArchitecturalImpactAnalysis) -> String {
    if analysis.breaking_changes.is_empty() {
        return format_no_impact_message();
    }
    
    let mut context = String::new();
    
    // 标题和概要
    context.push_str("## 🏗️ 架构影响分析\n\n");
    context.push_str(&format!("**风险级别**: {} {}\n", 
        analysis.risk_level.emoji(), 
        analysis.risk_level.description()));
    
    context.push_str(&format!("**变更摘要**: {}\n\n", analysis.summary));
    
    // 风险摘要
    let risk_summary = super::risk_assessment::generate_risk_summary(analysis);
    context.push_str(&format!("### 📊 风险评估\n{}\n\n", risk_summary));
    
    // 按风险级别分组显示变更
    context.push_str("### 🔍 检测到的变更\n\n");
    
    let critical_changes = get_changes_by_risk(&analysis.breaking_changes, RiskLevel::Critical);
    let high_changes = get_changes_by_risk(&analysis.breaking_changes, RiskLevel::High);
    let medium_changes = get_changes_by_risk(&analysis.breaking_changes, RiskLevel::Medium);
    let low_changes = get_changes_by_risk(&analysis.breaking_changes, RiskLevel::Low);
    
    if !critical_changes.is_empty() {
        context.push_str("#### 🚨 紧急风险变更\n");
        for change in critical_changes {
            context.push_str(&format_change_detail(change));
        }
        context.push('\n');
    }
    
    if !high_changes.is_empty() {
        context.push_str("#### ⚠️ 高风险变更\n");
        for change in high_changes {
            context.push_str(&format_change_detail(change));
        }
        context.push('\n');
    }
    
    if !medium_changes.is_empty() {
        context.push_str("#### ⚡ 中等风险变更\n");
        for change in medium_changes {
            context.push_str(&format_change_detail(change));
        }
        context.push('\n');
    }
    
    if !low_changes.is_empty() {
        context.push_str("#### 💡 低风险变更\n");
        for change in low_changes {
            context.push_str(&format_change_detail(change));
        }
        context.push('\n');
    }
    
    // 建议和缓解措施
    let recommendations = super::risk_assessment::generate_mitigation_recommendations(analysis);
    if !recommendations.is_empty() {
        context.push_str("### 💡 建议和缓解措施\n\n");
        for (i, recommendation) in recommendations.iter().enumerate() {
            context.push_str(&format!("{}. {}\n", i + 1, recommendation));
        }
        context.push('\n');
    }
    
    // 元数据
    context.push_str("### 📋 分析元数据\n\n");
    context.push_str(&format!("- **分析文件数**: {} 个\n", analysis.metadata.analyzed_files));
    context.push_str(&format!("- **总变更数**: {} 个\n", analysis.metadata.total_changes));
    context.push_str(&format!("- **分析耗时**: {} ms\n", analysis.metadata.analysis_duration_ms));
    
    if !analysis.metadata.affected_files.is_empty() {
        context.push_str("- **受影响文件**: ");
        context.push_str(&analysis.metadata.affected_files.join(", "));
        context.push('\n');
    }
    
    context.push_str("\n---\n\n");
    context.push_str("**AI 评审提示**: 在进行代码评审时，请特别关注上述架构影响变更，评估其对项目整体架构的影响，并给出相应的改进建议。\n");
    
    context
}

/// 格式化无影响的消息
fn format_no_impact_message() -> String {
    "## 🏗️ 架构影响分析\n\n✅ **无架构影响**: 此次代码变更没有检测到显著的架构影响，可以安全地进行。\n\n---\n\n**AI 评审提示**: 虽然没有检测到架构风险，但仍建议关注代码质量、性能和可维护性。\n".to_string()
}

/// 根据风险级别过滤变更
fn get_changes_by_risk(changes: &[BreakingChange], target_risk: RiskLevel) -> Vec<&BreakingChange> {
    changes.iter()
        .filter(|change| {
            let change_risk = super::risk_assessment::assess_breaking_change_risk(change);
            change_risk == target_risk
        })
        .collect()
}

/// 格式化单个变更的详细信息
fn format_change_detail(change: &BreakingChange) -> String {
    let mut detail = String::new();
    
    detail.push_str(&format!("- **{}**: `{}`\n", 
        change.change_type.description(), 
        change.component));
    
    detail.push_str(&format!("  - 📝 描述: {}\n", change.description));
    detail.push_str(&format!("  - 📍 文件: `{}`\n", change.file_path));
    detail.push_str(&format!("  - 🎯 影响范围: {:?}\n", change.impact_level));
    
    // 显示变更前后对比
    if let (Some(before), Some(after)) = (&change.before, &change.after) {
        detail.push_str("  - 🔄 变更对比:\n");
        detail.push_str(&format!("    - 变更前: `{}`\n", before));
        detail.push_str(&format!("    - 变更后: `{}`\n", after));
    }
    
    // 显示建议
    if !change.suggestions.is_empty() {
        detail.push_str("  - 💡 建议:\n");
        for suggestion in &change.suggestions {
            detail.push_str(&format!("    - {}\n", suggestion));
        }
    }
    
    detail.push('\n');
    detail
}

/// 生成简化的 AI 上下文（用于长度限制的场景）
pub fn format_condensed_ai_context(analysis: &ArchitecturalImpactAnalysis) -> String {
    if analysis.breaking_changes.is_empty() {
        return "✅ 无架构风险变更".to_string();
    }
    
    let mut context = String::new();
    
    context.push_str(&format!("🏗️ 架构影响: {} ({}个变更)\n", 
        analysis.risk_level.description(),
        analysis.breaking_changes.len()));
    
    // 只显示高风险变更
    let high_risk_changes: Vec<_> = analysis.breaking_changes.iter()
        .filter(|change| {
            let risk = super::risk_assessment::assess_breaking_change_risk(change);
            matches!(risk, RiskLevel::Critical | RiskLevel::High)
        })
        .collect();
    
    if !high_risk_changes.is_empty() {
        context.push_str("⚠️ 高风险变更:\n");
        for change in high_risk_changes {
            context.push_str(&format!("- {} `{}`\n", 
                change.change_type.description(), 
                change.component));
        }
    }
    
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{BreakingChange, BreakingChangeType, ImpactLevel};

    fn create_test_change() -> BreakingChange {
        BreakingChange {
            change_type: BreakingChangeType::FunctionSignatureChanged,
            component: "test_function".to_string(),
            description: "函数签名发生变化".to_string(),
            impact_level: ImpactLevel::Module,
            suggestions: vec!["考虑向后兼容".to_string()],
            before: Some("fn test(a: i32)".to_string()),
            after: Some("fn test(a: i32, b: bool)".to_string()),
            file_path: "src/test.rs".to_string(),
        }
    }

    #[test]
    fn test_format_no_impact() {
        let analysis = ArchitecturalImpactAnalysis::new();
        let context = format_for_ai_context(&analysis);
        assert!(context.contains("无架构影响"));
    }

    #[test]
    fn test_format_with_changes() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        analysis.add_breaking_change(create_test_change());
        
        let context = format_for_ai_context(&analysis);
        assert!(context.contains("架构影响分析"));
        assert!(context.contains("test_function"));
        assert!(context.contains("函数签名发生变化"));
    }

    #[test]
    fn test_condensed_format() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        analysis.add_breaking_change(create_test_change());
        
        let context = format_condensed_ai_context(&analysis);
        assert!(context.contains("架构影响"));
        assert!(context.len() < 500); // 确保是简化版本
    }
}
