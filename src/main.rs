use aw_mcp_server::{ActivityWatchClient, ActivityWatchMcpServer};
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing (logs to stderr so it doesn't interfere with stdio transport)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Get ActivityWatch server URL from environment (optional)
    let base_url = env::var("ACTIVITYWATCH_URL")
        .unwrap_or_else(|_| "http://localhost:5600/api/0".to_string());

    // Create ActivityWatch API client
    let client = ActivityWatchClient::new(&base_url);

    // Create MCP server
    let server = ActivityWatchMcpServer::new(client);

    eprintln!("ActivityWatch MCP Server starting...");
    eprintln!("Connecting to ActivityWatch at: {}", base_url);

    // Run with stdio transport
    let service = server.serve(stdio()).await?;

    // Wait for service to complete
    service.waiting().await?;

    Ok(())
}
