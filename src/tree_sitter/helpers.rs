//! Helper utilities for Tree-sitter analysis.

use crate::tree_sitter::StructuralSummary;

/// Optimized parameter parsing that minimizes allocations and handles nested generics/arrays.
pub(crate) fn parse_parameters_optimized(params_text: &str) -> Vec<String> {
    if params_text.is_empty() {
        return Vec::new();
    }

    let trimmed = params_text.trim_matches(|c| c == '(' || c == ')');
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut params = Vec::with_capacity(5); // pre-allocate
    let mut start = 0;
    let mut bracket_level: i32 = 0;

    for (i, ch) in trimmed.char_indices() {
        match ch {
            '(' | '<' | '[' => bracket_level += 1,
            ')' | '>' | ']' => bracket_level = bracket_level.saturating_sub(1),
            ',' if bracket_level == 0 => {
                if start < i {
                    let param = trimmed[start..i].trim();
                    if !param.is_empty() {
                        params.push(param.to_string());
                    }
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // Last parameter
    if start < trimmed.len() {
        let param = trimmed[start..].trim();
        if !param.is_empty() {
            params.push(param.to_string());
        }
    }

    params
}

/// Optimized complexity hint calculation that reduces repeated computations.
pub(crate) fn calculate_complexity_hints_optimized(summary: &StructuralSummary) -> Vec<String> {
    let mut hints = Vec::with_capacity(10);

    // Function count
    let func_count = summary.functions.len();
    if func_count > 50 {
        hints.push(format!("文件包含{func_count}个函数，建议考虑拆分"));
    }

    // Class count
    let class_count = summary.classes.len();
    if class_count > 10 {
        hints.push(format!("文件包含{class_count}个类，建议考虑模块化"));
    }

    // Long functions
    for func in &summary.functions {
        let line_count = func.line_end.saturating_sub(func.line_start);
        if line_count > 100 {
            hints.push(format!("函数{}过长({}行)，建议拆分", func.name, line_count));
        }
    }

    // Too many parameters
    for func in &summary.functions {
        if func.parameters.len() > 5 {
            hints.push(format!(
                "函数{}参数过多({}个)，建议使用对象封装",
                func.name,
                func.parameters.len()
            ));
        }
    }

    hints
}
