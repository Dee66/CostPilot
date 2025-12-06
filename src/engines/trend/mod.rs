// Trend engine module for cost tracking and visualization

mod html_generator;
mod snapshot_manager;
mod snapshot_types;
mod svg_generator;

pub use html_generator::HtmlGenerator;
pub use snapshot_manager::SnapshotManager;
pub use snapshot_types::*;
pub use svg_generator::{SvgConfig, SvgGenerator};

use crate::engines::baselines::{BaselinesManager, BaselineViolation};
use crate::engines::detection::ResourceChange;
use crate::engines::prediction::PredictionEngine;
use crate::errors::CostPilotError;

/// Main trend engine for cost tracking
pub struct TrendEngine {
    snapshot_manager: SnapshotManager,
    svg_generator: SvgGenerator,
}

impl TrendEngine {
    /// Create a new trend engine with storage directory
    pub fn new<P: AsRef<std::path::Path>>(storage_dir: P) -> Self {
        Self {
            snapshot_manager: SnapshotManager::new(storage_dir),
            svg_generator: SvgGenerator::new(),
        }
    }

    /// Create a snapshot from resource changes
    pub fn create_snapshot(
        &self,
        changes: &[ResourceChange],
        commit_hash: Option<String>,
        branch: Option<String>,
    ) -> Result<CostSnapshot, CostPilotError> {
        let id = SnapshotManager::generate_snapshot_id();
        
        // Calculate total cost
        let prediction_engine = PredictionEngine::new();
        let total_cost = prediction_engine.predict_total_cost(changes)?;
        
        let mut snapshot = CostSnapshot::new(id, total_cost.monthly);
        snapshot.commit_hash = commit_hash;
        snapshot.branch = branch;
        
        // Group by module (simplified - uses resource_id prefix)
        let mut modules = std::collections::HashMap::new();
        for change in changes {
            let module_name = self.extract_module_name(&change.resource_id);
            let entry = modules.entry(module_name.clone()).or_insert((0usize, 0.0));
            entry.0 += 1; // resource count
            
            // Estimate cost for this resource
            let resource_cost = prediction_engine.predict_resource_cost(change)
                .unwrap_or_else(|_| crate::engines::prediction::CostEstimate {
                    hourly: 0.0,
                    daily: 0.0,
                    monthly: 0.0,
                    confidence: 0.0,
                });
            entry.1 += resource_cost.monthly;
        }
        
        for (name, (count, cost)) in modules {
            snapshot.add_module(name, cost, count);
        }
        
        // Group by service (extract from resource_type)
        let mut services = std::collections::HashMap::new();
        for change in changes {
            let service = self.extract_service_name(&change.resource_type);
            let resource_cost = prediction_engine.predict_resource_cost(change)
                .unwrap_or_else(|_| crate::engines::prediction::CostEstimate {
                    hourly: 0.0,
                    daily: 0.0,
                    monthly: 0.0,
                    confidence: 0.0,
                });
            *services.entry(service).or_insert(0.0) += resource_cost.monthly;
        }
        
        for (service, cost) in services {
            snapshot.add_service(service, cost);
        }
        
        Ok(snapshot)
    }

    /// Save a snapshot to storage
    pub fn save_snapshot(&self, snapshot: &CostSnapshot) -> Result<std::path::PathBuf, CostPilotError> {
        self.snapshot_manager.write_snapshot(snapshot)
    }

    /// Load trend history from storage
    pub fn load_history(&self) -> Result<TrendHistory, CostPilotError> {
        self.snapshot_manager.load_history()
    }

    /// Generate SVG graph from history
    pub fn generate_svg(&self) -> Result<String, CostPilotError> {
        let history = self.load_history()?;
        
        self.svg_generator
            .generate(&history)
            .map_err(|e| CostPilotError::GenerationError(e))
    }

    /// Generate HTML file with embedded SVG
    pub fn generate_html<P: AsRef<std::path::Path>>(
        &self,
        output_path: P,
        title: &str,
    ) -> Result<(), CostPilotError> {
        let svg = self.generate_svg()?;
        HtmlGenerator::generate_file(output_path, &svg, title)
    }

