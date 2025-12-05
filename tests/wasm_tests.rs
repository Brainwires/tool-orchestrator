//! WASM-specific tests
//!
//! Run with: wasm-pack test --headless --chrome --features wasm --no-default-features
//! Or:       wasm-pack test --headless --firefox --features wasm --no-default-features
//! Or:       wasm-pack test --node --features wasm --no-default-features

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

// Note: Not configuring run_in_browser to allow both browser and Node.js testing

use tool_orchestrator::{WasmExecutionLimits, WasmOrchestrator};

// ============================================================================
// ExecutionLimits Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_execution_limits_constructor() {
    let limits = WasmExecutionLimits::new();
    assert_eq!(limits.max_operations(), 100_000);
    assert_eq!(limits.max_tool_calls(), 50);
    assert_eq!(limits.timeout_ms(), 30_000);
}

#[wasm_bindgen_test]
fn test_execution_limits_quick() {
    let limits = WasmExecutionLimits::quick();
    assert_eq!(limits.max_operations(), 10_000);
    assert_eq!(limits.max_tool_calls(), 10);
    assert_eq!(limits.timeout_ms(), 5_000);
}

#[wasm_bindgen_test]
fn test_execution_limits_extended() {
    let limits = WasmExecutionLimits::extended();
    assert_eq!(limits.max_operations(), 500_000);
    assert_eq!(limits.max_tool_calls(), 100);
    assert_eq!(limits.timeout_ms(), 120_000);
}

#[wasm_bindgen_test]
fn test_execution_limits_setters() {
    let mut limits = WasmExecutionLimits::new();

    limits.set_max_operations(50_000);
    assert_eq!(limits.max_operations(), 50_000);

    limits.set_max_tool_calls(25);
    assert_eq!(limits.max_tool_calls(), 25);

    limits.set_timeout_ms(10_000);
    assert_eq!(limits.timeout_ms(), 10_000);

    limits.set_max_string_size(5_000_000);
    assert_eq!(limits.max_string_size(), 5_000_000);

    limits.set_max_array_size(5_000);
    assert_eq!(limits.max_array_size(), 5_000);
}

// ============================================================================
// WasmOrchestrator Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_orchestrator_creation() {
    let orchestrator = WasmOrchestrator::new();
    assert!(orchestrator.registered_tools().is_empty());
}

#[wasm_bindgen_test]
fn test_simple_script_execution() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let result = orchestrator.execute("40 + 2", &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("\"success\":true"));
    assert!(result_string.contains("42"));
}

#[wasm_bindgen_test]
fn test_string_interpolation() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let result = orchestrator.execute(r#"let x = "World"; `Hello, ${x}!`"#, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("Hello, World!"));
}

#[wasm_bindgen_test]
fn test_loop_execution() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let sum = 0;
        for i in 1..=10 {
            sum += i;
        }
        sum
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("55"));
}

#[wasm_bindgen_test]
fn test_array_operations() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let arr = [1, 2, 3, 4, 5];
        let doubled = [];
        for item in arr {
            doubled.push(item * 2);
        }
        `Result: ${doubled}`
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_map_operations() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let config = #{
            name: "test",
            value: 42,
            active: true
        };
        `Name: ${config.name}, Value: ${config.value}`
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_compilation_error() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let result = orchestrator.execute("let x = ", &limits);
    assert!(result.is_ok()); // Returns Ok with error in result

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("\"success\":false"));
    assert!(result_string.contains("Compilation error"));
}

#[wasm_bindgen_test]
fn test_max_operations_exceeded() {
    let orchestrator = WasmOrchestrator::new();
    let mut limits = WasmExecutionLimits::new();
    limits.set_max_operations(100);

    // This infinite-like loop should exceed operations limit
    let script = r#"
        let x = 0;
        loop {
            x += 1;
            if x > 10000 { break; }
        }
        x
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("\"success\":false"));
    assert!(result_string.contains("maximum operations"));
}

#[wasm_bindgen_test]
fn test_tool_registration() {
    let mut orchestrator = WasmOrchestrator::new();

    // Create a JavaScript function for testing
    let greet_fn = js_sys::Function::new_with_args(
        "input",
        r#"return "Hello, " + JSON.parse(input) + "!""#
    );

    orchestrator.register_tool("greet", greet_fn);

    let tools = orchestrator.registered_tools();
    assert_eq!(tools.len(), 1);
    assert!(tools.contains(&"greet".to_string()));
}

#[wasm_bindgen_test]
fn test_tool_execution() {
    let mut orchestrator = WasmOrchestrator::new();

    // Create a simple echo tool
    let echo_fn = js_sys::Function::new_with_args(
        "input",
        r#"return "Echo: " + input"#
    );

    orchestrator.register_tool("echo", echo_fn);

    let limits = WasmExecutionLimits::new();
    let result = orchestrator.execute(r#"echo("test")"#, &limits);

    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("Echo:"));
    assert!(result_string.contains("\"success\":true"));
}

