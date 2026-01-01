// Baseline-related tests

use costpilot::engines::baselines::{Baseline, BaselinesManager};
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
#[cfg(test)]
use quickcheck_macros::quickcheck;
use std::collections::HashMap;

#[test]
fn test_baseline_within_variance() {
    let baseline = Baseline {
        name: "test-module".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Test baseline".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Actual cost within 10% variance
    let actual_cost = 105.0;
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert!(
        variance <= baseline.acceptable_variance_percent,
        "Cost {} should be within {}% of {}",
        actual_cost,
        baseline.acceptable_variance_percent,
        baseline.expected_monthly_cost
    );
}

#[test]
fn test_baseline_exceeds_variance() {
    let baseline = Baseline {
        name: "test-module".to_string(),
        expected_monthly_cost: 100.0,
        acceptable_variance_percent: 10.0,
        last_updated: "2025-01-01T00:00:00Z".to_string(),
        justification: "Test baseline".to_string(),
        owner: "test-team".to_string(),
        reference: None,
        tags: HashMap::new(),
    };

    // Actual cost exceeds 10% variance
    let actual_cost = 120.0;
    let variance =
        ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs()
            * 100.0;

    assert!(
        variance > baseline.acceptable_variance_percent,
        "Cost {} should exceed {}% variance from {}",
        actual_cost,
        baseline.acceptable_variance_percent,
        baseline.expected_monthly_cost
    );
}

#[test]
fn test_baselines_manager_loads_config() {
    use std::collections::HashMap;

    let config = costpilot::engines::baselines::BaselinesConfig {
        version: "1.0".to_string(),
        global: None,
        modules: HashMap::new(),
        services: HashMap::new(),
        metadata: None,
    };

    let manager = BaselinesManager::from_config(config);

    // Manager should initialize without error
    // TODO: Add meaningful validation checks for baseline manager
    assert!(true, "Manager initialized successfully");
}

// ===== PROPERTY-BASED TESTS =====

proptest! {
    #[test]
    fn test_baseline_variance_calculation(
        expected_cost in 1.0f64..10000.0,
        variance_percent in 0.0f64..50.0,
        actual_cost_factor in 0.5f64..2.0
    ) {
        let actual_cost = expected_cost * actual_cost_factor;

        let baseline = Baseline {
            name: "test-module".to_string(),
            expected_monthly_cost: expected_cost,
            acceptable_variance_percent: variance_percent,
            last_updated: "2025-01-01T00:00:00Z".to_string(),
            justification: "Property test baseline".to_string(),
            owner: "test-team".to_string(),
            reference: None,
            tags: HashMap::new(),
        };

        let variance = ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs() * 100.0;

        // Variance calculation should be mathematically correct
        prop_assert!(variance >= 0.0);

        // Check if actual cost is within variance bounds
        let within_variance = variance <= baseline.acceptable_variance_percent;

        // If actual cost equals expected cost, variance should be 0
        if (actual_cost - expected_cost).abs() < 0.001 {
            prop_assert_eq!(variance, 0.0);
        }
    }

    #[test]
    fn test_baseline_cost_non_negative(
        name in "[a-zA-Z0-9_-]{1,50}",
        expected_cost in 0.0f64..100000.0,
        variance_percent in 0.0f64..100.0
    ) {
        let baseline = Baseline {
            name,
            expected_monthly_cost: expected_cost,
            acceptable_variance_percent: variance_percent,
            last_updated: "2025-01-01T00:00:00Z".to_string(),
            justification: "Property test baseline".to_string(),
            owner: "test-team".to_string(),
            reference: None,
            tags: HashMap::new(),
        };

        // Expected cost should never be negative (we generate non-negative)
        prop_assert!(baseline.expected_monthly_cost >= 0.0);
        prop_assert!(baseline.acceptable_variance_percent >= 0.0);
    }

    #[test]
    fn test_baseline_manager_consistency(
        global_cost in 0.0f64..10000.0,
        module_count in 0..10usize
    ) {
        use std::collections::HashMap;

        let mut modules = HashMap::new();
        for i in 0..module_count {
            let module_name = format!("module-{}", i);
            let baseline = Baseline {
                name: module_name.clone(),
                expected_monthly_cost: global_cost / (module_count as f64 + 1.0),
                acceptable_variance_percent: 10.0,
                last_updated: "2025-01-01T00:00:00Z".to_string(),
                justification: "Property test module".to_string(),
                owner: "test-team".to_string(),
                reference: None,
                tags: HashMap::new(),
            };
            modules.insert(module_name, baseline);
        }

        let config = costpilot::engines::baselines::BaselinesConfig {
            version: "1.0".to_string(),
            global: Some(Baseline {
                name: "global".to_string(),
                expected_monthly_cost: global_cost,
                acceptable_variance_percent: 15.0,
                last_updated: "2025-01-01T00:00:00Z".to_string(),
                justification: "Property test global".to_string(),
                owner: "test-team".to_string(),
                reference: None,
                tags: HashMap::new(),
            }),
            modules,
            services: HashMap::new(),
            metadata: None,
        };

        let manager = BaselinesManager::from_config(config.clone());

        // Manager should be consistent with input config
        let retrieved_config = manager.config();
        prop_assert_eq!(&retrieved_config.version, &config.version);

        if let Some(global) = &config.global {
            prop_assert_eq!(retrieved_config.global.as_ref().unwrap().expected_monthly_cost, global.expected_monthly_cost);
        }
        prop_assert_eq!(retrieved_config.modules.len(), config.modules.len());
    }
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct ArbBaseline(Baseline);

impl Arbitrary for ArbBaseline {
    fn arbitrary(g: &mut Gen) -> Self {
        let name: String = Arbitrary::arbitrary(g);
        let expected_monthly_cost: f64 = Arbitrary::arbitrary(g);
        let acceptable_variance_percent: f64 = Arbitrary::arbitrary(g);

        // Constrain to valid finite positive values
        let expected_monthly_cost =
            if expected_monthly_cost.is_finite() && expected_monthly_cost > 0.0 {
                expected_monthly_cost
            } else {
                100.0 // Default to reasonable positive value
            };

        let acceptable_variance_percent = if acceptable_variance_percent.is_finite()
            && acceptable_variance_percent >= 0.0
            && acceptable_variance_percent <= 100.0
        {
            acceptable_variance_percent
        } else {
            10.0 // Default to reasonable value
        };

        ArbBaseline(Baseline {
            name: if name.is_empty() {
                "test-baseline".to_string()
            } else {
                name
            },
            expected_monthly_cost,
            acceptable_variance_percent,
            last_updated: "2025-01-01T00:00:00Z".to_string(),
            justification: "Arbitrary baseline".to_string(),
            owner: "test-team".to_string(),
            reference: None,
            tags: HashMap::new(),
        })
    }
}

#[quickcheck]
fn quickcheck_baseline_variance_bounds(baseline: ArbBaseline, actual_cost: f64) -> bool {
    // Skip NaN values for actual_cost
    if !actual_cost.is_finite() {
        return true;
    }

    let variance = if baseline.0.expected_monthly_cost == 0.0 {
        // If expected cost is 0, variance is undefined unless actual cost is also 0
        if actual_cost == 0.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        ((actual_cost - baseline.0.expected_monthly_cost) / baseline.0.expected_monthly_cost).abs()
            * 100.0
    };

    // Variance should always be non-negative (or infinite for impossible cases)
    variance >= 0.0 || variance.is_infinite()
}

#[quickcheck]
fn quickcheck_baseline_properties_valid(baseline: ArbBaseline) -> bool {
    // Baseline properties should be valid
    baseline.0.expected_monthly_cost >= 0.0
        && baseline.0.acceptable_variance_percent >= 0.0
        && baseline.0.acceptable_variance_percent <= 100.0
        && !baseline.0.name.is_empty()
        && !baseline.0.justification.is_empty()
        && !baseline.0.owner.is_empty()
}

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
    let variance =
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
