# Tool Orchestrator - Universal Programmatic Tool Calling

A model-agnostic implementation of Anthropic's "Programmatic Tool Calling" pattern. Instead of sequential tool calls consuming tokens, any LLM writes Rhai scripts that orchestrate multiple tools efficiently.

## Why Universal?

Anthropic's programmatic tool calling requires Claude + their Python sandbox. This crate provides the same benefits for **any LLM provider**:

- Claude (all versions, not just 4.5)
- OpenAI (GPT-4, etc.)
- Local models (Ollama, llama.cpp)
- Any future provider

## Multi-Target Architecture

This crate produces **three outputs** from a single codebase:

| Target | Description | Use Case |
|--------|-------------|----------|
| **Rust Library** | Native Rust crate | CLI tools, server-side apps |
| **WASM Package** | Browser/Node.js module | Web apps, npm packages |
| **MCP Server** | stdio-based Model Context Protocol server | Easy integration with AI assistants |

## Benefits

- **37% token reduction** - intermediate results don't pollute context
- **Batch operations** - process multiple items in loops
- **Conditional logic** - if/else based on tool results
- **Data transformation** - process and filter between tool calls
- **Model agnostic** - works with any LLM that can write code

## How It Works

1. Register tool executor functions (your actual tool implementations)
2. LLM writes a Rhai script that calls those tools
3. Script executes locally with safety limits
4. Only the final result enters the conversation context

```
┌─────────────┐      ┌──────────────────┐      ┌─────────────┐
│   Any LLM   │ ──▶  │  Rhai Script     │ ──▶  │   Tools     │
│             │      │                  │      │             │
│  "Write a   │      │ let files =      │      │ read_file   │
│   script    │      │   list_dir(".")  │      │ list_dir    │
│   that..."  │      │ for f in files { │      │ search_code │
│             │      │   ...            │      │ git_status  │
└─────────────┘      └──────────────────┘      └─────────────┘
                              │
                              ▼
                     ┌──────────────────┐
                     │  Final Result    │
                     │  (only this goes │
                     │   back to LLM)   │
                     └──────────────────┘
```

## Installation & Building

### Rust Library (default)

```bash
# Add to Cargo.toml
cargo add tool-orchestrator

# Or build from source
cargo build
```

### WASM Package

```bash
# Build for web (browser)
wasm-pack build --target web --features wasm --no-default-features

# Build for Node.js
wasm-pack build --target nodejs --features wasm --no-default-features

# The package is generated in ./pkg/
```

### MCP Server

```bash
# Build the MCP server binary
cargo build --release --features mcp-server --bin tool-orchestrator-mcp

# Run the server (uses stdio transport)
./target/release/tool-orchestrator-mcp
```

## Usage

### Rust Library

```rust
use tool_orchestrator::{ToolOrchestrator, ExecutionLimits};

// Create orchestrator
let mut orchestrator = ToolOrchestrator::new();

// Register tools as executor functions
orchestrator.register_executor("read_file", |input| {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("");
    std::fs::read_to_string(path).map_err(|e| e.to_string())
});

orchestrator.register_executor("list_directory", |input| {
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");
    let entries: Vec<String> = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path().display().to_string()))
        .collect();
    Ok(entries.join("\n"))
});

// Execute a Rhai script (written by any LLM)
let script = r#"
    let files = list_directory("src");
    let rust_files = [];
    for file in files.split("\n") {
        if file.ends_with(".rs") {
            rust_files.push(file);
        }
    }
    `Found ${rust_files.len()} Rust files: ${rust_files}`
"#;

let result = orchestrator.execute(script, ExecutionLimits::default())?;
println!("Output: {}", result.output);           // Final result only
println!("Tool calls: {:?}", result.tool_calls); // Audit trail
```

### WASM (JavaScript/TypeScript)

