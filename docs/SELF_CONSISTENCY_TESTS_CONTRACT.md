# Self-Consistency Tests Contract

**Version:** 1.0.0
**Status:** Enforced
**Last Updated:** 2025-12-06

---

## Overview

CostPilot engines must be **internally consistent**. Self-consistency tests catch bugs where different systems disagree about the same data. These are **meta-tests** that validate the logic across engines.

---

## Core Principle

**If multiple engines analyze the same input, their outputs must be logically consistent.**

Examples:
- Detection says "EC2 instance added" → Prediction must show cost for that EC2
- Prediction says "confidence: 95%" → Explain engine must show strong evidence
- Mapping says "resource A depends on B" → Grouping must respect this
- Policy says "CRITICAL violation" → Severity must match confidence/delta

---

## Consistency Checks

### 1. Detection ↔ Prediction Consistency

**Rule:** Every detected resource must have a prediction.

```rust
#[test]
fn test_detection_prediction_consistency() {
    let plan = sample_terraform_plan();

    // Run detection
    let detected_resources = detect_resources(&plan).unwrap();

    // Run prediction
    let predictions = predict_costs(&plan).unwrap();

    // Every detected resource must have a prediction
    for resource in detected_resources {
        let prediction = predictions
            .iter()
            .find(|p| p.resource_id == resource.id);

        assert!(
            prediction.is_some(),
            "Resource {} detected but no prediction found",
            resource.id
        );

        // Prediction cost must be > 0 for detected resources
        if let Some(pred) = prediction {
            assert!(
                pred.p50 > 0.0,
                "Resource {} detected but prediction is zero",
                resource.id
            );
        }
    }
}
```

**Rule:** Detected cost change must match prediction delta.

```rust
#[test]
fn test_detection_cost_change_matches_prediction_delta() {
    let baseline = sample_baseline();
    let plan = sample_modified_plan();

    // Detection detects cost change
    let cost_changes = detect_cost_changes(&baseline, &plan).unwrap();

    // Prediction predicts delta
    let predictions = predict_costs(&plan).unwrap();
    let baseline_predictions = predict_costs(&baseline).unwrap();

    for change in cost_changes {
        let new_pred = predictions
            .iter()
            .find(|p| p.resource_id == change.resource_id)
            .unwrap();

        let old_pred = baseline_predictions
            .iter()
            .find(|p| p.resource_id == change.resource_id)
            .unwrap();

        let predicted_delta = new_pred.p50 - old_pred.p50;

        // Deltas must match within 5%
        let tolerance = predicted_delta.abs() * 0.05;
        assert!(
            (change.delta - predicted_delta).abs() <= tolerance,
            "Detection delta ({}) doesn't match prediction delta ({})",
            change.delta,
            predicted_delta
        );
    }
}
```

---

### 2. Prediction ↔ Explain Consistency

**Rule:** High confidence must have strong evidence.

```rust
#[test]
fn test_prediction_confidence_matches_explain_evidence() {
    let resource = sample_ec2_resource();

    // Prediction
    let prediction = predict_cost(&resource).unwrap();

    // Explanation
    let explanation = explain_prediction(&resource, &prediction).unwrap();

    if prediction.confidence >= 0.9 {
        // High confidence must have:
        // - At least 3 reasoning steps
        assert!(
            explanation.reasoning.len() >= 3,
            "High confidence ({}) but only {} reasoning steps",
            prediction.confidence,
            explanation.reasoning.len()
        );

        // - Heuristic provenance (not cold-start)
        assert!(
            matches!(
                explanation.provenance.confidence_source,
                ConfidenceSource::Heuristic { .. }
            ),
            "High confidence but using cold-start inference"
        );

        // - No fallback reason
        assert!(
            explanation.provenance.fallback_reason.is_none(),
            "High confidence but has fallback reason"
        );
    }
}
```

**Rule:** Low confidence must explain why.

