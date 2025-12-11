// Trend diff generator - compares two snapshots and generates human-readable diff

use crate::engines::trend::snapshot_types::{CostSnapshot, ModuleCost, Regression};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Diff between two cost snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDiff {
    /// Earlier snapshot ID
    pub from_snapshot: String,

    /// Later snapshot ID
    pub to_snapshot: String,

    /// Time range
    pub time_range: String,

    /// Total cost change
    pub total_cost_delta: f64,

    /// Percentage change
    pub total_cost_percent: f64,

    /// Module-level changes
    pub module_changes: Vec<ModuleChange>,

    /// Service-level changes
    pub service_changes: Vec<ServiceChange>,

    /// New regressions introduced
    pub new_regressions: Vec<Regression>,

    /// Summary
    pub summary: DiffSummary,
}

/// Change in a specific module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleChange {
    /// Module name
    pub module: String,

    /// Cost before
    pub cost_before: f64,

    /// Cost after
    pub cost_after: f64,

    /// Delta
    pub delta: f64,

    /// Percentage change
    pub percent: f64,

    /// Change type
    pub change_type: ChangeType,
}

/// Change in a specific service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChange {
    /// Service name (e.g., "EC2", "RDS")
    pub service: String,

    /// Cost before
    pub cost_before: f64,

    /// Cost after
    pub cost_after: f64,

    /// Delta
    pub delta: f64,

    /// Percentage change
    pub percent: f64,
}

/// Type of change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// New module added
    Added,

    /// Module removed
    Removed,

    /// Cost increased
    Increased,

    /// Cost decreased
    Decreased,

    /// No change
    Unchanged,
}

/// Summary statistics for diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Number of modules added
    pub modules_added: usize,

    /// Number of modules removed
    pub modules_removed: usize,

    /// Number of modules with cost increases
    pub modules_increased: usize,

    /// Number of modules with cost decreases
    pub modules_decreased: usize,

    /// Number of modules unchanged
    pub modules_unchanged: usize,

    /// New regressions count
    pub new_regressions: usize,

    /// Trend direction
    pub trend: TrendDirection,
}

/// Overall trend direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Costs increasing
    Rising,

    /// Costs decreasing
    Falling,

    /// Costs stable
    Stable,
}

/// Trend diff generator
pub struct TrendDiffGenerator;

impl TrendDiffGenerator {
    /// Generate diff between two snapshots
    pub fn generate_diff(from: &CostSnapshot, to: &CostSnapshot) -> TrendDiff {
        let total_cost_delta = to.total_monthly_cost - from.total_monthly_cost;
        let total_cost_percent = if from.total_monthly_cost > 0.0 {
            (total_cost_delta / from.total_monthly_cost) * 100.0
        } else {
            0.0
        };

        let module_changes = Self::calculate_module_changes(&from.modules, &to.modules);
        let service_changes = Self::calculate_service_changes(&from.services, &to.services);

        // Find new regressions (in "to" but not in "from")
        let from_regression_ids: std::collections::HashSet<_> = from
            .regressions
            .iter()
            .map(|r| format!("{}_{}", r.regression_type.clone() as u8, r.affected))
            .collect();

        let new_regressions: Vec<_> = to
            .regressions
            .iter()
            .filter(|r| {
                !from_regression_ids.contains(&format!(
                    "{}_{}",
                    r.regression_type.clone() as u8,
                    r.affected
                ))
            })
            .cloned()
            .collect();

        let summary = Self::generate_summary(&module_changes, &new_regressions, total_cost_delta);

        let time_range = format!("{} → {}", from.timestamp, to.timestamp);

        TrendDiff {
            from_snapshot: from.id.clone(),
            to_snapshot: to.id.clone(),
            time_range,
            total_cost_delta,
            total_cost_percent,
            module_changes,
            service_changes,
            new_regressions,
            summary,
        }
    }

    /// Calculate module-level changes
    fn calculate_module_changes(
        from_modules: &HashMap<String, ModuleCost>,
        to_modules: &HashMap<String, ModuleCost>,
    ) -> Vec<ModuleChange> {
        let mut changes = Vec::new();

        // Find all unique module names
        let mut all_modules: std::collections::HashSet<String> = std::collections::HashSet::new();
        all_modules.extend(from_modules.keys().cloned());
        all_modules.extend(to_modules.keys().cloned());

        for module_name in all_modules {
            let from_cost = from_modules
                .get(&module_name)
                .map(|m| m.monthly_cost)
                .unwrap_or(0.0);
            let to_cost = to_modules
                .get(&module_name)
                .map(|m| m.monthly_cost)
                .unwrap_or(0.0);

            let delta = to_cost - from_cost;
            let percent = if from_cost > 0.0 {
                (delta / from_cost) * 100.0
            } else if to_cost > 0.0 {
                100.0
            } else {
                0.0
            };

            let change_type = if from_cost == 0.0 && to_cost > 0.0 {
                ChangeType::Added
            } else if from_cost > 0.0 && to_cost == 0.0 {
                ChangeType::Removed
            } else if delta > 0.01 {
                ChangeType::Increased
            } else if delta < -0.01 {
                ChangeType::Decreased
            } else {
                ChangeType::Unchanged
            };

            changes.push(ModuleChange {
                module: module_name,
                cost_before: from_cost,
                cost_after: to_cost,
                delta,
                percent,
                change_type,
            });
        }

        // Sort by absolute delta (largest changes first)
        changes.sort_by(|a, b| b.delta.abs().partial_cmp(&a.delta.abs()).unwrap());

        changes
    }

