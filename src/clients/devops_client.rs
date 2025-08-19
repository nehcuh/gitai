use crate::errors::AppError;
use crate::types::devops::{DevOpsResponse, WorkItem};
use futures::future::join_all;
use reqwest::{header, Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

const DEFAULT_RETRY_COUNT: u32 = 3;
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug)]
pub struct DevOpsClient {
    base_url: String,
    token: String,
    client: Client,
    retry_count: u32,
    timeout: Duration,
}

impl DevOpsClient {
    pub fn new(base_url: String, token: String) -> Self {
        DevOpsClient {
            base_url,
            token,
            client: Client::new(),
            retry_count: DEFAULT_RETRY_COUNT,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
        }
    }

    async fn make_request_with_retry(
        &self,
        url: &str,
    ) -> Result<DevOpsResponse, AppError> {
        let mut last_error: Option<String> = None;
        let mut wait_time = Duration::from_secs(1);

        for attempt in 0..self.retry_count {
            if attempt > 0 {
                sleep(wait_time).await;
                wait_time *= 2; // Exponential backoff
            }

            let response_result = self
                .client
                .get(url)
                .header(header::AUTHORIZATION, format!("token {}", self.token))
                .header(header::ACCEPT, "application/json")
                .header(header::CONTENT_TYPE, "application/json")
                .timeout(self.timeout)
                .send()
                .await;

            match response_result {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        // Attempt to parse the response body as DevOpsResponse
                        match response.json::<DevOpsResponse>().await {
                            Ok(data) => return Ok(data),
                            Err(e) => {
                                // If parsing fails, create error message
                                last_error = Some(format!("Failed to parse JSON response: {}", e));
                                // Continue to retry as the server might have sent a malformed JSON temporarily
                            }
                        }
                    } else {
                        // Map HTTP status codes to specific error messages
                        last_error = Some(match status {
                            StatusCode::UNAUTHORIZED => "Authentication failed".to_string(),
                            StatusCode::NOT_FOUND => "Resource not found (404)".to_string(),
                            StatusCode::TOO_MANY_REQUESTS => "Rate limit exceeded".to_string(),
                            s if s.is_server_error() => format!("Server error: {}", s.as_u16()),
                            s if s.is_client_error() => format!("Client error: {}", s.as_u16()),
                            _ => format!("Unhandled HTTP status: {}", status),
                        });
                    }
                }
                Err(e) => {
                    // Handle reqwest::Error
                    if e.is_timeout() {
                        last_error = Some("Request timeout".to_string());
                    } else {
                        last_error = Some(format!("Network error: {}", e));
                    }
                }
            }
        }
        Err(AppError::DevOps(last_error.unwrap_or_else(|| "Request failed after multiple retries".to_string())))
    }

    pub async fn get_work_item(
        &self,
        space_id: u32,
        item_id: u32,
    ) -> Result<WorkItem, AppError> {
        let url = format!(
            "{}/external/collaboration/api/project/{}/issues/{}",
            self.base_url, space_id, item_id
        );

        match self.make_request_with_retry(&url).await {
            Err(AppError::DevOps(msg)) if msg.contains("404") => {
                // Refine 404 from make_request_with_retry to WorkItemNotFound
                Err(AppError::DevOps(format!("Work item not found: {}", item_id)))
            }
            Err(e) => Err(e), // Propagate other errors
            Ok(response) => {
                if response.code != 0 {
                    Err(AppError::DevOps(format!("API logical error: code={}, message={}", 
                        response.code, 
                        response.msg.unwrap_or_else(|| "No message provided".to_string()))))
                } else {
                    match response.data {
                        Some(work_item) => Ok(work_item),
                        None => Err(AppError::DevOps(
                            "WorkItem data is missing in API response when code is 0".to_string(),
                        )),
                    }
                }
            }
        }
    }

    pub async fn get_work_items(
        &self,
        space_id: u32,
        item_ids: &[u32],
    ) -> Result<Vec<WorkItem>, AppError> {
        let futures: Vec<_> = item_ids
            .iter()
            .map(|&id| self.get_work_item(space_id, id))
            .collect();

        let results = join_all(futures).await;

        // Separate successful results from errors
        let mut work_items = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(work_item) => work_items.push(work_item),
                Err(e) => errors.push(e),
            }
        }

        // If all requests failed, return an error
        if work_items.is_empty() && !errors.is_empty() {
            return Err(AppError::DevOps(format!(
                "Failed to fetch any work items. First error: {}",
                errors[0]
            )));
        }

        // Log partial failures but return successful results
        if !errors.is_empty() {
            tracing::warn!(
                "Successfully fetched {} out of {} work items. {} errors occurred.",
                work_items.len(),
                item_ids.len(),
                errors.len()
            );
        }

        Ok(work_items)
    }
}