```typescript
import init, { WasmOrchestrator, ExecutionLimits } from 'tool-orchestrator';

await init();

const orchestrator = new WasmOrchestrator();

// Register a JavaScript function as a tool
orchestrator.register_tool('get_weather', (inputJson: string) => {
  const input = JSON.parse(inputJson);
  // Your implementation here
  return JSON.stringify({ temp: 72, condition: 'sunny' });
});

// Execute a Rhai script
const limits = new ExecutionLimits();
const result = orchestrator.execute(`
  let weather = get_weather("San Francisco");
  \`Current weather: \${weather}\`
`, limits);

console.log(result);
// { success: true, output: "Current weather: ...", tool_calls: [...] }
```

### MCP Server

The MCP server exposes these tools via the Model Context Protocol:

| Tool | Description |
|------|-------------|
| `execute_script` | Execute a Rhai script with registered tools |
| `register_tool` | Register a shell command as a callable tool |
| `unregister_tool` | Remove a registered tool |
| `list_tools` | List all registered shell tools |

#### Claude Desktop Configuration

Add to `~/.config/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "tool-orchestrator": {
      "command": "/path/to/tool-orchestrator-mcp"
    }
  }
}
```

#### Example MCP Usage

```json
// Register a shell tool
{
  "name": "register_tool",
  "arguments": {
    "name": "list_files",
    "description": "List files in a directory",
    "command": "ls -la $input"
  }
}

// Execute a script using registered tools
{
  "name": "execute_script",
  "arguments": {
    "script": "let files = list_files(\".\"); `Files: ${files}`"
  }
}
```

## Safety & Sandboxing

The orchestrator includes built-in limits to prevent runaway scripts:

| Limit | Default | Description |
|-------|---------|-------------|
| `max_operations` | 100,000 | Prevents infinite loops |
| `max_tool_calls` | 50 | Limits tool invocations |
| `timeout_ms` | 30,000 | Execution timeout |
| `max_string_size` | 10MB | Maximum string length |
| `max_array_size` | 10,000 | Maximum array elements |

```rust
// Preset profiles
let quick = ExecutionLimits::quick();      // 10k ops, 10 calls, 5s
let extended = ExecutionLimits::extended(); // 500k ops, 100 calls, 2m

// Custom limits
let limits = ExecutionLimits::default()
    .with_max_operations(50_000)
    .with_max_tool_calls(25)
    .with_timeout_ms(10_000);
```

## Rhai Syntax Quick Reference

Rhai is a Rust-like scripting language that's easy for LLMs to write:

```rhai
// Variables
let x = 42;
let name = "Claude";

// String interpolation (backticks)
let greeting = `Hello, ${name}!`;

// Arrays and loops
let items = [1, 2, 3, 4, 5];
let sum = 0;
for item in items {
    sum += item;
}

// Conditionals
if sum > 10 {
    "Large sum"
} else {
    "Small sum"
}

// Maps (objects)
let config = #{
    debug: true,
    limit: 100
};

// Tool calls (registered functions)
let content = read_file("README.md");
let files = list_directory("src");
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `native` | Yes | Thread-safe with `Arc<Mutex>` (for native Rust) |
| `wasm` | No | Single-threaded with `Rc<RefCell>` (for browser/Node.js) |
| `mcp-server` | No | MCP stdio server binary using `rmcp` SDK |

## Integration Example

The orchestrator integrates with AI agents via a tool definition:

```rust
// Register as "execute_script" tool for the LLM
Tool {
    name: "execute_script",
    description: "Execute a Rhai script for programmatic tool orchestration...",
    input_schema: /* script parameter */,
    requires_approval: false,  // Scripts are sandboxed
    defer_loading: false,      // Primary tool - always available
}
```

When the LLM needs to perform multi-step operations, it writes a Rhai script instead of making sequential individual tool calls. The script executes locally, and only the final result enters the context window.

## Comparison with Anthropic's Approach

| Feature | Anthropic's | Tool Orchestrator |
|---------|-------------|-------------------|
| Language | Python | Rhai (Rust-like) |
| Execution | Their sandbox | Your local process |
| Models | Claude 4.5 only | Any LLM |
| Runtime | Server-side | Client-side |
| Dependencies | API call | Pure Rust, no runtime |
| Targets | Python only | Rust, WASM, MCP |

## License

MIT
