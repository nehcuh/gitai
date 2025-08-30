// review 模块 - 模块化重构
// 将原本超过1000行的 review.rs 拆分为多个职责清晰的子模块

pub mod executor;
pub mod cache;
pub mod converter;
pub mod analyzer;
pub mod types;

// 重新导出核心类型和函数
pub use types::{ReviewResult, Finding, Severity, ReviewConfig, ReviewCache};
pub use executor::{execute_review, execute_review_with_result};
pub use analyzer::{perform_structural_analysis, perform_architectural_impact_analysis};
pub use cache::{check_cache, save_cache, build_cache_key};
pub use converter::{convert_analysis_result, convert_analysis_result_with_critical_check};

// 保持向后兼容
#[deprecated(note = "Use static functions execute_review and execute_review_with_result instead")]
pub use executor::ReviewExecutor;
