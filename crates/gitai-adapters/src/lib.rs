//! gitai-adapters
//! External integration adapters traits and common facades.

// #![warn(missing_docs)]  // 暂时关闭文档警告

pub mod ai;
pub mod devops;

pub use ai::*;
pub use devops::*;
