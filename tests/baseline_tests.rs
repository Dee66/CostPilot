// Baseline behavior tests

use costpilot::engines::baseline::BaselineManager;
use costpilot::engines::shared::models::CostSnapshot;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_baseline_file_missing_returns_none() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("nonexistent.json");
    
    let manager = BaselineManager::new(baseline_path.clone());
    let result = manager.load_baseline();
    
    assert!(result.is_none());
}

#[test]
fn test_baseline_file_missing_allows_operation() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("nonexistent.json");
    
    let manager = BaselineManager::new(baseline_path);
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    // Should not panic or error when baseline is missing
    let diff = manager.compare_to_baseline(&snapshot);
    assert!(diff.is_ok());
}

#[test]
fn test_baseline_missing_uses_current_as_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let manager = BaselineManager::new(baseline_path);
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let diff = manager.compare_to_baseline(&snapshot).unwrap();
    
    // When baseline is missing, diff should be zero or treat current as baseline
    assert_eq!(diff.cost_change, 0.0);
}

#[test]
fn test_baseline_missing_creates_on_save() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let manager = BaselineManager::new(baseline_path.clone());
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    manager.save_baseline(&snapshot).unwrap();
    
    assert!(baseline_path.exists());
}

#[test]
fn test_baseline_missing_no_regression_detected() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let manager = BaselineManager::new(baseline_path);
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 2000.0,
        resource_count: 20,
        resources: vec![],
    };
    
    let regressions = manager.detect_regressions(&snapshot);
    
    // No regressions when baseline is missing
    assert!(regressions.is_empty());
}

#[test]
fn test_baseline_override_with_explicit_path() {
    let temp_dir = TempDir::new().unwrap();
    let default_path = temp_dir.path().join("baseline.json");
    let override_path = temp_dir.path().join("override.json");
    
    let manager = BaselineManager::new(default_path);
    
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    manager.save_baseline_to(&snapshot, &override_path).unwrap();
    
    assert!(override_path.exists());
}

#[test]
fn test_baseline_override_loads_from_explicit_path() {
    let temp_dir = TempDir::new().unwrap();
    let override_path = temp_dir.path().join("override.json");
    
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1500.0,
        resource_count: 15,
        resources: vec![],
    };
    
    let manager = BaselineManager::new(override_path.clone());
    manager.save_baseline(&snapshot).unwrap();
    
    let loaded = manager.load_baseline().unwrap();
    assert_eq!(loaded.total_monthly_cost, 1500.0);
}

#[test]
fn test_baseline_override_via_env_var() {
    let temp_dir = TempDir::new().unwrap();
    let default_path = temp_dir.path().join("baseline.json");
    let override_path = temp_dir.path().join("env_override.json");
    
    std::env::set_var("COSTPILOT_BASELINE", override_path.to_str().unwrap());
    
    let manager = BaselineManager::from_env(default_path);
    let snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 2000.0,
        resource_count: 20,
        resources: vec![],
    };
    
    manager.save_baseline(&snapshot).unwrap();
    
    assert!(override_path.exists());
    
    std::env::remove_var("COSTPILOT_BASELINE");
}

#[test]
fn test_baseline_override_precedence() {
    let temp_dir = TempDir::new().unwrap();
    let default_path = temp_dir.path().join("baseline.json");
    let override_path = temp_dir.path().join("override.json");
    
    // Save different snapshots to each
    let default_snapshot = CostSnapshot {
        timestamp: "2025-12-07T09:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let override_snapshot = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1500.0,
        resource_count: 15,
        resources: vec![],
    };
    
    let default_manager = BaselineManager::new(default_path.clone());
    default_manager.save_baseline(&default_snapshot).unwrap();
    
    let override_manager = BaselineManager::new(override_path.clone());
    override_manager.save_baseline(&override_snapshot).unwrap();
    
    // Load from override path - should get override snapshot
    let loaded = override_manager.load_baseline().unwrap();
    assert_eq!(loaded.total_monthly_cost, 1500.0);
}

#[test]
fn test_regression_classifier_uses_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let baseline = CostSnapshot {
        timestamp: "2025-12-01T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let manager = BaselineManager::new(baseline_path);
    manager.save_baseline(&baseline).unwrap();
    
    let current = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1500.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let regressions = manager.detect_regressions(&current);
    
    assert!(!regressions.is_empty());
    assert!(regressions.iter().any(|r| r.regression_type == "cost_spike"));
}

#[test]
fn test_regression_classifier_cost_spike_detection() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let baseline = CostSnapshot {
        timestamp: "2025-12-01T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let manager = BaselineManager::new(baseline_path);
    manager.save_baseline(&baseline).unwrap();
    
    let current = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 2000.0, // 100% increase
        resource_count: 10,
        resources: vec![],
    };
    
    let regressions = manager.detect_regressions(&current);
    
    let cost_spike = regressions.iter().find(|r| r.regression_type == "cost_spike");
    assert!(cost_spike.is_some());
    assert!(cost_spike.unwrap().severity >= 7.0);
}

#[test]
fn test_slo_uses_baseline_for_budget() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let baseline = CostSnapshot {
        timestamp: "2025-12-01T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let manager = BaselineManager::new(baseline_path);
    manager.save_baseline(&baseline).unwrap();
    
    let slo_config = manager.generate_slo_config(1.2); // 20% budget buffer
    
    assert_eq!(slo_config.budget_limit, 1200.0);
}

#[test]
fn test_slo_baseline_drift_detection() {
    let temp_dir = TempDir::new().unwrap();
    let baseline_path = temp_dir.path().join("baseline.json");
    
    let baseline = CostSnapshot {
        timestamp: "2025-12-01T10:00:00Z".to_string(),
        total_monthly_cost: 1000.0,
        resource_count: 10,
        resources: vec![],
    };
    
    let manager = BaselineManager::new(baseline_path);
    manager.save_baseline(&baseline).unwrap();
    
    let current = CostSnapshot {
        timestamp: "2025-12-07T10:00:00Z".to_string(),
        total_monthly_cost: 1300.0, // 30% increase
        resource_count: 10,
        resources: vec![],
    };
    
    let drift = manager.calculate_drift(&current);
    
    assert_eq!(drift.percentage, 30.0);
    assert_eq!(drift.absolute, 300.0);
}
