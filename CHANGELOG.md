# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-24

### Changed
- **Version bump to 1.0.0** - Production-ready release
- Added compile-time guard preventing simultaneous use of `native` and `wasm` features
- Added `categories`, `documentation`, `readme`, and `rust-version` fields to Cargo.toml

### Fixed
- Compilation error when both `native` and `wasm` features were enabled together

## [0.1.0] - 2025-12-05

### Development History

#### December 5, 2025 - Production Ready Release

**Real-time Timeout Enforcement** (`f9adc53`)
- Implemented `on_progress` callback for true wall-clock timeout
- Both native and WASM modules now enforce timeouts in real-time
- Updated error handling to properly detect `ErrorTerminated`
- Fixed timeout test to use CPU-intensive loop instead of `thread::sleep`

**Dependency Updates & WASM Compatibility** (`773872c`)
- Upgraded Rhai from 1.20 to 1.23
- Replaced `instant` crate with `web-time` for WASM compatibility
- Enhanced README with Anthropic background and token savings analysis

**Test Suite Expansion** (`a3d474f`)
- Upgraded to Rust 2024 edition
- Added comprehensive tests for `ExecutionLimits` builder pattern
- Added tests for all `dynamic_to_json` type conversions
- Added README badges for version, license, and Rust version

**Architecture Refactor** (`942c401`)
- Removed experimental MCP server implementation
- Focused on core programmatic tool calling pattern
- Simplified multi-target architecture
- Updated tests for cleaner API

**Multi-Target Support** (`1e852a2`)
- Added WASM target with `wasm-bindgen`
- Feature-gated thread safety (`native` vs `wasm`)
- Native: `Arc<Mutex>` for thread safety
- WASM: `Rc<RefCell>` for single-threaded environments

#### December 3, 2025 - Documentation

**README Enhancement** (`9e3763e`)
- Expanded documentation on universal tool orchestration
- Added detailed benefit explanations
- Improved usage examples
- Documented safety limits and sandboxing

#### December 2, 2025 - Initial Development

**Initial Implementation** (`7d31c92`)
- Core `ToolOrchestrator` struct with tool registration
- `ExecutionLimits` for sandboxed execution
- Rhai integration for script execution
- `dynamic_to_json` conversion for tool arguments
- Initial test suite

---

### Added

- Core `ToolOrchestrator` with tool registration and script execution
- `ExecutionLimits` for sandboxing with configurable limits:
  - `max_operations` - prevents infinite loops (default: 100,000)
  - `max_tool_calls` - limits tool invocations (default: 50)
  - `timeout_ms` - real-time timeout via `on_progress` callback (default: 30s)
  - `max_string_size` - prevents memory bombs (default: 10MB)
  - `max_array_size` - limits collection sizes (default: 10,000)
  - `max_map_size` - limits map sizes (default: 1,000)
- Preset limit profiles: `quick()`, `default()`, `extended()`
- Multi-target architecture:
  - Native Rust library with `Arc<Mutex>` thread safety
  - WASM package with `Rc<RefCell>` for browser/Node.js
- Comprehensive test suite (64 tests):
  - 39 native tests
  - 25 WASM tests
- Security features:
  - Rhai sandbox (no filesystem, network, shell access)
  - Real-time timeout enforcement via `on_progress`
  - Configurable resource limits
- Documentation:
  - README with problem/solution framing
  - Security considerations section
  - LLM system prompt template
  - Rhai syntax cheatsheet
  - API documentation (rustdoc)
  - CONTRIBUTING.md guide
- Examples:
  - `expense_aggregation` - Classic batch processing use case
  - `file_operations` - File system orchestration
  - `multi_api` - Multiple API orchestration with conditionals
- Benchmarks:
  - Token comparison benchmark demonstrating savings

### Technical Details

- Built on [Rhai](https://rhai.rs/) scripting engine v1.23
- Rust 2024 edition
- Zero runtime dependencies for native builds
- Pure Rust implementation

### References

- Implements patterns from [Anthropic's Advanced Tool Use](https://www.anthropic.com/engineering/advanced-tool-use)
- Inspired by [CodeAct](https://arxiv.org/abs/2402.01030) research on code-based tool orchestration

[Unreleased]: https://github.com/Brainwires/tool-orchestrator/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Brainwires/tool-orchestrator/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/Brainwires/tool-orchestrator/releases/tag/v0.1.0
