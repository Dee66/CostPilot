// Shared data models for CostPilot

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost impact details for a resource change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostImpact {
    pub delta: f64,
    pub confidence: f64,
    pub heuristic_source: Option<String>,
}

/// Resource change detected in IaC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChange {
    pub resource_id: String,
    pub resource_type: String,
    pub action: ChangeAction,
    #[serde(default)]
    pub module_path: Option<String>,
    #[serde(default)]
    pub old_config: Option<serde_json::Value>,
    #[serde(default)]
    pub new_config: Option<serde_json::Value>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    /// Optional cost estimate (populated when prediction runs)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub monthly_cost: Option<f64>,
    /// Optional config reference (deprecated, use old_config/new_config)
    #[serde(skip, default)]
    pub config: Option<serde_json::Value>,
    /// Optional cost impact details
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cost_impact: Option<CostImpact>,
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
    #[serde(default)]
    pub monthly_cost: f64,
    #[serde(default)]
    pub prediction_interval_low: f64,
    #[serde(default)]
    pub prediction_interval_high: f64,
    #[serde(default)]
    pub confidence_score: f64,
    #[serde(default)]
    pub heuristic_reference: Option<String>,
    #[serde(default)]
    pub cold_start_inference: bool,
    #[serde(default)]
    pub one_time: Option<f64>,
    #[serde(default)]
    pub breakdown: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub hourly: Option<f64>,
    #[serde(default)]
    pub daily: Option<f64>,
}

/// Total cost summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalCost {
    pub monthly: f64,
    pub prediction_interval_low: f64,
    pub prediction_interval_high: f64,
    pub confidence_score: f64,
    pub resource_count: usize,
}

/// Regression classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RegressionType {
    #[default]
    Configuration,
    Scaling,
    Provisioning,
    TrafficInferred,
    IndirectCost,
    #[serde(alias = "traffic")]
    Traffic,
    #[serde(alias = "indirect")]
    Indirect,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// Detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub rule_id: String,
    #[serde(default)]
    pub severity: Severity,
    pub resource_id: String,
    #[serde(default)]
    pub regression_type: RegressionType,
    #[serde(default)]
    pub severity_score: u32,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub fix_snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub estimated_cost: Option<f64>,
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
    pub timestamp: Option<String>,
    pub heuristics_version: String,
    pub policy_version: Option<String>,
    pub deterministic: bool,
}

// Builder pattern for test compatibility
#[allow(dead_code)]
pub struct CostEstimateBuilder {
    resource_id: Option<String>,
    monthly_cost: Option<f64>,
    prediction_interval_low: Option<f64>,
    prediction_interval_high: Option<f64>,
    confidence_score: Option<f64>,
    heuristic_reference: Option<String>,
    cold_start_inference: bool,
}

impl CostEstimateBuilder {
    pub fn new() -> Self {
        Self {
            resource_id: None,
            monthly_cost: None,
            prediction_interval_low: None,
            prediction_interval_high: None,
            confidence_score: None,
            heuristic_reference: None,
            cold_start_inference: false,
        }
    }

    pub fn resource_id(mut self, val: impl Into<String>) -> Self {
        self.resource_id = Some(val.into());
        self
    }

    pub fn monthly_cost(mut self, val: f64) -> Self {
        self.monthly_cost = Some(val);
        self
    }

    pub fn prediction_interval_low(mut self, val: f64) -> Self {
        self.prediction_interval_low = Some(val);
        self
    }

    pub fn prediction_interval_high(mut self, val: f64) -> Self {
        self.prediction_interval_high = Some(val);
        self
    }

    pub fn confidence_score(mut self, val: f64) -> Self {
        self.confidence_score = Some(val);
        self
    }

    pub fn heuristic_reference(mut self, val: impl Into<String>) -> Self {
        self.heuristic_reference = Some(val.into());
        self
    }

    pub fn cold_start_inference(mut self, val: bool) -> Self {
        self.cold_start_inference = val;
        self
    }

    pub fn build(self) -> CostEstimate {
        // Priority: explicit canonical > defaults
        let monthly_cost = self.monthly_cost.unwrap_or(0.0);

        let prediction_interval_low = self.prediction_interval_low.unwrap_or(monthly_cost * 0.8);

        let prediction_interval_high = self.prediction_interval_high.unwrap_or(monthly_cost * 1.2);

        let confidence_score = self.confidence_score.unwrap_or(0.85);

        CostEstimate {
            resource_id: self.resource_id.unwrap_or_else(|| "unknown".to_string()),
            monthly_cost,
            prediction_interval_low,
            prediction_interval_high,
            confidence_score,
            heuristic_reference: self.heuristic_reference,
            cold_start_inference: self.cold_start_inference,
            one_time: None,
            breakdown: None,
            hourly: None,
            daily: None,
        }
    }
}