    /// Detect regressions by comparing with baseline
    pub fn detect_regressions(
        &self,
        snapshot: &CostSnapshot,
        baseline: &CostSnapshot,
        threshold_percent: f64,
    ) -> Vec<Regression> {
        let mut regressions = Vec::new();

        // Check total cost increase
        let total_increase = snapshot.total_monthly_cost - baseline.total_monthly_cost;
        let total_percent = if baseline.total_monthly_cost > 0.0 {
            (total_increase / baseline.total_monthly_cost) * 100.0
        } else {
            0.0
        };

        if total_percent > threshold_percent {
            regressions.push(Regression {
                regression_type: RegressionType::CostIncrease,
                affected: "total".to_string(),
                baseline_cost: baseline.total_monthly_cost,
                current_cost: snapshot.total_monthly_cost,
                increase_amount: total_increase,
                increase_percent: total_percent,
                severity: if total_percent > 50.0 {
                    "CRITICAL"
                } else if total_percent > 25.0 {
                    "HIGH"
                } else {
                    "MEDIUM"
                }
                .to_string(),
            });
        }

        // Check module-level regressions
        for (module_name, current_module) in &snapshot.modules {
            if let Some(baseline_module) = baseline.modules.get(module_name) {
                let increase = current_module.monthly_cost - baseline_module.monthly_cost;
                let percent = if baseline_module.monthly_cost > 0.0 {
                    (increase / baseline_module.monthly_cost) * 100.0
                } else {
                    0.0
                };

                if percent > threshold_percent {
                    regressions.push(Regression {
                        regression_type: RegressionType::CostIncrease,
                        affected: module_name.clone(),
                        baseline_cost: baseline_module.monthly_cost,
                        current_cost: current_module.monthly_cost,
                        increase_amount: increase,
                        increase_percent: percent,
                        severity: if percent > 50.0 {
                            "HIGH"
                        } else {
                            "MEDIUM"
                        }
                        .to_string(),
                    });
                }
            } else {
                // New module
                regressions.push(Regression {
                    regression_type: RegressionType::NewResource,
                    affected: module_name.clone(),
                    baseline_cost: 0.0,
                    current_cost: current_module.monthly_cost,
                    increase_amount: current_module.monthly_cost,
                    increase_percent: 100.0,
                    severity: "MEDIUM".to_string(),
                });
            }
        }

        regressions
    }

    /// Detect baseline violations by comparing snapshot against baselines
    pub fn detect_baseline_violations(
        &self,
        snapshot: &CostSnapshot,
        baselines: &BaselinesManager,
    ) -> Vec<BaselineViolation> {
        let mut violations = Vec::new();

        // Check global baseline
        if let Some(global_violation) = baselines.compare_total_cost(snapshot.total_monthly_cost) {
            violations.push(global_violation);
        }

        // Extract module costs from snapshot
        let mut module_costs = std::collections::HashMap::new();
        for (module_name, module_cost) in &snapshot.modules {
            module_costs.insert(module_name.clone(), module_cost.monthly_cost);
        }

        // Check module baselines
        let comparison = baselines.compare_module_costs(&module_costs);
        violations.extend(comparison.violations);

        violations
    }

    /// Create snapshot with baseline comparison
    pub fn create_snapshot_with_baselines(
        &self,
        changes: &[ResourceChange],
        commit_hash: Option<String>,
        branch: Option<String>,
        baselines: &BaselinesManager,
    ) -> Result<CostSnapshot, CostPilotError> {
        // Create base snapshot
        let mut snapshot = self.create_snapshot(changes, commit_hash, branch)?;

        // Detect baseline violations
        let violations = self.detect_baseline_violations(&snapshot, baselines);

        // Convert violations to regressions format for compatibility
        for violation in violations {
            let regression_type = if violation.actual_cost > violation.expected_cost {
                RegressionType::BudgetExceeded
            } else {
                RegressionType::CostIncrease // Using existing type for below-baseline
            };

            snapshot.regressions.push(Regression {
                regression_type,
                affected: violation.name.clone(),
                baseline_cost: violation.expected_cost,
                current_cost: violation.actual_cost,
                increase_amount: violation.actual_cost - violation.expected_cost,
                increase_percent: violation.variance_percent,
                severity: violation.severity,
            });
        }

        Ok(snapshot)
    }

    /// Compare snapshot against baselines and annotate
    pub fn annotate_with_baselines(
        &self,
        snapshot: &mut CostSnapshot,
        baselines: &BaselinesManager,
    ) {
        let violations = self.detect_baseline_violations(snapshot, baselines);

        // Add violations as regressions
        for violation in violations {
            let regression_type = if violation.actual_cost > violation.expected_cost {
                RegressionType::BudgetExceeded
            } else {
                RegressionType::CostIncrease
            };

            snapshot.regressions.push(Regression {
                regression_type,
                affected: violation.name.clone(),
                baseline_cost: violation.expected_cost,
                current_cost: violation.actual_cost,
                increase_amount: violation.actual_cost - violation.expected_cost,
                increase_percent: violation.variance_percent,
                severity: violation.severity,
            });
        }
    }

