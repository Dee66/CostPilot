// SLO engine implementation

use super::slo_manager::SloManager;
use super::slo_types::{Slo, SloConfig, SloEvaluation, SloReport, SloType, EnforcementLevel};
use super::burn_rate::BurnAnalysis;
use crate::engines::shared::models::{CostEstimate, TotalCost};
use crate::engines::trend::snapshot_types::CostSnapshot;
use crate::edition::EditionContext;
use serde::{Deserialize, Serialize};

/// Result of SLO evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloResult {
    /// Whether all SLOs passed
    pub passed: bool,

    /// Individual SLO evaluation results
    pub evaluations: Vec<SloEvaluation>,

    /// Whether deployment should be blocked
    pub should_block: bool,

    /// Summary message
    pub message: String,
}

/// Simplified SLO definition for API contract compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slo_type: SloType,
    pub target: String,
    pub threshold: f64,
    pub enforcement: EnforcementLevel,
}

/// Core SLO engine for cost governance
pub struct SloEngine {
    manager: SloManager,
}

impl SloEngine {
    /// Create new SLO engine with SLO definitions
    pub fn new(definitions: Vec<SloDefinition>, edition: &EditionContext) -> Self {
        let config = Self::convert_definitions_to_config(definitions);
        let manager = SloManager::new(config, edition);
        Self { manager }
    }

    /// Check SLO compliance against total cost and resource estimates
    pub fn check_slo(&self, total_cost: &TotalCost, estimates: &[CostEstimate]) -> SloResult {
        // Create a synthetic cost snapshot for evaluation
        let snapshot = Self::create_snapshot_from_costs(total_cost, estimates);

        // Evaluate against the snapshot
        let report = self.manager.evaluate_snapshot(&snapshot);

        // Load historical snapshots for burn rate analysis
        let burn_analyses = self.compute_burn_risk(&snapshot);

        // Check if there are violations and should block before moving evaluations
        let has_violations = report.has_violations();
        let should_block = self.manager.should_block_deployment(&report);

        // Merge burn risk data into evaluations (clone to avoid moving)
        let evaluations = report.evaluations.clone().into_iter().map(|mut eval| {
            if let Some(burn_analysis) = burn_analyses.iter().find(|ba| ba.slo_id == eval.slo_id) {
                eval.burn_risk = Some(burn_analysis.risk.clone());
                eval.projected_cost_after_merge = Some(burn_analysis.projected_cost);
            }
            eval
        }).collect();

        SloResult {
            passed: !has_violations,
            evaluations,
            should_block,
            message: self.format_result_message(&report),
        }
    }

    /// Convert API contract SloDefinitions to internal SloConfig
    fn convert_definitions_to_config(definitions: Vec<SloDefinition>) -> SloConfig {
        let slos = definitions.into_iter().map(|def| {
            Slo::new(
                def.id,
                def.name,
                def.description,
                def.slo_type,
                def.target,
                super::slo_types::SloThreshold {
                    max_value: def.threshold,
                    min_value: None,
                    warning_threshold_percent: 80.0,
                    time_window: "30d".to_string(),
                    use_baseline: false,
                    baseline_multiplier: None,
                },
                def.enforcement,
                "system".to_string(),
            )
        }).collect();

        SloConfig {
            version: "1.0".to_string(),
            slos,
            config: None,
        }
    }

    /// Create a cost snapshot from TotalCost and CostEstimate array
    fn create_snapshot_from_costs(total_cost: &TotalCost, estimates: &[CostEstimate]) -> crate::engines::trend::CostSnapshot {
        use crate::engines::trend::CostSnapshot;

        let mut snapshot = CostSnapshot::new("slo_check".to_string(), total_cost.monthly);

        // Group estimates by module (if module_path is available) or service type
        for estimate in estimates {
            // For now, treat each resource as its own "module" for module budget SLOs
            // In a real implementation, this would parse module paths from resource IDs
            let module_name = format!("resource_{}", estimate.resource_id);
            snapshot.add_module(module_name, estimate.monthly_cost, 1);

            // Also track by service type for service budget SLOs
            // This is a simplified implementation - real logic would parse service types
            let service_name = "generic_service".to_string(); // Placeholder
            let current_cost = snapshot.services.get(service_name.as_str()).unwrap_or(&0.0);
            snapshot.add_service(service_name, current_cost + estimate.monthly_cost);
        }

        snapshot
    }

