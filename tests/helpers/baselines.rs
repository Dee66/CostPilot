/// Baseline test helpers for deterministic comparison
/// 
/// Provides consistent module weight calculation and violation formatting.

use std::collections::HashMap;

/// Get deterministic module weights for baseline comparison tests
pub fn make_test_module_weights() -> HashMap<String, f64> {
    let mut weights = HashMap::new();
    weights.insert("compute".to_string(), 1.0);
    weights.insert("storage".to_string(), 0.8);
    weights.insert("network".to_string(), 0.6);
    weights
}

/// Format violation string matching current baseline engine format
pub fn format_baseline_violation(
    module_path: &str,
    actual: f64,
    expected: f64,
    threshold: f64,
) -> String {
    let delta = actual - expected;
    let pct = (delta / expected * 100.0).abs();
    format!(
        "Module '{}' exceeded baseline by ${:.2} ({:.1}% over ${:.2} threshold)",
        module_path, delta, pct, threshold
    )
}
