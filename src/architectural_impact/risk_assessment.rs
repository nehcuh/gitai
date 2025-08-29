// 风险评估模块
// 基于检测到的变更评估潜在风险

use super::{BreakingChange, RiskLevel, ArchitecturalImpactAnalysis};

/// 评估单个破坏性变更的风险级别
pub fn assess_breaking_change_risk(change: &BreakingChange) -> RiskLevel {
    use super::BreakingChangeType;
    
    match change.change_type {
        // 紧急风险：可能导致编译失败
        BreakingChangeType::FunctionRemoved |
        BreakingChangeType::InterfaceChanged => RiskLevel::Critical,
        
        // 高风险：破坏向后兼容性
        BreakingChangeType::FunctionSignatureChanged |
        BreakingChangeType::ParameterCountChanged |
        BreakingChangeType::ReturnTypeChanged |
        BreakingChangeType::VisibilityChanged => RiskLevel::High,
        
        // 中等风险：需要注意的结构性变更
        BreakingChangeType::StructureChanged |
        BreakingChangeType::ModuleStructureChanged => RiskLevel::Medium,
        
        // 低风险：新增功能
        BreakingChangeType::FunctionAdded => RiskLevel::Low,
    }
}

/// 计算整体风险分数（0-100）
pub fn calculate_risk_score(analysis: &ArchitecturalImpactAnalysis) -> u8 {
    if analysis.breaking_changes.is_empty() {
        return 0;
    }
    
    let mut total_score = 0;
    let mut weight_sum = 0;
    
    for change in &analysis.breaking_changes {
        let (score, weight) = match assess_breaking_change_risk(change) {
            RiskLevel::Critical => (90, 10),  // 关键变更权重最高
            RiskLevel::High => (70, 8),
            RiskLevel::Medium => (40, 5),
            RiskLevel::Low => (15, 2),
            RiskLevel::None => (0, 1),
        };
        
        total_score += score * weight;
        weight_sum += weight;
    }
    
    if weight_sum == 0 {
        0
    } else {
        (total_score / weight_sum).min(100) as u8
    }
}

/// 生成风险摘要报告
pub fn generate_risk_summary(analysis: &ArchitecturalImpactAnalysis) -> String {
    let risk_score = calculate_risk_score(analysis);
    let change_count = analysis.breaking_changes.len();
    
    if change_count == 0 {
        return "✅ 未检测到架构风险".to_string();
    }
    
    let risk_emoji = match analysis.risk_level {
        RiskLevel::Critical => "🚨",
        RiskLevel::High => "⚠️",
        RiskLevel::Medium => "⚡",
        RiskLevel::Low => "💡",
        RiskLevel::None => "✅",
    };
    
    let risk_desc = analysis.risk_level.description();
    
    format!(
        "{} 风险评分：{}/100\n📊 变更数量：{} 个\n📈 风险级别：{}",
        risk_emoji, risk_score, change_count, risk_desc
    )
}

/// 生成风险缓解建议
pub fn generate_mitigation_recommendations(analysis: &ArchitecturalImpactAnalysis) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if analysis.breaking_changes.is_empty() {
        return recommendations;
    }
    
    let risk_score = calculate_risk_score(analysis);
    
    // 基于风险分数的通用建议
    if risk_score >= 70 {
        recommendations.push("🔍 建议进行全面的集成测试".to_string());
        recommendations.push("📋 制定详细的回滚计划".to_string());
        recommendations.push("👥 考虑分阶段发布以降低风险".to_string());
    } else if risk_score >= 40 {
        recommendations.push("🧪 增加单元测试覆盖率".to_string());
        recommendations.push("📝 更新相关文档和API说明".to_string());
    } else if risk_score >= 15 {
        recommendations.push("✅ 确认变更符合预期".to_string());
        recommendations.push("📚 考虑更新使用示例".to_string());
    }
    
    // 基于具体变更类型的建议
    for change in &analysis.breaking_changes {
        let change_suggestions = super::breaking_changes::generate_suggestions(&change.change_type, &change.component);
        for suggestion in change_suggestions {
            if !recommendations.contains(&suggestion) {
                recommendations.push(suggestion);
            }
        }
    }
    
    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{BreakingChange, BreakingChangeType, ImpactLevel};

    fn create_test_change(change_type: BreakingChangeType) -> BreakingChange {
        BreakingChange {
            change_type,
            component: "test_component".to_string(),
            description: "test change".to_string(),
            impact_level: ImpactLevel::Module,
            suggestions: vec![],
            before: None,
            after: None,
            file_path: "test.rs".to_string(),
        }
    }

    #[test]
    fn test_risk_assessment() {
        let critical_change = create_test_change(BreakingChangeType::FunctionRemoved);
        assert_eq!(assess_breaking_change_risk(&critical_change), RiskLevel::Critical);
        
        let low_change = create_test_change(BreakingChangeType::FunctionAdded);
        assert_eq!(assess_breaking_change_risk(&low_change), RiskLevel::Low);
    }

    #[test]
    fn test_risk_score_calculation() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        
        // 空分析应该返回0分
        assert_eq!(calculate_risk_score(&analysis), 0);
        
        // 添加一个高风险变更
        analysis.add_breaking_change(create_test_change(BreakingChangeType::FunctionSignatureChanged));
        let score = calculate_risk_score(&analysis);
        assert!(score > 50);
    }

    #[test]
    fn test_mitigation_recommendations() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        analysis.add_breaking_change(create_test_change(BreakingChangeType::FunctionRemoved));
        
        let recommendations = generate_mitigation_recommendations(&analysis);
        assert!(!recommendations.is_empty());
    }
}
