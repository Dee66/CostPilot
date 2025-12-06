// Shared data models for CostPilot

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource change detected in IaC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    pub resource_id: String,
    pub resource_type: String,
    pub action: ChangeAction,
    pub module_path: Option<String>,
    pub old_config: Option<serde_json::Value>,
    pub new_config: Option<serde_json::Value>,
    pub tags: HashMap<String, String>,
}

/// Type of change action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeAction {
    Create,
    Update,
    Delete,
    Replace,
    NoOp,
}

/// Cost estimate for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub resource_id: String,
    pub monthly_cost: f64,
    pub prediction_interval_low: f64,
    pub prediction_interval_high: f64,
    pub confidence_score: f64,
    pub heuristic_reference: Option<String>,
    pub cold_start_inference: bool,
}

/// Regression classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegressionType {
    Configuration,
    Scaling,
    Provisioning,
    TrafficInferred,
    IndirectCost,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub rule_id: String,
    pub severity: Severity,
    pub resource_id: String,
    pub regression_type: RegressionType,
    pub severity_score: u32,
    pub message: String,
}

/// Scan result containing all analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub resource_changes: Vec<ResourceChange>,
    pub cost_estimates: Vec<CostEstimate>,
    pub detections: Vec<Detection>,
    pub total_monthly_delta: f64,
    pub metadata: ScanMetadata,
}

/// Metadata about the scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub timestamp: String,
    pub heuristics_version: String,
    pub policy_version: Option<String>,
    pub deterministic: bool,
}
