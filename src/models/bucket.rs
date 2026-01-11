use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output format for responses
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Human-readable markdown format
    #[default]
    Markdown,
    /// Machine-readable JSON format
    Json,
}

/// ActivityWatch Bucket - a container for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    /// Unique bucket identifier
    pub id: String,

    /// Name of the watcher/client that created the bucket
    #[serde(default)]
    pub client: Option<String>,

    /// Type of events stored (e.g., "currentwindow", "afkstatus")
    #[serde(rename = "type", default)]
    pub bucket_type: Option<String>,

    /// Hostname where the bucket was created
    #[serde(default)]
    pub hostname: Option<String>,

    /// When the bucket was created
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,

    /// Additional metadata
    #[serde(default)]
    pub data: Option<HashMap<String, serde_json::Value>>,

    /// Last updated timestamp
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
}

/// ActivityWatch Event - a timestamped activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID (optional, assigned by server)
    #[serde(default)]
    pub id: Option<i64>,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Duration in seconds
    pub duration: f64,

    /// Event-specific data (e.g., app name, window title)
    pub data: HashMap<String, serde_json::Value>,
}

impl Bucket {
    /// Format bucket information as markdown
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![format!("## {}", self.id)];

        if let Some(ref client) = self.client {
            lines.push(format!("- **Client**: {}", client));
        }
        if let Some(ref bucket_type) = self.bucket_type {
            lines.push(format!("- **Type**: {}", bucket_type));
        }
        if let Some(ref hostname) = self.hostname {
            lines.push(format!("- **Hostname**: {}", hostname));
        }
        if let Some(ref created) = self.created {
            lines.push(format!("- **Created**: {}", created.format("%Y-%m-%d %H:%M:%S")));
        }
        if let Some(ref last_updated) = self.last_updated {
            lines.push(format!(
                "- **Last Updated**: {}",
                last_updated.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        lines.join("\n")
    }
}

impl Event {
    /// Format event information as markdown
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![];

        // Format timestamp and duration
        lines.push(format!(
            "### {} ({:.1}s)",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.duration
        ));

        // Format data fields
        for (key, value) in &self.data {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            lines.push(format!("- **{}**: {}", key, value_str));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_format_defaults_to_markdown() {
        let format = ResponseFormat::default();
        assert!(matches!(format, ResponseFormat::Markdown));
    }

    #[test]
    fn bucket_deserializes_correctly() {
        let json = r#"{
            "id": "aw-watcher-window_test",
            "client": "aw-watcher-window",
            "type": "currentwindow",
            "hostname": "testhost",
            "created": "2024-01-01T00:00:00Z"
        }"#;

        let bucket: Bucket = serde_json::from_str(json).unwrap();
        assert_eq!(bucket.id, "aw-watcher-window_test");
        assert_eq!(bucket.client, Some("aw-watcher-window".to_string()));
        assert_eq!(bucket.bucket_type, Some("currentwindow".to_string()));
    }

    #[test]
    fn event_deserializes_correctly() {
        let json = r#"{
            "id": 1,
            "timestamp": "2024-01-01T12:00:00Z",
            "duration": 60.5,
            "data": {
                "app": "Firefox",
                "title": "Test Page"
            }
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, Some(1));
        assert_eq!(event.duration, 60.5);
        assert_eq!(
            event.data.get("app").and_then(|v| v.as_str()),
            Some("Firefox")
        );
    }
}
