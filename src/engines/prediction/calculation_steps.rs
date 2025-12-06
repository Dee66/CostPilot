// Document calculation steps for explainability

use crate::engines::shared::models::{ResourceChange, CostEstimate};
use serde::{Deserialize, Serialize};

/// A step in the cost calculation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationStep {
    pub step_number: usize,
    pub operation: String,
    pub input: String,
    pub output: String,
    pub reasoning: String,
}

/// Full calculation breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationBreakdown {
    pub resource_id: String,
    pub resource_type: String,
    pub steps: Vec<CalculationStep>,
    pub final_estimate: f64,
    pub confidence: f64,
    pub cold_start_used: bool,
}

/// Document calculation steps for a cost estimate
pub fn document_calculation(
    change: &ResourceChange,
    estimate: &CostEstimate,
    steps: Vec<CalculationStep>,
) -> CalculationBreakdown {
    CalculationBreakdown {
        resource_id: change.resource_id.clone(),
        resource_type: change.resource_type.clone(),
        steps,
        final_estimate: estimate.estimate,
        confidence: estimate.confidence,
        cold_start_used: false, // Set by caller
    }
}

/// Create a step for EC2 instance calculation
pub fn ec2_calculation_step(
    step: usize,
    instance_type: &str,
    hourly_rate: f64,
    hours: f64,
) -> CalculationStep {
    CalculationStep {
        step_number: step,
        operation: "EC2 Instance Cost".to_string(),
        input: format!("instance_type={}, hourly_rate=${:.4}, hours={}", instance_type, hourly_rate, hours),
        output: format!("${:.2}/month", hourly_rate * hours),
        reasoning: format!(
            "EC2 {} instance runs at ${:.4}/hour for {} hours/month",
            instance_type, hourly_rate, hours
        ),
    }
}

/// Create a step for RDS instance calculation
pub fn rds_calculation_step(
    step: usize,
    engine: &str,
    instance_class: &str,
    hourly_rate: f64,
    hours: f64,
) -> CalculationStep {
    CalculationStep {
        step_number: step,
        operation: "RDS Instance Cost".to_string(),
        input: format!("engine={}, instance_class={}, hourly_rate=${:.4}, hours={}", engine, instance_class, hourly_rate, hours),
        output: format!("${:.2}/month", hourly_rate * hours),
        reasoning: format!(
            "RDS {} {} instance runs at ${:.4}/hour for {} hours/month",
            engine, instance_class, hourly_rate, hours
        ),
    }
}

/// Create a step for storage calculation
pub fn storage_calculation_step(
    step: usize,
    size_gb: f64,
    cost_per_gb: f64,
    storage_type: &str,
) -> CalculationStep {
    CalculationStep {
        step_number: step,
        operation: format!("{} Storage Cost", storage_type),
        input: format!("size_gb={}, cost_per_gb=${:.4}", size_gb, cost_per_gb),
        output: format!("${:.2}/month", size_gb * cost_per_gb),
        reasoning: format!(
            "{} GB {} storage at ${:.4}/GB/month",
            size_gb, storage_type, cost_per_gb
        ),
    }
}

/// Create a step for DynamoDB calculation
pub fn dynamodb_calculation_step(
    step: usize,
    billing_mode: &str,
    rcu: Option<i64>,
    wcu: Option<i64>,
    rcu_cost: f64,
    wcu_cost: f64,
) -> CalculationStep {
    let (input, output, reasoning) = match billing_mode {
        "PROVISIONED" => {
            let rcu = rcu.unwrap_or(5);
            let wcu = wcu.unwrap_or(5);
            let cost = (rcu as f64 * rcu_cost) + (wcu as f64 * wcu_cost);
            (
                format!("billing_mode={}, rcu={}, wcu={}", billing_mode, rcu, wcu),
                format!("${:.2}/month", cost),
                format!(
                    "Provisioned capacity: {} RCU at ${:.4} + {} WCU at ${:.4}",
                    rcu, rcu_cost, wcu, wcu_cost
                ),
            )
        }
        _ => {
            (
                format!("billing_mode={}", billing_mode),
                "$0.00/month (base)".to_string(),
                "On-demand billing - pay per request (requests not estimated in plan)".to_string(),
            )
        }
    };

    CalculationStep {
        step_number: step,
        operation: "DynamoDB Cost".to_string(),
        input,
        output,
        reasoning,
    }
}

