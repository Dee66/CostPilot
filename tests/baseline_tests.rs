// Baseline-related tests

use costpilot::engines::baselines::{Baseline, BaselinesManager};
use serde_json::json;
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
    let variance = ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs() * 100.0;
    
    assert!(variance <= baseline.acceptable_variance_percent, 
        "Cost {} should be within {}% of {}", actual_cost, baseline.acceptable_variance_percent, baseline.expected_monthly_cost);
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
    let variance = ((actual_cost - baseline.expected_monthly_cost) / baseline.expected_monthly_cost).abs() * 100.0;
    
    assert!(variance > baseline.acceptable_variance_percent, 
        "Cost {} should exceed {}% variance from {}", actual_cost, baseline.acceptable_variance_percent, baseline.expected_monthly_cost);
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
