//! Change and breaking change related types

use crate::risk::{RiskLevel, Severity};
use serde::{Deserialize, Serialize};

/// 统一的破坏性变更结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// 变更类型
    pub change_type: BreakingChangeType,
    /// 受影响的组件
    pub component: String,
    /// 变更描述
    pub description: String,
    /// 风险级别
    pub risk_level: RiskLevel,
    /// 影响级别
    pub impact_level: ImpactLevel,
    /// 受影响的依赖
    pub affected_dependencies: Vec<String>,
    /// 迁移建议
    pub migration_path: Option<String>,
    /// 修复建议
    pub suggestions: Vec<String>,
    /// 变更前的状态
    pub before: Option<String>,
    /// 变更后的状态
    pub after: Option<String>,
    /// 文件路径
    pub file_path: String,
    /// 行号范围
    pub line_range: Option<(usize, usize)>,
}

/// 破坏性变更类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakingChangeType {
    /// API 签名变更
    ApiSignatureChange,
    /// API 移除
    ApiRemoval,
    /// 函数签名变更
    FunctionSignatureChanged,
    /// 函数移除
    FunctionRemoved,
    /// 类型定义变更
    TypeDefinitionChanged,
    /// 类型移除
    TypeRemoved,
    /// 模块路径变更
    ModulePathChanged,
    /// 模块移除
    ModuleRemoved,
    /// 字段变更
    FieldChanged,
    /// 字段移除
    FieldRemoved,
    /// 方法签名变更
    MethodSignatureChanged,
    /// 方法移除
    MethodRemoved,
    /// 枚举变体变更
    EnumVariantChanged,
    /// 枚举变体移除
    EnumVariantRemoved,
    /// 泛型约束变更
    GenericConstraintChanged,
    /// 默认值变更
    DefaultValueChanged,
    /// 可见性变更
    VisibilityChanged,
    /// 依赖版本变更
    DependencyVersionChanged,
    /// 配置格式变更
    ConfigurationFormatChanged,
    /// 其他破坏性变更
    Other(String),
}

/// 影响级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    /// 非常高 - 影响核心功能
    Critical,
    /// 高 - 影响主要功能
    High,
    /// 中等 - 影响部分功能
    Medium,
    /// 低 - 影响较小
    Low,
    /// 最小 - 几乎无影响
    Minimal,
}

impl From<ImpactLevel> for Severity {
    fn from(impact: ImpactLevel) -> Self {
        match impact {
            ImpactLevel::Critical => Severity::Critical,
            ImpactLevel::High => Severity::High,
            ImpactLevel::Medium => Severity::Medium,
            ImpactLevel::Low => Severity::Low,
            ImpactLevel::Minimal => Severity::Info,
        }
    }
}

/// 代码变更统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeStatistics {
    /// 新增行数
    pub lines_added: usize,
    /// 删除行数
    pub lines_removed: usize,
    /// 修改的文件数
    pub files_changed: usize,
    /// 新增文件数
    pub files_added: usize,
    /// 删除文件数
    pub files_removed: usize,
    /// 重命名文件数
    pub files_renamed: usize,
    /// 二进制文件变更数
    pub binary_files_changed: usize,
}