/// Create a step for Lambda calculation
pub fn lambda_calculation_step(
    step: usize,
    memory_mb: i64,
    invocations: i64,
    gb_seconds: f64,
    gb_second_cost: f64,
    request_cost: f64,
) -> CalculationStep {
    let compute_cost = gb_seconds * gb_second_cost;
    let request_cost_total = (invocations as f64) * request_cost;
    let total = compute_cost + request_cost_total;

    CalculationStep {
        step_number: step,
        operation: "Lambda Cost".to_string(),
        input: format!(
            "memory_mb={}, invocations={}, gb_seconds={:.2}",
            memory_mb, invocations, gb_seconds
        ),
        output: format!("${:.2}/month", total),
        reasoning: format!(
            "Compute: {:.2} GB-seconds at ${:.6} = ${:.2}, Requests: {} invocations at ${:.10} = ${:.2}",
            gb_seconds, gb_second_cost, compute_cost, invocations, request_cost, request_cost_total
        ),
    }
}

/// Create a step for NAT Gateway calculation
pub fn nat_gateway_calculation_step(
    step: usize,
    hours: f64,
    hourly_rate: f64,
    data_gb: f64,
    data_rate: f64,
) -> CalculationStep {
    let fixed_cost = hours * hourly_rate;
    let data_cost = data_gb * data_rate;
    let total = fixed_cost + data_cost;

    CalculationStep {
        step_number: step,
        operation: "NAT Gateway Cost".to_string(),
        input: format!("hours={}, hourly_rate=${:.4}, data_gb={}, data_rate=${:.4}", hours, hourly_rate, data_gb, data_rate),
        output: format!("${:.2}/month", total),
        reasoning: format!(
            "Fixed: {} hours at ${:.4}/hour = ${:.2}, Data: {} GB at ${:.4}/GB = ${:.2}",
            hours, hourly_rate, fixed_cost, data_gb, data_rate, data_cost
        ),
    }
}

/// Create a step for Load Balancer calculation
pub fn load_balancer_calculation_step(
    step: usize,
    lb_type: &str,
    hours: f64,
    hourly_rate: f64,
    lcu_hours: f64,
    lcu_rate: f64,
) -> CalculationStep {
    let fixed_cost = hours * hourly_rate;
    let lcu_cost = lcu_hours * lcu_rate;
    let total = fixed_cost + lcu_cost;

    CalculationStep {
        step_number: step,
        operation: format!("{} Cost", lb_type),
        input: format!("hours={}, hourly_rate=${:.4}, lcu_hours={}, lcu_rate=${:.4}", hours, hourly_rate, lcu_hours, lcu_rate),
        output: format!("${:.2}/month", total),
        reasoning: format!(
            "Fixed: {} hours at ${:.4}/hour = ${:.2}, LCU: {} LCU-hours at ${:.4} = ${:.2}",
            hours, hourly_rate, fixed_cost, lcu_hours, lcu_rate, lcu_cost
        ),
    }
}

/// Create a step for S3 calculation
pub fn s3_calculation_step(
    step: usize,
    storage_gb: f64,
    storage_class: &str,
    cost_per_gb: f64,
) -> CalculationStep {
    let cost = storage_gb * cost_per_gb;

    CalculationStep {
        step_number: step,
        operation: "S3 Storage Cost".to_string(),
        input: format!("storage_gb={}, storage_class={}, cost_per_gb=${:.4}", storage_gb, storage_class, cost_per_gb),
        output: format!("${:.2}/month", cost),
        reasoning: format!(
            "{} GB in {} class at ${:.4}/GB/month",
            storage_gb, storage_class, cost_per_gb
        ),
    }
}

/// Create a step for cold start inference
pub fn cold_start_step(
    step: usize,
    resource_type: &str,
    inferred_value: &str,
    reasoning: &str,
) -> CalculationStep {
    CalculationStep {
        step_number: step,
        operation: "Cold Start Inference".to_string(),
        input: format!("resource_type={}, missing_value", resource_type),
        output: format!("inferred_value={}", inferred_value),
        reasoning: reasoning.to_string(),
    }
}

