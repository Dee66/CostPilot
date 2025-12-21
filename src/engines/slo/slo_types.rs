use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Burn risk classification (forward declaration for slo_types)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BurnRisk {
    /// No breach predicted
    Low,

    /// Breach possible but not imminent (>14 days)
    Medium,

    /// Breach likely within 14 days
    High,

    /// Breach imminent (<7 days)
    Critical,
}

impl BurnRisk {
    /// Get numeric severity (0-3)
    pub fn severity(&self) -> u8 {
        match self {
            BurnRisk::Low => 0,
            BurnRisk::Medium => 1,
            BurnRisk::High => 2,
            BurnRisk::Critical => 3,
        }
    }

    /// Check if action is required
    pub fn requires_action(&self) -> bool {
        matches!(self, BurnRisk::High | BurnRisk::Critical)
    }
}

/// A Service Level Objective for cost management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slo {
    /// Unique identifier for this SLO
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this SLO enforces
    pub description: String,

    /// Type of SLO check
    pub slo_type: SloType,

    /// Target entity (module name, service name, or "global")
    pub target: String,

    /// Threshold configuration
    pub threshold: SloThreshold,

    /// Enforcement level
    pub enforcement: EnforcementLevel,

    /// Owner responsible for this SLO
    pub owner: String,

    /// When this SLO was created
    pub created_at: String,

    /// When this SLO was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,

    /// Tags for categorization
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub tags: HashMap<String, String>,
}

/// Type of SLO
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SloType {
    /// Monthly cost limit for total infrastructure
    MonthlyBudget,

    /// Monthly cost limit for a specific module
    ModuleBudget,

    /// Monthly cost limit for a specific service type
    ServiceBudget,

    /// Cost per resource limit
    ResourceBudget,

    /// Cost increase rate limit (percentage)
    CostGrowthRate,

    /// Resource count limit
    ResourceCount,
}

/// SLO threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloThreshold {
    /// Maximum allowed value
    pub max_value: f64,

    /// Minimum allowed value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,

    /// Warning threshold (percentage of max, e.g., 80.0 for 80%)
    #[serde(default = "default_warning_threshold")]
    pub warning_threshold_percent: f64,

    /// Time window for evaluation (e.g., "30d" for 30 days)
    #[serde(default = "default_time_window")]
    pub time_window: String,

    /// Use baseline as threshold source
    #[serde(default)]
    pub use_baseline: bool,

    /// Baseline multiplier if using baseline (e.g., 1.2 for 120% of baseline)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_multiplier: Option<f64>,
}

fn default_warning_threshold() -> f64 {
    80.0
}

fn default_time_window() -> String {
    "30d".to_string()
}

/// Enforcement level for SLO violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementLevel {
    /// Log only - no action taken
    Observe,

    /// Warn but allow deployment
    Warn,

    /// Block deployment on violation
    Block,

    /// Block and require approval to override
    StrictBlock,
}

/// Result of SLO evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloEvaluation {
    /// SLO that was evaluated
    pub slo_id: String,

    /// SLO name
    pub slo_name: String,

    /// Evaluation status
    pub status: SloStatus,

    /// Actual measured value
    pub actual_value: f64,

    /// Threshold that was checked against
    pub threshold_value: f64,

    /// Percentage of threshold used (e.g., 85.5 for 85.5%)
    pub threshold_usage_percent: f64,

    /// Time of evaluation
    pub evaluated_at: String,

    /// Message describing the result
    pub message: String,

    /// Affected resources or modules
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub affected: Vec<String>,

    /// Burn risk level (Low/Medium/High/Critical)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burn_risk: Option<BurnRisk>,

    /// Projected cost after merge/deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_cost_after_merge: Option<f64>,
}

/// SLO evaluation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SloStatus {
    /// Within SLO limits
    Pass,

    /// Approaching threshold (warning level)
    Warning,

    /// Exceeded threshold
    Violation,

    /// No data available for evaluation
    NoData,
}

/// Container for all SLOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloConfig {
    /// Schema version
    pub version: String,

    /// All SLOs
    pub slos: Vec<Slo>,

    /// Global configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<SloGlobalConfig>,
}