```rust
#[test]
fn test_low_confidence_has_fallback_reason() {
    let resource = sample_unsupported_resource();

    let prediction = predict_cost(&resource).unwrap();
    let explanation = explain_prediction(&resource, &prediction).unwrap();

    if prediction.confidence < 0.5 {
        // Low confidence must have fallback reason
        assert!(
            explanation.provenance.fallback_reason.is_some(),
            "Low confidence ({}) but no fallback reason",
            prediction.confidence
        );

        // Must use cold-start
        assert!(
            matches!(
                explanation.provenance.confidence_source,
                ConfidenceSource::ColdStart { .. }
            ),
            "Low confidence but not using cold-start"
        );
    }
}
```

---

### 3. Mapping ↔ Grouping Consistency

**Rule:** Grouping must respect dependency graph.

```rust
#[test]
fn test_grouping_respects_dependencies() {
    let plan = sample_terraform_plan();

    // Build dependency graph
    let graph = build_dependency_graph(&plan).unwrap();

    // Group resources
    let groups = group_resources(&plan, GroupBy::Module).unwrap();

    // Check: if A depends on B, they must be in topologically sorted order
    for edge in &graph.edges {
        let from_group = find_group_for_resource(&groups, &edge.from).unwrap();
        let to_group = find_group_for_resource(&groups, &edge.to).unwrap();

        // If different groups, check topological order
        if from_group.id != to_group.id {
            let from_index = groups.iter().position(|g| g.id == from_group.id).unwrap();
            let to_index = groups.iter().position(|g| g.id == to_group.id).unwrap();

            assert!(
                from_index >= to_index,
                "Resource {} (group {}) depends on {} (group {}), but order is wrong",
                edge.from,
                from_index,
                edge.to,
                to_index
            );
        }
    }
}
```

**Rule:** Cycle detection must match graph structure.

```rust
#[test]
fn test_cycle_detection_consistent() {
    let plan = sample_terraform_plan_with_cycle();

    // Build graph
    let graph = build_dependency_graph(&plan).unwrap();

    // Check for cycles (mapping engine)
    let has_cycle = graph.has_cycle();

    // Grouping must detect same cycles
    let grouping_result = group_resources(&plan, GroupBy::Dependencies);

    if has_cycle {
        // Grouping should fail or warn
        assert!(
            grouping_result.is_err() || grouping_result.unwrap().warnings.iter().any(|w| w.contains("cycle")),
            "Mapping detected cycle but grouping didn't"
        );
    }
}
```

---

### 4. Policy ↔ Severity Consistency

**Rule:** Severity must match cost delta and confidence.

```rust
#[test]
fn test_severity_matches_cost_delta() {
    let deltas = vec![
        (100.0, Severity::Info),      // <5%
        (500.0, Severity::Low),       // 5-20%
        (1500.0, Severity::Medium),   // 20-50%
        (3000.0, Severity::High),     // 50-100%
        (7000.0, Severity::Critical), // >100%
    ];

    let baseline_cost = 3000.0;

    for (new_cost, expected_severity) in deltas {
        let delta = CostDelta::new(baseline_cost, new_cost, Interval::Month);
        let computed_severity = delta.severity();

        assert_eq!(
            computed_severity,
            expected_severity,
            "Delta {}% should be {:?} but got {:?}",
            delta.percentage,
            expected_severity,
            computed_severity
        );
    }
}
```

**Rule:** Policy violation severity must match cost impact.

```rust
#[test]
fn test_policy_violation_severity_consistent() {
    let plan = sample_terraform_plan();

    // Evaluate policy
    let policy_result = evaluate_policy(&plan, &sample_policy()).unwrap();

    // Check predictions
    let predictions = predict_costs(&plan).unwrap();

    for violation in &policy_result.violations {
        // Find affected resource
        let resource = plan.resources
            .iter()
            .find(|r| r.id == violation.resource_id)
            .unwrap();

        let prediction = predictions
            .iter()
            .find(|p| p.resource_id == resource.id)
            .unwrap();

        // Severity should correlate with cost
        match violation.severity {
            Severity::Critical => {
                assert!(
                    prediction.p50 >= 1000.0,
                    "CRITICAL violation but cost is only ${:.2}",
                    prediction.p50
                );
            }
            Severity::High => {
                assert!(
                    prediction.p50 >= 100.0,
                    "HIGH violation but cost is only ${:.2}",
                    prediction.p50
                );
            }
            _ => {}
        }
    }
}
```

