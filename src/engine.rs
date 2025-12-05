//! Rhai engine setup and tool orchestration

use std::collections::HashMap;

#[cfg(feature = "native")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "native")]
use std::time::Instant;

#[cfg(feature = "wasm")]
use std::cell::RefCell;
#[cfg(feature = "wasm")]
use std::rc::Rc;
#[cfg(feature = "wasm")]
use instant::Instant;

use rhai::{Engine, EvalAltResult, Scope};

use crate::sandbox::ExecutionLimits;
use crate::types::{OrchestratorError, OrchestratorResult, ToolCall};

// ============================================================================
// Type aliases for thread-safety primitives (feature-gated)
// ============================================================================

#[cfg(feature = "native")]
pub type SharedVec<T> = Arc<Mutex<Vec<T>>>;
#[cfg(feature = "native")]
pub type SharedCounter = Arc<Mutex<usize>>;
#[cfg(feature = "native")]
pub type ToolExecutor = Arc<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>;

#[cfg(feature = "wasm")]
pub type SharedVec<T> = Rc<RefCell<Vec<T>>>;
#[cfg(feature = "wasm")]
pub type SharedCounter = Rc<RefCell<usize>>;
#[cfg(feature = "wasm")]
pub type ToolExecutor = Rc<dyn Fn(serde_json::Value) -> Result<String, String>>;

// ============================================================================
// Helper functions for shared state (feature-gated)
// ============================================================================

#[cfg(feature = "native")]
fn new_shared_vec<T>() -> SharedVec<T> {
    Arc::new(Mutex::new(Vec::new()))
}

#[cfg(feature = "wasm")]
fn new_shared_vec<T>() -> SharedVec<T> {
    Rc::new(RefCell::new(Vec::new()))
}

#[cfg(feature = "native")]
fn new_shared_counter() -> SharedCounter {
    Arc::new(Mutex::new(0))
}

#[cfg(feature = "wasm")]
fn new_shared_counter() -> SharedCounter {
    Rc::new(RefCell::new(0))
}

#[cfg(feature = "native")]
fn clone_shared<T: ?Sized>(shared: &Arc<T>) -> Arc<T> {
    Arc::clone(shared)
}

#[cfg(feature = "wasm")]
fn clone_shared<T: ?Sized>(shared: &Rc<T>) -> Rc<T> {
    Rc::clone(shared)
}

#[cfg(feature = "native")]
fn lock_vec<T: Clone>(shared: &SharedVec<T>) -> Vec<T> {
    shared.lock().unwrap().clone()
}

#[cfg(feature = "wasm")]
fn lock_vec<T: Clone>(shared: &SharedVec<T>) -> Vec<T> {
    shared.borrow().clone()
}

#[cfg(feature = "native")]
fn push_to_vec<T>(shared: &SharedVec<T>, item: T) {
    shared.lock().unwrap().push(item);
}

#[cfg(feature = "wasm")]
fn push_to_vec<T>(shared: &SharedVec<T>, item: T) {
    shared.borrow_mut().push(item);
}

#[cfg(feature = "native")]
fn increment_counter(shared: &SharedCounter, max: usize) -> Result<(), ()> {
    let mut c = shared.lock().unwrap();
    if *c >= max {
        return Err(());
    }
    *c += 1;
    Ok(())
}

#[cfg(feature = "wasm")]
fn increment_counter(shared: &SharedCounter, max: usize) -> Result<(), ()> {
    let mut c = shared.borrow_mut();
    if *c >= max {
        return Err(());
    }
    *c += 1;
    Ok(())
}

// ============================================================================
// ToolOrchestrator
// ============================================================================

/// Tool orchestrator - executes Rhai scripts with tool access
pub struct ToolOrchestrator {
    #[allow(dead_code)]
    engine: Engine,
    executors: HashMap<String, ToolExecutor>,
}

impl ToolOrchestrator {
    /// Create a new tool orchestrator
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Disable unsafe operations
        engine.set_max_expr_depths(64, 64);