impl CostEstimate {
    /// Create new cost estimate (canonical API)
    pub fn new(resource_id: String, monthly_cost: f64) -> Self {
        Self {
            resource_id,
            monthly_cost,
            prediction_interval_low: monthly_cost * 0.8,
            prediction_interval_high: monthly_cost * 1.2,
            confidence_score: 0.85,
            heuristic_reference: None,
            cold_start_inference: false,
            one_time: None,
            breakdown: None,
            hourly: None,
            daily: None,
        }
    }

    /// Builder pattern entry point
    pub fn builder() -> CostEstimateBuilder {
        CostEstimateBuilder::new()
    }
}

impl Default for CostEstimateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CostEstimate {
    fn default() -> Self {
        Self {
            resource_id: "unknown".to_string(),
            monthly_cost: 0.0,
            prediction_interval_low: 0.0,
            prediction_interval_high: 0.0,
            confidence_score: 0.0,
            heuristic_reference: None,
            cold_start_inference: false,
            one_time: None,
            breakdown: None,
            hourly: None,
            daily: None,
        }
    }
}

#[allow(dead_code)]
pub struct ResourceChangeBuilder {
    resource_id: Option<String>,
    resource_type: Option<String>,
    action: Option<ChangeAction>,
    module_path: Option<String>,
    old_config: Option<serde_json::Value>,
    new_config: Option<serde_json::Value>,
    tags: HashMap<String, String>,
    monthly_cost: Option<f64>,
    cost_impact: Option<CostImpact>,
}

impl ResourceChangeBuilder {
    pub fn new() -> Self {
        Self {
            resource_id: None,
            resource_type: None,
            action: None,
            module_path: None,
            old_config: None,
            new_config: None,
            tags: HashMap::new(),
            monthly_cost: None,
            cost_impact: None,
        }
    }

    pub fn resource_id(mut self, val: impl Into<String>) -> Self {
        self.resource_id = Some(val.into());
        self
    }

    pub fn resource_type(mut self, val: impl Into<String>) -> Self {
        self.resource_type = Some(val.into());
        self
    }

    pub fn action(mut self, val: ChangeAction) -> Self {
        self.action = Some(val);
        self
    }

    pub fn module_path(mut self, val: impl Into<String>) -> Self {
        self.module_path = Some(val.into());
        self
    }

    pub fn old_config(mut self, val: serde_json::Value) -> Self {
        self.old_config = Some(val);
        self
    }

    pub fn new_config(mut self, val: serde_json::Value) -> Self {
        self.new_config = Some(val);
        self
    }

    pub fn tags(mut self, val: HashMap<String, String>) -> Self {
        self.tags = val;
        self
    }

    pub fn monthly_cost(mut self, val: f64) -> Self {
        self.monthly_cost = Some(val);
        self
    }

    pub fn cost_impact(mut self, val: CostImpact) -> Self {
        self.cost_impact = Some(val);
        self
    }

    pub fn build(self) -> ResourceChange {
        ResourceChange {
            resource_id: self.resource_id.unwrap_or_else(|| "unknown".to_string()),
            resource_type: self.resource_type.unwrap_or_else(|| "unknown".to_string()),
            action: self.action.unwrap_or(ChangeAction::NoOp),
            module_path: self.module_path,
            old_config: self.old_config,
            new_config: self.new_config,
            tags: self.tags,
            monthly_cost: self.monthly_cost,
            config: None,
            cost_impact: self.cost_impact,
        }
    }
}

impl ResourceChange {
    /// Create new resource change (canonical API)
    pub fn new(resource_id: String, resource_type: String, action: ChangeAction) -> Self {
        Self {
            resource_id,
            resource_type,
            action,
            module_path: None,
            old_config: None,
            new_config: None,
            tags: HashMap::new(),
            monthly_cost: None,
            config: None,
            cost_impact: None,
        }
    }

    /// Builder pattern entry point
    pub fn builder() -> ResourceChangeBuilder {
        ResourceChangeBuilder::new()
    }
}

impl Default for ResourceChangeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct DetectionBuilder {
    rule_id: Option<String>,
    severity: Option<Severity>,
    resource_id: Option<String>,
    regression_type: Option<RegressionType>,
    severity_score: Option<u32>,
    message: Option<String>,
    fix_snippet: Option<String>,
    estimated_cost: Option<f64>,
}

