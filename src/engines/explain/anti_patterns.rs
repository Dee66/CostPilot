// Anti-pattern detection - MVP top 5 patterns

use crate::engines::shared::models::{CostEstimate, ResourceChange};
use serde::{Deserialize, Serialize};

/// Anti-pattern detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub pattern_id: String,
    pub pattern_name: String,
    pub description: String,
    pub severity: String,
    pub detected_in: String,
    pub evidence: Vec<String>,
    pub suggested_fix: Option<String>,
    pub cost_impact: Option<f64>,
}

/// Detect anti-patterns in resource change (MVP top 5)
pub fn detect_anti_patterns(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // 1. NAT Gateway overuse
    if let Some(pattern) = detect_nat_gateway_overuse(change, estimate) {
        patterns.push(pattern);
    }

    // 2. Overprovisioned EC2
    if let Some(pattern) = detect_overprovisioned_ec2(change, estimate) {
        patterns.push(pattern);
    }

    // 3. S3 missing lifecycle
    if let Some(pattern) = detect_s3_missing_lifecycle(change) {
        patterns.push(pattern);
    }

    // 4. Unbounded Lambda concurrency
    if let Some(pattern) = detect_unbounded_lambda_concurrency(change) {
        patterns.push(pattern);
    }

    // 5. DynamoDB pay-per-request default
    if let Some(pattern) = detect_dynamodb_pay_per_request_default(change) {
        patterns.push(pattern);
    }

    patterns
}

/// Pattern 1: NAT Gateway overuse
fn detect_nat_gateway_overuse(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    if change.resource_type != "aws_nat_gateway" {
        return None;
    }

    // NAT Gateways are expensive (~$32.85/month + data transfer)
    let mut evidence =
        vec!["NAT Gateway incurs fixed hourly charges plus data transfer costs".to_string()];

    if let Some(est) = estimate {
        evidence.push(format!("Estimated cost: ${:.2}/month", est.monthly_cost));
    }

    Some(AntiPattern {
        pattern_id: "NAT_GATEWAY_OVERUSE".to_string(),
        pattern_name: "NAT Gateway Overuse".to_string(),
        description: "NAT Gateways are expensive. Consider VPC endpoints or consolidating across AZs.".to_string(),
        severity: "HIGH".to_string(),
        detected_in: change.resource_id.clone(),
        evidence,
        suggested_fix: Some(
            "Use VPC endpoints for AWS services (S3, DynamoDB) to avoid NAT Gateway data transfer. \
            Consider single NAT Gateway if high availability not critical.".to_string()
        ),
        cost_impact: estimate.map(|e| e.monthly_cost),
    })
}

/// Pattern 2: Overprovisioned EC2
fn detect_overprovisioned_ec2(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    if change.resource_type != "aws_instance" {
        return None;
    }

    // Check for large instance types that might be overprovisioned
    if let Some(config) = &change.new_config {
        if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
            // Detect potentially oversized instances
            let is_large = instance_type.contains("large")
                || instance_type.contains("xlarge")
                || instance_type.starts_with("m5.")
                || instance_type.starts_with("c5.")
                || instance_type.starts_with("r5.");

            if is_large {
                let mut evidence = vec![
                    format!("Instance type: {}", instance_type),
                    "Large instance types should be right-sized based on actual workload"
                        .to_string(),
                ];

                if let Some(est) = estimate {
                    evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
                }

                return Some(AntiPattern {
                    pattern_id: "OVERPROVISIONED_EC2".to_string(),
                    pattern_name: "Potentially Overprovisioned EC2".to_string(),
                    description: "Large EC2 instance detected. Verify sizing matches actual workload requirements.".to_string(),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence,
                    suggested_fix: Some(
                        "Use AWS Compute Optimizer to analyze utilization patterns and recommend right-sized instances. \
                        Start with smaller instances and scale up as needed.".to_string()
                    ),
                    cost_impact: estimate.map(|e| e.monthly_cost),
                });
            }
        }
    }

    None
}

/// Pattern 3: S3 missing lifecycle
fn detect_s3_missing_lifecycle(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_s3_bucket" {
        return None;
    }

    // Check if lifecycle rules are missing
    let has_lifecycle = if let Some(config) = &change.new_config {
        config.get("lifecycle_rule").is_some() || config.get("lifecycle_configuration").is_some()
    } else {
        false
    };

    if !has_lifecycle {
        return Some(AntiPattern {
            pattern_id: "S3_MISSING_LIFECYCLE".to_string(),
            pattern_name: "S3 Missing Lifecycle Policy".to_string(),
            description: "S3 bucket created without lifecycle rules. Objects will remain in standard storage indefinitely.".to_string(),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "No lifecycle_rule or lifecycle_configuration found".to_string(),
                "Data will accumulate in standard storage (most expensive tier)".to_string(),
            ],
            suggested_fix: Some(
                "Add lifecycle rules to transition objects to cheaper storage classes: \
                Intelligent-Tiering (automatic), Glacier (archive), or Deep Archive (long-term). \
                Consider expiration rules for temporary/logs data.".to_string()
            ),
            cost_impact: None,
        });
    }

    None
}

