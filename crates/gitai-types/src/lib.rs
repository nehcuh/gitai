// GitAI Shared Types
// 统一的类型定义，避免重复和不一致

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============ 严重程度相关 ============

/// 统一的严重程度枚举
/// 用于表示问题、风险、发现等的严重性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// 紧急/关键 - 必须立即处理
    Critical,
    /// 高 - 需要尽快处理
    High,
    /// 中等 - 应该处理但不紧急
    Medium,
    /// 低 - 可以延后处理
    Low,
    /// 信息 - 仅供参考
    Info,
}

impl Severity {
    /// 转换为数值表示（用于排序和比较）
    pub fn to_score(&self) -> u8 {
        match self {
            Severity::Critical => 5,
            Severity::High => 4,
            Severity::Medium => 3,
            Severity::Low => 2,
            Severity::Info => 1,
        }
    }

    /// 转换为颜色代码（用于终端输出）
    pub fn to_color(&self) -> &'static str {
        match self {
            Severity::Critical => "\x1b[91m", // Bright Red
            Severity::High => "\x1b[31m",     // Red
            Severity::Medium => "\x1b[93m",   // Bright Yellow
            Severity::Low => "\x1b[33m",      // Yellow
            Severity::Info => "\x1b[36m",     // Cyan
        }
    }

    /// 转换为 emoji 表示
    pub fn to_emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
            Severity::Info => "ℹ️",
        }
    }
}

/// 统一的风险级别枚举
/// 与 Severity 相似但用于更高层次的风险评估
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// 紧急风险 - 可能导致系统崩溃或数据丢失
    Critical,
    /// 高风险 - 可能破坏功能或兼容性
    High,
    /// 中风险 - 需要关注但影响有限
    Medium,
    /// 低风险 - 影响较小
    Low,
    /// 无风险 - 安全的变更
    None,
}

impl From<Severity> for RiskLevel {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Critical => RiskLevel::Critical,
            Severity::High => RiskLevel::High,
            Severity::Medium => RiskLevel::Medium,
            Severity::Low => RiskLevel::Low,
            Severity::Info => RiskLevel::None,
        }
    }
}

impl From<RiskLevel> for Severity {
    fn from(risk: RiskLevel) -> Self {
        match risk {
            RiskLevel::Critical => Severity::Critical,
            RiskLevel::High => Severity::High,
            RiskLevel::Medium => Severity::Medium,
            RiskLevel::Low => Severity::Low,
            RiskLevel::None => Severity::Info,
        }
    }
}

// ============ 发现和问题相关 ============

/// 统一的发现/问题结构
/// 用于代码审查、安全扫描等场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题标题
    pub title: String,
    /// 详细描述
    pub description: String,
    /// 严重程度
    pub severity: Severity,
    /// 问题类别
    pub category: FindingCategory,
    /// 文件路径
    pub file_path: Option<PathBuf>,
    /// 行号
    pub line: Option<usize>,
    /// 列号
    pub column: Option<usize>,
    /// 代码片段
    pub code_snippet: Option<String>,
    /// 规则ID（如果适用）
    pub rule_id: Option<String>,
    /// 修复建议
    pub suggestions: Vec<String>,
    /// 相关链接
    pub references: Vec<String>,
    /// 元数据
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

/// 发现类别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    /// 安全问题
    Security,
    /// 性能问题
    Performance,
    /// 代码质量
    Quality,
    /// 最佳实践
    BestPractice,
    /// 代码风格
    Style,
    /// 文档问题
    Documentation,
    /// 兼容性问题
    Compatibility,
    /// 架构问题
    Architecture,
    /// 其他
    Other(String),
}

// ============ 破坏性变更相关 ============

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
    /// 函数新增
    FunctionAdded,
    /// 可见性变更
    VisibilityChanged,
    /// 参数数量变更
    ParameterCountChanged,
    /// 返回类型变更
    ReturnTypeChanged,
    /// 数据结构变更
    DataStructureChange,
    /// 接口变更
    InterfaceChange,
    /// 行为变更
    BehaviorChange,
    /// 模块结构变更
    ModuleStructureChanged,
    /// 其他
    Other(String),
}

/// 影响级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactLevel {
    /// 项目级影响
    Project,
    /// 模块级影响
    Module,
    /// 本地影响
    Local,
    /// 最小影响
    Minimal,
}

// ============ 依赖和架构相关 ============

/// 依赖类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    /// 导入/引用
    Import,
    /// 继承
    Inheritance,
    /// 组合
    Composition,
    /// 方法调用
    MethodCall,
    /// 类型依赖
    TypeDependency,
    /// 运行时依赖
    Runtime,
    /// 编译时依赖
    CompileTime,
    /// 测试依赖
    Test,
    /// 其他
    Other(String),
}

/// 节点类型（用于依赖图）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// 模块
    Module,
    /// 类
    Class,
    /// 接口
    Interface,
    /// 函数
    Function,
    /// 结构体
    Struct,
    /// 枚举
    Enum,
    /// Trait
    Trait,
    /// 包
    Package,
    /// 文件
    File,
    /// 其他
    Other(String),
}

