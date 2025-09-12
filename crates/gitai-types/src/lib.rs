//! GitAI Shared Types
//! 统一的类型定义，避免重复和不一致

#![warn(missing_docs)]

pub mod change;
pub mod common;
pub mod error;
pub mod risk;

// Re-export commonly used types
pub use change::*;
pub use common::*;
pub use error::*;
pub use risk::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        assert_eq!(
            ComplexityLevel::from_cyclomatic(3),
            ComplexityLevel::VeryLow
        );
        assert_eq!(
            ComplexityLevel::from_cyclomatic(15),
            ComplexityLevel::Medium
        );
        assert_eq!(
            ComplexityLevel::from_cyclomatic(100),
            ComplexityLevel::VeryHigh
        );
    }

    #[test]
    fn test_scorable_trait() {
        assert_eq!(Severity::Critical.score(), 1.0);
        assert_eq!(Severity::Info.score(), 0.2);
        assert_eq!(RiskLevel::None.score(), 0.0);
        assert_eq!(ComplexityLevel::VeryHigh.score(), 1.0);
    }
}
