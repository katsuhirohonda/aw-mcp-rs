# ActivityWatch MCP Server

A Rust-based MCP (Model Context Protocol) server for [ActivityWatch](https://activitywatch.net/) - the open-source automated time tracker.

## Features

Query your ActivityWatch time tracking data through MCP tools:

- **aw_list_buckets** - List all ActivityWatch buckets (data containers)
- **aw_get_bucket** - Get detailed information about a specific bucket
- **aw_get_events** - Retrieve events from a bucket with optional time filtering
- **aw_get_event_count** - Count events in a bucket

## Prerequisites

- [ActivityWatch](https://activitywatch.net/) running on your machine (default: `http://localhost:5600`)
- Rust toolchain (for building from source)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/aw-mcp-rs.git
cd aw-mcp-rs
cargo build --release
```

The binary will be available at `target/release/aw-mcp-server`.

## Usage

### Running the Server

```bash
# Default (connects to localhost:5600)
./target/release/aw-mcp-server

# Custom ActivityWatch URL
ACTIVITYWATCH_URL=http://localhost:5600/api/0 ./target/release/aw-mcp-server
```

### Claude Code Configuration

Add to your `~/.claude.json`:

```json
{
  "mcpServers": {
    "activitywatch": {
      "command": "/path/to/aw-mcp-server"
    }
  }
}
```

## MCP Tools

### aw_list_buckets

List all ActivityWatch buckets.

```json
{
  "response_format": "markdown"  // or "json"
}
```

### aw_get_bucket

Get a specific bucket by ID.

```json
{
  "bucket_id": "aw-watcher-window_hostname",
  "response_format": "markdown"
}
```

### aw_get_events

Get events from a bucket with optional filters.

```json
{
  "bucket_id": "aw-watcher-window_hostname",
  "limit": 10,
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-01T23:59:59Z",
  "response_format": "markdown"
}
```

### aw_get_event_count

Count events in a bucket.

```json
{
  "bucket_id": "aw-watcher-window_hostname",
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-01T23:59:59Z"
}
```

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## License

MIT

## References

- [ActivityWatch Documentation](https://docs.activitywatch.net/)
- [ActivityWatch REST API](https://docs.activitywatch.net/en/latest/api/rest.html)
- [MCP Protocol](https://modelcontextprotocol.io/)
