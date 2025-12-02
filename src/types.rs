//! Core types for tool orchestration

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result from executing an orchestration script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResult {
    /// Whether execution completed successfully
    pub success: bool,
    /// Output from the script (final expression value)
    pub output: String,
    /// All tool calls made during execution
    pub tool_calls: Vec<ToolCall>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if execution failed
    pub error: Option<String>,
}

impl OrchestratorResult {
    /// Create a successful result
    pub fn success(output: String, tool_calls: Vec<ToolCall>, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            output,
            tool_calls,
            execution_time_ms,
            error: None,
        }
    }

    /// Create a failed result
    pub fn error(error: String, tool_calls: Vec<ToolCall>, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            output: String::new(),
            tool_calls,
            execution_time_ms,
            error: Some(error),
        }
    }
}

/// Record of a single tool call during script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool that was called
    pub tool_name: String,
    /// Input passed to the tool (as JSON)
    pub input: serde_json::Value,
    /// Output returned by the tool
    pub output: String,
    /// Whether the call succeeded
    pub success: bool,
    /// Execution time for this call in milliseconds
    pub duration_ms: u64,
}

impl ToolCall {
    /// Create a new tool call record
    pub fn new(
        tool_name: String,
        input: serde_json::Value,
        output: String,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        Self {
            tool_name,
            input,
            output,
            success,
            duration_ms,
        }
    }
}

/// Errors that can occur during orchestration
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Script compilation failed: {0}")]
    CompilationError(String),

    #[error("Script execution failed: {0}")]
    ExecutionError(String),

    #[error("Script exceeded maximum operations ({0})")]
    MaxOperationsExceeded(u64),

    #[error("Script exceeded maximum tool calls ({0})")]
    MaxToolCallsExceeded(usize),

    #[error("Script execution timed out after {0}ms")]
    Timeout(u64),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_result_success() {
        let result = OrchestratorResult::success(
            "output".to_string(),
            vec![],
            100,
        );
        assert!(result.success);
        assert_eq!(result.output, "output");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_orchestrator_result_error() {
        let result = OrchestratorResult::error(
            "failed".to_string(),
            vec![],
            50,
        );
        assert!(!result.success);
        assert_eq!(result.error, Some("failed".to_string()));
    }

    #[test]
    fn test_tool_call_new() {
        let call = ToolCall::new(
            "test_tool".to_string(),
            serde_json::json!({"arg": "value"}),
            "result".to_string(),
            true,
            10,
        );
        assert_eq!(call.tool_name, "test_tool");
        assert!(call.success);
    }
}