/// Global SLO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloGlobalConfig {
    /// Default enforcement level
    #[serde(default = "default_enforcement")]
    pub default_enforcement: EnforcementLevel,

    /// Enable SLO inheritance from parent modules
    #[serde(default)]
    pub enable_inheritance: bool,

    /// Baseline file path for baseline-aware SLOs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_file: Option<String>,
}

fn default_enforcement() -> EnforcementLevel {
    EnforcementLevel::Warn
}

/// SLO evaluation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloReport {
    /// When the report was generated
    pub generated_at: String,

    /// All evaluations in this report
    pub evaluations: Vec<SloEvaluation>,

    /// Summary statistics
    pub summary: SloSummary,

    /// Metadata about the evaluation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Summary of SLO evaluations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloSummary {
    /// Total SLOs evaluated
    pub total_slos: usize,

    /// Number passing
    pub pass_count: usize,

    /// Number in warning state
    pub warning_count: usize,

    /// Number violated
    pub violation_count: usize,

    /// Number with no data
    pub no_data_count: usize,

    /// Overall status (Pass if all pass, Violation if any violation)
    pub overall_status: SloStatus,
}

impl Slo {
    /// Create a new SLO
    pub fn new(
        id: String,
        name: String,
        description: String,
        slo_type: SloType,
        target: String,
        threshold: SloThreshold,
        enforcement: EnforcementLevel,
        owner: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            slo_type,
            target,
            threshold,
            enforcement,
            owner,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            tags: HashMap::new(),
        }
    }

    /// Check if SLO should block deployment
    pub fn should_block(&self) -> bool {
        matches!(
            self.enforcement,
            EnforcementLevel::Block | EnforcementLevel::StrictBlock
        )
    }

    /// Check if SLO requires strict approval for override
    pub fn requires_strict_approval(&self) -> bool {
        self.enforcement == EnforcementLevel::StrictBlock
    }

    /// Get warning threshold value
    pub fn warning_threshold(&self) -> f64 {
        self.threshold.max_value * (self.threshold.warning_threshold_percent / 100.0)
    }

    /// Check if value is in warning range
    pub fn is_warning(&self, value: f64) -> bool {
        let warning_threshold = self.warning_threshold();
        value >= warning_threshold && value < self.threshold.max_value
    }

    /// Check if value violates SLO
    pub fn is_violation(&self, value: f64) -> bool {
        value > self.threshold.max_value
    }

    /// Evaluate value against this SLO
    pub fn evaluate(&self, value: f64) -> SloEvaluation {
        let status = if self.is_violation(value) {
            SloStatus::Violation
        } else if self.is_warning(value) {
            SloStatus::Warning
        } else {
            SloStatus::Pass
        };

        let threshold_usage = (value / self.threshold.max_value) * 100.0;
        let message = match status {
            SloStatus::Pass => format!(
                "Within SLO: ${:.2} of ${:.2} ({:.1}%)",
                value, self.threshold.max_value, threshold_usage
            ),
            SloStatus::Warning => format!(
                "Approaching limit: ${:.2} of ${:.2} ({:.1}%)",
                value, self.threshold.max_value, threshold_usage
            ),
            SloStatus::Violation => format!(
                "SLO violated: ${:.2} exceeds ${:.2} ({:.1}%)",
                value, self.threshold.max_value, threshold_usage
            ),
            SloStatus::NoData => "No data available".to_string(),
        };

        SloEvaluation {
            slo_id: self.id.clone(),
            slo_name: self.name.clone(),
            status,
            actual_value: value,
            threshold_value: self.threshold.max_value,
            threshold_usage_percent: threshold_usage,
            evaluated_at: Utc::now().to_rfc3339(),
            message,
            affected: vec![self.target.clone()],
            burn_risk: None,
            projected_cost_after_merge: None,
        }
    }
}

