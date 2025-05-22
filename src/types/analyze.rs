use std::collections::HashMap;

use crate::tree_sitter_analyzer::analyzer::ChangedFile;

// Represents the entire Git diff
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub changed_files: Vec<ChangedFile>,
    pub metadata: Option<HashMap<String, String>>,
}

/// 分析深度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisDepth {
    /// 基础分析
    Basic,
    /// 标准分析
    Normal,
    /// 深度分析
    Deep,
}