---

### 5. Heuristic ↔ Confidence Consistency

**Rule:** Stale heuristics must reduce confidence.

```rust
#[test]
fn test_stale_heuristics_reduce_confidence() {
    use chrono::{Utc, Duration};

    let fresh_heuristic = Heuristic {
        updated_at: Utc::now(),
        ..sample_heuristic()
    };

    let stale_heuristic = Heuristic {
        updated_at: Utc::now() - Duration::days(180),  // 6 months old
        ..sample_heuristic()
    };

    let resource = sample_ec2_resource();

    let fresh_prediction = predict_with_heuristic(&resource, &fresh_heuristic).unwrap();
    let stale_prediction = predict_with_heuristic(&resource, &stale_heuristic).unwrap();

    assert!(
        stale_prediction.confidence < fresh_prediction.confidence,
        "Stale heuristic should have lower confidence"
    );

    // Explanation must mention staleness
    let explanation = explain_prediction(&resource, &stale_prediction).unwrap();
    assert!(
        matches!(
            explanation.provenance.fallback_reason,
            Some(FallbackReason::HeuristicStale)
        ),
        "Stale heuristic must have fallback reason"
    );
}
```

**Rule:** Missing heuristics must use cold-start.

```rust
#[test]
fn test_missing_heuristic_uses_cold_start() {
    let resource = Resource {
        region: "eu-north-1".to_string(),  // Unsupported region
        instance_type: "t3.xlarge".to_string(),
        ..sample_ec2_resource()
    };

    let prediction = predict_cost(&resource).unwrap();

    // Must use cold-start
    let explanation = explain_prediction(&resource, &prediction).unwrap();
    assert!(
        matches!(
            explanation.provenance.confidence_source,
            ConfidenceSource::ColdStart { .. }
        ),
        "Missing heuristic must use cold-start"
    );

    // Must have fallback reason
    assert!(
        matches!(
            explanation.provenance.fallback_reason,
            Some(FallbackReason::RegionNotSupported) | Some(FallbackReason::HeuristicMissing)
        ),
        "Missing heuristic must have fallback reason"
    );

    // Confidence should be lower
    assert!(
        prediction.confidence < 0.7,
        "Cold-start should have lower confidence"
    );
}
```

---

### 6. Regression Classifier ↔ Prediction Delta

**Rule:** Classifier must match delta sign.

```rust
#[test]
fn test_regression_classifier_matches_delta_sign() {
    let baseline = sample_baseline();
    let plan = sample_modified_plan();

    // Classify regression
    let regression = classify_regression(&baseline, &plan).unwrap();

    // Compute delta
    let baseline_cost = predict_total_cost(&baseline).unwrap();
    let plan_cost = predict_total_cost(&plan).unwrap();
    let delta = plan_cost - baseline_cost;

    // Check consistency
    match regression.regression_type {
        RegressionType::CostIncrease => {
            assert!(
                delta > 0.0,
                "Classified as CostIncrease but delta is {}",
                delta
            );
        }
        RegressionType::CostDecrease => {
            assert!(
                delta < 0.0,
                "Classified as CostDecrease but delta is {}",
                delta
            );
        }
        RegressionType::NewResource => {
            assert!(
                delta >= 0.0,
                "New resource should increase cost, but delta is {}",
                delta
            );
        }
        _ => {}
    }
}
```

---

### 7. Float Math Consistency

**Rule:** Prediction intervals must be ordered.

```rust
#[test]
fn test_prediction_intervals_ordered() {
    let resources = sample_resources(100);

    for resource in resources {
        let prediction = predict_cost(&resource).unwrap();

        // p10 ≤ p50 ≤ p90 ≤ p99
        assert!(
            prediction.p10 <= prediction.p50,
            "p10 ({}) > p50 ({})",
            prediction.p10,
            prediction.p50
        );

        assert!(
            prediction.p50 <= prediction.p90,
            "p50 ({}) > p90 ({})",
            prediction.p50,
            prediction.p90
        );

        assert!(
            prediction.p90 <= prediction.p99,
            "p90 ({}) > p99 ({})",
            prediction.p90,
            prediction.p99
        );
    }
}
```

