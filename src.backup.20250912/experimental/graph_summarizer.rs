// Experimental: LLM-friendly graph summarization types (not wired yet)
// This file is intentionally not included in lib.rs to avoid build changes.

#[allow(dead_code)]
pub mod graph_summary {
    #[derive(Clone, Debug, Default)]
    pub struct GraphStats {
        pub nodes: usize,
        pub edges: usize,
        pub avg_degree: f32,
        pub components: usize,
    }

    #[derive(Clone, Debug)]
    pub enum Scope {
        SeedOnly,
        Module,
        Community,
        Full,
    }

    impl Default for Scope {
        fn default() -> Self { Scope::Community }
    }

    #[derive(Clone, Debug, Default)]
    pub struct GraphSummaryParams {
        pub seeds: Vec<String>,
        pub radius: usize,          // default: 1
        pub top_k: usize,           // default: 200
        pub with_communities: bool, // default: true
        pub with_paths: bool,       // default: true
        pub path_samples: usize,    // default: 5
        pub path_max_hops: usize,   // default: 5
        pub budget_tokens: usize,   // default: 3000
        pub include_filters: Vec<String>,
        pub exclude_filters: Vec<String>,
        pub scope: Scope,           // SeedOnly | Module | Community | Full
        pub language_filters: Vec<String>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct NodeLabel {
        pub id: String,
        pub label: String,
        pub module: Option<String>,
        pub pr_score: Option<f32>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct CommunitySummary {
        pub id: String,
        pub name: String,
        pub size: usize,
        pub cross_edges: usize,
        pub samples: Vec<NodeLabel>, // limited
    }

    #[derive(Clone, Debug, Default)]
    pub struct CrossEdgeBucket {
        pub src_comm: String,
        pub dst_comm: String,
        pub types: Vec<(String, usize)>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct GraphSummary {
        pub graph_stats: GraphStats,
        pub seeds_preview: Vec<NodeLabel>,
        pub top_nodes: Vec<NodeLabel>,
        pub communities: Vec<CommunitySummary>,
        pub cross_edges_summary: Vec<CrossEdgeBucket>,
        pub path_examples: Vec<Vec<NodeLabel>>, // paths as sequences of nodes
        pub impacted_summary: Vec<(String, usize, f32)>, // (name, size, ratio)
        pub truncated: bool,
    }

    pub trait GraphSummarizer {
        fn summarize(&self, params: &GraphSummaryParams) -> GraphSummary;
    }
}

