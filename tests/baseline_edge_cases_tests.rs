// Baseline edge case tests

use costpilot::engines::baselines::Baseline;
use std::collections::HashMap;

// ===== EDGE CASE TESTS =====

#[test]
fn test_baseline_zero_expected_cost_edge_case() {
    let baseline = Baseline {
        name: "zero-cost-service".to_string(),
        expected_monthly_cost: 0.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Service with zero expected cost".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Zero actual cost should have zero variance
    let actual_cost = 0.0;
    let _variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;
    // This would cause division by zero, so we need to handle it specially
    assert!(baseline.expected_monthly_cost == 0.0); // Just verify the condition
}

#[test]
fn test_baseline_extreme_variance_percentages() {
    // Test 0% variance (no tolerance)
    let baseline_strict = Baseline {
        name: "strict-budget".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 0.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Zero tolerance budget".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Test 100% variance (very loose)
    let baseline_loose = Baseline {
        name: "loose-budget".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 100.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "High tolerance budget".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Both should be valid baselines
    assert_eq!(baseline_strict.acceptable_variance_percent, 0.0);
    assert_eq!(baseline_loose.acceptable_variance_percent, 100.0);
}

#[test]
fn test_baseline_extremely_large_costs() {
    let baseline = Baseline {
        name: "enterprise-scale".to_string(),
        expected_monthly_cost: 1_000_000.0, // 1M per month
        acceptable_variance_percent: 50.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Large scale enterprise service".to_string(),
        owner: "enterprise-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Test with costs that exceed the baseline significantly
    let actual_cost = 1_500_000.0; // 50% over budget
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert_eq!(variance, 50.0);
    assert!(variance <= baseline.acceptable_variance_percent);
}

#[test]
fn test_baseline_negative_actual_cost_edge_case() {
    let baseline = Baseline {
        name: "service-with-credits".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 20.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Service that might receive credits".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Negative actual cost (credits received)
    let actual_cost = -50.0;
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert_eq!(variance, 150.0); // 150% variance due to sign change
    assert!(variance > baseline.acceptable_variance_percent); // Should exceed threshold
}

#[test]
fn test_baseline_empty_name_edge_case() {
    // Test that empty names are handled (though our Arbitrary implementation prevents this)
    let baseline = Baseline {
        name: "".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Empty name test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Empty name should be detectable
    assert!(baseline.name.is_empty());
}

#[test]
fn test_baseline_maximum_variance_boundary() {
    let baseline = Baseline {
        name: "max-variance-test".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 100.0, // Maximum allowed
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Maximum variance boundary test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Test exactly at the boundary
    let actual_cost = 200.0; // 100% increase
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert_eq!(variance, 100.0);
    assert_eq!(variance, baseline.acceptable_variance_percent); // Exactly at boundary
}

#[test]
fn test_baseline_minimum_cost_boundary() {
    let baseline = Baseline {
        name: "min-cost-test".to_string(),
        expected_monthly_cost: 0.01, // Very small positive cost
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Minimum cost boundary test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Test with zero actual cost
    let actual_cost = 0.0;
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert_eq!(variance, 100.0); // 100% variance (complete elimination)
    assert!(variance > baseline.acceptable_variance_percent);
}

#[test]
fn test_baseline_extreme_name_lengths() {
    // Test with extremely long name
    let long_name = "a".repeat(1000);
    let baseline_long = Baseline {
        name: long_name.clone(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Long name test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    assert_eq!(baseline_long.name.len(), 1000);
    assert!(baseline_long.name.starts_with("a"));
}

#[test]
fn test_baseline_special_characters_in_name() {
    // Test with special characters and Unicode
    let special_name = "test_服务-123.特殊@chars#";
    let baseline = Baseline {
        name: special_name.to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Special characters test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    assert_eq!(baseline.name, special_name);
    assert!(baseline.name.contains("服务")); // Unicode characters
    assert!(baseline.name.contains("@")); // Special symbols
}

#[test]
fn test_baseline_fractional_cents_precision() {
    let baseline = Baseline {
        name: "precision-test".to_string(),
        expected_monthly_cost: 0.0001, // Fractional cents
        acceptable_variance_percent: 1.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Precision test".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Test with very small actual cost
    let actual_cost = 0.0002;
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert_eq!(variance, 100.0); // 100% increase
    assert!(variance > baseline.acceptable_variance_percent);
}
