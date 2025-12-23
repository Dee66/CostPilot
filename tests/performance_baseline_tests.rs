// Performance baseline tests for Windows validation
//
// These tests establish basic performance baselines for CostPilot engines
// to ensure cross-platform consistency.

use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn test_prediction_engine_performance() {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();

    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();

    // Single prediction
    let start = Instant::now();
    let total_cost = prediction_engine.predict_total_cost(&changes).unwrap();
    let duration = start.elapsed();

    println!("Single EC2 prediction: {:?}", duration);
    println!("Total cost: ${:.2}", total_cost.monthly);
    // Note: Cost may be 0.0 if heuristics don't cover this resource type
    assert!(duration < Duration::from_millis(100)); // Should be fast
}

#[test]
fn test_detection_engine_performance() {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");

    let start = Instant::now();
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();
    let duration = start.elapsed();

    println!("Plan parsing: {:?}", duration);
    println!("Detected {} changes", changes.len());
    assert!(!changes.is_empty());
    assert!(duration < Duration::from_millis(50)); // Should be very fast
}

#[test]
fn test_policy_engine_performance() {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();
    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();
    let total_cost_summary = prediction_engine.predict_total_cost(&changes).unwrap();

    let total_cost = costpilot::engines::shared::models::CostEstimate {
        resource_id: "total".to_string(),
        monthly_cost: total_cost_summary.monthly,
        prediction_interval_low: total_cost_summary.prediction_interval_low,
        prediction_interval_high: total_cost_summary.prediction_interval_high,
        confidence_score: total_cost_summary.confidence_score,
        heuristic_reference: None,
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    let policy_config = costpilot::engines::policy::PolicyConfig {
        version: "1.0".to_string(),
        metadata: Default::default(),
        budgets: Default::default(),
        resources: Default::default(),
        slos: vec![],
        enforcement: costpilot::engines::policy::EnforcementConfig {
            mode: "advisory".to_string(),
            fail_on_violation: false,
        },
    };

    let edition = costpilot::edition::EditionContext::free();
    let policy_engine = costpilot::engines::policy::PolicyEngine::new(policy_config, &edition);

    let start = Instant::now();
    let result = policy_engine.evaluate(&changes, &total_cost);
    let duration = start.elapsed();

    println!("Policy evaluation: {:?}", duration);
    println!("Violations found: {}", result.violations.len());
    assert!(duration < Duration::from_millis(20)); // Should be very fast
}

#[test]
fn test_full_scan_pipeline_performance() {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");

    let start = Instant::now();

    // Detection
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();

    // Prediction
    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();
    let total_cost_summary = prediction_engine.predict_total_cost(&changes).unwrap();

    let total_cost = costpilot::engines::shared::models::CostEstimate {
        resource_id: "total".to_string(),
        monthly_cost: total_cost_summary.monthly,
        prediction_interval_low: total_cost_summary.prediction_interval_low,
        prediction_interval_high: total_cost_summary.prediction_interval_high,
        confidence_score: total_cost_summary.confidence_score,
        heuristic_reference: None,
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    // Policy evaluation
    let policy_config = costpilot::engines::policy::PolicyConfig {
        version: "1.0".to_string(),
        metadata: Default::default(),
        budgets: Default::default(),
        resources: Default::default(),
        slos: vec![],
        enforcement: costpilot::engines::policy::EnforcementConfig {
            mode: "advisory".to_string(),
            fail_on_violation: false,
        },
    };

    let edition = costpilot::edition::EditionContext::free();
    let policy_engine = costpilot::engines::policy::PolicyEngine::new(policy_config, &edition);
    let _result = policy_engine.evaluate(&changes, &total_cost);

    let duration = start.elapsed();

    println!("Full scan pipeline: {:?}", duration);
    println!("Processed {} changes", changes.len());
    assert!(duration < Duration::from_millis(200)); // Should complete within reasonable time
}
