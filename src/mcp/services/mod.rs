// MCP 服务实现模块
//
// 包含 GitAI 各种核心功能的 MCP 服务实现

pub mod review;
pub mod commit;
pub mod scan;
pub mod analysis;
pub mod dependency;

// 重新导出服务
pub use review::ReviewService;
pub use commit::CommitService;
pub use scan::ScanService;
pub use analysis::AnalysisService;
pub use dependency::DependencyService;
