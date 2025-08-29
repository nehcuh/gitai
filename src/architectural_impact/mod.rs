use serde::{Deserialize, Serialize};

pub mod ast_comparison;
pub mod breaking_changes;
pub mod risk_assessment;
pub mod ai_context;
pub mod git_state_analyzer;

// 重新导出git_state_analyzer模块的公共类型
pub use git_state_analyzer::{GitStateAnalyzer, ArchitecturalImpact};

/// 架构影响分析的主要结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImpactAnalysis {
    /// 检测到的破坏性变更列表
    pub breaking_changes: Vec<BreakingChange>,
    /// 整体风险级别
    pub risk_level: RiskLevel,
    /// 影响摘要
    pub summary: String,
    /// AI 友好的上下文信息
    pub ai_context: String,
    /// 额外的元数据
    pub metadata: ImpactMetadata,
}

/// 破坏性变更的详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// 变更类型
    pub change_type: BreakingChangeType,
    /// 受影响的组件名称
    pub component: String,
    /// 变更详细描述
    pub description: String,
    /// 影响级别
    pub impact_level: ImpactLevel,
    /// 给开发者的建议
    pub suggestions: Vec<String>,
    /// 变更前的状态
    pub before: Option<String>,
    /// 变更后的状态
    pub after: Option<String>,
    /// 文件路径
    pub file_path: String,
}

/// 破坏性变更的类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakingChangeType {
    /// 函数签名发生变化
    FunctionSignatureChanged,
    /// 函数被移除
    FunctionRemoved,
    /// 新增函数
    FunctionAdded,
    /// 可见性发生变化
    VisibilityChanged,
    /// 参数数量发生变化
    ParameterCountChanged,
    /// 返回类型发生变化
    ReturnTypeChanged,
    /// 结构体/类定义发生变化
    StructureChanged,
    /// 接口/trait 发生变化
    InterfaceChanged,
    /// 模块结构发生变化
    ModuleStructureChanged,
}

/// 风险级别枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 紧急风险 - 导致编译失败或运行时错误
    Critical,
    /// 高风险 - 可能破坏向后兼容性
    High,
    /// 中风险 - 需要注意但不会立即破坏功能
    Medium,
    /// 低风险 - 微小影响或改进
    Low,
    /// 无风险 - 纯粹的添加或改进
    None,
}

/// 影响级别枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// 项目级影响 - 影响整个项目
    Project,
    /// 模块级影响 - 影响特定模块
    Module,
    /// 本地影响 - 仅影响局部区域
    Local,
    /// 微小影响 - 几乎不产生影响
    Minimal,
}

/// 影响分析的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMetadata {
    /// 分析的文件数量
    pub analyzed_files: usize,
    /// 检测到的变更总数
    pub total_changes: usize,
    /// 分析耗时（毫秒）
    pub analysis_duration_ms: u64,
    /// 变更涉及的文件列表
    pub affected_files: Vec<String>,
    /// Git commit 相关信息
    pub git_info: Option<GitInfo>,
}

/// Git 相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// 当前提交的 hash
    pub current_commit: String,
    /// 对比的基准提交 hash
    pub base_commit: String,
    /// 分支名称
    pub branch: Option<String>,
}

impl Default for ArchitecturalImpactAnalysis {
    fn default() -> Self {
        Self {
            breaking_changes: Vec::new(),
            risk_level: RiskLevel::None,
            summary: String::new(),
            ai_context: String::new(),
            metadata: ImpactMetadata {
                analyzed_files: 0,
                total_changes: 0,
                analysis_duration_ms: 0,
                affected_files: Vec::new(),
                git_info: None,
            },
        }
    }
}

