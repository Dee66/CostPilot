// Policy fuzz tests

#[cfg(test)]
mod policy_fuzz_tests {
    use proptest::prelude::*;
    use costpilot::engines::policy::{PolicyEngine, Policy, PolicyRule};
    use costpilot::engines::shared::models::{Detection, Severity, ChangeAction};
    use serde_json::json;

    fn arb_policy_rule() -> impl Strategy<Value = PolicyRule> {
        (
            any::<String>(),
            any::<String>(),
            prop::option::of(0.0f64..10000.0f64),
            prop::option::of(any::<String>()),
        ).prop_map(|(id, resource_type, max_cost, severity)| {
            PolicyRule {
                rule_id: id,
                resource_type,
                max_monthly_cost: max_cost,
                required_tags: vec![],
                severity: severity.unwrap_or_else(|| "medium".to_string()),
                message: "Policy violation".to_string(),
            }
        })
    }

    fn arb_policy() -> impl Strategy<Value = Policy> {
        prop::collection::vec(arb_policy_rule(), 0..20).prop_map(|rules| {
            Policy {
                version: "1.0".to_string(),
                rules,
            }
        })
    }

    fn arb_detection() -> impl Strategy<Value = Detection> {
        (
            any::<String>(),
            any::<String>(),
            any::<String>(),
            0.0f64..10.0f64,
            0.0f64..1.0f64,
        ).prop_map(|(id, resource_type, issue, severity, confidence)| {
            let sev = if severity < 3.0 {
                Severity::Low
            } else if severity < 6.0 {
                Severity::Medium
            } else {
                Severity::High
            };
            Detection::builder()
                .resource_id(id)
                .rule_id("test_rule")
                .message(issue)
                .severity(sev)
                .estimated_cost(100.0)
                .build()
        })
    }

    proptest! {
        #[test]
        fn fuzz_policy_evaluate_never_panics(
            policy in arb_policy(),
            detections in prop::collection::vec(arb_detection(), 0..50)
        ) {
            let engine = PolicyEngine::new(policy);
            let _ = engine.evaluate(&detections);
        }

        #[test]
        fn fuzz_policy_validation_never_panics(
            policy in arb_policy()
        ) {
            let engine = PolicyEngine::new(policy);
            let _ = engine.validate();
        }

        #[test]
        fn fuzz_policy_rule_matching(
            policy in arb_policy(),
            resource_type in any::<String>(),
            cost in 0.0f64..100000.0f64
        ) {
            let engine = PolicyEngine::new(policy);
            let _ = engine.check_resource(&resource_type, cost);
        }

        #[test]
        fn fuzz_policy_deterministic(
            policy in arb_policy(),
            detections in prop::collection::vec(arb_detection(), 1..10)
        ) {
            let engine = PolicyEngine::new(policy.clone());
            let result1 = engine.evaluate(&detections);
            let result2 = engine.evaluate(&detections);

            prop_assert_eq!(result1.len(), result2.len());
        }

        #[test]
        fn fuzz_policy_empty_rules(
            detections in prop::collection::vec(arb_detection(), 0..20)
        ) {
            let policy = Policy {
                version: "1.0".to_string(),
                rules: vec![],
            };
            let engine = PolicyEngine::new(policy);
            let violations = engine.evaluate(&detections);
            prop_assert_eq!(violations.len(), 0);
        }

        #[test]
        fn fuzz_policy_extreme_costs(
            max_cost in prop::num::f64::ANY,
            actual_cost in prop::num::f64::ANY
        ) {
            let rule = PolicyRule {
                rule_id: "test".to_string(),
                resource_type: "aws_instance".to_string(),
                max_monthly_cost: Some(max_cost.abs()),
                required_tags: vec![],
                severity: "high".to_string(),
                message: "Test".to_string(),
            };

            let policy = Policy {
                version: "1.0".to_string(),
                rules: vec![rule],
            };

            let engine = PolicyEngine::new(policy);
            let _ = engine.check_resource("aws_instance", actual_cost.abs());
        }

        #[test]
        fn fuzz_policy_unicode_resource_types(
            resource_type in "\\PC{1,100}"
        ) {
            let rule = PolicyRule {
                rule_id: "test".to_string(),
                resource_type: resource_type.clone(),
                max_monthly_cost: Some(100.0),
                required_tags: vec![],
                severity: "high".to_string(),
                message: "Test".to_string(),
            };

            let policy = Policy {
                version: "1.0".to_string(),
                rules: vec![rule],
            };

            let engine = PolicyEngine::new(policy);
            let _ = engine.check_resource(&resource_type, 50.0);
        }
    }
}
