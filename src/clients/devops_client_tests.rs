#![cfg(test)]
use super::DevOpsClient; // Assuming devops_client_tests.rs is in the same directory as devops_client.rs, or mod.rs makes it available
use crate::errors::DevOpsError;
use crate::types::devops::{DevOpsResponse, IssueTypeDetail, WorkItem};
use httpmock::prelude::*;
use std::time::Duration;

// Helper function to create a common IssueTypeDetail
fn common_issue_type_detail() -> IssueTypeDetail {
    IssueTypeDetail {
        id: 1,
        name: "User Story".to_string(),
        icon_type: "story".to_string(),
        issue_type: "REQUIREMENT".to_string(),
    }
}

// Helper function to create a WorkItem
fn create_mock_work_item(id: u32, name: &str, description: &str) -> WorkItem {
    WorkItem {
        id,
        name: name.to_string(),
        description: description.to_string(),
        issue_type_detail: common_issue_type_detail(),
        r#type: "REQUIREMENT".to_string(),
        status_name: "New".to_string(),
        priority: 1,
    }
}

#[tokio::test]
async fn test_devops_client_new() {
    let base_url = "http://localhost".to_string();
    let token = "test_token".to_string();
    let _client = DevOpsClient::new(base_url.clone(), token.clone());
    // Test that the constructor can be called and an instance is created.
    // Correctness of field initialization is implicitly tested by other tests
    // that rely on these fields for making mocked API calls.
    // No panic means the basic construction worked.
}

#[tokio::test]
async fn test_get_work_item_success() {
    let server = MockServer::start_async().await;
    let base_url = server.base_url();
    let token = "test_token".to_string();
    let client = DevOpsClient::new(base_url.clone(), token.clone());

    let space_id = 123;
    let item_id = 456;

    let mock_item = create_mock_work_item(item_id, "Test Story", "A test user story.");
    let api_response = DevOpsResponse {
        code: 0,
        msg: None,
        data: Some(mock_item.clone()),
    };

    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!(
                    "/external/collaboration/api/project/{}/issues/{}",
                    space_id, item_id
                ))
                .header("Authorization", &format!("token {}", token))
                .header("Accept", "application/json")
                .header("Content-Type", "application/json");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&api_response);
        })
        .await;

    let result = client.get_work_item(space_id, item_id).await;
    assert!(result.is_ok());
    let fetched_item = result.unwrap();
    assert_eq!(fetched_item, mock_item);
}

#[tokio::test]
async fn test_get_work_item_api_logical_error() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_id = 1;

    let api_response = DevOpsResponse {
        code: 1,
        msg: Some("API Error Occurred".to_string()),
        data: None,
    };

    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, item_id
            ));
            then.status(200) // HTTP success, but logical error in response body
                .json_body_obj(&api_response);
        })
        .await;

    let result = client.get_work_item(space_id, item_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DevOpsError::ApiLogicalError { code, message } => {
            assert_eq!(code, 1);
            assert_eq!(message, "API Error Occurred");
        }
        _ => panic!("Expected ApiLogicalError"),
    }
}

#[tokio::test]
async fn test_get_work_item_not_found_404() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_id = 1;

    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, item_id
            ));
            then.status(404);
        })
        .await;

    let result = client.get_work_item(space_id, item_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DevOpsError::WorkItemNotFound { item_id: returned_id } => {
            assert_eq!(returned_id, item_id);
        }
        _ => panic!("Expected WorkItemNotFound"),
    }
}

#[tokio::test]
async fn test_get_work_item_authentication_error_401() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());

    server
        .mock_async(|when, then| {
            when.method(GET); // Match any GET
            then.status(401);
        })
        .await;

    let result = client.get_work_item(1, 1).await;
    assert!(matches!(result, Err(DevOpsError::AuthenticationError)));
}

#[tokio::test]
async fn test_get_work_item_rate_limit_error_429() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());

    server
        .mock_async(|when, then| {
            when.method(GET);
            then.status(429);
        })
        .await;

    let result = client.get_work_item(1, 1).await;
    assert!(matches!(result, Err(DevOpsError::RateLimitExceeded)));
}

#[tokio::test]
async fn test_get_work_item_server_error_500() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());

    server
        .mock_async(|when, then| {
            when.method(GET);
            then.status(500);
        })
        .await;

    let result = client.get_work_item(1, 1).await;
    match result.unwrap_err() {
        DevOpsError::ServerError { status_code } => assert_eq!(status_code, 500),
        _ => panic!("Expected ServerError"),
    }
}

#[tokio::test]
async fn test_get_work_item_parse_error() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());

    server
        .mock_async(|when, then| {
            when.method(GET);
            then.status(200).body("this is not json");
        })
        .await;

    let result = client.get_work_item(1, 1).await;
    assert!(matches!(result, Err(DevOpsError::ParseError(_))));
}

#[tokio::test]
async fn test_get_work_item_unexpected_structure_data_null() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());

    let api_response = DevOpsResponse {
        code: 0,
        msg: None,
        data: None, // data is null
    };
    server
        .mock_async(|when, then| {
            when.method(GET);
            then.status(200).json_body_obj(&api_response);
        })
        .await;

    let result = client.get_work_item(1, 1).await;
    match result.unwrap_err() {
        DevOpsError::UnexpectedResponseStructure(msg) => {
            assert!(msg.contains("WorkItem data is missing"));
        }
        _ => panic!("Expected UnexpectedResponseStructure"),
    }
}