#[wasm_bindgen_test]
fn test_multiple_tools() {
    let mut orchestrator = WasmOrchestrator::new();

    let add_fn = js_sys::Function::new_with_args(
        "input",
        r#"var arr = JSON.parse(input); return String(arr[0] + arr[1])"#
    );

    let multiply_fn = js_sys::Function::new_with_args(
        "input",
        r#"var arr = JSON.parse(input); return String(arr[0] * arr[1])"#
    );

    orchestrator.register_tool("add", add_fn);
    orchestrator.register_tool("multiply", multiply_fn);

    let tools = orchestrator.registered_tools();
    assert_eq!(tools.len(), 2);

    let limits = WasmExecutionLimits::new();
    let result = orchestrator.execute(
        r#"
            let sum = add([5, 3]);
            let product = multiply([4, 7]);
            `Sum: ${sum}, Product: ${product}`
        "#,
        &limits
    );

    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_tool_error_handling() {
    let mut orchestrator = WasmOrchestrator::new();

    // Create a tool that throws an error
    let error_fn = js_sys::Function::new_with_args(
        "input",
        r#"throw new Error("Intentional error")"#
    );

    orchestrator.register_tool("fail", error_fn);

    let limits = WasmExecutionLimits::new();
    let result = orchestrator.execute(r#"fail("test")"#, &limits);

    assert!(result.is_ok()); // Should not throw, error is captured

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    // Tool errors are returned as output, script still succeeds
    assert!(result_string.contains("\"success\":true"));
}

#[wasm_bindgen_test]
fn test_conditional_logic() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let value = 75;
        let grade = if value >= 90 {
            "A"
        } else if value >= 80 {
            "B"
        } else if value >= 70 {
            "C"
        } else {
            "F"
        };
        grade
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("\"C\"") || result_string.contains("\\\"C\\\""));
}

#[wasm_bindgen_test]
fn test_function_definition() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        fn factorial(n) {
            if n <= 1 { 1 }
            else { n * factorial(n - 1) }
        }
        factorial(5)
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("120"));
}

#[wasm_bindgen_test]
fn test_empty_script() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let result = orchestrator.execute("", &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("\"success\":true"));
}

#[wasm_bindgen_test]
fn test_execution_records_timing() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let result = orchestrator.execute("let x = 1 + 1; x", &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    // Should contain execution_time_ms field
    assert!(result_string.contains("execution_time_ms"));
}

#[wasm_bindgen_test]
fn test_tool_with_json_object() {
    let mut orchestrator = WasmOrchestrator::new();

    // Tool that parses and processes JSON object
    let process_fn = js_sys::Function::new_with_args(
        "input",
        r#"
            var obj = JSON.parse(input);
            return "Name: " + obj.name + ", Age: " + obj.age;
        "#
    );

    orchestrator.register_tool("process", process_fn);

    let limits = WasmExecutionLimits::new();
    let result = orchestrator.execute(
        r#"process(#{ name: "Alice", age: 30 })"#,
        &limits
    );

    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("Alice"));
    assert!(result_string.contains("30"));
}

#[wasm_bindgen_test]
fn test_max_tool_calls_limit() {
    let mut orchestrator = WasmOrchestrator::new();

    let count_fn = js_sys::Function::new_with_args("input", r#"return "1""#);
    orchestrator.register_tool("count", count_fn);

    let mut limits = WasmExecutionLimits::new();
    limits.set_max_tool_calls(3);

    // Try to call tool 4 times
    let result = orchestrator.execute(
        r#"
            let a = count("1");
            let b = count("2");
            let c = count("3");
            count("4")
        "#,
        &limits
    );

    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    // Fourth call should hit the limit
    assert!(result_string.contains("Maximum tool calls"));
}

#[wasm_bindgen_test]
fn test_tool_records_calls() {
    let mut orchestrator = WasmOrchestrator::new();

    let echo_fn = js_sys::Function::new_with_args("input", r#"return input"#);
    orchestrator.register_tool("echo", echo_fn);

    let limits = WasmExecutionLimits::new();
    let result = orchestrator.execute(
        r#"
            let a = echo("first");
            let b = echo("second");
            "done"
        "#,
        &limits
    );

    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    // Should record 2 tool calls - look for tool_calls array with echo entries
    assert!(result_string.contains("tool_calls"));
    // The tool_name field is serialized as "tool_name" not "name"
    let echo_count = result_string.matches("echo").count();
    // Should have at least 2 occurrences of "echo" (the tool name appears in each call)
    assert!(echo_count >= 2, "Expected at least 2 echo occurrences, got {}: {}", echo_count, result_string);
}

#[wasm_bindgen_test]
fn test_string_methods() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let s = "Hello, World!";
        let upper = s.to_upper();
        let lower = s.to_lower();
        let len = s.len();
        `Upper: ${upper}, Lower: ${lower}, Length: ${len}`
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    assert!(result_string.contains("HELLO, WORLD!"));
    assert!(result_string.contains("hello, world!"));
    assert!(result_string.contains("13"));
}

#[wasm_bindgen_test]
fn test_nested_data_structures() {
    let orchestrator = WasmOrchestrator::new();
    let limits = WasmExecutionLimits::new();

    let script = r#"
        let users = [
            #{ name: "Alice", scores: [90, 85, 92] },
            #{ name: "Bob", scores: [78, 88, 95] }
        ];

        let total = 0;
        for user in users {
            for score in user.scores {
                total += score;
            }
        }
        `Total: ${total}`
    "#;

    let result = orchestrator.execute(script, &limits);
    assert!(result.is_ok());

    let result_js = result.unwrap();
    let result_str = js_sys::JSON::stringify(&result_js).unwrap();
    let result_string: String = result_str.into();

    // 90+85+92+78+88+95 = 528
    assert!(result_string.contains("528"));
}