    /// Calculate service-level changes
    fn calculate_service_changes(
        from_services: &HashMap<String, f64>,
        to_services: &HashMap<String, f64>,
    ) -> Vec<ServiceChange> {
        let mut changes = Vec::new();

        let mut all_services: std::collections::HashSet<String> = std::collections::HashSet::new();
        all_services.extend(from_services.keys().cloned());
        all_services.extend(to_services.keys().cloned());

        for service_name in all_services {
            let from_cost = from_services.get(&service_name).copied().unwrap_or(0.0);
            let to_cost = to_services.get(&service_name).copied().unwrap_or(0.0);

            let delta = to_cost - from_cost;
            let percent = if from_cost > 0.0 {
                (delta / from_cost) * 100.0
            } else if to_cost > 0.0 {
                100.0
            } else {
                0.0
            };

            changes.push(ServiceChange {
                service: service_name,
                cost_before: from_cost,
                cost_after: to_cost,
                delta,
                percent,
            });
        }

        // Sort by absolute delta
        changes.sort_by(|a, b| b.delta.abs().partial_cmp(&a.delta.abs()).unwrap());

        changes
    }

    /// Generate summary statistics
    fn generate_summary(
        module_changes: &[ModuleChange],
        new_regressions: &[Regression],
        total_cost_delta: f64,
    ) -> DiffSummary {
        let mut modules_added = 0;
        let mut modules_removed = 0;
        let mut modules_increased = 0;
        let mut modules_decreased = 0;
        let mut modules_unchanged = 0;

        for change in module_changes {
            match change.change_type {
                ChangeType::Added => modules_added += 1,
                ChangeType::Removed => modules_removed += 1,
                ChangeType::Increased => modules_increased += 1,
                ChangeType::Decreased => modules_decreased += 1,
                ChangeType::Unchanged => modules_unchanged += 1,
            }
        }

        let trend = if total_cost_delta > 1.0 {
            TrendDirection::Rising
        } else if total_cost_delta < -1.0 {
            TrendDirection::Falling
        } else {
            TrendDirection::Stable
        };

        DiffSummary {
            modules_added,
            modules_removed,
            modules_increased,
            modules_decreased,
            modules_unchanged,
            new_regressions: new_regressions.len(),
            trend,
        }
    }