impl ArchitecturalImpactAnalysis {
    /// 创建新的架构影响分析实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算整体风险级别
    pub fn calculate_overall_risk(&mut self) {
        if self.breaking_changes.is_empty() {
            self.risk_level = RiskLevel::None;
            return;
        }

        let has_critical = self.breaking_changes.iter().any(|change| {
            matches!(
                change.change_type,
                BreakingChangeType::FunctionRemoved | BreakingChangeType::InterfaceChanged
            )
        });

        let has_high = self.breaking_changes.iter().any(|change| {
            matches!(
                change.change_type,
                BreakingChangeType::FunctionSignatureChanged
                    | BreakingChangeType::ParameterCountChanged
                    | BreakingChangeType::ReturnTypeChanged
                    | BreakingChangeType::VisibilityChanged
            )
        });

        let has_medium = self.breaking_changes.iter().any(|change| {
            matches!(
                change.change_type,
                BreakingChangeType::StructureChanged | BreakingChangeType::ModuleStructureChanged
            )
        });

        self.risk_level = if has_critical {
            RiskLevel::Critical
        } else if has_high {
            RiskLevel::High
        } else if has_medium {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
    }

    /// 添加破坏性变更
    pub fn add_breaking_change(&mut self, change: BreakingChange) {
        self.breaking_changes.push(change);
        self.calculate_overall_risk();
    }

    /// 生成简要摘要
    pub fn generate_summary(&mut self) {
        let count = self.breaking_changes.len();
        if count == 0 {
            self.summary = "未检测到架构影响变更".to_string();
            return;
        }

        let risk_desc = match self.risk_level {
            RiskLevel::Critical => "紧急",
            RiskLevel::High => "高",
            RiskLevel::Medium => "中等",
            RiskLevel::Low => "低",
            RiskLevel::None => "无",
        };

        self.summary = format!(
            "检测到 {} 个架构影响变更，风险级别：{}",
            count, risk_desc
        );
    }

    /// 检查是否有高风险变更
    pub fn has_high_risk_changes(&self) -> bool {
        matches!(self.risk_level, RiskLevel::Critical | RiskLevel::High)
    }

    /// 获取特定类型的变更
    pub fn get_changes_by_type(&self, change_type: BreakingChangeType) -> Vec<&BreakingChange> {
        self.breaking_changes
            .iter()
            .filter(|change| change.change_type == change_type)
            .collect()
    }

    /// 生成 AI 友好的上下文
    pub fn generate_ai_context(&mut self) {
        self.ai_context = crate::architectural_impact::ai_context::format_for_ai_context(self);
    }

    /// 获取 AI 上下文
    pub fn get_ai_context(&self) -> &str {
        &self.ai_context
    }
}

impl RiskLevel {
    /// 获取风险级别的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Critical => "紧急 - 可能导致编译失败或运行时错误",
            RiskLevel::High => "高风险 - 可能破坏向后兼容性",
            RiskLevel::Medium => "中等风险 - 需要注意的架构变更",
            RiskLevel::Low => "低风险 - 轻微的架构调整",
            RiskLevel::None => "无风险 - 无架构影响",
        }
    }

    /// 获取风险级别的emoji表示
    pub fn emoji(&self) -> &'static str {
        match self {
            RiskLevel::Critical => "🚨",
            RiskLevel::High => "⚠️",
            RiskLevel::Medium => "⚡",
            RiskLevel::Low => "💡",
            RiskLevel::None => "✅",
        }
    }
}

impl BreakingChangeType {
    /// 获取变更类型的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            BreakingChangeType::FunctionSignatureChanged => "函数签名变更",
            BreakingChangeType::FunctionRemoved => "函数移除",
            BreakingChangeType::FunctionAdded => "函数新增",
            BreakingChangeType::VisibilityChanged => "可见性变更",
            BreakingChangeType::ParameterCountChanged => "参数数量变更",
            BreakingChangeType::ReturnTypeChanged => "返回类型变更",
            BreakingChangeType::StructureChanged => "结构定义变更",
            BreakingChangeType::InterfaceChanged => "接口定义变更",
            BreakingChangeType::ModuleStructureChanged => "模块结构变更",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architectural_impact_analysis_creation() {
        let analysis = ArchitecturalImpactAnalysis::new();
        assert_eq!(analysis.breaking_changes.len(), 0);
        assert_eq!(analysis.risk_level, RiskLevel::None);
    }

    #[test]
    fn test_risk_level_calculation() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        
        // 添加一个高风险变更
        let change = BreakingChange {
            change_type: BreakingChangeType::FunctionSignatureChanged,
            component: "test_function".to_string(),
            description: "函数签名发生变化".to_string(),
            impact_level: ImpactLevel::Module,
            suggestions: vec!["考虑向后兼容".to_string()],
            before: Some("fn test(a: i32)".to_string()),
            after: Some("fn test(a: i32, b: bool)".to_string()),
            file_path: "src/test.rs".to_string(),
        };

        analysis.add_breaking_change(change);
        assert_eq!(analysis.risk_level, RiskLevel::High);
        assert_eq!(analysis.breaking_changes.len(), 1);
    }

    #[test]
    fn test_summary_generation() {
        let mut analysis = ArchitecturalImpactAnalysis::new();
        analysis.generate_summary();
        assert_eq!(analysis.summary, "未检测到架构影响变更");

        let change = BreakingChange {
            change_type: BreakingChangeType::FunctionAdded,
            component: "new_function".to_string(),
            description: "新增函数".to_string(),
            impact_level: ImpactLevel::Local,
            suggestions: vec![],
            before: None,
            after: Some("fn new_function()".to_string()),
            file_path: "src/new.rs".to_string(),
        };

        analysis.add_breaking_change(change);
        analysis.generate_summary();
        assert!(analysis.summary.contains("1 个架构影响变更"));
    }
}
