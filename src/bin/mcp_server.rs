//! MCP Server binary entry point
//!
//! Run with: cargo run --features mcp-server --bin tool-orchestrator-mcp

use rmcp::{transport::stdio, ServiceExt};
use tool_orchestrator::mcp::ToolOrchestratorService;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to stderr (stdout is for MCP protocol)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    info!(
        "Tool Orchestrator MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create the service and serve via stdio
    let service = ToolOrchestratorService::new().serve(stdio()).await?;

    info!("MCP server running, waiting for requests...");

    // Wait for shutdown
    service.waiting().await?;

    info!("MCP server shutting down");
    Ok(())
}