// Test for retry logic (simplified: one failure then success)
#[tokio::test]
async fn test_get_work_item_retry_success_on_second_attempt() {
    let server = MockServer::start_async().await;
    // Client configured with retry_count = 3 by default in its `new`
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_id = 1;

    let mock_item = create_mock_work_item(item_id, "Retry Item", "Fetched after retry.");
    let success_response = DevOpsResponse {
        code: 0,
        msg: None,
        data: Some(mock_item.clone()),
    };

    // First call fails with 500
    let mock_500 = server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, item_id
            ));
            then.status(500);
        })
        .await;

    // Second call succeeds
    let mock_200 = server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, item_id
            ));
            then.status(200).json_body_obj(&success_response);
        })
        .await;
    
    // The client's retry logic should handle this sequence.
    // We expect the first call to hit mock_500, then retry, hit mock_200.
    // To ensure this, we make mock_500 expect only 1 hit.
    mock_500.expect_hits_async(1).await;
    mock_200.expect_hits_async(1).await;


    let result = client.get_work_item(space_id, item_id).await;
    assert!(result.is_ok(), "Result was: {:?}", result.err());
    assert_eq!(result.unwrap(), mock_item);
}

#[tokio::test]
async fn test_get_work_items_success() {
    let server = MockServer::start_async().await;
    let base_url = server.base_url();
    let token = "test_token_items".to_string();
    let client = DevOpsClient::new(base_url.clone(), token.clone());

    let space_id = 789;
    let item_ids = vec![101, 102];

    let mock_item_101 = create_mock_work_item(101, "Item 101", "First item.");
    let api_response_101 = DevOpsResponse {
        code: 0,
        msg: None,
        data: Some(mock_item_101.clone()),
    };

    let mock_item_102 = create_mock_work_item(102, "Item 102", "Second item.");
    let api_response_102 = DevOpsResponse {
        code: 0,
        msg: None,
        data: Some(mock_item_102.clone()),
    };

    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/external/collaboration/api/project/{}/issues/{}", space_id, 101))
                .header("Authorization", &format!("token {}", token));
            then.status(200).json_body_obj(&api_response_101);
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/external/collaboration/api/project/{}/issues/{}", space_id, 102))
                .header("Authorization", &format!("token {}", token));
            then.status(200).json_body_obj(&api_response_102);
        })
        .await;

    let results = client.get_work_items(space_id, &item_ids).await;
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap(), &mock_item_101);
    assert!(results[1].is_ok());
    assert_eq!(results[1].as_ref().unwrap(), &mock_item_102);
}

#[tokio::test]
async fn test_get_work_items_partial_failure() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_ids = vec![101, 404]; // 101 will succeed, 404 will fail

    let mock_item_101 = create_mock_work_item(101, "Item 101", "Partial success item.");
    let api_response_101 = DevOpsResponse {
        code: 0,
        msg: None,
        data: Some(mock_item_101.clone()),
    };

    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, 101
            ));
            then.status(200).json_body_obj(&api_response_101);
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, 404 // This ID will trigger a 404
            ));
            then.status(404);
        })
        .await;

    let results = client.get_work_items(space_id, &item_ids).await;
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap(), &mock_item_101);
    assert!(results[1].is_err());
    match results[1].as_ref().unwrap_err() {
        DevOpsError::WorkItemNotFound { item_id } => assert_eq!(*item_id, 404),
        _ => panic!("Expected WorkItemNotFound for the second item"),
    }
}

#[tokio::test]
async fn test_get_work_items_all_fail() {
    let server = MockServer::start_async().await;
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_ids = vec![503, 504]; // Both will fail

    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, 503
            ));
            then.status(500); // Server error for 503
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, 504
            ));
            then.status(401); // Auth error for 504
        })
        .await;

    let results = client.get_work_items(space_id, &item_ids).await;
    assert_eq!(results.len(), 2);
    assert!(matches!(results[0].as_ref().unwrap_err(), DevOpsError::ServerError { status_code: 500 }));
    assert!(matches!(results[1].as_ref().unwrap_err(), DevOpsError::AuthenticationError));
}

#[tokio::test]
async fn test_get_work_items_empty_list() {
    let server = MockServer::start_async().await; // Server needed for client new, but won't be hit
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_ids: Vec<u32> = Vec::new();

    let results = client.get_work_items(space_id, &item_ids).await;
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_get_work_item_network_error() {
    let server = MockServer::start_async().await;
    let base_url = server.base_url(); // Get base_url before stopping
    let client = DevOpsClient::new(base_url, "token".to_string());
    
    server.stop_async().await; // Stop the server

    let result = client.get_work_item(1, 1).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DevOpsError::NetworkError(_) => {} // Expected
        e => panic!("Expected NetworkError, got {:?}", e),
    }
}

// Test for retry logic with persistent failure
#[tokio::test]
async fn test_get_work_item_retry_persistent_failure() {
    let server = MockServer::start_async().await;
    // Client configured with retry_count = 3 by default
    let client = DevOpsClient::new(server.base_url(), "token".to_string());
    let space_id = 1;
    let item_id = 1;

    // Mock will return 500 for all attempts
    let mock_500 = server
        .mock_async(|when, then| {
            when.method(GET).path(format!(
                "/external/collaboration/api/project/{}/issues/{}",
                space_id, item_id
            ));
            then.status(500);
        })
        .await;
    
    // The client should try 3 times (1 initial + 2 retries).
    // So, the mock should be hit 3 times.
    mock_500.expect_hits_async(3).await; // Default retry_count is 3

    let result = client.get_work_item(space_id, item_id).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        DevOpsError::ServerError { status_code } => assert_eq!(status_code, 500),
        _ => panic!("Expected ServerError after retries"),
    }
}
