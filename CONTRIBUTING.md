# Contributing to tool-orchestrator

Thank you for your interest in contributing to tool-orchestrator! This document provides guidelines and information for contributors.

## Code of Conduct

Please be respectful and constructive in all interactions. We're building tools to help AI agents work more effectively, and we value collaboration.

## Getting Started

### Prerequisites

- Rust (latest stable, 2024 edition)
- wasm-pack (for WASM builds)
- Node.js (for WASM testing)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Brainwires/tool-orchestrator.git
cd tool-orchestrator

# Build native library
cargo build

# Run tests
cargo test

# Build WASM package
wasm-pack build --target web --features wasm --no-default-features
```

## Project Structure

```
tool-orchestrator/
├── src/
│   ├── lib.rs          # Public API and re-exports
│   ├── engine.rs       # Core ToolOrchestrator implementation
│   ├── sandbox.rs      # Sandbox configuration
│   ├── types.rs        # Type definitions
│   └── wasm/
│       └── mod.rs      # WASM bindings
├── examples/           # Runnable examples
├── benches/            # Benchmarks
└── tests/              # Integration tests
```

## Development Workflow

### Making Changes

1. **Fork the repository** and create a feature branch
2. **Write tests** for any new functionality
3. **Run the full test suite** before submitting:
   ```bash
   # Native tests
   cargo test

   # WASM tests (requires wasm-pack)
   wasm-pack test --node --features wasm --no-default-features
   ```
4. **Update documentation** if you've changed the public API
5. **Submit a pull request** with a clear description

### Code Style

- Follow standard Rust conventions (rustfmt)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address any warnings
- Use meaningful variable names
- Add comments for complex logic

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features
```

### Commit Messages

Use clear, descriptive commit messages:

```
feat: Add support for async tool execution
fix: Handle timeout edge case in WASM builds
docs: Update README with new examples
test: Add integration tests for multi-tool scripts
```

Prefixes:
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Build/tooling changes

## Testing

### Running Tests

```bash
# Run all native tests
cargo test

# Run specific test
cargo test test_basic_execution

# Run with output
cargo test -- --nocapture

# Run WASM tests
wasm-pack test --node --features wasm --no-default-features
```

### Writing Tests

- Place unit tests in the same file as the code they test
- Use `#[cfg(test)]` module convention
- Test both success and error cases
- Include edge cases (empty inputs, limits exceeded, etc.)

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_success() {
        // Test normal operation
    }

    #[test]
    fn test_feature_error_case() {
        // Test error handling
    }
}
```

## Architecture Guidelines

### Adding New Features

When adding features, consider:

1. **Security**: Does this maintain the sandbox guarantees?
2. **Both targets**: Will this work in native AND WASM?
3. **Limits**: Is this bounded by ExecutionLimits?
4. **Error handling**: Are errors informative and recoverable?

### Thread Safety

The crate uses conditional compilation for thread safety:

- **Native** (`--features native`): Uses `Arc<Mutex<T>>` for thread safety
- **WASM** (`--features wasm`): Uses `Rc<RefCell<T>>` (single-threaded)

When adding shared state, use the appropriate wrapper:

```rust
#[cfg(feature = "native")]
type SharedState<T> = Arc<Mutex<T>>;

#[cfg(feature = "wasm")]
type SharedState<T> = Rc<RefCell<T>>;
```

### Performance

- Use benchmarks to validate performance claims
- The `on_progress` callback is called frequently - keep it lightweight
- Avoid allocations in hot paths

Run benchmarks:
```bash
cargo bench
```

## Pull Request Process

1. **Ensure all tests pass** on both native and WASM targets
2. **Update CHANGELOG.md** with your changes under `[Unreleased]`
3. **Update documentation** for any API changes
4. **Request review** from maintainers
5. **Address feedback** promptly

### PR Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] `cargo fmt` run
- [ ] `cargo clippy` passes
- [ ] Native tests pass (`cargo test`)
- [ ] WASM tests pass (`wasm-pack test`)

## Reporting Issues

When reporting bugs, please include:

1. **Environment**: Rust version, OS, native or WASM
2. **Minimal reproduction**: Smallest code that shows the issue
3. **Expected behavior**: What should happen
4. **Actual behavior**: What actually happens
5. **Error messages**: Full error output if applicable

## Feature Requests

For feature requests:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case**: Why is this needed?
3. **Propose a solution**: How might this work?
4. **Consider alternatives**: Other ways to solve this?

## Questions?

- Open a GitHub issue for questions
- Tag with `question` label
- Check existing issues first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to tool-orchestrator! 🚀