impl DetectionBuilder {
    pub fn new() -> Self {
        Self {
            rule_id: None,
            severity: None,
            resource_id: None,
            regression_type: None,
            severity_score: None,
            message: None,
            fix_snippet: None,
            estimated_cost: None,
        }
    }

    pub fn rule_id(mut self, val: impl Into<String>) -> Self {
        self.rule_id = Some(val.into());
        self
    }

    pub fn severity(mut self, val: Severity) -> Self {
        self.severity = Some(val);
        self
    }

    pub fn resource_id(mut self, val: impl Into<String>) -> Self {
        self.resource_id = Some(val.into());
        self
    }

    pub fn regression_type(mut self, val: RegressionType) -> Self {
        self.regression_type = Some(val);
        self
    }

    pub fn severity_score(mut self, val: u32) -> Self {
        self.severity_score = Some(val);
        self
    }

    pub fn message(mut self, val: impl Into<String>) -> Self {
        self.message = Some(val.into());
        self
    }

    pub fn fix_snippet(mut self, val: impl Into<String>) -> Self {
        self.fix_snippet = Some(val.into());
        self
    }

    pub fn estimated_cost(mut self, val: f64) -> Self {
        self.estimated_cost = Some(val);
        self
    }

    pub fn build(self) -> Detection {
        let severity = self.severity.unwrap_or(Severity::Low);
        let message = self.message.unwrap_or_default();
        let estimated_cost = self.estimated_cost;

        let severity_score = self.severity_score.unwrap_or(match severity {
            Severity::Critical => 90,
            Severity::High => 70,
            Severity::Medium => 50,
            Severity::Low => 30,
        });

        Detection {
            rule_id: self.rule_id.unwrap_or_else(|| "unknown".to_string()),
            severity,
            resource_id: self.resource_id.unwrap_or_else(|| "unknown".to_string()),
            regression_type: self
                .regression_type
                .unwrap_or(RegressionType::Configuration),
            severity_score,
            message,
            fix_snippet: self.fix_snippet,
            estimated_cost,
        }
    }
}

impl Detection {
    /// Create new detection (canonical API)
    pub fn new(rule_id: String, severity: Severity, resource_id: String, message: String) -> Self {
        let severity_score = match severity {
            Severity::Critical => 90,
            Severity::High => 70,
            Severity::Medium => 50,
            Severity::Low => 30,
        };

        Self {
            rule_id,
            severity,
            resource_id,
            regression_type: RegressionType::Configuration,
            severity_score,
            message,
            fix_snippet: None,
            estimated_cost: None,
        }
    }

    /// Builder pattern entry point
    pub fn builder() -> DetectionBuilder {
        DetectionBuilder::new()
    }
}

impl Default for DetectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct ScanResultBuilder {
    resource_changes: Vec<ResourceChange>,
    cost_estimates: Vec<CostEstimate>,
    detections: Vec<Detection>,
    total_monthly_delta: f64,
    metadata: Option<ScanMetadata>,
}

impl ScanResultBuilder {
    pub fn new() -> Self {
        Self {
            resource_changes: Vec::new(),
            cost_estimates: Vec::new(),
            detections: Vec::new(),
            total_monthly_delta: 0.0,
            metadata: None,
        }
    }

    pub fn resource_changes(mut self, val: Vec<ResourceChange>) -> Self {
        self.resource_changes = val;
        self
    }

    pub fn cost_estimates(mut self, val: Vec<CostEstimate>) -> Self {
        self.cost_estimates = val;
        self
    }

    pub fn detections(mut self, val: Vec<Detection>) -> Self {
        self.detections = val;
        self
    }

    pub fn total_monthly_delta(mut self, val: f64) -> Self {
        self.total_monthly_delta = val;
        self
    }

    pub fn metadata(mut self, val: ScanMetadata) -> Self {
        self.metadata = Some(val);
        self
    }

    pub fn build(self) -> ScanResult {
        let metadata = self.metadata.unwrap_or(ScanMetadata {
            timestamp: None,
            heuristics_version: "1.0.0".to_string(),
            policy_version: None,
            deterministic: true,
        });

        ScanResult {
            resource_changes: self.resource_changes,
            cost_estimates: self.cost_estimates,
            detections: self.detections,
            total_monthly_delta: self.total_monthly_delta,
            metadata,
        }
    }
}

impl Default for ScanResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanResult {
    /// Builder pattern entry point
    pub fn builder() -> ScanResultBuilder {
        ScanResultBuilder::new()
    }
}
