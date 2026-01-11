use crate::models::{Bucket, Event};
use reqwest::Client;
use rmcp::ErrorData as McpError;
use std::collections::HashMap;
use std::time::Duration;

/// ActivityWatch API client
#[derive(Clone)]
pub struct ActivityWatchClient {
    client: Client,
    base_url: String,
}

impl ActivityWatchClient {
    /// Create a new ActivityWatch API client
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Get all buckets
    pub async fn get_buckets(&self) -> Result<HashMap<String, Bucket>, McpError> {
        let response = self
            .client
            .get(format!("{}/buckets/", self.base_url))
            .send()
            .await
            .map_err(handle_api_error)?;

        handle_response(response).await
    }

    /// Get a specific bucket by ID
    pub async fn get_bucket(&self, bucket_id: &str) -> Result<Bucket, McpError> {
        let response = self
            .client
            .get(format!("{}/buckets/{}", self.base_url, bucket_id))
            .send()
            .await
            .map_err(handle_api_error)?;

        handle_response(response).await
    }

    /// Get events from a bucket
    pub async fn get_events(
        &self,
        bucket_id: &str,
        limit: Option<i32>,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<Event>, McpError> {
        let mut url = format!("{}/buckets/{}/events", self.base_url, bucket_id);
        let mut params = Vec::new();

        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(s) = start {
            params.push(format!("start={}", s));
        }
        if let Some(e) = end {
            params.push(format!("end={}", e));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(handle_api_error)?;

        handle_response(response).await
    }

    /// Get event count for a bucket
    pub async fn get_event_count(
        &self,
        bucket_id: &str,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<i64, McpError> {
        let mut url = format!("{}/buckets/{}/events/count", self.base_url, bucket_id);
        let mut params = Vec::new();

        if let Some(s) = start {
            params.push(format!("start={}", s));
        }
        if let Some(e) = end {
            params.push(format!("end={}", e));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(handle_api_error)?;

        handle_response(response).await
    }
}

/// Handle API response and convert to result
async fn handle_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, McpError> {
    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(match status.as_u16() {
            404 => McpError::invalid_params(
                format!(
                    "Resource not found. Please check the bucket ID. Details: {}",
                    body
                ),
                None,
            ),
            400 => McpError::invalid_params(
                format!("Bad request. Please check your parameters. Details: {}", body),
                None,
            ),
            500 => McpError::internal_error(
                format!("ActivityWatch server error: {}", body),
                None,
            ),
            _ => McpError::internal_error(
                format!("API request failed with status {}: {}", status, body),
                None,
            ),
        });
    }

    response.json().await.map_err(|e| {
        McpError::internal_error(format!("Failed to parse API response: {}", e), None)
    })
}

/// Convert reqwest errors to MCP errors with clear messages
pub fn handle_api_error(error: reqwest::Error) -> McpError {
    if error.is_timeout() {
        McpError::internal_error("Request timed out. Please try again.".to_string(), None)
    } else if error.is_connect() {
        McpError::internal_error(
            "Failed to connect to ActivityWatch. Is aw-server running?".to_string(),
            None,
        )
    } else {
        McpError::internal_error(format!("Network error: {}", error), None)
    }
}