/// Pattern 4: Unbounded Lambda concurrency
fn detect_unbounded_lambda_concurrency(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_lambda_function" {
        return None;
    }

    // Check if reserved_concurrent_executions is set
    let has_concurrency_limit = if let Some(config) = &change.new_config {
        config.get("reserved_concurrent_executions").is_some()
    } else {
        false
    };

    if !has_concurrency_limit {
        return Some(AntiPattern {
            pattern_id: "UNBOUNDED_LAMBDA_CONCURRENCY".to_string(),
            pattern_name: "Unbounded Lambda Concurrency".to_string(),
            description:
                "Lambda function without concurrency limits can cause unexpected cost spikes."
                    .to_string(),
            severity: "HIGH".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "No reserved_concurrent_executions configured".to_string(),
                "Function can scale to account-level limits (1000 concurrent by default)"
                    .to_string(),
                "Traffic spikes or bugs can cause runaway costs".to_string(),
            ],
            suggested_fix: Some(
                "Set reserved_concurrent_executions based on expected peak load. \
                Start conservative (e.g., 10-100) and adjust based on CloudWatch metrics. \
                Consider provisioned concurrency for predictable traffic."
                    .to_string(),
            ),
            cost_impact: None,
        });
    }

    None
}

/// Pattern 5: DynamoDB pay-per-request default
fn detect_dynamodb_pay_per_request_default(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_dynamodb_table" {
        return None;
    }

    // Check billing mode
    if let Some(config) = &change.new_config {
        let billing_mode = config
            .get("billing_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("PAY_PER_REQUEST"); // Default is on-demand

        if billing_mode == "PAY_PER_REQUEST" {
            // Check if this is intentional or just using defaults
            let has_explicit_config = config.get("billing_mode").is_some();

            if !has_explicit_config {
                return Some(AntiPattern {
                    pattern_id: "DYNAMODB_PAY_PER_REQUEST_DEFAULT".to_string(),
                    pattern_name: "DynamoDB On-Demand by Default".to_string(),
                    description: "DynamoDB table using pay-per-request mode (potentially by default). \
                                 Provisioned mode is often cheaper for predictable workloads.".to_string(),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        "Billing mode: PAY_PER_REQUEST (on-demand)".to_string(),
                        "On-demand pricing is ~5x more expensive than provisioned for consistent workloads".to_string(),
                    ],
                    suggested_fix: Some(
                        "If workload has predictable traffic patterns, switch to PROVISIONED billing mode \
                        with autoscaling. On-demand is best for unpredictable/spiky workloads or <1M requests/month.".to_string()
                    ),
                    cost_impact: None,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ResourceChange, ChangeAction, CostEstimate};
    use serde_json::json;

    #[test]
    fn test_nat_gateway_pattern() {
        let change = ResourceChange::builder()
            .resource_id("aws_nat_gateway.test")
            .resource_type("aws_nat_gateway")
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let estimate = CostEstimate::builder()
            .resource_id("test")
            .monthly_cost(32.85)
            .prediction_interval_low(24.64)
            .prediction_interval_high(41.06)
            .confidence_score(0.95)
            .build();

        let patterns = detect_anti_patterns(&change, Some(&estimate));
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "NAT_GATEWAY_OVERUSE");
        assert_eq!(patterns[0].severity, "HIGH");
    }

    #[test]
    fn test_overprovisioned_ec2() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "m5.4xlarge"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "OVERPROVISIONED_EC2");
    }

    #[test]
    fn test_s3_missing_lifecycle() {
        let change = ResourceChange::builder()
            .resource_id("aws_s3_bucket.test")
            .resource_type("aws_s3_bucket")
            .action(ChangeAction::Create)
            .new_config(json!({"bucket": "test-bucket"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "S3_MISSING_LIFECYCLE");
    }

    #[test]
    fn test_unbounded_lambda() {
        let change = ResourceChange::builder()
            .resource_id("aws_lambda_function.test")
            .resource_type("aws_lambda_function")
            .action(ChangeAction::Create)
            .new_config(json!({"function_name": "test", "memory_size": 256}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "UNBOUNDED_LAMBDA_CONCURRENCY");
    }

    #[test]
    fn test_dynamodb_pay_per_request() {
        let change = ResourceChange::builder()
            .resource_id("aws_dynamodb_table.test")
            .resource_type("aws_dynamodb_table")
            .action(ChangeAction::Create)
            .new_config(json!({"name": "test-table"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "DYNAMODB_PAY_PER_REQUEST_DEFAULT");
    }

    #[test]
    fn test_no_patterns() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.micro"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 0);
    }
}
