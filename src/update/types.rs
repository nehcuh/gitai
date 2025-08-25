use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub updates: Vec<UpdateItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItem {
    pub name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDownloadResult {
    pub sources: Vec<String>,
    pub total_rules: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSetupResult {
    pub count: usize,
    pub templates: Vec<String>,
}