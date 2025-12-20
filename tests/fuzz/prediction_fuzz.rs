// Prediction fuzz tests

#[cfg(test)]
mod prediction_fuzz_tests {
    use proptest::prelude::*;
    use costpilot::engines::prediction::PredictionEngine;
    use costpilot::engines::shared::models::{ResourceChange, Severity, ChangeAction};
    use serde_json::json;

    // Generate arbitrary resource changes for fuzzing
    fn arb_resource_change() -> impl Strategy<Value = ResourceChange> {
        (
            any::<String>(),
            prop::option::of(any::<String>()),
            prop::option::of(any::<String>()),
            prop::option::of(any::<f64>()),
        ).prop_map(|(id, old_type, new_type, monthly_cost)| {
            ResourceChange::builder()
                .resource_id(id)
                .resource_type(new_type.unwrap_or_else(|| "aws_instance".to_string()))
                .action(ChangeAction::Create)
                .old_config(old_type.map(|t| json!({"type": t})).unwrap_or(json!(null)))
                .new_config(json!({"type": "t3.medium"}))
                .monthly_cost(monthly_cost.unwrap_or(0.0).abs())
                .build()
        })
    }

    proptest! {
        #[test]
        fn fuzz_prediction_never_panics(
            resource in arb_resource_change()
        ) {
            let engine = PredictionEngine::new();
            let _ = engine.predict(&resource);
        }

        #[test]
        fn fuzz_prediction_cost_non_negative(
            resource in arb_resource_change()
        ) {
            let engine = PredictionEngine::new();
            if let Ok(prediction) = engine.predict(&resource) {
                prop_assert!(prediction.predicted_cost >= 0.0);
                prop_assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
            }
        }

        #[test]
        fn fuzz_prediction_confidence_bounds(
            resource in arb_resource_change()
        ) {
            let engine = PredictionEngine::new();
            if let Ok(prediction) = engine.predict(&resource) {
                prop_assert!(prediction.confidence >= 0.0);
                prop_assert!(prediction.confidence <= 1.0);
            }
        }

        #[test]
        fn fuzz_prediction_deterministic(
            resource in arb_resource_change()
        ) {
            let engine = PredictionEngine::new();
            let result1 = engine.predict(&resource);
            let result2 = engine.predict(&resource);

            match (result1, result2) {
                (Ok(p1), Ok(p2)) => {
                    prop_assert_eq!(p1.predicted_cost, p2.predicted_cost);
                    prop_assert_eq!(p1.confidence, p2.confidence);
                }
                (Err(_), Err(_)) => {},
                _ => prop_assert!(false, "Inconsistent results"),
            }
        }

        #[test]
        fn fuzz_batch_prediction_no_panic(
            resources in prop::collection::vec(arb_resource_change(), 0..100)
        ) {
            let engine = PredictionEngine::new();
            let _ = engine.predict_batch(&resources);
        }

        #[test]
        fn fuzz_prediction_extreme_costs(
            cost in prop::num::f64::ANY
        ) {
            let resource = ResourceChange::builder()
                .resource_id("test")
                .resource_type("aws_instance")
                .action(ChangeAction::Create)
                .new_config(json!({"instance_type": "t3.medium"}))
                .monthly_cost(cost.abs())
                .build();

            let engine = PredictionEngine::new();
            let _ = engine.predict(&resource);
        }

        #[test]
        fn fuzz_prediction_unicode_strings(
            id in "\\PC{1,100}",
            resource_type in "\\PC{1,50}"
        ) {
            let resource = ResourceChange::builder()
                .resource_id(id)
                .resource_type(resource_type)
                .action(ChangeAction::Create)
                .new_config(json!({}))
                .monthly_cost(10.0)
                .build();

            let engine = PredictionEngine::new();
            let _ = engine.predict(&resource);
        }
    }
}
