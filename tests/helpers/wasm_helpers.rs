// Test helpers for WASM-related tests

use costpilot::engines::shared::models::{CostEstimate, ResourceChange, ChangeAction};
use serde_json::json;

/// Create a test CostEstimate with canonical fields
pub fn make_test_cost_estimate(monthly_cost: f64) -> CostEstimate {
    CostEstimate::builder()
        .resource_id("test_resource".to_string())
        .monthly_cost(monthly_cost)
        .prediction_interval_low(monthly_cost * 0.9)
        .prediction_interval_high(monthly_cost * 1.1)
        .confidence_score(0.95)
        .heuristic_reference("test".to_string())
        .cold_start_inference(false)
        .build()
}

/// Create a test ResourceChange with canonical fields
pub fn make_test_resource_change(resource_type: &str, monthly_cost: Option<f64>) -> ResourceChange {
    let mut builder = ResourceChange::builder()
        .resource_id(format!("test_{}", resource_type))
        .action(ChangeAction::Create)
        .new_config(json!({"type": resource_type}));
    
    if let Some(cost) = monthly_cost {
        builder = builder.monthly_cost(cost);
    }
    
    builder.build()
}

/// Get sample WASM bytes for testing (minimal valid WASM header)
pub fn wasm_input_bytes(_name: &str) -> Vec<u8> {
    // Minimal valid WASM module (magic number + version)
    vec![
        0x00, 0x61, 0x73, 0x6D, // Magic number: \0asm
        0x01, 0x00, 0x00, 0x00, // Version: 1
    ]
}
