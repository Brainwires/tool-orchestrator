//! MCP Server implementation using the official rmcp SDK
//!
//! Exposes tool orchestration via the Model Context Protocol.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::sandbox::ExecutionLimits;
use crate::types::OrchestratorResult;
use crate::ToolOrchestrator;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Parameters for the execute_script tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecuteScriptParams {
    /// The Rhai script to execute
    #[schemars(description = "The Rhai script to execute")]
    pub script: String,
    /// Maximum number of operations (prevents infinite loops)
    #[serde(default)]
    #[schemars(description = "Maximum number of operations (prevents infinite loops)")]
    pub max_operations: Option<u64>,
    /// Maximum number of tool calls allowed
    #[serde(default)]
    #[schemars(description = "Maximum number of tool calls allowed")]
    pub max_tool_calls: Option<usize>,
    /// Timeout in milliseconds
    #[serde(default)]
    #[schemars(description = "Timeout in milliseconds")]
    pub timeout_ms: Option<u64>,
}

/// Parameters for registering a server-side tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RegisterToolParams {
    /// Name of the tool
    #[schemars(description = "Name of the tool")]
    pub name: String,
    /// Description of what the tool does
    #[schemars(description = "Description of what the tool does")]
    pub description: String,
    /// Shell command to execute (use $input for the JSON input)
    #[schemars(description = "Shell command to execute (use $input for the JSON input)")]
    pub command: String,
}

/// Parameters for unregistering a tool
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UnregisterToolParams {
    /// Name of the tool to unregister
    #[schemars(description = "Name of the tool to unregister")]
    pub name: String,
}

/// Result of script execution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecuteScriptResult {
    /// Whether execution completed successfully
    pub success: bool,
    /// Output from the script
    pub output: String,
    /// Number of tool calls made
    pub tool_calls_count: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if execution failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<OrchestratorResult> for ExecuteScriptResult {
    fn from(r: OrchestratorResult) -> Self {
        Self {
            success: r.success,
            output: r.output,
            tool_calls_count: r.tool_calls.len(),
            execution_time_ms: r.execution_time_ms,
            error: r.error,
        }
    }
}

// ============================================================================
// Registered Tool (server-side shell command)
// ============================================================================

#[derive(Debug, Clone)]
struct RegisteredShellTool {
    name: String,
    #[allow(dead_code)]
    description: String,
    command: String,
}

impl RegisteredShellTool {
    fn execute(&self, input: &serde_json::Value) -> Result<String, String> {
        use std::process::Command;

        let input_str = serde_json::to_string(input).unwrap_or_default();
        let cmd = self.command.replace("$input", &input_str);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }
}

// ============================================================================
// MCP Service
// ============================================================================

/// Tool Orchestrator MCP Service
///
/// Exposes tool orchestration capabilities via MCP.
#[derive(Clone)]
pub struct ToolOrchestratorService {
    /// Default execution limits
    default_limits: Arc<Mutex<ExecutionLimits>>,
    /// Registered shell tools
    shell_tools: Arc<Mutex<HashMap<String, RegisteredShellTool>>>,
    /// Tool router for rmcp
    tool_router: ToolRouter<Self>,
}

impl ToolOrchestratorService {
    pub fn new() -> Self {
        Self {
            default_limits: Arc::new(Mutex::new(ExecutionLimits::default())),
            shell_tools: Arc::new(Mutex::new(HashMap::new())),
            tool_router: Self::tool_router(),
        }
    }

    /// Set default execution limits
    pub async fn set_default_limits(&self, limits: ExecutionLimits) {
        let mut default = self.default_limits.lock().await;
        *default = limits;
    }
}

#[allow(dead_code)]
fn mcp_error(message: impl Into<String>) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(message.into()),
        data: None,
    }
}

#[tool_router]
impl ToolOrchestratorService {
    /// Execute a Rhai script with registered tools
    ///
    /// The script can call any tools that have been registered via register_tool.
    /// Returns the script's output along with execution metrics.
    #[tool(description = "Execute a Rhai script that can orchestrate multiple tool calls")]
    async fn execute_script(
        &self,
        Parameters(params): Parameters<ExecuteScriptParams>,
    ) -> Result<CallToolResult, McpError> {
        // Build limits
        let default = self.default_limits.lock().await;
        let mut limits = default.clone();
        drop(default);

        if let Some(v) = params.max_operations {
            limits.max_operations = v;
        }
        if let Some(v) = params.max_tool_calls {
            limits.max_tool_calls = v;
        }
        if let Some(v) = params.timeout_ms {
            limits.timeout_ms = v;
        }

        // Create orchestrator and register shell tools
        let mut orchestrator = ToolOrchestrator::new();
        let tools = self.shell_tools.lock().await;

        for tool in tools.values() {
            let tool_clone = tool.clone();
            orchestrator.register_executor(&tool.name, move |input| tool_clone.execute(&input));
        }
        drop(tools);

        // Execute
        match orchestrator.execute(&params.script, limits) {
            Ok(result) => {
                let exec_result = ExecuteScriptResult::from(result);
                let json = serde_json::to_string_pretty(&exec_result)
                    .unwrap_or_else(|_| exec_result.output.clone());
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => {
                let error_result = ExecuteScriptResult {
                    success: false,
                    output: String::new(),
                    tool_calls_count: 0,
                    execution_time_ms: 0,
                    error: Some(e.to_string()),
                };
                let json =
                    serde_json::to_string_pretty(&error_result).unwrap_or_else(|_| e.to_string());
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
        }
    }

    /// Register a shell command as a tool
    ///
    /// The command can use $input to receive the JSON-encoded input.
    /// Example: "curl -s https://api.example.com/data?q=$input"
    #[tool(description = "Register a shell command as a callable tool")]
    async fn register_tool(
        &self,
        Parameters(params): Parameters<RegisterToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let tool = RegisteredShellTool {
            name: params.name.clone(),
            description: params.description,
            command: params.command,
        };

        let mut tools = self.shell_tools.lock().await;
        tools.insert(params.name.clone(), tool);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Tool '{}' registered successfully",
            params.name
        ))]))
    }

    /// Unregister a previously registered tool
    #[tool(description = "Unregister a tool by name")]
    async fn unregister_tool(
        &self,
        Parameters(params): Parameters<UnregisterToolParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut tools = self.shell_tools.lock().await;
        let removed = tools.remove(&params.name).is_some();

        if removed {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Tool '{}' unregistered",
                params.name
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Tool '{}' not found",
                params.name
            ))]))
        }
    }

    /// List all registered tools
    #[tool(description = "List all registered shell tools")]
    async fn list_tools(&self) -> Result<CallToolResult, McpError> {
        let tools = self.shell_tools.lock().await;
        let names: Vec<&str> = tools.keys().map(|s| s.as_str()).collect();

        if names.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(
                "No tools registered",
            )]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Registered tools: {}",
                names.join(", ")
            ))]))
        }
    }
}

impl Default for ToolOrchestratorService {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for ToolOrchestratorService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "tool-orchestrator".to_string(),
                title: Some("Tool Orchestrator".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Tool Orchestrator MCP Server - Execute Rhai scripts that orchestrate multiple tools"
                    .to_string(),
            ),
        }
    }
}
