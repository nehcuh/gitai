// review 模块 - 模块化重构
// 将原本超过1000行的 review.rs 拆分为多个职责清晰的子模块

pub mod analyzer;
pub mod cache;
pub mod converter;
pub mod executor;
pub mod types;

// 重新导出核心类型和函数
pub use analyzer::{perform_architectural_impact_analysis, perform_structural_analysis};
pub use cache::{build_cache_key, check_cache, save_cache};
pub use converter::{convert_analysis_result, convert_analysis_result_with_critical_check};
pub use executor::{execute_review, execute_review_with_result};
pub use types::{Finding, ReviewCache, ReviewConfig, ReviewResult, Severity};

// 保持向后兼容
#[deprecated(
    note = "Use static functions execute_review and execute_review_with_result instead"
)]
pub use executor::ReviewExecutor;
