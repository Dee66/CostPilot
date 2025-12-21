use super::slo_types::{
    EnforcementLevel, Slo, SloConfig, SloEvaluation, SloReport, SloStatus, SloType,
};
use crate::engines::baselines::BaselinesManager;
use crate::engines::trend::CostSnapshot;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manages SLO evaluation and enforcement
pub struct SloManager {
    config: SloConfig,
    baselines: Option<BaselinesManager>,
    edition: crate::edition::EditionContext,
}

impl SloManager {
    /// Create new SloManager with config
    pub fn new(config: SloConfig, edition: &crate::edition::EditionContext) -> Self {
        Self {
            config,
            baselines: None,
            edition: edition.clone(),
        }
    }

    /// Load SLO configuration from JSON file
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
        edition: &crate::edition::EditionContext,
    ) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read SLO config: {}", e))?;

        let config: SloConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse SLO JSON: {}", e))?;

        Ok(Self {
            config,
            baselines: None,
            edition: edition.clone(),
        })
    }

    /// Get reference to SLO configuration
    pub fn config(&self) -> &SloConfig {
        &self.config
    }

    /// Create from existing config
    pub fn from_config(config: SloConfig, edition: &crate::edition::EditionContext) -> Self {
        Self {
            config,
            baselines: None,
            edition: edition.clone(),
        }
    }

    /// Load with baselines for baseline-aware SLOs
    pub fn with_baselines(
        slo_path: impl AsRef<Path>,
        baseline_path: impl AsRef<Path>,
        edition: &crate::edition::EditionContext,
    ) -> Result<Self, String> {
        let mut manager = Self::load_from_file(slo_path, edition)?;
        let baselines = BaselinesManager::load_from_file(baseline_path)?;
        manager.baselines = Some(baselines);
        Ok(manager)
    }

    /// Save SLO configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Failed to serialize SLO config: {}", e))?;

        fs::write(path.as_ref(), json).map_err(|e| format!("Failed to write SLO config: {}", e))?;

        Ok(())
    }

    /// Validate SLO configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.config.slos.is_empty() {
            errors.push("No SLOs defined in configuration".to_string());
        }

        for slo in &self.config.slos {
            // Validate ID
            if slo.id.is_empty() {
                errors.push("SLO has empty ID".to_string());
            }

            // Validate name
            if slo.name.is_empty() {
                errors.push(format!("SLO '{}' has empty name", slo.id));
            }

            // Validate threshold
            if slo.threshold.max_value < 0.0 {
                errors.push(format!("SLO '{}' has negative max_value", slo.id));
            }

            if let Some(min_value) = slo.threshold.min_value {
                if min_value < 0.0 {
                    errors.push(format!("SLO '{}' has negative min_value", slo.id));
                }
                if min_value >= slo.threshold.max_value {
                    errors.push(format!("SLO '{}' min_value >= max_value", slo.id));
                }
            }

            if slo.threshold.warning_threshold_percent < 0.0
                || slo.threshold.warning_threshold_percent > 100.0
            {
                errors.push(format!("SLO '{}' warning threshold must be 0-100%", slo.id));
            }

            // Validate baseline settings
            if slo.threshold.use_baseline {
                if self.baselines.is_none() {
                    errors.push(format!(
                        "SLO '{}' uses baseline but no baseline manager loaded",
                        slo.id
                    ));
                }
                if let Some(multiplier) = slo.threshold.baseline_multiplier {
                    if multiplier <= 0.0 {
                        errors.push(format!(
                            "SLO '{}' baseline_multiplier must be positive",
                            slo.id
                        ));
                    }
                }
            }

            // Validate owner
            if slo.owner.is_empty() {
                errors.push(format!("SLO '{}' has no owner", slo.id));
            }
        }

        // Check for duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for slo in &self.config.slos {
            if !seen_ids.insert(&slo.id) {
                errors.push(format!("Duplicate SLO ID: '{}'", slo.id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Evaluate all SLOs against a snapshot
    pub fn evaluate_snapshot(&self, snapshot: &CostSnapshot) -> SloReport {
        let mut evaluations = Vec::new();

        for slo in &self.config.slos {
            // Gate enforcement for premium
            let mut slo_copy = slo.clone();
            if slo_copy.enforcement == EnforcementLevel::Block && !self.edition.is_premium() {
                eprintln!(
                    "⚠️  Free edition: SLO '{}' downgraded from Block to Warn (validate-only)",
                    slo_copy.id
                );
                eprintln!("   Upgrade to Premium to enforce SLO blocking");
                slo_copy.enforcement = EnforcementLevel::Warn;
            }

            if let Some(evaluation) = self.evaluate_slo(&slo_copy, snapshot) {
                evaluations.push(evaluation);
            }
        }

        SloReport::new(evaluations)
    }

    /// Evaluate a single SLO against snapshot
    fn evaluate_slo(&self, slo: &Slo, snapshot: &CostSnapshot) -> Option<SloEvaluation> {
        match slo.slo_type {
            SloType::MonthlyBudget => {
                if slo.target == "global" {
                    let threshold = self.get_effective_threshold(slo, snapshot.total_monthly_cost);
                    Some(self.evaluate_value(slo, snapshot.total_monthly_cost, threshold))
                } else {
                    None
                }
            }
            SloType::ModuleBudget => {
                if let Some(module) = snapshot.modules.get(&slo.target) {
                    let threshold = self.get_effective_threshold(slo, module.monthly_cost);
                    Some(self.evaluate_value(slo, module.monthly_cost, threshold))
                } else {
                    Some(SloEvaluation {
                        slo_id: slo.id.clone(),
                        slo_name: slo.name.clone(),
                        status: SloStatus::NoData,
                        actual_value: 0.0,
                        threshold_value: slo.threshold.max_value,
                        threshold_usage_percent: 0.0,
                        evaluated_at: chrono::Utc::now().to_rfc3339(),
                        message: format!("Module '{}' not found in snapshot", slo.target),
                        affected: vec![slo.target.clone()],
                        burn_risk: None,
                        projected_cost_after_merge: None,
                    })
                }
            }
            SloType::ServiceBudget => {
                if let Some(cost) = snapshot.services.get(&slo.target) {
                    let threshold = self.get_effective_threshold(slo, *cost);
                    Some(self.evaluate_value(slo, *cost, threshold))
                } else {
                    Some(SloEvaluation {
                        slo_id: slo.id.clone(),
                        slo_name: slo.name.clone(),
                        status: SloStatus::NoData,
                        actual_value: 0.0,
                        threshold_value: slo.threshold.max_value,
                        threshold_usage_percent: 0.0,
                        evaluated_at: chrono::Utc::now().to_rfc3339(),
                        message: format!("Service '{}' not found in snapshot", slo.target),
                        affected: vec![slo.target.clone()],
                        burn_risk: None,
                        projected_cost_after_merge: None,
                    })
                }
            }
            SloType::ResourceCount => {
                let total_resources: usize =
                    snapshot.modules.values().map(|m| m.resource_count).sum();
                Some(self.evaluate_value(slo, total_resources as f64, slo.threshold.max_value))
            }
            SloType::CostGrowthRate => {
                // Growth rate requires historical data - return NoData for now
                Some(SloEvaluation {
                    slo_id: slo.id.clone(),
                    slo_name: slo.name.clone(),
                    status: SloStatus::NoData,
                    actual_value: 0.0,
                    threshold_value: slo.threshold.max_value,
                    threshold_usage_percent: 0.0,
                    evaluated_at: chrono::Utc::now().to_rfc3339(),
                    message: "Growth rate calculation requires historical data".to_string(),
                    affected: vec![],
                    burn_risk: None,
                    projected_cost_after_merge: None,
                })
            }
            SloType::ResourceBudget => {
                // Per-resource budget - not directly applicable to snapshots
                Some(SloEvaluation {
                    slo_id: slo.id.clone(),
                    slo_name: slo.name.clone(),
                    status: SloStatus::NoData,
                    actual_value: 0.0,
                    threshold_value: slo.threshold.max_value,
                    threshold_usage_percent: 0.0,
                    evaluated_at: chrono::Utc::now().to_rfc3339(),
                    message: "Resource-level budget checks not implemented".to_string(),
                    affected: vec![],
                    burn_risk: None,
                    projected_cost_after_merge: None,
                })
            }
        }
    }

    /// Get effective threshold (considering baselines if enabled)
    fn get_effective_threshold(&self, slo: &Slo, _actual_value: f64) -> f64 {
        if slo.threshold.use_baseline {
            if let Some(baselines) = &self.baselines {
                let baseline_value = match slo.slo_type {
                    SloType::MonthlyBudget if slo.target == "global" => baselines
                        .config()
                        .global
                        .as_ref()
                        .map(|b| b.expected_monthly_cost),
                    SloType::ModuleBudget => baselines
                        .config()
                        .get_module_baseline(&slo.target)
                        .map(|b| b.expected_monthly_cost),
                    SloType::ServiceBudget => baselines
                        .config()
                        .get_service_baseline(&slo.target)
                        .map(|b| b.expected_monthly_cost),
                    _ => None,
                };

                if let Some(baseline) = baseline_value {
                    let multiplier = slo.threshold.baseline_multiplier.unwrap_or(1.0);
                    return baseline * multiplier;
                }
            }
        }

        slo.threshold.max_value
    }

    /// Evaluate value against SLO threshold
    fn evaluate_value(&self, slo: &Slo, value: f64, threshold: f64) -> SloEvaluation {
        let warning_threshold = threshold * (slo.threshold.warning_threshold_percent / 100.0);

        let status = if value > threshold {
            SloStatus::Violation
        } else if value >= warning_threshold {
            SloStatus::Warning
        } else {
            SloStatus::Pass
        };

        let threshold_usage = (value / threshold) * 100.0;
        let message = match status {
            SloStatus::Pass => format!(
                "Within SLO: ${:.2} of ${:.2} ({:.1}%)",
                value, threshold, threshold_usage
            ),
            SloStatus::Warning => format!(
                "Approaching limit: ${:.2} of ${:.2} ({:.1}%)",
                value, threshold, threshold_usage
            ),
            SloStatus::Violation => format!(
                "SLO violated: ${:.2} exceeds ${:.2} ({:.1}%)",
                value, threshold, threshold_usage
            ),
            SloStatus::NoData => "No data available".to_string(),
        };

        SloEvaluation {
            slo_id: slo.id.clone(),
            slo_name: slo.name.clone(),
            status,
            actual_value: value,
            threshold_value: threshold,
            threshold_usage_percent: threshold_usage,
            evaluated_at: chrono::Utc::now().to_rfc3339(),
            message,
            affected: vec![slo.target.clone()],
            burn_risk: None,
            projected_cost_after_merge: None,
        }
    }

    /// Evaluate module costs against module SLOs
    pub fn evaluate_module_costs(&self, module_costs: &HashMap<String, f64>) -> SloReport {
        let mut evaluations = Vec::new();

        for slo in &self.config.slos {
            if slo.slo_type == SloType::ModuleBudget {
                if let Some(cost) = module_costs.get(&slo.target) {
                    let threshold = self.get_effective_threshold(slo, *cost);
                    evaluations.push(self.evaluate_value(slo, *cost, threshold));
                }
            }
        }

        SloReport::new(evaluations)
    }

    /// Check if deployment should be blocked
    pub fn should_block_deployment(&self, report: &SloReport) -> bool {
        report.should_block_deployment(&self.config)
    }

    /// Get blocking violations from report
    pub fn get_blocking_violations<'a>(&self, report: &'a SloReport) -> Vec<&'a SloEvaluation> {
        report.blocking_violations(&self.config)
    }

    /// Get baselines manager if loaded
    pub fn baselines(&self) -> Option<&BaselinesManager> {
        self.baselines.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::slo::slo_types::SloThreshold;
    use crate::engines::trend::ModuleCost;

    fn create_test_slo() -> Slo {
        Slo::new(
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
        )
    }

    fn create_test_snapshot(total_cost: f64) -> CostSnapshot {
        let mut snapshot = CostSnapshot::new("test".to_string(), total_cost);

        let mut modules = HashMap::new();
        modules.insert(
            "module.vpc".to_string(),
            ModuleCost {
                name: "module.vpc".to_string(),
                monthly_cost: total_cost * 0.3,
                resource_count: 5,
                change_from_previous: None,
                change_percent: None,
                services: vec![],
            },
        );
        snapshot.modules = modules;

        snapshot
    }

    #[test]
    fn test_manager_creation() {
        let config = SloConfig::new();
        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        assert_eq!(manager.config.slos.len(), 0);
    }

    #[test]
    fn test_validate_success() {
        let mut config = SloConfig::new();
        config.add_slo(create_test_slo());

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_validate_negative_threshold() {
        let mut config = SloConfig::new();
        let mut slo = create_test_slo();
        slo.threshold.max_value = -100.0;
        config.add_slo(slo);

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let result = manager.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_config() {
        let config = SloConfig::new();
        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let result = manager.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("No SLOs defined"));
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let mut config = SloConfig::new();
        config.add_slo(create_test_slo());
        config.add_slo(create_test_slo()); // Same ID

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let result = manager.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_snapshot_pass() {
        let mut config = SloConfig::new();
        config.add_slo(create_test_slo());

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(5000.0);

        let report = manager.evaluate_snapshot(&snapshot);
        assert_eq!(report.summary.pass_count, 1);
        assert_eq!(report.summary.violation_count, 0);
    }

    #[test]
    fn test_evaluate_snapshot_violation() {
        let mut config = SloConfig::new();
        config.add_slo(create_test_slo());

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(15000.0);

        let report = manager.evaluate_snapshot(&snapshot);
        assert_eq!(report.summary.violation_count, 1);
        assert!(report.has_violations());
    }

    #[test]
    fn test_evaluate_snapshot_warning() {
        let mut config = SloConfig::new();
        config.add_slo(create_test_slo());

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(8500.0); // 85% of 10000

        let report = manager.evaluate_snapshot(&snapshot);
        assert_eq!(report.summary.warning_count, 1);
    }

    #[test]
    fn test_evaluate_module_budget() {
        let mut config = SloConfig::new();
        let module_slo = Slo::new(
            "slo-module".to_string(),
            "VPC Budget".to_string(),
            "VPC monthly budget".to_string(),
            SloType::ModuleBudget,
            "module.vpc".to_string(),
            SloThreshold {
                max_value: 3000.0,
                min_value: None,
                warning_threshold_percent: 80.0,
                time_window: "30d".to_string(),
                use_baseline: false,
                baseline_multiplier: None,
            },
            EnforcementLevel::Warn,
            "network@example.com".to_string(),
        );
        config.add_slo(module_slo);

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(10000.0); // module.vpc = 3000

        let report = manager.evaluate_snapshot(&snapshot);
        assert_eq!(report.evaluations.len(), 1);
        assert_eq!(report.evaluations[0].status, SloStatus::Warning);
    }

    #[test]
    fn test_blocking_deployment() {
        let mut config = SloConfig::new();
        let mut slo = create_test_slo();
        slo.enforcement = EnforcementLevel::Block;
        config.add_slo(slo);

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(15000.0);

        let report = manager.evaluate_snapshot(&snapshot);
        assert!(manager.should_block_deployment(&report));
    }

    #[test]
    fn test_no_blocking_for_warn() {
        let mut config = SloConfig::new();
        let mut slo = create_test_slo();
        slo.enforcement = EnforcementLevel::Warn;
        config.add_slo(slo);

        let edition = crate::edition::EditionContext::free();
        let manager = SloManager::from_config(config, &edition);
        let snapshot = create_test_snapshot(15000.0);

        let report = manager.evaluate_snapshot(&snapshot);
        assert!(!manager.should_block_deployment(&report));
    }
}
