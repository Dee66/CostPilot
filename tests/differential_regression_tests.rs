use costpilot::engines::detection::DetectionEngine;
use costpilot::engines::explain::PredictionExplainer;
use costpilot::engines::prediction::PredictionEngine;
use costpilot::engines::shared::models::{ChangeAction, ResourceChange};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod differential_regression_tests {
    use super::*;

    fn create_test_resource_change() -> ResourceChange {
        ResourceChange {
            resource_id: "test-aws-instance-regression-1".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Update,
            module_path: None,
            old_config: Some(serde_json::json!({
                "instance_type": "t2.micro",
                "ami": "ami-12345"
            })),
            new_config: Some(serde_json::json!({
                "instance_type": "t3.medium",
                "ami": "ami-67890"
            })),
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        }
    }

    fn get_snapshot_path(test_name: &str) -> String {
        format!("tests/snapshots/{}.json", test_name)
    }

    fn load_snapshot(test_name: &str) -> Option<Value> {
        let path = get_snapshot_path(test_name);
        if Path::new(&path).exists() {
            let content = fs::read_to_string(&path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    fn save_snapshot(test_name: &str, data: &Value) {
        let path = get_snapshot_path(test_name);
        let dir = Path::new(&path).parent().unwrap();
        fs::create_dir_all(dir).unwrap();
        let json = serde_json::to_string_pretty(data).unwrap();
        fs::write(&path, json).unwrap();
    }

    #[test]
    fn test_detect_output_regression_protection() {
        let engine = DetectionEngine::new();
        let change = create_test_resource_change();

        let detections = engine.detect(&[change]).unwrap();
        let current_output: Value = serde_json::to_value(&detections).unwrap();

        let test_name = "detect_output_regression";
        if let Some(previous_output) = load_snapshot(test_name) {
            // Compare outputs - in a real regression test, you'd want more sophisticated
            // comparison that allows for expected changes
            assert_eq!(
                current_output, previous_output,
                "Detect output has changed! This indicates a potential regression. \
                 If this change is expected, update the snapshot by deleting the snapshot file \
                 and re-running the test."
            );
        } else {
            // First run - save snapshot
            save_snapshot(test_name, &current_output);
        }
    }

    #[test]
    fn test_predict_output_regression_protection() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = create_test_resource_change();

        let estimates = engine.predict(&[change]).unwrap();
        let current_output: Value = serde_json::to_value(&estimates).unwrap();

        let test_name = "predict_output_regression";
        if let Some(previous_output) = load_snapshot(test_name) {
            assert_eq!(
                current_output, previous_output,
                "Predict output has changed! This indicates a potential regression. \
                 If this change is expected, update the snapshot by deleting the snapshot file \
                 and re-running the test."
            );
        } else {
            save_snapshot(test_name, &current_output);
        }
    }

    #[test]
    fn test_explain_output_regression_protection() {
        let mut prediction_engine = PredictionEngine::new().unwrap();
        let heuristics = prediction_engine.heuristics().clone();
        let explain_engine = PredictionExplainer::new(&heuristics);

        let change = create_test_resource_change();
        let estimates = prediction_engine
            .predict(std::slice::from_ref(&change))
            .unwrap();

        let explanations = explain_engine.explain(&change, &estimates[0]);
        let current_output: Value = serde_json::to_value(&explanations).unwrap();

        let test_name = "explain_output_regression";
        if let Some(previous_output) = load_snapshot(test_name) {
            assert_eq!(
                current_output, previous_output,
                "Explain output has changed! This indicates a potential regression. \
                 If this change is expected, update the snapshot by deleting the snapshot file \
                 and re-running the test."
            );
        } else {
            save_snapshot(test_name, &current_output);
        }
    }

    #[test]
    fn test_output_schema_stability() {
        // Test that outputs maintain expected structure/schema
        let engine = DetectionEngine::new();
        let change = create_test_resource_change();

        let detections = engine.detect(&[change]).unwrap();

        // Verify that all detections have required fields
        for detection in &detections {
            assert!(
                !detection.resource_id.is_empty(),
                "Detection missing resource_id"
            );
            assert!(!detection.message.is_empty(), "Detection missing message");
            assert!(
                detection.severity_score <= 100,
                "Detection severity_score out of valid range: {}",
                detection.severity_score
            );
            // Severity is an enum, so we just check it's one of the valid values
            // The enum itself ensures validity
        }
    }

    #[test]
    fn test_no_silent_drift_in_cost_calculations() {
        let mut engine = PredictionEngine::new().unwrap();
        let change = create_test_resource_change();

        // Run prediction multiple times to ensure consistency
        let estimates1 = engine.predict(std::slice::from_ref(&change)).unwrap();
        let estimates2 = engine.predict(std::slice::from_ref(&change)).unwrap();

        assert_eq!(
            estimates1.len(),
            estimates2.len(),
            "Number of estimates changed between runs"
        );

        for (e1, e2) in estimates1.iter().zip(estimates2.iter()) {
            assert_eq!(
                e1.resource_id, e2.resource_id,
                "Resource ID changed between runs for {}",
                e1.resource_id
            );
            assert_eq!(
                e1.monthly_cost, e2.monthly_cost,
                "Monthly cost changed between runs for {}: {} vs {}",
                e1.resource_id, e1.monthly_cost, e2.monthly_cost
            );
            assert_eq!(
                e1.confidence_score, e2.confidence_score,
                "Confidence score changed between runs for {}",
                e1.resource_id
            );
        }
    }

    #[test]
    fn test_boundary_case_regression_protection() {
        // Test edge cases that might be prone to regression
        let mut engine = PredictionEngine::new().unwrap();

        let boundary_cases = vec![
            ResourceChange {
                resource_id: "boundary-zero-cost".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({"instance_type": "t2.nano"})),
                tags: HashMap::new(),
                monthly_cost: Some(0.0), // Explicitly zero
                config: None,
                cost_impact: None,
            },
            ResourceChange {
                resource_id: "boundary-max-cost".to_string(),
                resource_type: "aws_instance".to_string(),
                action: ChangeAction::Create,
                module_path: None,
                old_config: None,
                new_config: Some(serde_json::json!({"instance_type": "m5.24xlarge"})),
                tags: HashMap::new(),
                monthly_cost: None,
                config: None,
                cost_impact: None,
            },
        ];

        for change in boundary_cases {
            let estimates = engine.predict(&[change]).unwrap();
            assert!(
                !estimates.is_empty(),
                "No estimates returned for boundary case"
            );

            let estimate = &estimates[0];
            assert!(
                estimate.monthly_cost >= 0.0,
                "Negative cost for boundary case: {}",
                estimate.monthly_cost
            );
            assert!(
                estimate.confidence_score >= 0.0 && estimate.confidence_score <= 1.0,
                "Invalid confidence for boundary case: {}",
                estimate.confidence_score
            );
        }
    }
}
