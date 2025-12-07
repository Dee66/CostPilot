// Prediction engine - deterministic cost estimation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::engines::shared::models::{ResourceChange, CostEstimate, ChangeAction};
use crate::engines::shared::error_model::{Result, CostPilotError, ErrorCategory};
use crate::engines::prediction::cold_start::ColdStartInference;
use crate::engines::prediction::confidence::calculate_confidence;
use crate::engines::prediction::heuristics_loader::HeuristicsLoader;
use crate::engines::performance::budgets::{PerformanceTracker, PerformanceBudgets, BudgetViolation, TimeoutAction};

/// Cost heuristics database
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CostHeuristics {
    pub version: String,
    pub last_updated: String,
    pub compute: ComputeHeuristics,
    pub storage: StorageHeuristics,
    pub database: DatabaseHeuristics,
    pub networking: NetworkingHeuristics,
    pub cold_start_defaults: ColdStartDefaults,
    pub prediction_intervals: PredictionIntervals,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComputeHeuristics {
    pub ec2: HashMap<String, InstanceCost>,
    pub lambda: LambdaCost,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstanceCost {
    pub hourly: f64,
    pub monthly: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LambdaCost {
    pub price_per_gb_second: f64,
    pub price_per_request: f64,
    pub free_tier_requests: u64,
    pub free_tier_compute_gb_seconds: u64,
    pub default_memory_mb: u32,
    pub default_duration_ms: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageHeuristics {
    pub s3: S3Cost,
    pub ebs: HashMap<String, EbsCost>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Cost {
    pub standard: S3Tier,
    pub glacier: S3Tier,
    pub requests: S3Requests,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Tier {
    pub per_gb: Option<f64>,
    pub first_50tb_per_gb: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Requests {
    pub put_copy_post_list_per_1000: f64,
    pub get_select_per_1000: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EbsCost {
    pub per_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseHeuristics {
    pub rds: RdsCost,
    pub dynamodb: DynamoDbCost,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RdsCost {
    pub mysql: HashMap<String, InstanceCost>,
    pub postgres: HashMap<String, InstanceCost>,
    pub storage_gp2_per_gb: f64,
    pub storage_gp3_per_gb: f64,
    pub backup_per_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamoDbCost {
    pub on_demand: DynamoDbOnDemand,
    pub provisioned: DynamoDbProvisioned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamoDbOnDemand {
    pub write_request_unit: f64,
    pub read_request_unit: f64,
    pub storage_per_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamoDbProvisioned {
    pub write_capacity_unit_hourly: f64,
    pub read_capacity_unit_hourly: f64,
    pub storage_per_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkingHeuristics {
    pub nat_gateway: NatGatewayCost,
    pub load_balancer: LoadBalancerCost,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NatGatewayCost {
    pub hourly: f64,
    pub monthly: f64,
    pub data_processing_per_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoadBalancerCost {
    pub alb: LoadBalancerType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoadBalancerType {
    pub hourly: f64,
    pub monthly: f64,
    pub lcu_hourly: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColdStartDefaults {
    pub dynamodb_unknown_rcu: u32,
    pub dynamodb_unknown_wcu: u32,
    pub lambda_default_invocations: u64,
    pub nat_gateway_default_gb: u32,
    pub s3_default_gb: u32,
    pub ec2_default_utilization: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredictionIntervals {
    pub range_factor: f64,
}

/// Main prediction engine
pub struct PredictionEngine {
    heuristics: CostHeuristics,
    cold_start: ColdStartInference,
    verbose: bool,
    performance_tracker: Option<PerformanceTracker>,
}

impl PredictionEngine {
    /// Create a new prediction engine with automatic heuristics discovery
    pub fn new() -> Result<Self> {
        let loader = HeuristicsLoader::new();
        let heuristics = loader.load()?;
        
        Ok(Self {
            cold_start: ColdStartInference::new(&heuristics.cold_start_defaults),
            heuristics,
            verbose: false,
            performance_tracker: None,
        })
    }

    /// Create a new prediction engine from specific heuristics file
    pub fn from_file(heuristics_path: &Path) -> Result<Self> {
        let loader = HeuristicsLoader::new();
        let heuristics = loader.load_from_file(heuristics_path)?;

        Ok(Self {
            cold_start: ColdStartInference::new(&heuristics.cold_start_defaults),
            heuristics,
            verbose: false,
            performance_tracker: None,
        })
    }

    /// Create prediction engine from heuristics object (for testing)
    pub fn with_heuristics(heuristics: CostHeuristics) -> Self {
        Self {
            cold_start: ColdStartInference::new(&heuristics.cold_start_defaults),
            heuristics,
            verbose: false,
            performance_tracker: None,
        }
    }

    /// Enable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable performance tracking with budgets
    pub fn with_performance_tracking(mut self, budgets: PerformanceBudgets) -> Self {
        self.performance_tracker = Some(PerformanceTracker::new(budgets.prediction, "Prediction".to_string()));
        self
    }

    /// Predict costs for resource changes
    pub fn predict(&mut self, changes: &[ResourceChange]) -> Result<Vec<CostEstimate>> {
        // Check budget before starting
        if let Some(tracker) = &self.performance_tracker {
            if let Err(violation) = tracker.check_budget() {
                return self.handle_budget_violation(violation);
            }
        }

        let mut estimates = Vec::new();

        for change in changes {
            // Check budget periodically during processing
            if let Some(tracker) = &self.performance_tracker {
                if let Err(violation) = tracker.check_budget() {
                    return self.handle_budget_violation_with_partial(violation, estimates);
                }
            }

            if let Some(estimate) = self.predict_resource(change)? {
                estimates.push(estimate);
            }
        }

        // Mark completion and collect metrics
        if let Some(tracker) = &mut self.performance_tracker {
            let _metrics = tracker.complete();
            // TODO: Log or return metrics
        }

        Ok(estimates)
    }

    /// Handle budget violation based on timeout action
    fn handle_budget_violation(&self, violation: BudgetViolation) -> Result<Vec<CostEstimate>> {
        match violation.action {
            TimeoutAction::PartialResults => {
                if self.verbose {
                    eprintln!("⚠️  Budget exceeded: {:?} ({}ms budget, {}ms elapsed)",
                        violation.violation_type, violation.budget_value, violation.actual_value);
                    eprintln!("   Returning empty results");
                }
                Ok(Vec::new())
            }
            TimeoutAction::Error => {
                Err(CostPilotError::new(
                    "PREDICT_TIMEOUT",
                    ErrorCategory::Timeout,
                    &format!("Prediction exceeded budget: {:?} ({}ms budget, {}ms elapsed)",
                        violation.violation_type, violation.budget_value, violation.actual_value)
                ))
            }
            TimeoutAction::CircuitBreak => {
                Err(CostPilotError::new(
                    "PREDICT_CIRCUIT_BREAK",
                    ErrorCategory::CircuitBreaker,
                    &format!("Circuit breaker triggered: {:?} ({}ms budget, {}ms elapsed)",
                        violation.violation_type, violation.budget_value, violation.actual_value)
                ))
            }
        }
    }

    /// Handle budget violation with partial results
    fn handle_budget_violation_with_partial(&self, violation: BudgetViolation, partial: Vec<CostEstimate>) -> Result<Vec<CostEstimate>> {
        match violation.action {
            TimeoutAction::PartialResults => {
                if self.verbose {
                    eprintln!("⚠️  Budget exceeded: {:?} ({}ms budget, {}ms elapsed)",
                        violation.violation_type, violation.budget_value, violation.actual_value);
                    eprintln!("   Returning {} partial results", partial.len());
                }
                Ok(partial)
            }
            TimeoutAction::Error => {
                Err(CostPilotError::new(
                    "PREDICT_TIMEOUT",
                    ErrorCategory::Timeout,
                    &format!("Prediction exceeded budget: {:?} ({}ms budget, {}ms elapsed) - {} partial results discarded",
                        violation.violation_type, violation.budget_value, violation.actual_value, partial.len())
                ))
            }
            TimeoutAction::CircuitBreak => {
                Err(CostPilotError::new(
                    "PREDICT_CIRCUIT_BREAK",
                    ErrorCategory::CircuitBreaker,
                    &format!("Circuit breaker triggered: {:?} ({}ms budget, {}ms elapsed) - {} partial results discarded",
                        violation.violation_type, violation.budget_value, violation.actual_value, partial.len())
                ))
            }
        }
    }

    /// Predict cost for a single resource
    fn predict_resource(&self, change: &ResourceChange) -> Result<Option<CostEstimate>> {
        let monthly_cost = match change.resource_type.as_str() {
            "aws_instance" => self.predict_ec2(change)?,
            "aws_rds_instance" => self.predict_rds(change)?,
            "aws_dynamodb_table" => self.predict_dynamodb(change)?,
            "aws_nat_gateway" => self.predict_nat_gateway(change)?,
            "aws_lb" | "aws_alb" => self.predict_load_balancer(change)?,
            "aws_s3_bucket" => self.predict_s3(change)?,
            "aws_lambda_function" => self.predict_lambda(change)?,
            _ => {
                if self.verbose {
                    println!("Unknown resource type: {}", change.resource_type);
                }
                return Ok(None);
            }
        };

        let (old_cost, cold_start_used) = match change.action {
            ChangeAction::Delete => (monthly_cost, false),
            _ => (0.0, false),
        };

        let cost_delta = monthly_cost - old_cost;
        let confidence = calculate_confidence(change, cold_start_used, &change.resource_type);

        let range_factor = self.heuristics.prediction_intervals.range_factor;
        let interval = monthly_cost * range_factor;

        Ok(Some(CostEstimate {
            resource_id: change.resource_id.clone(),
            monthly_cost: cost_delta,
            prediction_interval_low: (monthly_cost - interval).max(0.0),
            prediction_interval_high: monthly_cost + interval,
            confidence_score: confidence,
            heuristic_reference: Some(format!("v{}", self.heuristics.version)),
            cold_start_inference: cold_start_used,
        }))
    }

    /// Predict EC2 instance cost
    fn predict_ec2(&self, change: &ResourceChange) -> Result<f64> {
        let config = change.new_config.as_ref().ok_or_else(|| {
            CostPilotError::new("PREDICT_003", ErrorCategory::InvalidInput, "Missing EC2 configuration")
        })?;

        let instance_type = config
            .get("instance_type")
            .and_then(|v| v.as_str())
            .unwrap_or("t3.micro");

        let cost = self.heuristics.compute.ec2.get(instance_type)
            .map(|c| c.monthly)
            .unwrap_or_else(|| {
                // Use cold start for unknown instance types
                self.cold_start.estimate_ec2_cost(instance_type)
            });

        Ok(cost)
    }

    /// Predict RDS instance cost
    fn predict_rds(&self, change: &ResourceChange) -> Result<f64> {
        let config = change.new_config.as_ref().ok_or_else(|| {
            CostPilotError::new("PREDICT_004", ErrorCategory::InvalidInput, "Missing RDS configuration")
        })?;

        let instance_class = config
            .get("instance_class")
            .and_then(|v| v.as_str())
            .unwrap_or("db.t3.micro");

        let engine = config
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("mysql");

        let instances = match engine {
            "postgres" => &self.heuristics.database.rds.postgres,
            _ => &self.heuristics.database.rds.mysql,
        };

        let instance_cost = instances.get(instance_class)
            .map(|c| c.monthly)
            .unwrap_or(50.0); // Conservative default

        // Add storage cost
        let storage_gb = config
            .get("allocated_storage")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0);

        let storage_cost = storage_gb * self.heuristics.database.rds.storage_gp2_per_gb;

        Ok(instance_cost + storage_cost)
    }

    /// Predict DynamoDB table cost
    fn predict_dynamodb(&self, change: &ResourceChange) -> Result<f64> {
        let config = change.new_config.as_ref().ok_or_else(|| {
            CostPilotError::new("PREDICT_005", ErrorCategory::InvalidInput, "Missing DynamoDB configuration")
        })?;

        let billing_mode = config
            .get("billing_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("PAY_PER_REQUEST");

        if billing_mode == "PROVISIONED" {
            let read_capacity = config
                .get("read_capacity")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.heuristics.cold_start_defaults.dynamodb_unknown_rcu as f64);

            let write_capacity = config
                .get("write_capacity")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.heuristics.cold_start_defaults.dynamodb_unknown_wcu as f64);

            let read_cost = read_capacity * self.heuristics.database.dynamodb.provisioned.read_capacity_unit_hourly * 730.0;
            let write_cost = write_capacity * self.heuristics.database.dynamodb.provisioned.write_capacity_unit_hourly * 730.0;

            Ok(read_cost + write_cost)
        } else {
            // On-demand: use conservative estimate
            Ok(25.0) // Default monthly estimate for on-demand
        }
    }

    /// Predict NAT Gateway cost
    fn predict_nat_gateway(&self, _change: &ResourceChange) -> Result<f64> {
        let base_cost = self.heuristics.networking.nat_gateway.monthly;
        let data_gb = self.heuristics.cold_start_defaults.nat_gateway_default_gb as f64;
        let data_cost = data_gb * self.heuristics.networking.nat_gateway.data_processing_per_gb;
        
        Ok(base_cost + data_cost)
    }

    /// Predict Load Balancer cost
    fn predict_load_balancer(&self, _change: &ResourceChange) -> Result<f64> {
        let base_cost = self.heuristics.networking.load_balancer.alb.monthly;
        let lcu_cost = self.heuristics.networking.load_balancer.alb.lcu_hourly * 730.0 * 2.0; // Assume 2 LCUs
        
        Ok(base_cost + lcu_cost)
    }

    /// Predict S3 bucket cost
    fn predict_s3(&self, _change: &ResourceChange) -> Result<f64> {
        let storage_gb = self.heuristics.cold_start_defaults.s3_default_gb as f64;
        let storage_cost = storage_gb * self.heuristics.storage.s3.standard.first_50tb_per_gb.unwrap_or(0.023);
        
        Ok(storage_cost)
    }

    /// Predict Lambda function cost
    fn predict_lambda(&self, change: &ResourceChange) -> Result<f64> {
        let config = change.new_config.as_ref().ok_or_else(|| {
            CostPilotError::new("PREDICT_006", ErrorCategory::InvalidInput, "Missing Lambda configuration")
        })?;

        let memory_mb = config
            .get("memory_size")
            .and_then(|v| v.as_f64())
            .unwrap_or(self.heuristics.compute.lambda.default_memory_mb as f64);

        let invocations = self.heuristics.compute.lambda.default_memory_mb as f64 / 1000.0;
        
        // Request cost
        let request_cost = invocations * self.heuristics.compute.lambda.price_per_request;
        
        // Compute cost (GB-seconds)
        let duration_seconds = self.heuristics.compute.lambda.default_duration_ms as f64 / 1000.0;
        let gb_seconds = (memory_mb / 1024.0) * duration_seconds * invocations;
        let compute_cost = gb_seconds * self.heuristics.compute.lambda.price_per_gb_second;
        
        Ok(request_cost + compute_cost)
    }

    /// Get heuristics version
    pub fn heuristics_version(&self) -> &str {
        &self.heuristics.version
    }

    /// Get reference to loaded heuristics
    pub fn heuristics(&self) -> &CostHeuristics {
        &self.heuristics
    }

    /// Check if engine is using default or custom heuristics
    pub fn is_using_custom_heuristics(&self) -> bool {
        // Could be enhanced to track source
        false
    }

    /// Generate explanation for a prediction
    pub fn explain(
        &self,
        change: &ResourceChange,
    ) -> Result<crate::engines::explain::ReasoningChain> {
        // First get the prediction
        let estimate = self.predict_resource(change)?
            .ok_or_else(|| CostPilotError::new(
                "PREDICT_EXPLAIN_001",
                ErrorCategory::InvalidInput,
                format!("Cannot explain prediction for unknown resource type: {}", change.resource_type),
            ))?;

        // Generate reasoning chain
        use crate::engines::explain::PredictionExplainer;
        let explainer = PredictionExplainer::from_engine(self);
        Ok(explainer.explain(change, &estimate))
    }

    /// Predict total cost for multiple resource changes (convenience method)
    pub fn predict_total_cost(&mut self, changes: &[ResourceChange]) -> Result<crate::engines::shared::models::TotalCost> {
        let estimates = self.predict(changes)?;
        
        // Calculate total
        let total_monthly: f64 = estimates.iter().map(|e| e.monthly_cost).sum();
        
        // Average confidence
        let avg_confidence = if !estimates.is_empty() {
            estimates.iter().map(|e| e.confidence_score).sum::<f64>() / estimates.len() as f64
        } else {
            0.0
        };

        // Calculate prediction intervals (simple approach: sum individual intervals)
        let total_low: f64 = estimates.iter().map(|e| e.prediction_interval_low).sum();
        let total_high: f64 = estimates.iter().map(|e| e.prediction_interval_high).sum();

        Ok(crate::engines::shared::models::TotalCost {
            monthly: total_monthly,
            prediction_interval_low: total_low,
            prediction_interval_high: total_high,
            confidence_score: avg_confidence,
            resource_count: estimates.len(),
        })
    }

    /// Predict cost for a single resource change (convenience method)
    pub fn predict_resource_cost(&self, change: &ResourceChange) -> Result<CostEstimate> {
        self.predict_resource(change)?
            .ok_or_else(|| CostPilotError::new(
                "PREDICT_RESOURCE_001",
                ErrorCategory::InvalidInput,
                format!("Cannot predict cost for unknown resource type: {}", change.resource_type),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_ec2_prediction() {
        // Test would require loading actual heuristics file
        // Skipped for now
    }
}
