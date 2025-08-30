// MCP 服务实现模块
//
// 包含 GitAI 各种核心功能的 MCP 服务实现

pub mod analysis;
pub mod commit;
pub mod dependency;
pub mod review;
pub mod scan;

// 重新导出服务
pub use analysis::AnalysisService;
pub use commit::CommitService;
pub use dependency::DependencyService;
pub use review::ReviewService;
pub use scan::ScanService;
