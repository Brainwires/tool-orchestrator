//! Execution limits and sandboxing for safe script execution

use serde::{Deserialize, Serialize};

/// Limits for safe script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Maximum number of operations (prevents infinite loops)
    pub max_operations: u64,
    /// Maximum number of tool calls allowed
    pub max_tool_calls: usize,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum string size in bytes
    pub max_string_size: usize,
    /// Maximum array size
    pub max_array_size: usize,
    /// Maximum map size
    pub max_map_size: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_operations: 100_000,
            max_tool_calls: 50,
            timeout_ms: 30_000, // 30 seconds
            max_string_size: 10_000_000, // 10MB
            max_array_size: 10_000,
            max_map_size: 1_000,
        }
    }
}

impl ExecutionLimits {
    /// Create new limits with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create quick execution limits for simple scripts
    pub fn quick() -> Self {
        Self {
            max_operations: 10_000,
            max_tool_calls: 10,
            timeout_ms: 5_000,
            ..Default::default()
        }
    }

    /// Create extended limits for complex orchestration
    pub fn extended() -> Self {
        Self {
            max_operations: 500_000,
            max_tool_calls: 100,
            timeout_ms: 120_000, // 2 minutes
            ..Default::default()
        }
    }

    /// Builder: set max operations
    pub fn with_max_operations(mut self, max: u64) -> Self {
        self.max_operations = max;
        self
    }

    /// Builder: set max tool calls
    pub fn with_max_tool_calls(mut self, max: usize) -> Self {
        self.max_tool_calls = max;
        self
    }

    /// Builder: set timeout
    pub fn with_timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Builder: set max string size
    pub fn with_max_string_size(mut self, size: usize) -> Self {
        self.max_string_size = size;
        self
    }

    /// Builder: set max array size
    pub fn with_max_array_size(mut self, size: usize) -> Self {
        self.max_array_size = size;
        self
    }

    /// Builder: set max map size
    pub fn with_max_map_size(mut self, size: usize) -> Self {
        self.max_map_size = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ExecutionLimits::default();
        assert_eq!(limits.max_operations, 100_000);
        assert_eq!(limits.max_tool_calls, 50);
        assert_eq!(limits.timeout_ms, 30_000);
    }

    #[test]
    fn test_quick_limits() {
        let limits = ExecutionLimits::quick();
        assert_eq!(limits.max_operations, 10_000);
        assert_eq!(limits.max_tool_calls, 10);
    }

    #[test]
    fn test_extended_limits() {
        let limits = ExecutionLimits::extended();
        assert_eq!(limits.max_operations, 500_000);
        assert_eq!(limits.max_tool_calls, 100);
    }

    #[test]
    fn test_builder_pattern() {
        let limits = ExecutionLimits::default()
            .with_max_operations(50_000)
            .with_max_tool_calls(25)
            .with_timeout_ms(10_000);

        assert_eq!(limits.max_operations, 50_000);
        assert_eq!(limits.max_tool_calls, 25);
        assert_eq!(limits.timeout_ms, 10_000);
    }
}
