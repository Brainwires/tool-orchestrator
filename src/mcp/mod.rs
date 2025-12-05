//! MCP (Model Context Protocol) server implementation
//!
//! This module provides a stdio-based MCP server using the official `rmcp` SDK.
//! It exposes the tool orchestrator's capabilities to AI clients.

mod server;

pub use server::ToolOrchestratorService;