**Rule:** Cost deltas must sum correctly.

```rust
#[test]
fn test_cost_deltas_sum_correctly() {
    let plan = sample_terraform_plan();

    let predictions = predict_costs(&plan).unwrap();

    // Sum individual predictions
    let sum_individual: f64 = predictions.iter().map(|p| p.p50).sum();

    // Total cost prediction
    let total = predict_total_cost(&plan).unwrap();

    // Must match within floating-point tolerance
    let tolerance = 0.01;  // 1 cent
    assert!(
        (sum_individual - total).abs() <= tolerance,
        "Individual costs sum to {:.2} but total is {:.2}",
        sum_individual,
        total
    );
}
```

---

### 8. JSON Schema Consistency

**Rule:** All JSON outputs must have schema_version.

```rust
#[test]
fn test_all_json_has_schema_version() {
    let outputs = vec![
        generate_scan_output(&sample_plan()).unwrap(),
        generate_explain_output(&sample_resource()).unwrap(),
        generate_policy_output(&sample_plan()).unwrap(),
        generate_map_output(&sample_plan()).unwrap(),
        generate_group_output(&sample_plan()).unwrap(),
    ];

    for output in outputs {
        let json: Value = serde_json::from_str(&output).unwrap();

        assert!(
            json.get("schema_version").is_some(),
            "JSON output missing schema_version: {}",
            output
        );

        let version = json["schema_version"].as_str().unwrap();
        assert!(
            version.starts_with("1."),
            "Invalid schema version: {}",
            version
        );
    }
}
```

**Rule:** JSON keys must be alphabetically sorted.

```rust
#[test]
fn test_json_keys_alphabetical() {
    let output = generate_scan_output(&sample_plan()).unwrap();
    let json: Value = serde_json::from_str(&output).unwrap();

    fn check_keys_sorted(value: &Value, path: &str) {
        if let Some(obj) = value.as_object() {
            let keys: Vec<_> = obj.keys().collect();
            let mut sorted_keys = keys.clone();
            sorted_keys.sort();

            assert_eq!(
                keys,
                sorted_keys,
                "Keys not sorted at path: {}",
                path
            );

            // Recursively check nested objects
            for (key, nested_value) in obj {
                check_keys_sorted(nested_value, &format!("{}.{}", path, key));
            }
        }
    }

    check_keys_sorted(&json, "root");
}
```

---

### 9. Error Code Consistency

**Rule:** Error categories must match code ranges.

```rust
#[test]
fn test_error_code_categories_consistent() {
    use strum::IntoEnumIterator;

    for code in ErrorCode::iter() {
        let computed_category = code.category();

        let code_num = code.as_str()[1..].parse::<u32>().unwrap();
        let expected_category = match code_num {
            1..=99 => ErrorCategory::Parse,
            100..=199 => ErrorCategory::Validation,
            200..=299 => ErrorCategory::Runtime,
            300..=399 => ErrorCategory::IO,
            400..=499 => ErrorCategory::Configuration,
            500..=599 => ErrorCategory::Internal,
            _ => ErrorCategory::Unknown,
        };

        assert_eq!(
            computed_category,
            expected_category,
            "Error code {} has wrong category",
            code.as_str()
        );
    }
}
```

---

### 10. CLI Exit Code Consistency

**Rule:** CLI exit codes must match error categories.

```rust
#[test]
fn test_cli_exit_codes_consistent() {
    let test_cases = vec![
        (ErrorCategory::Parse, 10),
        (ErrorCategory::Validation, 11),
        (ErrorCategory::Runtime, 12),
        (ErrorCategory::IO, 13),
        (ErrorCategory::Configuration, 14),
        (ErrorCategory::Internal, 15),
    ];

    for (category, expected_code) in test_cases {
        let error = ErrorSignature {
            category,
            ..sample_error()
        };

        assert_eq!(
            error.exit_code(),
            expected_code,
            "Category {:?} should have exit code {}",
            category,
            expected_code
        );
    }
}
```

