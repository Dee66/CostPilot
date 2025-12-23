use super::baseline_types::{Baseline, BaselineStatus, BaselineViolation, BaselinesConfig};
use crate::engines::shared::models::RegressionType;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Manages baseline cost expectations
pub struct BaselinesManager {
    config: BaselinesConfig,
}

/// Result of baseline comparison against snapshot
#[derive(Debug, Clone)]
pub struct BaselineComparisonResult {
    pub total_violations: usize,
    pub violations: Vec<BaselineViolation>,
    pub within_baseline_count: usize,
    pub no_baseline_count: usize,
}

impl BaselinesManager {
    /// Load baselines from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read baselines file: {}", e))?;

        let config: BaselinesConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse baselines JSON: {}", e))?;

        Ok(Self { config })
    }

    /// Create from existing config
    pub fn from_config(config: BaselinesConfig) -> Self {
        Self { config }
    }

    /// Save baselines to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Failed to serialize baselines: {}", e))?;

        fs::write(path.as_ref(), json)
            .map_err(|e| format!("Failed to write baselines file: {}", e))?;

        Ok(())
    }

    /// Validate baselines configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate global baseline
        if let Some(global) = &self.config.global {
            if global.expected_monthly_cost < 0.0 {
                errors.push("Global baseline has negative cost".to_string());
            }
            if global.acceptable_variance_percent < 0.0
                || global.acceptable_variance_percent > 100.0
            {
                errors.push("Global baseline variance must be 0-100%".to_string());
            }
        }

        // Validate module baselines
        for (name, baseline) in &self.config.modules {
            if baseline.expected_monthly_cost < 0.0 {
                errors.push(format!("Module '{}' has negative cost", name));
            }
            if baseline.acceptable_variance_percent < 0.0
                || baseline.acceptable_variance_percent > 100.0
            {
                errors.push(format!("Module '{}' variance must be 0-100%", name));
            }
            if baseline.justification.is_empty() {
                errors.push(format!("Module '{}' missing justification", name));
            }
            if baseline.owner.is_empty() {
                errors.push(format!("Module '{}' missing owner", name));
            }
        }

        // Validate service baselines
        for (name, baseline) in &self.config.services {
            if baseline.expected_monthly_cost < 0.0 {
                errors.push(format!("Service '{}' has negative cost", name));
            }
            if baseline.acceptable_variance_percent < 0.0
                || baseline.acceptable_variance_percent > 100.0
            {
                errors.push(format!("Service '{}' variance must be 0-100%", name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Classify regression type for a module based on its resource changes
    fn classify_module_regression(
        &self,
        module_name: &str,
        changes: &[crate::engines::detection::ResourceChange],
    ) -> RegressionType {
        // Get changes for this module
        let module_changes: Vec<_> = changes
            .iter()
            .filter(|change| change.module_path.as_deref() == Some(module_name))
            .collect();

        if module_changes.is_empty() {
            return RegressionType::IndirectCost;
        }

        // Check for provisioning (new resources)
        if module_changes.iter().any(|change| change.action == crate::engines::shared::models::ChangeAction::Create) {
            return RegressionType::Provisioning;
        }

        // Check for scaling changes
        if module_changes.iter().any(|change| {
            change.action == crate::engines::shared::models::ChangeAction::Update
                && Self::is_scaling_change(change)
        }) {
            return RegressionType::Scaling;
        }

        // Check for configuration changes
        if module_changes.iter().any(|change| {
            change.action == crate::engines::shared::models::ChangeAction::Update
                && Self::is_configuration_change(change)
        }) {
            return RegressionType::Configuration;
        }

        // Default to indirect cost
        RegressionType::IndirectCost
    }

    /// Check if a change represents scaling
    fn is_scaling_change(change: &crate::engines::detection::ResourceChange) -> bool {
        // Scaling typically involves instance count, capacity, etc.
        if let (Some(old_config), Some(new_config)) = (&change.old_config, &change.new_config) {
            // Check for common scaling attributes
            let scaling_attrs = ["instance_count", "desired_capacity", "replicas", "node_count"];
            for attr in &scaling_attrs {
                if Self::field_changed(old_config, new_config, attr) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a change represents configuration
    fn is_configuration_change(change: &crate::engines::detection::ResourceChange) -> bool {
        if let (Some(old_config), Some(new_config)) = (&change.old_config, &change.new_config) {
            // Check for configuration-type changes
            let config_attrs = ["billing_mode", "instance_type", "engine_version", "storage_type"];
            for attr in &config_attrs {
                if Self::field_changed(old_config, new_config, attr) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a field changed between old and new config
    fn field_changed(old_config: &serde_json::Value, new_config: &serde_json::Value, field: &str) -> bool {
        let old_val = old_config.get(field);
        let new_val = new_config.get(field);
        old_val != new_val
    }

    /// Classify regression type for global baseline based on all resource changes
    fn classify_global_regression(&self, changes: &[crate::engines::detection::ResourceChange]) -> RegressionType {
        if changes.is_empty() {
            return RegressionType::IndirectCost;
        }

        // Check for provisioning (new resources)
        if changes.iter().any(|change| change.action == crate::engines::shared::models::ChangeAction::Create) {
            return RegressionType::Provisioning;
        }

        // Check for scaling changes
        if changes.iter().any(|change| {
            change.action == crate::engines::shared::models::ChangeAction::Update
                && Self::is_scaling_change(change)
        }) {
            return RegressionType::Scaling;
        }

        // Check for configuration changes
        if changes.iter().any(|change| {
            change.action == crate::engines::shared::models::ChangeAction::Update
                && Self::is_configuration_change(change)
        }) {
            return RegressionType::Configuration;
        }

        // Default to indirect cost
        RegressionType::IndirectCost
    }

    /// Compare snapshot module costs against baselines
    pub fn compare_module_costs(
        &self,
        module_costs: &HashMap<String, f64>,
        changes: Option<&[crate::engines::detection::ResourceChange]>,
    ) -> BaselineComparisonResult {
        let mut violations = Vec::new();
        let mut within_count = 0;
        let mut no_baseline_count = 0;

        for (module_name, actual_cost) in module_costs {
            if let Some(baseline) = self.config.get_module_baseline(module_name) {
                match baseline.check_variance(*actual_cost) {
                    BaselineStatus::Within => {
                        within_count += 1;
                    }
                    BaselineStatus::Exceeded {
                        expected,
                        variance_percent,
                        ..
                    } => {
                        violations.push(BaselineViolation {
                            name: module_name.clone(),
                            baseline_type: "module".to_string(),
                            expected_cost: expected,
                            actual_cost: *actual_cost,
                            variance_percent,
                            acceptable_variance: baseline.acceptable_variance_percent,
                            severity: calculate_severity(variance_percent),
                            regression_type: changes.map_or(RegressionType::IndirectCost, |c| self.classify_module_regression(module_name, c)),
                            owner: baseline.owner.clone(),
                            justification: baseline.justification.clone(),
                        });
                    }
                    BaselineStatus::Below {
                        expected,
                        variance_percent,
                        ..
                    } => {
                        // Below baseline might be good, but flag for review
                        violations.push(BaselineViolation {
                            name: module_name.clone(),
                            baseline_type: "module".to_string(),
                            expected_cost: expected,
                            actual_cost: *actual_cost,
                            variance_percent,
                            acceptable_variance: baseline.acceptable_variance_percent,
                            severity: "Info".to_string(),
                            regression_type: changes.map_or(RegressionType::IndirectCost, |c| self.classify_module_regression(module_name, c)),
                            owner: baseline.owner.clone(),
                            justification: baseline.justification.clone(),
                        });
                    }
                    BaselineStatus::NoBaseline => {
                        no_baseline_count += 1;
                    }
                }
            } else {
                no_baseline_count += 1;
            }
        }

        BaselineComparisonResult {
            total_violations: violations.len(),
            violations,
            within_baseline_count: within_count,
            no_baseline_count,
        }
    }

    /// Compare total cost against global baseline
    pub fn compare_total_cost(&self, total_cost: f64, changes: Option<&[crate::engines::detection::ResourceChange]>) -> Option<BaselineViolation> {
        let global = self.config.global.as_ref()?;

        match global.check_variance(total_cost) {
            BaselineStatus::Exceeded {
                expected,
                variance_percent,
                ..
            } => Some(BaselineViolation {
                name: "global".to_string(),
                baseline_type: "global".to_string(),
                expected_cost: expected,
                actual_cost: total_cost,
                variance_percent,
                acceptable_variance: global.acceptable_variance_percent,
                severity: calculate_severity(variance_percent),
                regression_type: changes.map_or(RegressionType::IndirectCost, |c| self.classify_global_regression(c)),
                owner: global.owner.clone(),
                justification: global.justification.clone(),
            }),
            BaselineStatus::Below {
                expected,
                variance_percent,
                ..
            } => Some(BaselineViolation {
                name: "global".to_string(),
                baseline_type: "global".to_string(),
                expected_cost: expected,
                actual_cost: total_cost,
                variance_percent,
                acceptable_variance: global.acceptable_variance_percent,
                severity: "Info".to_string(),
                regression_type: changes.map_or(RegressionType::IndirectCost, |c| self.classify_global_regression(c)),
                owner: global.owner.clone(),
                justification: global.justification.clone(),
            }),
            _ => None,
        }
    }

    /// Get stale baselines that need review
    pub fn get_stale_baselines(&self) -> Vec<(&str, &Baseline)> {
        self.config.get_stale_baselines()
    }

    /// Get underlying config
    pub fn config(&self) -> &BaselinesConfig {
        &self.config
    }

    /// Update a module baseline
    pub fn update_module_baseline(&mut self, module_name: String, baseline: Baseline) {
        self.config.add_module(module_name, baseline);
    }

    /// Update global baseline
    pub fn update_global_baseline(&mut self, baseline: Baseline) {
        self.config.set_global(baseline);
    }
}

/// Calculate severity based on variance percentage
fn calculate_severity(variance_percent: f64) -> String {
    if variance_percent > 50.0 {
        "Critical".to_string()
    } else if variance_percent > 25.0 {
        "High".to_string()
    } else if variance_percent > 10.0 {
        "Medium".to_string()
    } else {
        "Low".to_string()
    }
}

impl BaselineComparisonResult {
    /// Filter violations by severity
    pub fn filter_by_severity(&self, severity: &str) -> Vec<&BaselineViolation> {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .collect()
    }

    /// Get critical violations
    pub fn critical_violations(&self) -> Vec<&BaselineViolation> {
        self.filter_by_severity("Critical")
    }

    /// Get high violations
    pub fn high_violations(&self) -> Vec<&BaselineViolation> {
        self.filter_by_severity("High")
    }

    /// Check if there are any critical violations
    pub fn has_critical_violations(&self) -> bool {
        !self.critical_violations().is_empty()
    }

    /// Format violations for display
    pub fn format_violations(&self) -> String {
        if self.violations.is_empty() {
            return "✅ All costs within baseline".to_string();
        }

        let mut output = String::new();
        output.push_str(&format!(
            "⚠️  {} baseline violations\n",
            self.total_violations
        ));

        for violation in &self.violations {
            output.push_str(&format!(
                "\n[{}] {}: ${:.2} vs ${:.2} expected ({:+.1}%)\n",
                violation.severity,
                violation.name,
                violation.actual_cost,
                violation.expected_cost,
                violation.variance_percent
            ));
            output.push_str(&format!("  Owner: {}\n", violation.owner));
            output.push_str(&format!("  Justification: {}\n", violation.justification));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_baseline() -> Baseline {
        Baseline::new(
            "module.vpc".to_string(),
            1000.0,
            "VPC baseline".to_string(),
            "platform-team".to_string(),
        )
    }

    fn create_test_config() -> BaselinesConfig {
        let mut config = BaselinesConfig::new();
        config.add_module("module.vpc".to_string(), create_test_baseline());
        config
    }

    #[test]
    fn test_validate_success() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_validate_negative_cost() {
        let mut config = BaselinesConfig::new();
        let mut baseline = create_test_baseline();
        baseline.expected_monthly_cost = -100.0;
        config.add_module("bad_module".to_string(), baseline);

        let manager = BaselinesManager::from_config(config);
        let result = manager.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_variance() {
        let mut config = BaselinesConfig::new();
        let mut baseline = create_test_baseline();
        baseline.acceptable_variance_percent = 150.0;
        config.add_module("bad_module".to_string(), baseline);

        let manager = BaselinesManager::from_config(config);
        let result = manager.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_module_costs_within() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);

        let mut costs = HashMap::new();
        costs.insert("module.vpc".to_string(), 1050.0); // Within 10%

        let result = manager.compare_module_costs(&costs, None);
        assert_eq!(result.within_baseline_count, 1);
        assert_eq!(result.total_violations, 0);
    }

    #[test]
    fn test_compare_module_costs_exceeded() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);

        let mut costs = HashMap::new();
        costs.insert("module.vpc".to_string(), 1500.0); // 50% over

        let result = manager.compare_module_costs(&costs, None);
        assert_eq!(result.total_violations, 1);
        assert_eq!(result.violations[0].severity, "High");
    }

    #[test]
    fn test_compare_module_costs_no_baseline() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);

        let mut costs = HashMap::new();
        costs.insert("module.unknown".to_string(), 500.0);

        let result = manager.compare_module_costs(&costs, None);
        assert_eq!(result.no_baseline_count, 1);
    }

    #[test]
    fn test_compare_total_cost() {
        let mut config = BaselinesConfig::new();
        let global = Baseline::new(
            "global".to_string(),
            5000.0,
            "Global baseline".to_string(),
            "finance-team".to_string(),
        );
        config.set_global(global);

        let manager = BaselinesManager::from_config(config);

        // Within baseline
        assert!(manager.compare_total_cost(5200.0, None).is_none());

        // Exceeded
        let violation = manager.compare_total_cost(6000.0, None);
        assert!(violation.is_some());
        assert_eq!(violation.unwrap().severity, "Medium");
    }

    #[test]
    fn test_severity_calculation() {
        assert_eq!(calculate_severity(5.0), "Low");
        assert_eq!(calculate_severity(15.0), "Medium");
        assert_eq!(calculate_severity(30.0), "High");
        assert_eq!(calculate_severity(60.0), "Critical");
    }

    #[test]
    fn test_format_violations() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);

        let mut costs = HashMap::new();
        costs.insert("module.vpc".to_string(), 1500.0);

        let result = manager.compare_module_costs(&costs, None);
        let formatted = result.format_violations();

        assert!(formatted.contains("1 baseline violations"));
        assert!(formatted.contains("module.vpc"));
        assert!(formatted.contains("High"));
    }

    #[test]
    fn test_filter_by_severity() {
        let config = create_test_config();
        let manager = BaselinesManager::from_config(config);

        let mut costs = HashMap::new();
        costs.insert("module.vpc".to_string(), 1600.0); // Critical

        let result = manager.compare_module_costs(&costs, None);
        assert_eq!(result.critical_violations().len(), 1);
        assert!(result.has_critical_violations());
    }
}
