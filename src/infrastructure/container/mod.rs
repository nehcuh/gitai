//! Container module (v2 as default)

pub mod v2;

// Re-export primary types from v2
pub use v2::{ContainerError, ServiceContainer};

// Lifecycle enum kept at container module level so v2 can reference it via `super::ServiceLifetime`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// Single instance for the entire application lifetime
    Singleton,
    /// New instance per resolve
    Transient,
    /// Per-scope instance (shared within active scope)
    Scoped,
}
