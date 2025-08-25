use crate::config::DevOpsConfig;
use serde::{Deserialize, Serialize};

/// Issue信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub labels: Vec<String>,
    pub url: String,
}

/// DevOps客户端
pub struct DevOpsClient {
    config: DevOpsConfig,
    client: reqwest::Client,
}

impl DevOpsClient {
    pub fn new(config: DevOpsConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .unwrap_or_default();
        
        Self { config, client }
    }
    
    /// 获取单个Issue
    pub async fn get_issue(&self, issue_id: &str) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let clean_id = issue_id.trim_start_matches('#');
        
        match self.config.platform.to_lowercase().as_str() {
            "coding" => self.get_coding_issue(clean_id).await,
            "github" => self.get_github_issue(clean_id).await,
            _ => Err(format!("Unsupported platform: {}", self.config.platform).into()),
        }
    }
    
    /// 获取多个Issues
    pub async fn get_issues(&self, ids: &[String]) -> Result<Vec<Issue>, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        
        for id in ids {
            match self.get_issue(id).await {
                Ok(issue) => issues.push(issue),
                Err(e) => eprintln!("Warning: Failed to fetch issue {}: {}", id, e),
            }
        }
        
        Ok(issues)
    }
    
    /// 获取Coding平台的Issue
    async fn get_coding_issue(&self, issue_id: &str) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/project/{}/issue/{}",
            self.config.base_url,
            self.config.project.as_deref().unwrap_or("default"),
            issue_id
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.config.token))
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch issue: {}", response.status()).into());
        }
        
        let issue_data: serde_json::Value = response.json().await?;
        
        Ok(Issue {
            id: issue_data["Issue"]["Id"].as_u64().unwrap_or(0).to_string(),
            title: issue_data["Issue"]["Title"].as_str().unwrap_or("").to_string(),
            description: issue_data["Issue"]["Content"].as_str().unwrap_or("").to_string(),
            status: issue_data["Issue"]["StatusName"].as_str().unwrap_or("").to_string(),
            priority: issue_data["Issue"]["PriorityName"].as_str().map(|s| s.to_string()),
            assignee: issue_data["Issue"]["Owner"]["Name"].as_str().map(|s| s.to_string()),
            created_at: issue_data["Issue"]["Created_at"].as_str().unwrap_or("").to_string(),
            updated_at: issue_data["Issue"]["Updated_at"].as_str().unwrap_or("").to_string(),
            labels: issue_data["Issue"]["LabelNames"]
                .as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default(),
            url: format!("{}/project/{}/issue/{}", self.config.base_url, 
                         self.config.project.as_deref().unwrap_or("default"), issue_id),
        })
    }
    
    /// 获取GitHub的Issue
    async fn get_github_issue(&self, issue_id: &str) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/repos/{}/issues/{}",
            self.config.project.as_deref().unwrap_or("owner/repo"),
            issue_id
        );
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.config.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch issue: {}", response.status()).into());
        }
        
        let issue_data: serde_json::Value = response.json().await?;
        
        Ok(Issue {
            id: issue_data["number"].as_u64().unwrap_or(0).to_string(),
            title: issue_data["title"].as_str().unwrap_or("").to_string(),
            description: issue_data["body"].as_str().unwrap_or("").to_string(),
            status: if issue_data["state"].as_str().unwrap_or("") == "open" {
                "open".to_string()
            } else {
                "closed".to_string()
            },
            priority: None,
            assignee: issue_data["assignee"]["login"].as_str().map(|s| s.to_string()),
            created_at: issue_data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: issue_data["updated_at"].as_str().unwrap_or("").to_string(),
            labels: issue_data["labels"]
                .as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_default(),
            url: issue_data["html_url"].as_str().unwrap_or("").to_string(),
        })
    }
}