impl SloConfig {
    /// Create a new SLO configuration
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            slos: Vec::new(),
            config: Some(SloGlobalConfig {
                default_enforcement: default_enforcement(),
                enable_inheritance: false,
                baseline_file: None,
            }),
        }
    }

    /// Add an SLO
    pub fn add_slo(&mut self, slo: Slo) {
        self.slos.push(slo);
    }

    /// Get SLO by ID
    pub fn get_slo(&self, id: &str) -> Option<&Slo> {
        self.slos.iter().find(|s| s.id == id)
    }

    /// Get SLOs by target
    pub fn get_slos_for_target(&self, target: &str) -> Vec<&Slo> {
        self.slos.iter().filter(|s| s.target == target).collect()
    }

    /// Get SLOs by type
    pub fn get_slos_by_type(&self, slo_type: &SloType) -> Vec<&Slo> {
        self.slos
            .iter()
            .filter(|s| &s.slo_type == slo_type)
            .collect()
    }

    /// Get global budget SLO
    pub fn get_global_budget_slo(&self) -> Option<&Slo> {
        self.slos
            .iter()
            .find(|s| s.slo_type == SloType::MonthlyBudget && s.target == "global")
    }
}

impl Default for SloConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SloReport {
    /// Create a new report
    pub fn new(evaluations: Vec<SloEvaluation>) -> Self {
        let pass_count = evaluations
            .iter()
            .filter(|e| e.status == SloStatus::Pass)
            .count();
        let warning_count = evaluations
            .iter()
            .filter(|e| e.status == SloStatus::Warning)
            .count();
        let violation_count = evaluations
            .iter()
            .filter(|e| e.status == SloStatus::Violation)
            .count();
        let no_data_count = evaluations
            .iter()
            .filter(|e| e.status == SloStatus::NoData)
            .count();

        let overall_status = if violation_count > 0 {
            SloStatus::Violation
        } else if warning_count > 0 {
            SloStatus::Warning
        } else if no_data_count > 0 {
            SloStatus::NoData
        } else {
            SloStatus::Pass
        };

        Self {
            generated_at: Utc::now().to_rfc3339(),
            evaluations,
            summary: SloSummary {
                total_slos: pass_count + warning_count + violation_count + no_data_count,
                pass_count,
                warning_count,
                violation_count,
                no_data_count,
                overall_status,
            },
            metadata: None,
        }
    }

    /// Check if any SLOs are violated
    pub fn has_violations(&self) -> bool {
        self.summary.violation_count > 0
    }

    /// Check if any SLOs should block deployment
    pub fn should_block_deployment(&self, slo_config: &SloConfig) -> bool {
        self.evaluations.iter().any(|eval| {
            if eval.status == SloStatus::Violation {
                if let Some(slo) = slo_config.get_slo(&eval.slo_id) {
                    return slo.should_block();
                }
            }
            false
        })
    }

    /// Get violations that require blocking
    pub fn blocking_violations(&self, slo_config: &SloConfig) -> Vec<&SloEvaluation> {
        self.evaluations
            .iter()
            .filter(|eval| {
                if eval.status == SloStatus::Violation {
                    if let Some(slo) = slo_config.get_slo(&eval.slo_id) {
                        return slo.should_block();
                    }
                }
                false
            })
            .collect()
    }

    /// Format report for display
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("📊 SLO Evaluation Report\n");
        output.push_str(&format!("Generated: {}\n\n", self.generated_at));

        output.push_str("Summary:\n");
        output.push_str(&format!("  Total SLOs: {}\n", self.summary.total_slos));
        output.push_str(&format!("  ✅ Pass: {}\n", self.summary.pass_count));
        output.push_str(&format!("  ⚠️  Warning: {}\n", self.summary.warning_count));
        output.push_str(&format!(
            "  ❌ Violation: {}\n",
            self.summary.violation_count
        ));
        if self.summary.no_data_count > 0 {
            output.push_str(&format!("  ❓ No Data: {}\n", self.summary.no_data_count));
        }
        output.push_str(&format!(
            "\nOverall Status: {:?}\n\n",
            self.summary.overall_status
        ));

