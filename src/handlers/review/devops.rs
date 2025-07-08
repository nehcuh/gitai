use crate::{
    clients::devops_client::DevOpsClient,
    config::AppConfig,
    errors::AppError,
    types::{devops::WorkItem, git::ReviewArgs},
};
use std::env;
use tracing;

/// Handles DevOps platform integration for work item fetching
pub struct DevOpsWorkItemFetcher {
    client: DevOpsClient,
}

impl DevOpsWorkItemFetcher {
    /// Create a new DevOps work item fetcher from configuration
    pub fn new(config: &AppConfig) -> Self {
        let client = match &config.account {
            Some(account_config) => {
                tracing::info!(
                    "使用配置文件中的 DevOps 配置: platform={}, base_url={}",
                    account_config.devops_platform,
                    account_config.base_url
                );
                DevOpsClient::new(account_config.base_url.clone(), account_config.token.clone())
            }
            None => {
                // Fallback to environment variables if no config found
                let devops_base_url = env::var("DEV_DEVOPS_API_BASE_URL")
                    .unwrap_or_else(|_| "https://codingcorp.devops.xxx.com.cn".to_string());
                let devops_token = env::var("DEV_DEVOPS_API_TOKEN")
                    .unwrap_or_else(|_| "your_placeholder_token".to_string());

                if devops_token == "your_placeholder_token" {
                    tracing::warn!(
                        "未找到 DevOps 配置且环境变量使用占位符。请在 ~/.config/gitai/config.toml 中配置 [account] 部分或设置环境变量。"
                    );
                } else {
                    tracing::info!("使用环境变量中的 DevOps 配置（配置文件中未找到 [account] 配置）");
                }
                DevOpsClient::new(devops_base_url, devops_token)
            }
        };

        Self { client }
    }

    /// Validate work item arguments
    pub fn validate_work_item_args(&self, args: &ReviewArgs) -> Result<(), AppError> {
        if (args.stories.is_some() || args.tasks.is_some() || args.defects.is_some())
            && args.space_id.is_none()
        {
            return Err(AppError::Generic(
                "When specifying stories, tasks, or defects, --space-id is required.".to_string(),
            ));
        }
        Ok(())
    }

    /// Fetch work items based on review arguments
    pub async fn fetch_work_items(&self, args: &ReviewArgs) -> Result<Vec<WorkItem>, AppError> {
        // Validate arguments first
        self.validate_work_item_args(args)?;

        // Collect all work item IDs
        let mut all_work_item_ids: Vec<u32> = Vec::new();
        if let Some(stories) = &args.stories {
            all_work_item_ids.extend(&stories.0);
        }
        if let Some(tasks) = &args.tasks {
            all_work_item_ids.extend(&tasks.0);
        }
        if let Some(defects) = &args.defects {
            all_work_item_ids.extend(&defects.0);
        }

        // Sort and deduplicate
        all_work_item_ids.sort_unstable();
        all_work_item_ids.dedup();

        // Return early if no work items to fetch
        if all_work_item_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch work items if space_id is provided
        if let Some(space_id) = args.space_id {
            tracing::info!(
                "Fetching work items from DevOps: Space ID {}, Item IDs: {:?}",
                space_id,
                all_work_item_ids
            );

            let results = self.client.get_work_items(space_id, &all_work_item_ids).await;

            let mut fetched_work_items = Vec::new();
            let mut fetch_errors = Vec::new();

            for result in results {
                match result {
                    Ok(work_item) => {
                        tracing::info!("Successfully fetched work item: {}", work_item.id);
                        fetched_work_items.push(work_item);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch work item: {:?}", e);
                        fetch_errors.push(e);
                    }
                }
            }

            if !fetch_errors.is_empty() {
                tracing::warn!(
                    "Failed to fetch {} out of {} work items",
                    fetch_errors.len(),
                    all_work_item_ids.len()
                );
            }

            if fetched_work_items.is_empty() && !fetch_errors.is_empty() {
                return Err(AppError::Generic(format!(
                    "Failed to fetch any work items. Errors: {:?}",
                    fetch_errors
                )));
            }

            Ok(fetched_work_items)
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if DevOps integration is configured and available
    pub fn is_available(&self) -> bool {
        // Simple check - could be expanded to ping the DevOps API
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::git::{DefectList, StoryList, TaskList};

    fn create_test_args() -> ReviewArgs {
        ReviewArgs {
            files: None,
            commits: None,
            range: None,
            depth: None,
            format: None,
            output: None,
            language: None,
            stories: Some(StoryList(vec![1, 2, 3])),
            tasks: Some(TaskList(vec![4, 5])),
            defects: Some(DefectList(vec![6])),
            space_id: Some(12345),
        }
    }

    #[test]
    fn test_validate_work_item_args_valid() {
        let config = AppConfig::default();
        let fetcher = DevOpsWorkItemFetcher::new(&config);
        let args = create_test_args();

        assert!(fetcher.validate_work_item_args(&args).is_ok());
    }

    #[test]
    fn test_validate_work_item_args_missing_space_id() {
        let config = AppConfig::default();
        let fetcher = DevOpsWorkItemFetcher::new(&config);
        let mut args = create_test_args();
        args.space_id = None;

        assert!(fetcher.validate_work_item_args(&args).is_err());
    }

    #[test]
    fn test_validate_work_item_args_no_work_items() {
        let config = AppConfig::default();
        let fetcher = DevOpsWorkItemFetcher::new(&config);
        let args = ReviewArgs {
            files: None,
            commits: None,
            range: None,
            depth: None,
            format: None,
            output: None,
            language: None,
            stories: None,
            tasks: None,
            defects: None,
            space_id: None,
        };

        assert!(fetcher.validate_work_item_args(&args).is_ok());
    }
}