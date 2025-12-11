// WASM runtime wrapper for sandboxed execution

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

/// Initialize WASM runtime
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
}

/// Sandbox limits for WASM execution
#[derive(Debug, Clone)]
pub struct SandboxLimits {
    /// Maximum memory in bytes (default: 256 MB)
    pub max_memory_bytes: usize,
    /// Maximum execution time in milliseconds (default: 2000 ms)
    pub max_execution_ms: u64,
    /// Maximum file size in bytes (default: 20 MB)
    pub max_file_size_bytes: usize,
    /// Maximum stack depth (default: 32)
    pub max_stack_depth: usize,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024,   // 256 MB
            max_execution_ms: 2000,                // 2 seconds
            max_file_size_bytes: 20 * 1024 * 1024, // 20 MB
            max_stack_depth: 32,
        }
    }
}

/// Engine-specific performance budgets
#[derive(Debug, Clone)]
pub struct EngineBudget {
    /// Engine name
    pub name: &'static str,
    /// Time budget in milliseconds
    pub time_budget_ms: u64,
    /// Memory budget in bytes
    pub memory_budget_bytes: usize,
}

impl EngineBudget {
    pub const PREDICTION: Self = Self {
        name: "prediction",
        time_budget_ms: 300,
        memory_budget_bytes: 64 * 1024 * 1024, // 64 MB
    };

    pub const DETECTION: Self = Self {
        name: "detection",
        time_budget_ms: 400,
        memory_budget_bytes: 128 * 1024 * 1024, // 128 MB
    };

    pub const POLICY: Self = Self {
        name: "policy",
        time_budget_ms: 200,
        memory_budget_bytes: 64 * 1024 * 1024, // 64 MB
    };

    pub const MAPPING: Self = Self {
        name: "mapping",
        time_budget_ms: 500,
        memory_budget_bytes: 128 * 1024 * 1024, // 128 MB
    };

    pub const GROUPING: Self = Self {
        name: "grouping",
        time_budget_ms: 400,
        memory_budget_bytes: 128 * 1024 * 1024, // 128 MB
    };

    pub const SLO: Self = Self {
        name: "slo",
        time_budget_ms: 150,
        memory_budget_bytes: 32 * 1024 * 1024, // 32 MB
    };
}

/// Validation result for inputs
#[derive(Debug)]
pub enum ValidationResult {
    Ok,
    ExceedsFileSize { size: usize, limit: usize },
    ExceedsStackDepth { depth: usize, limit: usize },
    InvalidJson { error: String },
}

/// Validate input size against sandbox limits
pub fn validate_input_size(data: &[u8], limits: &SandboxLimits) -> ValidationResult {
    if data.len() > limits.max_file_size_bytes {
        return ValidationResult::ExceedsFileSize {
            size: data.len(),
            limit: limits.max_file_size_bytes,
        };
    }
    ValidationResult::Ok
}

/// Validate JSON structure depth
pub fn validate_json_depth(json: &str, limits: &SandboxLimits) -> ValidationResult {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => {
            let depth = calculate_json_depth(&value);
            if depth > limits.max_stack_depth {
                ValidationResult::ExceedsStackDepth {
                    depth,
                    limit: limits.max_stack_depth,
                }
            } else {
                ValidationResult::Ok
            }
        }
        Err(e) => ValidationResult::InvalidJson {
            error: e.to_string(),
        },
    }
}

fn calculate_json_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            map.values().map(calculate_json_depth).max().unwrap_or(0) + 1
        }
        serde_json::Value::Array(arr) => {
            arr.iter().map(calculate_json_depth).max().unwrap_or(0) + 1
        }
        _ => 1,
    }
}

