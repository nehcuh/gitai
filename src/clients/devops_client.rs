use crate::errors::devops::ApiError;
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
    ) -> Result<DevOpsResponse, ApiError> {
        let mut last_error: Option<ApiError> = None;
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
                                // If parsing fails, map to ApiError::ParseError
                                // This is a terminal error for this attempt, but we might retry.
                                last_error = Some(ApiError::ParseError(e));
                                // Continue to retry as the server might have sent a malformed JSON temporarily
                            }
                        }
                    } else {
                        // Map HTTP status codes to specific ApiError variants
                        last_error = Some(match status {
                            StatusCode::UNAUTHORIZED => ApiError::AuthenticationError,
                            StatusCode::NOT_FOUND => ApiError::ServerError { status_code: 404 }, // As specified for get_work_item to refine
                            StatusCode::TOO_MANY_REQUESTS => ApiError::RateLimitExceeded,
                            s if s.is_server_error() => ApiError::ServerError { status_code: s.as_u16() },
                            s if s.is_client_error() => ApiError::ServerError { status_code: s.as_u16() }, // Or a more generic client error
                            _ => ApiError::UnexpectedResponseStructure(format!("Unhandled HTTP status: {}", status)),
                        });
                        // For client-side errors (4xx) or server-side errors (5xx), retrying might not always help,
                        // but the loop handles retries. Specific errors like 401, 404, 429 are distinct.
                    }
                }
                Err(e) => {
                    // Handle reqwest::Error
                    if e.is_timeout() {
                        last_error = Some(ApiError::TimeoutError);
                    } else {
                        last_error = Some(ApiError::NetworkError(e));
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| ApiError::UnexpectedResponseStructure("Request failed after multiple retries without a specific error.".to_string())))
    }

    pub async fn get_work_item(
        &self,
        space_id: u32,
        item_id: u32,
    ) -> Result<WorkItem, ApiError> {
        let url = format!(
            "{}/external/collaboration/api/project/{}/issues/{}",
            self.base_url, space_id, item_id
        );

        match self.make_request_with_retry(&url).await {
            Err(ApiError::ServerError { status_code: 404 }) => {
                // Refine 404 from make_request_with_retry to WorkItemNotFound
                Err(ApiError::WorkItemNotFound { item_id })
            }
            Err(e) => Err(e), // Propagate other errors
            Ok(response) => {
                if response.code != 0 {
                    Err(ApiError::ApiLogicalError {
                        code: response.code,
                        message: response.msg.unwrap_or_else(|| "No message provided".to_string()),
                    })
                } else {
                    match response.data {
                        Some(work_item) => Ok(work_item),
                        None => Err(ApiError::UnexpectedResponseStructure(
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
    ) -> Vec<Result<WorkItem, ApiError>> {
        if item_ids.is_empty() {
            return Vec::new();
        }

        let mut futures_vec = Vec::with_capacity(item_ids.len());
        for &item_id in item_ids {
            futures_vec.push(self.get_work_item(space_id, item_id));
        }

        join_all(futures_vec).await
    }
}
