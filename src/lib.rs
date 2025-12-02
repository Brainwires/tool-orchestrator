//! Tool Orchestrator - Rhai-based tool orchestration for AI agents
//!
//! Implements Anthropic's "Programmatic Tool Calling" pattern for token-efficient
//! tool orchestration. Instead of sequential tool calls, AI writes Rhai scripts
//! that orchestrate multiple tools, returning only the final result.
//!
//! ## Benefits
//! - **37% token reduction** - intermediate results don't pollute context
//! - **Parallel execution** - multiple tools in one pass
//! - **Complex orchestration** - loops, conditionals, data processing
//!
//! ## Example
//!
//! ```ignore
//! use tool_orchestrator::{ToolOrchestrator, ExecutionLimits};
//!
//! let mut orchestrator = ToolOrchestrator::new();
//! orchestrator.register_executor("greet", |input| {
//!     Ok(format!("Hello, {}!", input.as_str().unwrap_or("world")))
//! });
//!
//! let result = orchestrator.execute(
//!     r#"greet("Claude")"#,
//!     ExecutionLimits::default()
//! )?;
//!
//! assert_eq!(result.output, "Hello, Claude!");
//! ```

mod engine;
mod sandbox;
mod types;

pub use engine::{ToolExecutor, ToolOrchestrator};
pub use sandbox::ExecutionLimits;
pub use types::{OrchestratorError, OrchestratorResult, ToolCall};
