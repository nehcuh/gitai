// 破坏性变更检测模块
// 识别可能破坏向后兼容性的代码变更

use super::{BreakingChange, BreakingChangeType, ImpactLevel};
use crate::tree_sitter::{ClassInfo, FunctionInfo};

/// 检测函数相关的破坏性变更
pub fn detect_function_breaking_changes(
    _before_functions: &[FunctionInfo],
    _after_functions: &[FunctionInfo],
    _file_path: &str,
) -> Vec<BreakingChange> {
    // TODO: 实现函数破坏性变更检测逻辑
    // 1. 检测函数删除
    // 2. 检测函数签名变更
    // 3. 检测参数变更
    // 4. 检测返回类型变更
    // 5. 检测可见性变更

    Vec::new()
}

/// 检测类/结构体相关的破坏性变更
pub fn detect_class_breaking_changes(
    _before_classes: &[ClassInfo],
    _after_classes: &[ClassInfo],
    _file_path: &str,
) -> Vec<BreakingChange> {
    // TODO: 实现类/结构体破坏性变更检测逻辑
    // 1. 检测类/结构体删除
    // 2. 检测字段变更
    // 3. 检测可见性变更

    Vec::new()
}

/// 评估破坏性变更的影响级别
pub fn assess_change_impact(change_type: &BreakingChangeType, _component: &str) -> ImpactLevel {
    match change_type {
        BreakingChangeType::FunctionRemoved | BreakingChangeType::InterfaceChanged => {
            ImpactLevel::Project
        }

        BreakingChangeType::FunctionSignatureChanged
        | BreakingChangeType::ParameterCountChanged
        | BreakingChangeType::ReturnTypeChanged => ImpactLevel::Module,

        BreakingChangeType::VisibilityChanged | BreakingChangeType::StructureChanged => {
            ImpactLevel::Local
        }

        BreakingChangeType::FunctionAdded | BreakingChangeType::ModuleStructureChanged => {
            ImpactLevel::Minimal
        }
    }
}

/// 生成变更建议
pub fn generate_suggestions(change_type: &BreakingChangeType, component: &str) -> Vec<String> {
    match change_type {
        BreakingChangeType::FunctionRemoved => {
            vec![
                format!("考虑标记 '{}' 为 deprecated 而不是直接删除", component),
                "提供迁移指南和替代方案".to_string(),
                "确保所有调用方已经更新".to_string(),
            ]
        }

        BreakingChangeType::FunctionSignatureChanged => {
            vec![
                format!("为 '{}' 保留向后兼容的重载版本", component),
                "使用渐进式迁移策略".to_string(),
                "更新所有相关文档和示例".to_string(),
            ]
        }

        BreakingChangeType::ParameterCountChanged => {
            vec![
                "考虑使用可选参数或默认值".to_string(),
                "提供向后兼容的包装函数".to_string(),
            ]
        }

        _ => vec!["检查变更对下游依赖的影响".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_assessment() {
        assert_eq!(
            assess_change_impact(&BreakingChangeType::FunctionRemoved, "test_fn"),
            ImpactLevel::Project
        );

        assert_eq!(
            assess_change_impact(&BreakingChangeType::FunctionAdded, "new_fn"),
            ImpactLevel::Minimal
        );
    }

    #[test]
    fn test_suggestion_generation() {
        let suggestions =
            generate_suggestions(&BreakingChangeType::FunctionRemoved, "old_function");
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("deprecated"));
    }
}