    /// Rotate old snapshots
    pub fn rotate_snapshots(&self) -> Result<usize, CostPilotError> {
        self.snapshot_manager.rotate_snapshots()
    }

    /// Extract module name from resource ID
    fn extract_module_name(&self, resource_id: &str) -> String {
        // Extract module from resource ID like "module.vpc.aws_nat_gateway.main"
        if resource_id.starts_with("module.") {
            let parts: Vec<&str> = resource_id.split('.').collect();
            if parts.len() >= 2 {
                return format!("module.{}", parts[1]);
            }
        }
        "root".to_string()
    }

    /// Extract service name from resource type
    fn extract_service_name(&self, resource_type: &str) -> String {
        // Extract service from resource type like "aws_nat_gateway" -> "NAT Gateway"
        if resource_type.starts_with("aws_") {
            let service = resource_type.trim_start_matches("aws_");
            let words: Vec<String> = service
                .split('_')
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect();
            return words.join(" ");
        }
        resource_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_trend_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());
        
        // Should be able to load empty history
        let history = engine.load_history();
        assert!(history.is_ok());
    }

    #[test]
    fn test_extract_module_name() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());
        
        assert_eq!(
            engine.extract_module_name("module.vpc.aws_nat_gateway.main"),
            "module.vpc"
        );
        assert_eq!(
            engine.extract_module_name("aws_instance.web"),
            "root"
        );
    }

    #[test]
    fn test_extract_service_name() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());
        
        assert_eq!(
            engine.extract_service_name("aws_nat_gateway"),
            "Nat Gateway"
        );
        assert_eq!(
            engine.extract_service_name("aws_s3_bucket"),
            "S3 Bucket"
        );
    }

    #[test]
    fn test_detect_regressions() {
        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());
        
        let baseline = CostSnapshot::new("baseline".to_string(), 1000.0);
        let mut current = CostSnapshot::new("current".to_string(), 1300.0);
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        
        assert!(!regressions.is_empty());
        assert_eq!(regressions[0].increase_percent, 30.0);
    }

    #[test]
    fn test_detect_baseline_violations() {
        use crate::engines::baselines::{Baseline, BaselinesConfig, BaselinesManager};

        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());

        // Create baselines
        let mut config = BaselinesConfig::new();
        let global = Baseline::new(
            "global".to_string(),
            1000.0,
            "Global baseline".to_string(),
            "owner".to_string(),
        );
        config.set_global(global);

        let baselines = BaselinesManager::from_config(config);

        // Create snapshot that exceeds baseline
        let snapshot = CostSnapshot::new("test".to_string(), 1500.0);

        let violations = engine.detect_baseline_violations(&snapshot, &baselines);

        assert!(!violations.is_empty());
        assert_eq!(violations[0].baseline_type, "global");
        assert_eq!(violations[0].expected_cost, 1000.0);
        assert_eq!(violations[0].actual_cost, 1500.0);
    }

    #[test]
    fn test_annotate_with_baselines() {
        use crate::engines::baselines::{Baseline, BaselinesConfig, BaselinesManager};
        use std::collections::HashMap;

        let temp_dir = TempDir::new().unwrap();
        let engine = TrendEngine::new(temp_dir.path());

        // Create baselines with module
        let mut config = BaselinesConfig::new();
        let module_baseline = Baseline::new(
            "module.vpc".to_string(),
            1000.0,
            "VPC baseline".to_string(),
            "owner".to_string(),
        );
        config.add_module("module.vpc".to_string(), module_baseline);

        let baselines = BaselinesManager::from_config(config);

        // Create snapshot with module that exceeds baseline
        let mut snapshot = CostSnapshot::new("test".to_string(), 1500.0);
        let mut modules = HashMap::new();
        modules.insert(
            "module.vpc".to_string(),
            ModuleCost {
                name: "module.vpc".to_string(),
                monthly_cost: 1500.0,
                resource_count: 5,
                change_from_previous: None,
                change_percent: None,
            },
        );
        snapshot.modules = modules;

        engine.annotate_with_baselines(&mut snapshot, &baselines);

        assert!(!snapshot.regressions.is_empty());
        assert_eq!(snapshot.regressions[0].affected, "module.vpc");
        assert_eq!(snapshot.regressions[0].regression_type, RegressionType::BudgetExceeded);
    }
}
