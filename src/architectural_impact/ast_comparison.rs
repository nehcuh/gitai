// AST 对比引擎
// 用于比较代码变更前后的结构化差异

use crate::tree_sitter::StructuralSummary;
use super::{ArchitecturalImpactAnalysis, BreakingChange, BreakingChangeType, ImpactLevel};

/// 比较两个结构化摘要，识别架构影响变更
pub fn compare_structural_summaries(
    before: &StructuralSummary,
    after: &StructuralSummary,
) -> ArchitecturalImpactAnalysis {
    let mut analysis = ArchitecturalImpactAnalysis::new();
    
    // 记录开始时间
    let start_time = std::time::Instant::now();
    
    // 对比函数变化
    let function_changes = compare_functions(&before.functions, &after.functions);
    for change in function_changes {
        analysis.add_breaking_change(change);
    }
    
    // 对比类/结构体变化
    let class_changes = compare_classes(&before.classes, &after.classes);
    for change in class_changes {
        analysis.add_breaking_change(change);
    }
    
    // 对比导入/导出变化
    let import_changes = compare_imports(&before.imports, &after.imports);
    for change in import_changes {
        analysis.add_breaking_change(change);
    }
    
    // 更新元数据
    analysis.metadata.analyzed_files = 1;
    analysis.metadata.total_changes = analysis.breaking_changes.len();
    analysis.metadata.analysis_duration_ms = start_time.elapsed().as_millis() as u64;
    
    // 生成摘要和 AI 上下文
    analysis.generate_summary();
    analysis.generate_ai_context();
    
    analysis
}

/// 比较函数列表，识别变化
fn compare_functions(
    before_functions: &[crate::tree_sitter::FunctionInfo],
    after_functions: &[crate::tree_sitter::FunctionInfo],
) -> Vec<BreakingChange> {
    use std::collections::HashMap;
    
    let mut changes = Vec::new();
    
    // 创建函数名称到函数信息的映射
    let before_map: HashMap<String, &crate::tree_sitter::FunctionInfo> = before_functions
        .iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    
    let after_map: HashMap<String, &crate::tree_sitter::FunctionInfo> = after_functions
        .iter()
        .map(|f| (f.name.clone(), f))
        .collect();
    
    // 检测函数删除
    for (name, before_func) in &before_map {
        if !after_map.contains_key(name) {
            changes.push(BreakingChange {
                change_type: BreakingChangeType::FunctionRemoved,
                component: name.clone(),
                description: format!("函数 '{}' 被删除", name),
                impact_level: ImpactLevel::Project,
                suggestions: vec![
                    format!("考虑标记 '{}' 为 deprecated 而不是直接删除", name),
                    "提供迁移指南和替代方案".to_string(),
                ],
                before: Some(format_function_signature(before_func)),
                after: None,
                file_path: "unknown".to_string(),
            });
        }
    }
    
    // 检测函数新增和修改
    for (name, after_func) in &after_map {
        if let Some(before_func) = before_map.get(name) {
            // 函数存在，检测变化
            let func_changes = compare_single_function(before_func, after_func);
            changes.extend(func_changes);
        } else {
            // 新增函数
            changes.push(BreakingChange {
                change_type: BreakingChangeType::FunctionAdded,
                component: name.clone(),
                description: format!("新增函数 '{}'", name),
                impact_level: ImpactLevel::Minimal,
                suggestions: vec![
                    "确保新函数的API设计合理".to_string(),
                    "考虑添加相应的文档和示例".to_string(),
                ],
                before: None,
                after: Some(format_function_signature(after_func)),
                file_path: "unknown".to_string(),
            });
        }
    }
    
    changes
}