/// 架构模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturalPattern {
    /// 单一职责原则
    SingleResponsibility,
    /// 开闭原则
    OpenClosed,
    /// 里氏替换原则
    LiskovSubstitution,
    /// 接口隔离原则
    InterfaceSegregation,
    /// 依赖倒置原则
    DependencyInversion,
    /// 层次违规
    LayerViolation,
    /// 循环依赖
    CircularDependency,
    /// MVC
    Mvc,
    /// MVP
    Mvp,
    /// MVVM
    Mvvm,
    /// 微服务
    Microservice,
    /// 事件驱动
    EventDriven,
    /// 其他
    Other(String),
}

// ============ 代码质量相关 ============

/// 代码复杂度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityLevel {
    /// 非常复杂
    VeryHigh,
    /// 复杂
    High,
    /// 中等
    Medium,
    /// 简单
    Low,
    /// 非常简单
    VeryLow,
}

impl ComplexityLevel {
    /// 从圈复杂度值创建
    pub fn from_cyclomatic(value: usize) -> Self {
        match value {
            0..=5 => ComplexityLevel::VeryLow,
            6..=10 => ComplexityLevel::Low,
            11..=20 => ComplexityLevel::Medium,
            21..=50 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }
}

/// 代码质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// 圈复杂度
    pub cyclomatic_complexity: usize,
    /// 认知复杂度
    pub cognitive_complexity: usize,
    /// 代码行数
    pub lines_of_code: usize,
    /// 重复代码百分比
    pub duplication_percentage: f64,
    /// 测试覆盖率
    pub test_coverage: Option<f64>,
    /// 技术债务（小时）
    pub technical_debt_hours: f64,
    /// 可维护性指数
    pub maintainability_index: f64,
}

// ============ Git 相关 ============

/// Git 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// 当前提交
    pub current_commit: String,
    /// 基准提交
    pub base_commit: Option<String>,
    /// 分支名称
    pub branch: Option<String>,
    /// 作者
    pub author: Option<String>,
    /// 提交时间
    pub commit_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 提交消息
    pub commit_message: Option<String>,
}

// ============ 通用结构 ============

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// 文件路径
    pub file: PathBuf,
    /// 起始行
    pub start_line: usize,
    /// 结束行
    pub end_line: Option<usize>,
    /// 起始列
    pub start_column: Option<usize>,
    /// 结束列
    pub end_column: Option<usize>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间
    pub start: chrono::DateTime<chrono::Utc>,
    /// 结束时间
    pub end: chrono::DateTime<chrono::Utc>,
}

/// 统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Statistics {
    /// 总数
    pub total: usize,
    /// 成功数
    pub success: usize,
    /// 失败数
    pub failure: usize,
    /// 跳过数
    pub skipped: usize,
    /// 警告数
    pub warnings: usize,
    /// 错误数
    pub errors: usize,
}

// ============ 错误相关 ============

use thiserror::Error;

/// GitAI 统一错误类型
#[derive(Error, Debug)]
pub enum GitAIError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Git operation failed: {0}")]
    Git(String),
    
    #[error("AI service error: {0}")]
    AI(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Analysis error: {0}")]
    Analysis(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GitAIError>;

// ============ Traits ============

/// 可评分的 trait
pub trait Scorable {
    /// 计算评分
    fn score(&self) -> f64;
}

/// 可验证的 trait
pub trait Validatable {
    /// 验证是否有效
    fn validate(&self) -> Result<()>;
}

/// 可合并的 trait
pub trait Mergeable {
    /// 合并另一个实例
    fn merge(&mut self, other: Self);
}

// 为常见类型实现 trait
impl Scorable for Severity {
    fn score(&self) -> f64 {
        self.to_score() as f64 / 5.0
    }
}

impl Scorable for RiskLevel {
    fn score(&self) -> f64 {
        match self {
            RiskLevel::Critical => 1.0,
            RiskLevel::High => 0.8,
            RiskLevel::Medium => 0.5,
            RiskLevel::Low => 0.3,
            RiskLevel::None => 0.0,
        }
    }
}

impl Scorable for ComplexityLevel {
    fn score(&self) -> f64 {
        match self {
            ComplexityLevel::VeryHigh => 1.0,
            ComplexityLevel::High => 0.8,
            ComplexityLevel::Medium => 0.5,
            ComplexityLevel::Low => 0.3,
            ComplexityLevel::VeryLow => 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
    }

    #[test]
    fn test_risk_level_conversion() {
        assert_eq!(RiskLevel::from(Severity::Critical), RiskLevel::Critical);
        assert_eq!(RiskLevel::from(Severity::Info), RiskLevel::None);
    }

    #[test]
    fn test_complexity_from_cyclomatic() {
        assert_eq!(ComplexityLevel::from_cyclomatic(3), ComplexityLevel::VeryLow);
        assert_eq!(ComplexityLevel::from_cyclomatic(15), ComplexityLevel::Medium);
        assert_eq!(ComplexityLevel::from_cyclomatic(100), ComplexityLevel::VeryHigh);
    }

    #[test]
    fn test_scorable_trait() {
        assert_eq!(Severity::Critical.score(), 1.0);
        assert_eq!(Severity::Info.score(), 0.2);
        assert_eq!(RiskLevel::None.score(), 0.0);
        assert_eq!(ComplexityLevel::VeryHigh.score(), 1.0);
    }
}
