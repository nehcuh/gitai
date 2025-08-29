#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::tree_sitter::{StructuralSummary, FunctionInfo, ClassInfo};
    
    /// 创建测试用的 StructuralSummary
    fn create_test_summary() -> StructuralSummary {
        StructuralSummary {
            functions: vec![
                FunctionInfo {
                    name: "test_function".to_string(),
                    visibility: "public".to_string(),
                    is_async: false,
                    parameters: vec!["String".to_string()],
                    return_type: Some("i32".to_string()),
                    start_line: 10,
                    end_line: 20,
                    complexity: 5,
                    calls_count: 3,
                },
                FunctionInfo {
                    name: "helper_function".to_string(),
                    visibility: "private".to_string(),
                    is_async: true,
                    parameters: vec!["u64".to_string(), "bool".to_string()],
                    return_type: Some("Result<String>".to_string()),
                    start_line: 25,
                    end_line: 35,
                    complexity: 3,
                    calls_count: 1,
                },
            ],
            classes: vec![
                ClassInfo {
                    name: "TestClass".to_string(),
                    visibility: "public".to_string(),
                    methods_count: 5,
                    fields_count: 3,
                    is_abstract: false,
                    has_tests: true,
                    start_line: 40,
                    end_line: 100,
                },
            ],
            comments: vec![],
            imports: vec![],
            complexity_metrics: Default::default(),
            code_quality_metrics: Default::default(),
        }
    }
    
    /// 测试函数签名变更检测
    #[test]
    fn test_function_signature_change_detection() {
        let mut before = create_test_summary();
        let mut after = before.clone();
        
        // 修改函数参数
        after.functions[0].parameters.push("bool".to_string());
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        assert!(!analysis.breaking_changes.is_empty());
        assert_eq!(analysis.breaking_changes.len(), 1);
        assert_eq!(analysis.breaking_changes[0].component, "test_function");
        assert!(matches!(
            analysis.breaking_changes[0].change_type,
            BreakingChangeType::ParameterCountChanged
        ));
    }
    
    /// 测试函数删除检测
    #[test]
    fn test_function_removal_detection() {
        let before = create_test_summary();
        let mut after = before.clone();
        
        // 删除一个函数
        after.functions.remove(0);
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        assert!(!analysis.breaking_changes.is_empty());
        let removed_changes: Vec<_> = analysis.breaking_changes
            .iter()
            .filter(|c| matches!(c.change_type, BreakingChangeType::FunctionRemoved))
            .collect();
        
        assert_eq!(removed_changes.len(), 1);
        assert_eq!(removed_changes[0].component, "test_function");
    }
    
    /// 测试函数新增检测
    #[test]
    fn test_function_addition_detection() {
        let before = create_test_summary();
        let mut after = before.clone();
        
        // 添加新函数
        after.functions.push(FunctionInfo {
            name: "new_function".to_string(),
            visibility: "public".to_string(),
            is_async: false,
            parameters: vec![],
            return_type: Some("String".to_string()),
            start_line: 150,
            end_line: 160,
            complexity: 2,
            calls_count: 0,
        });
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        let added_changes: Vec<_> = analysis.breaking_changes
            .iter()
            .filter(|c| matches!(c.change_type, BreakingChangeType::FunctionAdded))
            .collect();
        
        assert_eq!(added_changes.len(), 1);
        assert_eq!(added_changes[0].component, "new_function");
    }
    
    /// 测试可见性变更检测
    #[test]
    fn test_visibility_change_detection() {
        let mut before = create_test_summary();
        let mut after = before.clone();
        
        // 修改函数可见性
        after.functions[1].visibility = "public".to_string();
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        let visibility_changes: Vec<_> = analysis.breaking_changes
            .iter()
            .filter(|c| matches!(c.change_type, BreakingChangeType::VisibilityChanged))
            .collect();
        
        assert!(!visibility_changes.is_empty());
        assert_eq!(visibility_changes[0].component, "helper_function");
    }
    
    /// 测试风险级别评估
    #[test]
    fn test_risk_level_assessment() {
        let before = create_test_summary();
        let mut after = before.clone();
        
        // 创建一个高风险变更（删除函数）
        after.functions.remove(0);
        
        let mut analysis = ast_comparison::compare_structural_summaries(&before, &after);
        analysis.calculate_overall_risk();
        
        assert_eq!(analysis.risk_level, RiskLevel::Critical);
    }
    
    /// 测试结构体变更检测
    #[test]
    fn test_class_change_detection() {
        let before = create_test_summary();
        let mut after = before.clone();
        
        // 修改类的方法数量
        after.classes[0].methods_count = 8;
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        let struct_changes: Vec<_> = analysis.breaking_changes
            .iter()
            .filter(|c| matches!(c.change_type, BreakingChangeType::StructureChanged))
            .collect();
        
        assert!(!struct_changes.is_empty());
        assert_eq!(struct_changes[0].component, "TestClass");
    }
    
    /// 测试 AI 上下文生成
    #[test]
    fn test_ai_context_generation() {
        let before = create_test_summary();
        let mut after = before.clone();
        
        // 创建多种变更
        after.functions[0].parameters.push("bool".to_string());
        after.functions.remove(1);
        
        let mut analysis = ast_comparison::compare_structural_summaries(&before, &after);
        analysis.generate_ai_context();
        
        let ai_context = analysis.get_ai_context();
        
        // 验证 AI 上下文包含关键信息
        assert!(ai_context.contains("架构影响分析"));
        assert!(ai_context.contains("高风险") || ai_context.contains("中风险"));
        assert!(!ai_context.is_empty());
    }
    
    /// 测试空变更的处理
    #[test]
    fn test_no_changes() {
        let before = create_test_summary();
        let after = before.clone();
        
        let analysis = ast_comparison::compare_structural_summaries(&before, &after);
        
        assert!(analysis.breaking_changes.is_empty());
        assert_eq!(analysis.risk_level, RiskLevel::None);
    }
    
    /// 测试 GitStateAnalyzer 的 diff 分析
    #[tokio::test]
    async fn test_git_diff_analysis() {
        let analyzer = git_state_analyzer::GitStateAnalyzer::new();
        
        let test_diff = r#"
diff --git a/src/test.rs b/src/test.rs
index abc123..def456 100644
--- a/src/test.rs
+++ b/src/test.rs
@@ -10,7 +10,7 @@
-fn old_function() {
+fn new_function(param: String) {
     println!("Hello");
 }
+
+struct NewStruct {
+    field: String,
+}
"#;
        
        let result = analyzer.analyze_git_diff(test_diff).await;
        
        assert!(result.is_ok());
        let impact = result.unwrap();
        
        // 应该检测到函数和结构体变更
        assert!(!impact.function_changes.is_empty());
        assert!(!impact.struct_changes.is_empty());
        assert!(!impact.impact_summary.affected_modules.is_empty());
    }
    
    /// 测试破坏性变更的影响级别
    #[test]
    fn test_impact_level_assessment() {
        use breaking_changes::assess_change_impact;
        
        // API 删除应该是项目级影响
        let impact = assess_change_impact(&BreakingChangeType::FunctionRemoved, "critical_api");
        assert_eq!(impact, ImpactLevel::Project);
        
        // 参数变更应该是模块级影响
        let impact = assess_change_impact(&BreakingChangeType::ParameterCountChanged, "helper");
        assert_eq!(impact, ImpactLevel::Module);
        
        // 新增函数应该是最小影响
        let impact = assess_change_impact(&BreakingChangeType::FunctionAdded, "new_api");
        assert_eq!(impact, ImpactLevel::Minimal);
    }
}