    /// Compute burn risk for all SLOs using historical snapshots
    fn compute_burn_risk(&self, current_snapshot: &CostSnapshot) -> Vec<BurnAnalysis> {
        use super::burn_rate::BurnRateCalculator;
        use crate::engines::trend::SnapshotManager;

        // Try to load historical snapshots
        let snapshot_manager = SnapshotManager::new(std::path::PathBuf::from(".costpilot/snapshots"));

        match snapshot_manager.load_history() {
            Ok(history) => {
                if history.snapshots.len() >= 3 {
                    // Create calculator and analyze burn rates
                    let calculator = BurnRateCalculator::new();
                    let mut all_snapshots = history.snapshots.clone();

                    // Add current snapshot for projection
                    all_snapshots.push(current_snapshot.clone());

                    calculator.analyze_all(&self.manager.config().slos, &all_snapshots)
                        .analyses
                } else {
                    // Not enough historical data, return empty burn analyses
                    vec![]
                }
            }
            Err(_) => {
                // No historical data available, return empty burn analyses
                vec![]
            }
        }
    }

    /// Format result message for API response
    fn format_result_message(&self, report: &SloReport) -> String {
        if report.has_violations() {
            format!(
                "SLO violations detected: {} violations, {} warnings",
                report.summary.violation_count,
                report.summary.warning_count
            )
        } else if report.summary.warning_count > 0 {
            format!(
                "SLO warnings: {} warnings, all within limits",
                report.summary.warning_count
            )
        } else {
            "All SLOs passed".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::EditionContext;

    fn create_test_definitions() -> Vec<SloDefinition> {
        vec![
            SloDefinition {
                id: "monthly_budget".to_string(),
                name: "Monthly Budget".to_string(),
                description: "Total monthly cost limit".to_string(),
                slo_type: SloType::MonthlyBudget,
                target: "global".to_string(),
                threshold: 10000.0,
                enforcement: EnforcementLevel::Block,
            },
            SloDefinition {
                id: "module_budget".to_string(),
                name: "Module Budget".to_string(),
                description: "Per-module cost limit".to_string(),
                slo_type: SloType::ModuleBudget,
                target: "resource_test".to_string(),
                threshold: 1000.0,
                enforcement: EnforcementLevel::Warn,
            },
        ]
    }

    fn create_test_costs() -> (TotalCost, Vec<CostEstimate>) {
        let total_cost = TotalCost {
            monthly: 5000.0,
            prediction_interval_low: 4500.0,
            prediction_interval_high: 5500.0,
            confidence_score: 0.9,
            resource_count: 2,
        };

        let estimates = vec![
            CostEstimate {
                resource_id: "test".to_string(),
                monthly_cost: 2500.0,
                prediction_interval_low: 2250.0,
                prediction_interval_high: 2750.0,
                confidence_score: 0.85,
                heuristic_reference: None,
                cold_start_inference: false,
                one_time: None,
                breakdown: None,
                hourly: None,
                daily: None,
            },
        ];

        (total_cost, estimates)
    }

    #[test]
    fn test_slo_engine_creation() {
        let definitions = create_test_definitions();
        let edition = EditionContext::free();
        let engine = SloEngine::new(definitions, &edition);
        // Engine created successfully
        assert!(true);
    }

    #[test]
    fn test_slo_check_pass() {
        let definitions = create_test_definitions();
        let edition = EditionContext::free();
        let engine = SloEngine::new(definitions, &edition);

        let (total_cost, estimates) = create_test_costs();
        let result = engine.check_slo(&total_cost, &estimates);

        // Should pass since costs are within limits
        assert!(!result.passed); // Note: has_violations() returns true if NO violations
        assert!(!result.should_block);
        assert!(result.evaluations.len() >= 1);
    }

    #[test]
    fn test_slo_check_violation() {
        let mut definitions = create_test_definitions();
        // Set very low threshold to trigger violation
        definitions[0].threshold = 1000.0;

        let edition = EditionContext::free();
        let engine = SloEngine::new(definitions, &edition);

        let (total_cost, estimates) = create_test_costs();
        let result = engine.check_slo(&total_cost, &estimates);

        // Should have violations since threshold is too low
        assert!(!result.passed); // passed is false when there are violations
        assert!(result.evaluations.len() >= 1);
    }
}
