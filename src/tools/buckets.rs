use crate::api::ActivityWatchClient;
use crate::constants::{CHARACTER_LIMIT, DEFAULT_EVENTS_LIMIT};
use crate::models::ResponseFormat;
use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::tool::Parameters,
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

/// ActivityWatch MCP Server
#[derive(Clone)]
pub struct ActivityWatchMcpServer {
    client: Arc<ActivityWatchClient>,
    tool_router: ToolRouter<Self>,
}

/// Input for listing all buckets
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListBucketsParams {
    /// Output format: "markdown" (default) or "json"
    #[serde(default)]
    pub response_format: ResponseFormat,
}

/// Input for getting a specific bucket
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBucketParams {
    /// The bucket ID to retrieve
    pub bucket_id: String,

    /// Output format: "markdown" (default) or "json"
    #[serde(default)]
    pub response_format: ResponseFormat,
}

/// Input for getting events from a bucket
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetEventsParams {
    /// The bucket ID to get events from
    pub bucket_id: String,

    /// Maximum number of events to return (default: 100)
    #[serde(default)]
    pub limit: Option<i32>,

    /// Start time (ISO 8601 format, e.g., "2024-01-01T00:00:00Z")
    #[serde(default)]
    pub start: Option<String>,

    /// End time (ISO 8601 format, e.g., "2024-01-01T23:59:59Z")
    #[serde(default)]
    pub end: Option<String>,

    /// Output format: "markdown" (default) or "json"
    #[serde(default)]
    pub response_format: ResponseFormat,
}

/// Input for getting event count
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetEventCountParams {
    /// The bucket ID to count events from
    pub bucket_id: String,

    /// Start time (ISO 8601 format)
    #[serde(default)]
    pub start: Option<String>,

    /// End time (ISO 8601 format)
    #[serde(default)]
    pub end: Option<String>,
}

#[tool_router]
impl ActivityWatchMcpServer {
    /// Create a new ActivityWatch MCP server
    pub fn new(client: ActivityWatchClient) -> Self {
        Self {
            client: Arc::new(client),
            tool_router: Self::tool_router(),
        }
    }

    /// List all ActivityWatch buckets.
    #[tool(description = "List all ActivityWatch buckets. Buckets are containers that group events by watcher type and hostname (e.g., aw-watcher-window_hostname for window tracking events).")]
    async fn aw_list_buckets(
        &self,
        Parameters(params): Parameters<ListBucketsParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.client.get_buckets().await {
            Ok(buckets) => {
                let response = match params.response_format {
                    ResponseFormat::Markdown => {
                        let mut lines = vec![
                            "# ActivityWatch Buckets".to_string(),
                            String::new(),
                            format!("Found {} buckets:", buckets.len()),
                            String::new(),
                        ];

                        for bucket in buckets.values() {
                            lines.push(bucket.to_markdown());
                            lines.push(String::new());
                        }

                        truncate_response(lines.join("\n"))
                    }
                    ResponseFormat::Json => {
                        serde_json::to_string_pretty(&buckets)
                            .unwrap_or_else(|_| "Error formatting JSON".to_string())
                    }
                };

                Ok(CallToolResult::success(vec![Content::text(response)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list buckets: {:?}",
                e
            ))])),
        }
    }

    /// Get a specific ActivityWatch bucket by ID.
    #[tool(description = "Get detailed information about a specific ActivityWatch bucket by its ID. Returns bucket metadata including type, hostname, and creation time.")]
    async fn aw_get_bucket(
        &self,
        Parameters(params): Parameters<GetBucketParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.bucket_id.trim().is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Bucket ID cannot be empty",
            )]));
        }

        match self.client.get_bucket(&params.bucket_id).await {
            Ok(bucket) => {
                let response = match params.response_format {
                    ResponseFormat::Markdown => {
                        let mut lines = vec!["# Bucket Details".to_string(), String::new()];
                        lines.push(bucket.to_markdown());
                        lines.join("\n")
                    }
                    ResponseFormat::Json => serde_json::to_string_pretty(&bucket)
                        .unwrap_or_else(|_| "Error formatting JSON".to_string()),
                };

                Ok(CallToolResult::success(vec![Content::text(response)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get bucket: {:?}",
                e
            ))])),
        }
    }

    /// Get events from an ActivityWatch bucket.
    #[tool(description = r#"Get events from an ActivityWatch bucket. Events contain timestamped activity data such as window titles, app names, or AFK status.