---

## Meta-Test Runner

```rust
/// Runs all self-consistency tests
#[test]
fn run_all_self_consistency_tests() {
    // Detection ↔ Prediction
    test_detection_prediction_consistency();
    test_detection_cost_change_matches_prediction_delta();

    // Prediction ↔ Explain
    test_prediction_confidence_matches_explain_evidence();
    test_low_confidence_has_fallback_reason();

    // Mapping ↔ Grouping
    test_grouping_respects_dependencies();
    test_cycle_detection_consistent();

    // Policy ↔ Severity
    test_severity_matches_cost_delta();
    test_policy_violation_severity_consistent();

    // Heuristic ↔ Confidence
    test_stale_heuristics_reduce_confidence();
    test_missing_heuristic_uses_cold_start();

    // Regression ↔ Delta
    test_regression_classifier_matches_delta_sign();

    // Float Math
    test_prediction_intervals_ordered();
    test_cost_deltas_sum_correctly();

    // JSON Schema
    test_all_json_has_schema_version();
    test_json_keys_alphabetical();

    // Error Codes
    test_error_code_categories_consistent();
    test_cli_exit_codes_consistent();
}
```

---

## CI Integration

```yaml
# .github/workflows/self-consistency-tests.yml
name: Self-Consistency Tests

on: [push, pull_request]

jobs:
  self-consistency:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run self-consistency tests
        run: cargo test --test self_consistency

      - name: Check for consistency violations
        run: |
          cargo test --test self_consistency -- --nocapture | \
            grep -i "consistency violation" && exit 1 || exit 0
```

---

## Fuzzing for Consistency

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_detection_prediction_consistency_fuzz(
        plan in arb_terraform_plan()
    ) {
        let detected = detect_resources(&plan).unwrap();
        let predicted = predict_costs(&plan).unwrap();

        // Every detected resource must have prediction
        for resource in detected {
            prop_assert!(
                predicted.iter().any(|p| p.resource_id == resource.id),
                "Resource {} detected but not predicted",
                resource.id
            );
        }
    }

    #[test]
    fn test_prediction_intervals_ordered_fuzz(
        resource in arb_resource()
    ) {
        let prediction = predict_cost(&resource).unwrap();

        prop_assert!(prediction.p10 <= prediction.p50);
        prop_assert!(prediction.p50 <= prediction.p90);
        prop_assert!(prediction.p90 <= prediction.p99);
    }
}
```

---

## Breaking This Contract

**Severity: CRITICAL (internal logic bugs)**

**Forbidden:**
- ❌ Detection finds resource, prediction doesn't
- ❌ High confidence with no evidence
- ❌ Severity doesn't match cost delta
- ❌ Prediction intervals out of order (p90 < p50)
- ❌ Cost deltas don't sum correctly
- ❌ Stale heuristics with high confidence

**Required:**
- ✅ All engines agree on same data
- ✅ Confidence matches evidence quality
- ✅ Severity correlates with impact
- ✅ Math is internally consistent
- ✅ Schemas validated across engines

---

## Benefits

### Bug Detection
- **Catches logic errors** - Engines disagree = bug
- **Early detection** - Fails in tests, not production
- **Clear failures** - Easy to pinpoint inconsistency
- **Prevents regressions** - Tests lock in consistency

### Code Quality
- **Forces alignment** - Engineers think about cross-engine logic
- **Documentation** - Tests show expected relationships
- **Refactoring safety** - Can't break consistency accidentally
- **Architecture validation** - Ensures clean separation

### User Trust
- **Correct outputs** - No conflicting information
- **Reliable** - Consistent behavior
- **Debuggable** - Easy to trace issues
- **Professional** - No obvious bugs

---

## Version History

- **1.0.0** (2025-12-06) - Initial self-consistency tests contract

---

**This contract ensures CostPilot engines are internally consistent and logically coherent.**