    /// Format diff as human-readable text
    pub fn format_text(diff: &TrendDiff) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "# Trend Diff: {} → {}\n\n",
            diff.from_snapshot, diff.to_snapshot
        ));
        output.push_str(&format!("Time Range: {}\n\n", diff.time_range));

        // Total cost change
        output.push_str("## Total Cost Change\n\n");
        let sign = if diff.total_cost_delta >= 0.0 {
            "+"
        } else {
            ""
        };
        output.push_str(&format!(
            "**{}${:.2}** ({}{:.1}%)\n\n",
            sign, diff.total_cost_delta, sign, diff.total_cost_percent
        ));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- Modules Added: {}\n",
            diff.summary.modules_added
        ));
        output.push_str(&format!(
            "- Modules Removed: {}\n",
            diff.summary.modules_removed
        ));
        output.push_str(&format!(
            "- Modules Increased: {}\n",
            diff.summary.modules_increased
        ));
        output.push_str(&format!(
            "- Modules Decreased: {}\n",
            diff.summary.modules_decreased
        ));
        output.push_str(&format!(
            "- Modules Unchanged: {}\n",
            diff.summary.modules_unchanged
        ));
        output.push_str(&format!(
            "- New Regressions: {}\n",
            diff.summary.new_regressions
        ));
        output.push_str(&format!("- Trend: {:?}\n\n", diff.summary.trend));

        // Module changes (top 10)
        if !diff.module_changes.is_empty() {
            output.push_str("## Top Module Changes\n\n");
            for change in diff.module_changes.iter().take(10) {
                let sign = if change.delta >= 0.0 { "+" } else { "" };
                output.push_str(&format!(
                    "- **{}**: {}${:.2} ({}{:.1}%) - {:?}\n",
                    change.module, sign, change.delta, sign, change.percent, change.change_type
                ));
            }
            output.push('\n');
        }

        // Service changes (top 5)
        if !diff.service_changes.is_empty() {
            output.push_str("## Top Service Changes\n\n");
            for change in diff.service_changes.iter().take(5) {
                let sign = if change.delta >= 0.0 { "+" } else { "" };
                output.push_str(&format!(
                    "- **{}**: {}${:.2} ({}{:.1}%)\n",
                    change.service, sign, change.delta, sign, change.percent
                ));
            }
            output.push('\n');
        }

        // New regressions
        if !diff.new_regressions.is_empty() {
            output.push_str("## New Regressions\n\n");
            for regression in &diff.new_regressions {
                output.push_str(&format!(
                    "- **{}** ({:?}): ${:.2} → ${:.2} (+${:.2}, +{:.1}%) - Severity: {}\n",
                    regression.affected,
                    regression.regression_type,
                    regression.baseline_cost,
                    regression.current_cost,
                    regression.increase_amount,
                    regression.increase_percent,
                    regression.severity
                ));
            }
        }

        output
    }

    /// Format diff as JSON
    pub fn format_json(diff: &TrendDiff) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_snapshot(
        id: &str,
        total_cost: f64,
        modules: HashMap<String, ModuleCost>,
    ) -> CostSnapshot {
        CostSnapshot {
            id: id.to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            commit_hash: None,
            branch: None,
            total_monthly_cost: total_cost,
            modules,
            services: HashMap::new(),
            regressions: Vec::new(),
            slo_violations: Vec::new(),
            metadata: None,
        }
    }

    #[test]
    fn test_generate_diff_cost_increase() {
        let mut from_modules = HashMap::new();
        from_modules.insert(
            "app".to_string(),
            ModuleCost {
                name: "app".to_string(),
                monthly_cost: 100.0,
                resource_count: 5,
                change_from_previous: None,
                change_percent: None,
                services: vec![],
            },
        );

        let mut to_modules = HashMap::new();
        to_modules.insert(
            "app".to_string(),
            ModuleCost {
                name: "app".to_string(),
                monthly_cost: 150.0,
                resource_count: 7,
                change_from_previous: Some(50.0),
                change_percent: Some(50.0),
                services: vec![],
            },
        );

        let from = create_test_snapshot("snap1", 100.0, from_modules);
        let to = create_test_snapshot("snap2", 150.0, to_modules);

        let diff = TrendDiffGenerator::generate_diff(&from, &to);

        assert_eq!(diff.total_cost_delta, 50.0);
        assert_eq!(diff.total_cost_percent, 50.0);
        assert_eq!(diff.summary.trend, TrendDirection::Rising);
        assert_eq!(diff.module_changes.len(), 1);
        assert_eq!(diff.module_changes[0].change_type, ChangeType::Increased);
    }

    #[test]
    fn test_generate_diff_new_module() {
        let from_modules = HashMap::new();

        let mut to_modules = HashMap::new();
        to_modules.insert(
            "database".to_string(),
            ModuleCost {
                name: "database".to_string(),
                monthly_cost: 200.0,
                resource_count: 3,
                change_from_previous: None,
                change_percent: None,
                services: vec![],
            },
        );

        let from = create_test_snapshot("snap1", 0.0, from_modules);
        let to = create_test_snapshot("snap2", 200.0, to_modules);

        let diff = TrendDiffGenerator::generate_diff(&from, &to);

        assert_eq!(diff.summary.modules_added, 1);
        assert_eq!(diff.module_changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_format_text() {
        let from_modules = HashMap::new();
        let to_modules = HashMap::new();

        let from = create_test_snapshot("snap1", 100.0, from_modules);
        let to = create_test_snapshot("snap2", 150.0, to_modules);

        let diff = TrendDiffGenerator::generate_diff(&from, &to);
        let text = TrendDiffGenerator::format_text(&diff);

        assert!(text.contains("Trend Diff"));
        assert!(text.contains("+$50.00"));
        assert!(text.contains("Summary"));
    }

    #[test]
    fn test_format_json() {
        let from_modules = HashMap::new();
        let to_modules = HashMap::new();

        let from = create_test_snapshot("snap1", 100.0, from_modules);
        let to = create_test_snapshot("snap2", 120.0, to_modules);

        let diff = TrendDiffGenerator::generate_diff(&from, &to);
        let json = TrendDiffGenerator::format_json(&diff).unwrap();

        assert!(json.contains("\"total_cost_delta\""));
        assert!(json.contains("\"summary\""));
    }
}