/// Create a step for confidence calculation
pub fn confidence_step(
    step: usize,
    base_confidence: f64,
    adjustments: Vec<(&str, f64)>,
    final_confidence: f64,
) -> CalculationStep {
    let mut reasoning = format!("Base confidence: {:.2}\n", base_confidence);
    for (reason, adjustment) in adjustments {
        reasoning.push_str(&format!("  - {}: ×{:.2}\n", reason, adjustment));
    }
    reasoning.push_str(&format!("Final confidence: {:.2}", final_confidence));

    CalculationStep {
        step_number: step,
        operation: "Confidence Calculation".to_string(),
        input: format!("base={:.2}", base_confidence),
        output: format!("{:.2}", final_confidence),
        reasoning,
    }
}

/// Create a step for prediction interval calculation
pub fn interval_step(
    step: usize,
    estimate: f64,
    interval_width: f64,
    lower: f64,
    upper: f64,
) -> CalculationStep {
    CalculationStep {
        step_number: step,
        operation: "Prediction Interval".to_string(),
        input: format!("estimate=${:.2}, width={:.0}%", estimate, interval_width * 100.0),
        output: format!("${:.2} - ${:.2}", lower, upper),
        reasoning: format!(
            "Estimate ${:.2} ± {:.0}% = [${:.2}, ${:.2}]",
            estimate, interval_width * 100.0, lower, upper
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ec2_step() {
        let step = ec2_calculation_step(1, "t3.micro", 0.0104, 730.0);
        assert_eq!(step.step_number, 1);
        assert!(step.reasoning.contains("t3.micro"));
        assert!(step.output.contains("$7.59"));
    }

    #[test]
    fn test_rds_step() {
        let step = rds_calculation_step(1, "mysql", "db.t3.small", 0.034, 730.0);
        assert_eq!(step.operation, "RDS Instance Cost");
        assert!(step.reasoning.contains("mysql"));
    }

    #[test]
    fn test_storage_step() {
        let step = storage_calculation_step(1, 100.0, 0.10, "gp3");
        assert!(step.output.contains("$10.00"));
    }

    #[test]
    fn test_dynamodb_provisioned_step() {
        let step = dynamodb_calculation_step(1, "PROVISIONED", Some(10), Some(10), 0.13, 0.65);
        assert!(step.reasoning.contains("Provisioned capacity"));
    }

    #[test]
    fn test_dynamodb_ondemand_step() {
        let step = dynamodb_calculation_step(1, "PAY_PER_REQUEST", None, None, 0.0, 0.0);
        assert!(step.reasoning.contains("On-demand"));
    }

    #[test]
    fn test_lambda_step() {
        let step = lambda_calculation_step(1, 256, 10000, 250.0, 0.0000166667, 0.0000002);
        assert!(step.reasoning.contains("GB-seconds"));
        assert!(step.reasoning.contains("invocations"));
    }

    #[test]
    fn test_nat_gateway_step() {
        let step = nat_gateway_calculation_step(1, 730.0, 0.045, 10.0, 0.045);
        assert!(step.reasoning.contains("Fixed:"));
        assert!(step.reasoning.contains("Data:"));
    }

    #[test]
    fn test_cold_start_step() {
        let step = cold_start_step(
            1,
            "aws_instance",
            "t3.medium",
            "Unknown instance type, using t3.medium as default",
        );
        assert_eq!(step.operation, "Cold Start Inference");
        assert!(step.reasoning.contains("default"));
    }

    #[test]
    fn test_confidence_step() {
        let step = confidence_step(
            1,
            0.95,
            vec![("Cold start used", 0.6), ("Unknown values", 0.75)],
            0.43,
        );
        assert!(step.reasoning.contains("Base confidence"));
        assert!(step.reasoning.contains("Final confidence"));
    }

    #[test]
    fn test_interval_step() {
        let step = interval_step(1, 100.0, 0.25, 75.0, 125.0);
        assert!(step.output.contains("$75.00"));
        assert!(step.output.contains("$125.00"));
    }
}
