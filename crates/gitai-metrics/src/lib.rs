//! GitAI Metrics Module
//!
//! This crate provides metrics collection and quality tracking functionality.

#![warn(missing_docs)]

pub mod collector;
pub mod storage;
pub mod tracker;

/// Re-export commonly used types
pub use tracker::*;
