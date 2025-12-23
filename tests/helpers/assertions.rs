/// Custom assertions for CostPilot tests
///
/// Provides domain-specific assertion helpers to make tests more readable
/// and maintainable.

/// Assert that a prediction is valid
/// - Confidence interval bounds are ordered correctly
/// - Cost is non-negative
/// - Confidence is between 0 and 1
pub fn assert_prediction_valid(
    low: f64,
    estimate: f64,
    high: f64,
    confidence: f64,
) {
    assert!(
        low <= estimate,
        "Low estimate ({}) must be <= central estimate ({})",
        low,
        estimate
    );
    assert!(
        estimate <= high,
        "Central estimate ({}) must be <= high estimate ({})",
        estimate,
        high
    );
    assert!(
        low >= 0.0,
        "Low estimate must be non-negative, got {}",
        low
    );
    assert!(
        confidence >= 0.0 && confidence <= 1.0,
        "Confidence must be between 0 and 1, got {}",
        confidence
    );
}

/// Assert that a confidence interval is never inverted
pub fn assert_interval_not_inverted(p10: f64, p50: f64, p90: f64, p99: f64) {
    assert!(
        p10 <= p50,
        "P10 ({}) must be <= P50 ({})",
        p10,
        p50
    );
    assert!(
        p50 <= p90,
        "P50 ({}) must be <= P90 ({})",
        p50,
        p90
    );
    assert!(
        p90 <= p99,
        "P90 ({}) must be <= P99 ({})",
        p90,
        p99
    );
}

/// Assert that a cost delta is valid
/// - Monthly delta matches (new - old)
/// - Percentage calculation is correct
pub fn assert_cost_delta_valid(old_cost: f64, new_cost: f64, delta: f64, percentage: f64) {
    let expected_delta = new_cost - old_cost;
    assert!(
        (delta - expected_delta).abs() < 0.01,
        "Delta ({}) should equal new_cost - old_cost ({} - {}) = {}",
        delta,
        new_cost,
        old_cost,
        expected_delta
    );

    if old_cost > 0.0 {
        let expected_percentage = ((new_cost - old_cost) / old_cost) * 100.0;
        assert!(
            (percentage - expected_percentage).abs() < 0.01,
            "Percentage ({}) should equal ((new - old) / old) * 100 = {}",
            percentage,
            expected_percentage
        );
    }
}

/// Assert that a graph is acyclic (no cycles)
pub fn assert_graph_acyclic(has_cycles: bool, cycle_nodes: &[String]) {
    assert!(
        !has_cycles,
        "Graph must be acyclic, found cycles involving nodes: {:?}",
        cycle_nodes
    );
}

/// Assert that policy evaluation result is valid
pub fn assert_policy_evaluation_valid(
    passed: bool,
    violations_count: usize,
    expected_violations: usize,
) {
    if expected_violations > 0 {
        assert!(
            !passed,
            "Policy should fail with {} violations",
            expected_violations
        );
        assert_eq!(
            violations_count, expected_violations,
            "Expected {} violations, got {}",
            expected_violations, violations_count
        );
    } else {
        assert!(
            passed,
            "Policy should pass with no violations, got {}",
            violations_count
        );
        assert_eq!(violations_count, 0, "Expected no violations");
    }
}

/// Assert that SLO check result is valid
pub fn assert_slo_check_valid(
    passed: bool,
    actual_value: f64,
    threshold: f64,
    should_block: bool,
) {
    if actual_value > threshold {
        assert!(
            !passed,
            "SLO should fail when actual ({}) > threshold ({})",
            actual_value,
            threshold
        );
    } else {
        assert!(
            passed,
            "SLO should pass when actual ({}) <= threshold ({})",
            actual_value,
            threshold
        );
    }

    if should_block {
        assert!(
            !passed,
            "SLO configured to block should not pass"
        );
    }
}

/// Assert that memory usage is within limits
pub fn assert_memory_within_limits(used_mb: f64, limit_mb: f64) {
    assert!(
        used_mb <= limit_mb,
        "Memory usage ({} MB) exceeds limit ({} MB)",
        used_mb,
        limit_mb
    );
    assert!(
        used_mb >= 0.0,
        "Memory usage must be non-negative, got {} MB",
        used_mb
    );
}

/// Assert that execution time is within budget
pub fn assert_time_within_budget(elapsed_ms: u128, budget_ms: u128) {
    assert!(
        elapsed_ms <= budget_ms,
        "Execution time ({} ms) exceeds budget ({} ms)",
        elapsed_ms,
        budget_ms
    );
}

/// Assert that a JSON value has required fields
pub fn assert_json_has_fields(value: &serde_json::Value, fields: &[&str]) {
    for field in fields {
        assert!(
            value.get(field).is_some(),
            "JSON object missing required field: {}",
            field
        );
    }
}

/// Assert that two f64 values are approximately equal
pub fn assert_approx_eq(actual: f64, expected: f64, tolerance: f64, message: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "{}: expected ~{}, got {} (diff: {})",
        message,
        expected,
        actual,
        diff
    );
}

/// Assert that a string matches a regex pattern
pub fn assert_matches_pattern(text: &str, pattern: &str) {
    let re = regex::Regex::new(pattern).expect("Invalid regex pattern");
    assert!(
        re.is_match(text),
        "Text '{}' does not match pattern '{}'",
        text,
        pattern
    );
}

/// Assert that a file size is within limits
pub fn assert_file_size_within_limit(size_bytes: usize, limit_bytes: usize) {
    assert!(
        size_bytes <= limit_bytes,
        "File size ({} bytes) exceeds limit ({} bytes)",
        size_bytes,
        limit_bytes
    );
}

/// Assert that an error has the expected error code
pub fn assert_error_code(error_code: &str, expected_code: &str) {
    assert_eq!(
        error_code, expected_code,
        "Expected error code '{}', got '{}'",
        expected_code, error_code
    );
}

/// Assert that deterministic execution produces identical outputs
pub fn assert_deterministic<T: PartialEq + std::fmt::Debug>(
    run1: &T,
    run2: &T,
    context: &str,
) {
    assert_eq!(
        run1, run2,
        "Deterministic execution failed for {}: outputs differ",
        context
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_prediction_valid() {
        assert_prediction_valid(10.0, 20.0, 30.0, 0.9);
    }

    #[test]
    #[should_panic(expected = "Low estimate")]
    fn test_assert_prediction_invalid_order() {
        assert_prediction_valid(30.0, 20.0, 10.0, 0.9);
    }

    #[test]
    fn test_assert_interval_not_inverted() {
        assert_interval_not_inverted(10.0, 20.0, 30.0, 40.0);
    }

    #[test]
    #[should_panic(expected = "P10")]
    fn test_assert_interval_inverted() {
        assert_interval_not_inverted(30.0, 20.0, 10.0, 5.0);
    }

    #[test]
    fn test_assert_cost_delta_valid() {
        assert_cost_delta_valid(100.0, 150.0, 50.0, 50.0);
    }

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(10.001, 10.0, 0.01, "Should be approximately equal");
    }
}