/// 比较单个函数的变化
fn compare_single_function(
    before: &crate::tree_sitter::FunctionInfo,
    after: &crate::tree_sitter::FunctionInfo,
) -> Vec<BreakingChange> {
    let mut changes = Vec::new();
    
    // 检查参数数量变化
    if before.parameters.len() != after.parameters.len() {
        changes.push(BreakingChange {
            change_type: BreakingChangeType::ParameterCountChanged,
            component: before.name.clone(),
            description: format!(
                "函数 '{}' 的参数数量从 {} 变为 {}",
                before.name,
                before.parameters.len(),
                after.parameters.len()
            ),
            impact_level: ImpactLevel::Module,
            suggestions: vec![
                "考虑使用可选参数或默认值".to_string(),
                "提供向后兼容的包装函数".to_string(),
            ],
            before: Some(format_function_signature(before)),
            after: Some(format_function_signature(after)),
            file_path: "unknown".to_string(),
        });
    }
    
    // 检查返回类型变化
    if before.return_type != after.return_type {
        changes.push(BreakingChange {
            change_type: BreakingChangeType::ReturnTypeChanged,
            component: before.name.clone(),
            description: format!(
                "函数 '{}' 的返回类型从 '{:?}' 变为 '{:?}'",
                before.name,
                before.return_type.as_deref().unwrap_or("void"),
                after.return_type.as_deref().unwrap_or("void")
            ),
            impact_level: ImpactLevel::Module,
            suggestions: vec![
                "评估对调用方的影响".to_string(),
                "考虑使用版本化进行渐进式变更".to_string(),
            ],
            before: Some(format_function_signature(before)),
            after: Some(format_function_signature(after)),
            file_path: "unknown".to_string(),
        });
    }
    
    // 检查可见性变化
    if before.visibility != after.visibility {
        changes.push(BreakingChange {
            change_type: BreakingChangeType::VisibilityChanged,
            component: before.name.clone(),
            description: format!(
                "函数 '{}' 的可见性从 '{:?}' 变为 '{:?}'",
                before.name,
                before.visibility.as_deref().unwrap_or("default"),
                after.visibility.as_deref().unwrap_or("default")
            ),
            impact_level: ImpactLevel::Local,
            suggestions: vec![
                "评估可见性变化的必要性".to_string(),
                "检查是否影响公共API".to_string(),
            ],
            before: Some(format_function_signature(before)),
            after: Some(format_function_signature(after)),
            file_path: "unknown".to_string(),
        });
    }
    
    // 检查参数类型变化（简化检测）
    if before.parameters != after.parameters {
        changes.push(BreakingChange {
            change_type: BreakingChangeType::FunctionSignatureChanged,
            component: before.name.clone(),
            description: format!("函数 '{}' 的签名发生变化", before.name),
            impact_level: ImpactLevel::Module,
            suggestions: vec![
                format!("为 '{}' 保留向后兼容的重载版本", before.name),
                "使用渐进式迁移策略".to_string(),
            ],
            before: Some(format_function_signature(before)),
            after: Some(format_function_signature(after)),
            file_path: "unknown".to_string(),
        });
    }
    
    changes
}

/// 比较类/结构体列表
fn compare_classes(
    before_classes: &[crate::tree_sitter::ClassInfo],
    after_classes: &[crate::tree_sitter::ClassInfo],
) -> Vec<BreakingChange> {
    use std::collections::HashMap;
    
    let mut changes = Vec::new();
    
    // 创建类名到类信息的映射
    let before_map: HashMap<String, &crate::tree_sitter::ClassInfo> = before_classes
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();
    
    let after_map: HashMap<String, &crate::tree_sitter::ClassInfo> = after_classes
        .iter()
        .map(|c| (c.name.clone(), c))
        .collect();
    
    // 检测类删除
    for (name, _before_class) in &before_map {
        if !after_map.contains_key(name) {
            changes.push(BreakingChange {
                change_type: BreakingChangeType::StructureChanged,
                component: name.clone(),
                description: format!("类/结构体 '{}' 被删除", name),
                impact_level: ImpactLevel::Project,
                suggestions: vec![
                    format!("考虑标记 '{}' 为 deprecated", name),
                    "提供迁移指南".to_string(),
                ],
                before: Some(format!("类/结构体 {}", name)),
                after: None,
                file_path: "unknown".to_string(),
            });
        }
    }
    
    // 检测类新增
    for (name, _after_class) in &after_map {
        if !before_map.contains_key(name) {
            changes.push(BreakingChange {
                change_type: BreakingChangeType::StructureChanged,
                component: name.clone(),
                description: format!("新增类/结构体 '{}'", name),
                impact_level: ImpactLevel::Minimal,
                suggestions: vec![
                    "确保新类的设计合理".to_string(),
                    "添加相应的文档".to_string(),
                ],
                before: None,
                after: Some(format!("类/结构体 {}", name)),
                file_path: "unknown".to_string(),
            });
        }
    }
    
    changes
}

/// 比较导入列表
fn compare_imports(
    before_imports: &[String],
    after_imports: &[String],
) -> Vec<BreakingChange> {
    use std::collections::HashSet;
    
    let mut changes = Vec::new();
    
    let before_set: HashSet<&String> = before_imports.iter().collect();
    let after_set: HashSet<&String> = after_imports.iter().collect();
    
    // 检测删除的导入
    for import in &before_set {
        if !after_set.contains(import) {
            changes.push(BreakingChange {
                change_type: BreakingChangeType::ModuleStructureChanged,
                component: format!("导入: {}", import),
                description: format!("删除了导入 '{}'", import),
                impact_level: ImpactLevel::Local,
                suggestions: vec![
                    "检查是否仍需要该依赖".to_string(),
                ],
                before: Some((*import).clone()),
                after: None,
                file_path: "unknown".to_string(),
            });
        }
    }
    
    // 检测新增的导入
    for import in &after_set {
        if !before_set.contains(import) {
            changes.push(BreakingChange {
                change_type: BreakingChangeType::ModuleStructureChanged,
                component: format!("导入: {}", import),
                description: format!("新增了导入 '{}'", import),
                impact_level: ImpactLevel::Minimal,
                suggestions: vec![
                    "确保新依赖是必要的".to_string(),
                ],
                before: None,
                after: Some((*import).clone()),
                file_path: "unknown".to_string(),
            });
        }
    }
    
    changes
}

