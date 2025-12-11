use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single cost snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSnapshot {
    /// Unique identifier for this snapshot
    pub id: String,

    /// ISO 8601 timestamp when snapshot was taken
    pub timestamp: String,

    /// Git commit hash (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,

    /// Branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Total monthly cost
    pub total_monthly_cost: f64,

    /// Breakdown by module
    pub modules: HashMap<String, ModuleCost>,

    /// Breakdown by service
    pub services: HashMap<String, f64>,

    /// Detected regressions from baseline
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub regressions: Vec<Regression>,

    /// SLO violations detected
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub slo_violations: Vec<SloViolation>,

    /// Metadata about the snapshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SnapshotMetadata>,
}

/// Cost information for a specific module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCost {
    /// Module name/path
    pub name: String,

    /// Monthly cost for this module
    pub monthly_cost: f64,

    /// Resource count in module
    pub resource_count: usize,

    /// Cost change from previous snapshot (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_from_previous: Option<f64>,

    /// Percentage change from previous
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,

    /// Service-level breakdown
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<ServiceCost>,
}

/// Service-level cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCost {
    pub name: String,
    pub monthly_cost: f64,
}

/// Detected cost regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    /// Type of regression
    pub regression_type: RegressionType,

    /// Affected resource or module
    pub affected: String,

    /// Baseline cost
    pub baseline_cost: f64,

    /// Current cost
    pub current_cost: f64,

    /// Cost increase amount
    pub increase_amount: f64,

    /// Percentage increase
    pub increase_percent: f64,

    /// Severity level
    pub severity: String,
}

/// Type of cost regression detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegressionType {
    /// New resource added
    NewResource,

    /// Resource cost increased
    CostIncrease,

    /// Module budget exceeded
    BudgetExceeded,

    /// Unexpected service cost
    UnexpectedService,
}

/// SLO violation detected in snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloViolation {
    /// SLO name
    pub slo_name: String,

    /// Affected resource or module
    pub affected: String,

    /// SLO limit
    pub limit: f64,

    /// Actual value
    pub actual: f64,

    /// Severity level
    pub severity: String,
}

/// Metadata about snapshot creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Who/what triggered the snapshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_by: Option<String>,

    /// CI/CD job or run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ci_run_id: Option<String>,

    /// PR number if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u32>,

    /// Environment (dev, staging, prod)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

/// Container for historical snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendHistory {
    /// Schema version
    pub version: String,

    /// List of snapshots (ordered by timestamp)
    pub snapshots: Vec<CostSnapshot>,

    /// Configuration for trend analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<TrendConfig>,
}

/// Configuration for trend analysis and visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    /// Maximum number of snapshots to retain
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: usize,

    /// Regression threshold percentage
    #[serde(default = "default_regression_threshold")]
    pub regression_threshold_percent: f64,

    /// Enable automatic snapshot rotation
    #[serde(default = "default_true")]
    pub enable_rotation: bool,

    /// Days to retain snapshots
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

fn default_max_snapshots() -> usize {
    100
}

fn default_regression_threshold() -> f64 {
    10.0
}

fn default_true() -> bool {
    true
}

fn default_retention_days() -> u32 {
    90
}

impl Default for TrendConfig {
    fn default() -> Self {
        Self {
            max_snapshots: default_max_snapshots(),
            regression_threshold_percent: default_regression_threshold(),
            enable_rotation: default_true(),
            retention_days: default_retention_days(),
        }
    }
}