        if !self.evaluations.is_empty() {
            output.push_str("Evaluations:\n");
            for eval in &self.evaluations {
                let icon = match eval.status {
                    SloStatus::Pass => "✅",
                    SloStatus::Warning => "⚠️",
                    SloStatus::Violation => "❌",
                    SloStatus::NoData => "❓",
                };
                output.push_str(&format!("{} {}: {}\n", icon, eval.slo_name, eval.message));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slo_creation() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Global Budget".to_string(),
            "Monthly budget limit".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Block,
            "finance@example.com".to_string(),
        );

        assert_eq!(slo.id, "slo-1");
        assert_eq!(slo.slo_type, SloType::MonthlyBudget);
        assert!(slo.should_block());
    }

    #[test]
    fn test_warning_threshold() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Warn,
            "owner".to_string(),
        );

        assert_eq!(slo.warning_threshold(), 8000.0);
        assert!(slo.is_warning(8500.0));
        assert!(!slo.is_warning(7500.0));
        assert!(!slo.is_warning(10500.0)); // This is violation
    }

    #[test]
    fn test_violation_check() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Block,
            "owner".to_string(),
        );

        assert!(!slo.is_violation(9000.0));
        assert!(slo.is_violation(11000.0));
    }

    #[test]
    fn test_evaluate_pass() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test SLO".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Warn,
            "owner".to_string(),
        );

        let eval = slo.evaluate(5000.0);
        assert_eq!(eval.status, SloStatus::Pass);
        assert_eq!(eval.actual_value, 5000.0);
        assert_eq!(eval.threshold_usage_percent, 50.0);
    }

    #[test]
    fn test_evaluate_warning() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test SLO".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Warn,
            "owner".to_string(),
        );

        let eval = slo.evaluate(9000.0);
        assert_eq!(eval.status, SloStatus::Warning);
        assert!(eval.message.contains("Approaching"));
    }

    #[test]
    fn test_evaluate_violation() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test SLO".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Block,
            "owner".to_string(),
        );

        let eval = slo.evaluate(12000.0);
        assert_eq!(eval.status, SloStatus::Violation);
        assert!(eval.message.contains("violated"));
    }

    #[test]
    fn test_slo_config() {
        let mut config = SloConfig::new();

        let slo = Slo::new(
            "slo-1".to_string(),
            "Global Budget".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Block,
            "owner".to_string(),
        );

        config.add_slo(slo);

        assert_eq!(config.slos.len(), 1);
        assert!(config.get_slo("slo-1").is_some());
        assert!(config.get_global_budget_slo().is_some());
    }

    #[test]
    fn test_enforcement_levels() {
        let observe_slo = Slo::new(
            "slo-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Observe,
            "owner".to_string(),
        );

        assert!(!observe_slo.should_block());
        assert!(!observe_slo.requires_strict_approval());

        let block_slo = Slo::new(
            "slo-2".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::StrictBlock,
            "owner".to_string(),
        );

        assert!(block_slo.should_block());
        assert!(block_slo.requires_strict_approval());
    }

    #[test]
    fn test_slo_report() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test SLO".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Block,
            "owner".to_string(),
        );

        let eval1 = slo.evaluate(5000.0);
        let eval2 = slo.evaluate(12000.0);

        let report = SloReport::new(vec![eval1, eval2]);

        assert_eq!(report.summary.total_slos, 2);
        assert_eq!(report.summary.pass_count, 1);
        assert_eq!(report.summary.violation_count, 1);
        assert!(report.has_violations());
        assert_eq!(report.summary.overall_status, SloStatus::Violation);
    }

    #[test]
    fn test_report_formatting() {
        let slo = Slo::new(
            "slo-1".to_string(),
            "Test SLO".to_string(),
            "Test".to_string(),
            SloType::MonthlyBudget,
            "global".to_string(),
            SloThreshold {
                max_value: 10000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Warn,
            "owner".to_string(),
        );

        let eval = slo.evaluate(5000.0);
        let report = SloReport::new(vec![eval]);
        let formatted = report.format();

        assert!(formatted.contains("SLO Evaluation Report"));
        assert!(formatted.contains("Total SLOs: 1"));
        assert!(formatted.contains("Pass: 1"));
    }
}