/// 格式化函数签名
fn format_function_signature(func: &crate::tree_sitter::FunctionInfo) -> String {
    let params = func.parameters.join(", ");
    let return_type = func.return_type.as_deref().unwrap_or("void");
    let visibility = func.visibility.as_deref().unwrap_or("");
    let async_str = if func.is_async { "async " } else { "" };
    
    format!("{}{}{} {}({}) -> {}", visibility, if visibility.is_empty() { "" } else { " " }, async_str, func.name, params, return_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_sitter::{FunctionInfo, StructuralSummary};
    use std::collections::HashMap;

    fn create_test_summary_with_function(name: &str, params: Vec<&str>, return_type: Option<&str>) -> StructuralSummary {
        let function = FunctionInfo {
            name: name.to_string(),
            parameters: params.into_iter().map(|p| p.to_string()).collect(),
            return_type: return_type.map(|rt| rt.to_string()),
            line_start: 1,
            line_end: 5,
            is_async: false,
            visibility: Some("public".to_string()),
        };

        StructuralSummary {
            language: "rust".to_string(),
            functions: vec![function],
            classes: vec![],
            imports: vec![],
            exports: vec![],
            comments: vec![],
            complexity_hints: vec![],
        }
    }

    #[test]
    fn test_empty_comparison() {
        let empty1 = StructuralSummary::default();
        let empty2 = StructuralSummary::default();
        
        let analysis = compare_structural_summaries(&empty1, &empty2);
        assert_eq!(analysis.breaking_changes.len(), 0);
    }

    #[test]
    fn test_function_addition_detection() {
        let before = StructuralSummary::default();
        let after = create_test_summary_with_function("new_function", vec![], None);
        
        let analysis = compare_structural_summaries(&before, &after);
        assert_eq!(analysis.breaking_changes.len(), 1);
        
        let change = &analysis.breaking_changes[0];
        assert_eq!(change.change_type, BreakingChangeType::FunctionAdded);
        assert_eq!(change.component, "new_function");
        assert_eq!(change.impact_level, ImpactLevel::Minimal);
    }
    
    #[test]
    fn test_function_removal_detection() {
        let before = create_test_summary_with_function("old_function", vec![], None);
        let after = StructuralSummary::default();
        
        let analysis = compare_structural_summaries(&before, &after);
        assert_eq!(analysis.breaking_changes.len(), 1);
        
        let change = &analysis.breaking_changes[0];
        assert_eq!(change.change_type, BreakingChangeType::FunctionRemoved);
        assert_eq!(change.component, "old_function");
        assert_eq!(change.impact_level, ImpactLevel::Project);
    }
    
    #[test]
    fn test_function_parameter_change_detection() {
        let before = create_test_summary_with_function("test_func", vec!["i32"], None);
        let after = create_test_summary_with_function("test_func", vec!["i32", "bool"], None);
        
        let analysis = compare_structural_summaries(&before, &after);
        assert!(!analysis.breaking_changes.is_empty());
        
        // 应该检测到参数数量变化
        let param_change = analysis.breaking_changes.iter()
            .find(|c| c.change_type == BreakingChangeType::ParameterCountChanged);
        assert!(param_change.is_some());
    }
    
    #[test]
    fn test_function_return_type_change_detection() {
        let before = create_test_summary_with_function("test_func", vec![], Some("i32"));
        let after = create_test_summary_with_function("test_func", vec![], Some("String"));
        
        let analysis = compare_structural_summaries(&before, &after);
        assert!(!analysis.breaking_changes.is_empty());
        
        // 应该检测到返回类型变化
        let return_type_change = analysis.breaking_changes.iter()
            .find(|c| c.change_type == BreakingChangeType::ReturnTypeChanged);
        assert!(return_type_change.is_some());
    }
    
    #[test]
    fn test_import_changes_detection() {
        let mut before = StructuralSummary::default();
        before.imports = vec!["std::collections::HashMap".to_string()];
        
        let mut after = StructuralSummary::default();
        after.imports = vec!["std::collections::HashSet".to_string()];
        
        let analysis = compare_structural_summaries(&before, &after);
        assert_eq!(analysis.breaking_changes.len(), 2); // 一个删除，一个新增
        
        // 检查是否有模块结构变更
        let module_changes: Vec<_> = analysis.breaking_changes.iter()
            .filter(|c| c.change_type == BreakingChangeType::ModuleStructureChanged)
            .collect();
        assert_eq!(module_changes.len(), 2);
    }
}