/// Memory tracker for monitoring usage
pub struct MemoryTracker {
    /// Initial memory usage (tracked for future reporting)
    _initial_usage: usize,
    peak_usage: usize,
    current_usage: usize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        let initial = Self::get_current_memory();
        Self {
            _initial_usage: initial,
            peak_usage: initial,
            current_usage: initial,
        }
    }

    pub fn update(&mut self) {
        self.current_usage = Self::get_current_memory();
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }
    }

    pub fn peak_usage_mb(&self) -> f64 {
        self.peak_usage as f64 / (1024.0 * 1024.0)
    }

    pub fn current_usage_mb(&self) -> f64 {
        self.current_usage as f64 / (1024.0 * 1024.0)
    }

    #[cfg(target_arch = "wasm32")]
    fn get_current_memory() -> usize {
        // In WASM, we can't directly measure memory usage
        // Return 0 as placeholder
        0
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_current_memory() -> usize {
        // On native platforms, use jemalloc or system allocator stats
        // For now, return 0 as placeholder
        0
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_limits_default() {
        let limits = SandboxLimits::default();
        assert_eq!(limits.max_memory_bytes, 256 * 1024 * 1024);
        assert_eq!(limits.max_execution_ms, 2000);
        assert_eq!(limits.max_file_size_bytes, 20 * 1024 * 1024);
        assert_eq!(limits.max_stack_depth, 32);
    }

    #[test]
    fn test_validate_input_size() {
        let limits = SandboxLimits::default();
        let small_data = vec![0u8; 1024]; // 1 KB
        let large_data = vec![0u8; 25 * 1024 * 1024]; // 25 MB

        assert!(matches!(
            validate_input_size(&small_data, &limits),
            ValidationResult::Ok
        ));

        assert!(matches!(
            validate_input_size(&large_data, &limits),
            ValidationResult::ExceedsFileSize { .. }
        ));
    }

    #[test]
    fn test_validate_json_depth() {
        let limits = SandboxLimits::default();

        let shallow_json = r#"{"key": "value"}"#;
        assert!(matches!(
            validate_json_depth(shallow_json, &limits),
            ValidationResult::Ok
        ));

        let deep_json = r#"{"a":{"b":{"c":{"d":{"e":"value"}}}}}"#;
        assert!(matches!(
            validate_json_depth(deep_json, &limits),
            ValidationResult::Ok
        ));

        let invalid_json = r#"{"key": invalid}"#;
        assert!(matches!(
            validate_json_depth(invalid_json, &limits),
            ValidationResult::InvalidJson { .. }
        ));
    }

    #[test]
    fn test_calculate_json_depth() {
        let flat = serde_json::json!({"key": "value"});
        assert_eq!(calculate_json_depth(&flat), 2);

        let nested = serde_json::json!({"a": {"b": {"c": "value"}}});
        assert_eq!(calculate_json_depth(&nested), 4);

        let array = serde_json::json!([1, 2, [3, 4, [5]]]);
        assert_eq!(calculate_json_depth(&array), 4);
    }

    #[test]
    fn test_engine_budgets() {
        assert_eq!(EngineBudget::PREDICTION.time_budget_ms, 300);
        assert_eq!(EngineBudget::DETECTION.time_budget_ms, 400);
        assert_eq!(EngineBudget::POLICY.time_budget_ms, 200);
        assert_eq!(EngineBudget::MAPPING.time_budget_ms, 500);
        assert_eq!(EngineBudget::GROUPING.time_budget_ms, 400);
        assert_eq!(EngineBudget::SLO.time_budget_ms, 150);

        // Total should not exceed 2000ms
        let total = EngineBudget::PREDICTION.time_budget_ms
            + EngineBudget::DETECTION.time_budget_ms
            + EngineBudget::POLICY.time_budget_ms
            + EngineBudget::MAPPING.time_budget_ms
            + EngineBudget::GROUPING.time_budget_ms
            + EngineBudget::SLO.time_budget_ms;

        assert!(total <= 2000, "Total engine budget exceeds 2000ms");
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();
        assert!(tracker.peak_usage_mb() >= 0.0);
        assert!(tracker.current_usage_mb() >= 0.0);
    }
}