impl CostSnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new(id: String, total_monthly_cost: f64) -> Self {
        Self {
            id,
            timestamp: Utc::now().to_rfc3339(),
            commit_hash: None,
            branch: None,
            total_monthly_cost,
            modules: HashMap::new(),
            services: HashMap::new(),
            regressions: Vec::new(),
            slo_violations: Vec::new(),
            metadata: None,
        }
    }

    /// Parse timestamp to DateTime
    pub fn get_timestamp(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.timestamp).map(|dt| dt.with_timezone(&Utc))
    }

    /// Add a module cost entry
    pub fn add_module(&mut self, name: String, monthly_cost: f64, resource_count: usize) {
        self.modules.insert(
            name.clone(),
            ModuleCost {
                name,
                monthly_cost,
                resource_count,
                change_from_previous: None,
                change_percent: None,
                services: Vec::new(),
            },
        );
    }

    /// Add a service cost entry
    pub fn add_service(&mut self, service_name: String, cost: f64) {
        self.services.insert(service_name, cost);
    }

    /// Add a regression
    pub fn add_regression(&mut self, regression: Regression) {
        self.regressions.push(regression);
    }

    /// Add an SLO violation
    pub fn add_slo_violation(&mut self, violation: SloViolation) {
        self.slo_violations.push(violation);
    }
}

impl TrendHistory {
    /// Create a new trend history
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            snapshots: Vec::new(),
            config: Some(TrendConfig::default()),
        }
    }

    /// Add a snapshot to history
    pub fn add_snapshot(&mut self, snapshot: CostSnapshot) {
        self.snapshots.push(snapshot);
        self.snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    /// Get the most recent snapshot
    pub fn latest(&self) -> Option<&CostSnapshot> {
        self.snapshots.last()
    }

    /// Get snapshots within a time range
    pub fn get_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&CostSnapshot> {
        self.snapshots
            .iter()
            .filter(|s| {
                if let Ok(ts) = s.get_timestamp() {
                    ts >= start && ts <= end
                } else {
                    false
                }
            })
            .collect()
    }

    /// Calculate cost delta between two snapshots
    pub fn calculate_delta(&self, from_id: &str, to_id: &str) -> Option<f64> {
        let from = self.snapshots.iter().find(|s| s.id == from_id)?;
        let to = self.snapshots.iter().find(|s| s.id == to_id)?;
        Some(to.total_monthly_cost - from.total_monthly_cost)
    }
}

impl Default for TrendHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = CostSnapshot::new("snap-001".to_string(), 1234.56);
        assert_eq!(snapshot.id, "snap-001");
        assert_eq!(snapshot.total_monthly_cost, 1234.56);
        assert!(snapshot.get_timestamp().is_ok());
    }

    #[test]
    fn test_add_module() {
        let mut snapshot = CostSnapshot::new("snap-001".to_string(), 1000.0);
        snapshot.add_module("vpc".to_string(), 500.0, 10);

        assert_eq!(snapshot.modules.len(), 1);
        assert_eq!(snapshot.modules.get("vpc").unwrap().monthly_cost, 500.0);
    }

    #[test]
    fn test_trend_history() {
        let mut history = TrendHistory::new();
        let snapshot1 = CostSnapshot::new("snap-001".to_string(), 1000.0);
        let snapshot2 = CostSnapshot::new("snap-002".to_string(), 1200.0);

        history.add_snapshot(snapshot1);
        history.add_snapshot(snapshot2);

        assert_eq!(history.snapshots.len(), 2);
        assert_eq!(history.latest().unwrap().id, "snap-002");
    }

    #[test]
    fn test_calculate_delta() {
        let mut history = TrendHistory::new();
        let snapshot1 = CostSnapshot::new("snap-001".to_string(), 1000.0);
        let snapshot2 = CostSnapshot::new("snap-002".to_string(), 1200.0);

        history.add_snapshot(snapshot1);
        history.add_snapshot(snapshot2);

        let delta = history.calculate_delta("snap-001", "snap-002");
        assert_eq!(delta, Some(200.0));
    }

    #[test]
    fn test_trend_config_defaults() {
        let config = TrendConfig::default();
        assert_eq!(config.max_snapshots, 100);
        assert_eq!(config.regression_threshold_percent, 10.0);
        assert!(config.enable_rotation);
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn test_regression_type_serialization() {
        let regression = Regression {
            regression_type: RegressionType::CostIncrease,
            affected: "module.vpc".to_string(),
            baseline_cost: 100.0,
            current_cost: 150.0,
            increase_amount: 50.0,
            increase_percent: 50.0,
            severity: "HIGH".to_string(),
        };

        let json = serde_json::to_string(&regression).unwrap();
        assert!(json.contains("cost_increase"));
    }
}
