//! Risk and severity related types

use serde::{Deserialize, Serialize};

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
