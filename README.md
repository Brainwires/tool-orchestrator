# Tool Orchestrator

Rhai-based tool orchestration for AI agents, implementing Anthropic's "Programmatic Tool Calling" pattern.

## Overview

Instead of sequential tool calls consuming tokens, the AI writes Rhai scripts that orchestrate multiple tools, returning only the final result.

### Benefits

- **37% token reduction** - intermediate results don't pollute context
- **Parallel tool execution** - multiple tools called in one pass
- **Complex orchestration** - loops, conditionals, data processing

## Usage

```rust
use tool_orchestrator::{ToolOrchestrator, ExecutionLimits};

// Create orchestrator
let mut orchestrator = ToolOrchestrator::new();

// Register tools
orchestrator.register_executor("read_file", |input| {
    let path = input.as_str().unwrap_or("");
    std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
});

orchestrator.register_executor("search_code", |input| {
    let pattern = input.as_str().unwrap_or("");
    // ... search implementation
    Ok("results".to_string())
});

// Execute a Rhai script
let script = r#"
    let todos = search_code("TODO");
    let count = todos.len();
    `Found ${count} TODOs`
"#;

let result = orchestrator.execute(script, ExecutionLimits::default())?;
println!("Output: {}", result.output);
println!("Tool calls: {:?}", result.tool_calls);
```

## Safety

The orchestrator includes built-in limits:

- **max_operations**: Prevents infinite loops
- **max_tool_calls**: Limits tool invocations
- **timeout_ms**: Execution timeout
- **max_string_size**: Maximum string length
- **max_array_size**: Maximum array size

```rust
let limits = ExecutionLimits::default()
    .with_max_operations(50_000)
    .with_max_tool_calls(25)
    .with_timeout_ms(10_000);
```

## Rhai Syntax

Rhai is a Rust-like scripting language:

```rhai
// Variables
let x = 42;
let name = "Claude";

// String interpolation
let greeting = `Hello, ${name}!`;

// Arrays and loops
let items = [1, 2, 3, 4, 5];
let sum = 0;
for item in items {
    sum += item;
}

// Maps (objects)
let config = #{
    debug: true,
    limit: 100
};

// Functions (in-script)
fn double(x) {
    x * 2
}
```

## License

MIT
