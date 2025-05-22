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