        Self {
            engine,
            executors: HashMap::new(),
        }
    }

    /// Register a tool executor function (native version - thread-safe)
    #[cfg(feature = "native")]
    pub fn register_executor<F>(&mut self, name: impl Into<String>, executor: F)
    where
        F: Fn(serde_json::Value) -> Result<String, String> + Send + Sync + 'static,
    {
        self.executors.insert(name.into(), Arc::new(executor));
    }

    /// Register a tool executor function (WASM version - single-threaded)
    #[cfg(feature = "wasm")]
    pub fn register_executor<F>(&mut self, name: impl Into<String>, executor: F)
    where
        F: Fn(serde_json::Value) -> Result<String, String> + 'static,
    {
        self.executors.insert(name.into(), Rc::new(executor));
    }

    /// Execute a Rhai script with the registered tools
    pub fn execute(
        &self,
        script: &str,
        limits: ExecutionLimits,
    ) -> Result<OrchestratorResult, OrchestratorError> {
        let start_time = Instant::now();
        let tool_calls: SharedVec<ToolCall> = new_shared_vec();
        let call_count: SharedCounter = new_shared_counter();

        // Create a new engine with limits for this execution
        let mut engine = Engine::new();

        // Apply limits
        engine.set_max_operations(limits.max_operations);
        engine.set_max_string_size(limits.max_string_size);
        engine.set_max_array_size(limits.max_array_size);
        engine.set_max_map_size(limits.max_map_size);
        engine.set_max_expr_depths(64, 64);

        // Register each tool as a Rhai function
        for (name, executor) in &self.executors {
            let exec = clone_shared(executor);
            let calls = clone_shared(&tool_calls);
            let count = clone_shared(&call_count);
            let max_calls = limits.max_tool_calls;
            let tool_name = name.clone();

            // Register as a function that takes a Dynamic and returns a String
            engine.register_fn(name.as_str(), move |input: rhai::Dynamic| -> String {
                let call_start = Instant::now();

                // Check call limit
                if increment_counter(&count, max_calls).is_err() {
                    return format!("ERROR: Maximum tool calls ({}) exceeded", max_calls);
                }

                // Convert Dynamic to JSON
                let json_input = dynamic_to_json(&input);

                // Execute the tool
                let (output, success) = match exec(json_input.clone()) {
                    Ok(result) => (result, true),
                    Err(e) => (format!("Tool error: {}", e), false),
                };

                // Record the call
                let duration_ms = call_start.elapsed().as_millis() as u64;
                let call = ToolCall::new(
                    tool_name.clone(),
                    json_input,
                    output.clone(),
                    success,
                    duration_ms,
                );
                push_to_vec(&calls, call);

                output
            });
        }

        // Compile the script
        let ast = engine
            .compile(script)
            .map_err(|e| OrchestratorError::CompilationError(e.to_string()))?;

        // Execute with timeout handling
        let mut scope = Scope::new();
        let result = engine
            .eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast)
            .map_err(|e| match *e {
                EvalAltResult::ErrorTooManyOperations(_) => {
                    OrchestratorError::MaxOperationsExceeded(limits.max_operations)
                }
                _ => OrchestratorError::ExecutionError(e.to_string()),
            })?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Check timeout (post-execution check, Rhai doesn't have built-in timeout)
        if execution_time_ms > limits.timeout_ms {
            return Err(OrchestratorError::Timeout(limits.timeout_ms));
        }

        // Convert result to string
        let output = if result.is_string() {
            result.into_string().unwrap_or_default()
        } else if result.is_unit() {
            String::new()
        } else {
            format!("{:?}", result)
        };

        let calls = lock_vec(&tool_calls);
        Ok(OrchestratorResult::success(output, calls, execution_time_ms))
    }

    /// Get list of registered tool names
    pub fn registered_tools(&self) -> Vec<&str> {
        self.executors.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Convert Rhai Dynamic to serde_json::Value
pub fn dynamic_to_json(value: &rhai::Dynamic) -> serde_json::Value {
    if value.is_string() {
        serde_json::Value::String(value.clone().into_string().unwrap_or_default())
    } else if value.is_int() {
        serde_json::Value::Number(serde_json::Number::from(value.clone().as_int().unwrap_or(0)))
    } else if value.is_float() {
        serde_json::json!(value.clone().as_float().unwrap_or(0.0))
    } else if value.is_bool() {
        serde_json::Value::Bool(value.clone().as_bool().unwrap_or(false))
    } else if value.is_array() {
        let arr: Vec<rhai::Dynamic> = value.clone().into_array().unwrap_or_default();
        serde_json::Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if value.is_map() {
        let map: rhai::Map = value.clone().cast();
        let mut json_map = serde_json::Map::new();
        for (k, v) in map.iter() {
            json_map.insert(k.to_string(), dynamic_to_json(v));
        }
        serde_json::Value::Object(json_map)
    } else if value.is_unit() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(format!("{:?}", value))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = ToolOrchestrator::new();
        assert!(orchestrator.registered_tools().is_empty());
    }

    #[test]
    fn test_register_executor() {
        let mut orchestrator = ToolOrchestrator::new();
        orchestrator.register_executor("test_tool", |_| Ok("success".to_string()));
        assert!(orchestrator.registered_tools().contains(&"test_tool"));
    }

    #[test]
    fn test_simple_script() {
        let orchestrator = ToolOrchestrator::new();
        let result = orchestrator
            .execute("let x = 1 + 2; x", ExecutionLimits::default())
            .unwrap();
        assert!(result.success);
        assert_eq!(result.output, "3");
    }

    #[test]
    fn test_string_interpolation() {
        let orchestrator = ToolOrchestrator::new();
        let result = orchestrator
            .execute(
                r#"let name = "world"; `Hello, ${name}!`"#,
                ExecutionLimits::default(),
            )
            .unwrap();
        assert!(result.success);
        assert_eq!(result.output, "Hello, world!");
    }

    #[test]
    fn test_tool_execution() {
        let mut orchestrator = ToolOrchestrator::new();
        orchestrator.register_executor("greet", |input| {
            let name = input.as_str().unwrap_or("stranger");
            Ok(format!("Hello, {}!", name))
        });

        let result = orchestrator
            .execute(r#"greet("Claude")"#, ExecutionLimits::default())
            .unwrap();

        assert!(result.success);
        assert_eq!(result.output, "Hello, Claude!");
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].tool_name, "greet");
    }

    #[test]
    fn test_max_operations_limit() {
        let orchestrator = ToolOrchestrator::new();
        let limits = ExecutionLimits::default().with_max_operations(10);

        // This should exceed the operations limit
        let result = orchestrator.execute(
            "let sum = 0; for i in 0..1000 { sum += i; } sum",
            limits,
        );

        assert!(matches!(
            result,
            Err(OrchestratorError::MaxOperationsExceeded(_))
        ));
    }

    #[test]
    fn test_compilation_error() {
        let orchestrator = ToolOrchestrator::new();
        let result = orchestrator.execute(
            "this is not valid rhai syntax {{{{",
            ExecutionLimits::default(),
        );

        assert!(matches!(result, Err(OrchestratorError::CompilationError(_))));
    }

    #[test]
    fn test_multiple_tool_calls() {
        let mut orchestrator = ToolOrchestrator::new();

        orchestrator.register_executor("add", |input| {
            if let Some(arr) = input.as_array() {
                let sum: i64 = arr.iter().filter_map(|v| v.as_i64()).sum();
                Ok(sum.to_string())
            } else {
                Err("Expected array".to_string())
            }
        });

        let script = r#"
            let a = add([1, 2, 3]);
            let b = add([4, 5, 6]);
            `Sum1: ${a}, Sum2: ${b}`
        "#;

        let result = orchestrator
            .execute(script, ExecutionLimits::default())
            .unwrap();

        assert!(result.success);
        assert_eq!(result.tool_calls.len(), 2);
        assert!(result.output.contains("Sum1: 6"));
        assert!(result.output.contains("Sum2: 15"));
    }
}