## Parameters
- `bucket_id`: The bucket ID (e.g., "aw-watcher-window_hostname")
- `limit`: Maximum events to return (default: 100)
- `start`: Start time in ISO 8601 format (e.g., "2024-01-01T00:00:00Z")
- `end`: End time in ISO 8601 format (e.g., "2024-01-01T23:59:59Z")

## Example
Get the last 10 window events:
```json
{
  "bucket_id": "aw-watcher-window_myhostname",
  "limit": 10
}
```"#)]
    async fn aw_get_events(
        &self,
        Parameters(params): Parameters<GetEventsParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.bucket_id.trim().is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Bucket ID cannot be empty",
            )]));
        }

        let limit = params.limit.unwrap_or(DEFAULT_EVENTS_LIMIT);

        match self
            .client
            .get_events(
                &params.bucket_id,
                Some(limit),
                params.start.as_deref(),
                params.end.as_deref(),
            )
            .await
        {
            Ok(events) => {
                let response = match params.response_format {
                    ResponseFormat::Markdown => {
                        let mut lines = vec![
                            format!("# Events from {}", params.bucket_id),
                            String::new(),
                            format!("Showing {} events:", events.len()),
                            String::new(),
                        ];

                        for event in &events {
                            lines.push(event.to_markdown());
                            lines.push(String::new());
                        }

                        if events.len() as i32 >= limit {
                            lines.push(format!(
                                "_Limit of {} reached. Use pagination to see more._",
                                limit
                            ));
                        }

                        truncate_response(lines.join("\n"))
                    }
                    ResponseFormat::Json => serde_json::to_string_pretty(&events)
                        .unwrap_or_else(|_| "Error formatting JSON".to_string()),
                };

                Ok(CallToolResult::success(vec![Content::text(response)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get events: {:?}",
                e
            ))])),
        }
    }

    /// Get the count of events in an ActivityWatch bucket.
    #[tool(description = "Get the total count of events in an ActivityWatch bucket. Useful for understanding data volume before fetching events. Optionally filter by time range.")]
    async fn aw_get_event_count(
        &self,
        Parameters(params): Parameters<GetEventCountParams>,
    ) -> Result<CallToolResult, McpError> {
        if params.bucket_id.trim().is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Bucket ID cannot be empty",
            )]));
        }

        match self
            .client
            .get_event_count(
                &params.bucket_id,
                params.start.as_deref(),
                params.end.as_deref(),
            )
            .await
        {
            Ok(count) => {
                let mut lines = vec![
                    format!("# Event Count for {}", params.bucket_id),
                    String::new(),
                    format!("**Total Events**: {}", count),
                ];

                if let Some(ref start) = params.start {
                    lines.push(format!("**From**: {}", start));
                }
                if let Some(ref end) = params.end {
                    lines.push(format!("**To**: {}", end));
                }

                Ok(CallToolResult::success(vec![Content::text(lines.join("\n"))]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get event count: {:?}",
                e
            ))])),
        }
    }
}

#[tool_handler]
impl rmcp::ServerHandler for ActivityWatchMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ActivityWatch MCP Server - Query your ActivityWatch time tracking data. Use aw_list_buckets to see available data sources, then aw_get_events to retrieve activity logs.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Truncate response if it exceeds the character limit
fn truncate_response(response: String) -> String {
    if response.len() > CHARACTER_LIMIT {
        let truncated = &response[..CHARACTER_LIMIT];
        format!(
            "{}\n\n_Response truncated at {} characters. Use more specific filters to reduce results._",
            truncated, CHARACTER_LIMIT
        )
    } else {
        response
    }
}
