//! Adapter registry for AI and DevOps integrations
//!
//! This module provides a minimal, trait-based integration point for external adapters
//! defined in the `gitai-adapters` crate. It allows `gitai-core` to depend only on
//! traits and accept pluggable implementations.

use std::sync::Arc;

/// Registry holding optional external adapters used by the core engine.
#[derive(Default, Clone)]
pub struct AdapterRegistry {
    /// AI adapter (e.g., OpenAI, Ollama) implementing text generation.
    pub ai: Option<Arc<dyn gitai_adapters::ai::AiAdapter>>,
    /// DevOps adapter (e.g., Coding, GitHub) for issues and pull requests.
    pub devops: Option<Arc<dyn gitai_adapters::devops::DevOpsAdapter>>,
}

impl AdapterRegistry {
    /// Create an empty registry (no adapters registered).
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an AI adapter.
    pub fn with_ai(mut self, ai: Arc<dyn gitai_adapters::ai::AiAdapter>) -> Self {
        self.ai = Some(ai);
        self
    }

    /// Register a DevOps adapter.
    pub fn with_devops(mut self, devops: Arc<dyn gitai_adapters::devops::DevOpsAdapter>) -> Self {
        self.devops = Some(devops);
        self
    }
}
