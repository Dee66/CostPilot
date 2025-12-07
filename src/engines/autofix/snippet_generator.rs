// Snippet generator - MVP deterministic, idempotent fix generation

use crate::engines::shared::models::{Detection, ResourceChange};
use crate::engines::explain::anti_patterns::AntiPattern;
use serde::{Deserialize, Serialize};

/// Fix snippet with human rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixSnippet {
    pub resource_id: String,
    pub resource_type: String,
    pub snippet: String,
    pub format: SnippetFormat,
    pub rationale: String,
    pub before_after: BeforeAfter,
    pub impact: String,
    pub deterministic: bool,
    pub idempotent: bool,
}

/// Snippet format (Terraform HCL, JSON, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnippetFormat {
    Terraform,
    CloudFormation,
    CDK,
}

/// Before/After comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeforeAfter {
    pub before: String,
    pub after: String,
    pub change_description: String,
}

pub struct SnippetGenerator;

impl SnippetGenerator {
    /// Generate fix snippet for a detection
    pub fn generate(
        detection: &Detection,
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
        estimate: Option<&CostEstimate>,
    ) -> Option<FixSnippet> {
        // Generate snippet based on resource type and detected issues
        match change.resource_type.as_str() {
            "aws_instance" => Self::generate_ec2_snippet(detection, change, anti_patterns, estimate),
            "aws_rds_instance" => Self::generate_rds_snippet(detection, change, estimate),
            "aws_lambda_function" => Self::generate_lambda_snippet(detection, change, anti_patterns, estimate),
            "aws_s3_bucket" => Self::generate_s3_snippet(detection, change, anti_patterns),
            "aws_dynamodb_table" => Self::generate_dynamodb_snippet(detection, change, anti_patterns),
            "aws_nat_gateway" => Self::generate_nat_gateway_snippet(detection, change),
            _ => None,
        }
    }

    /// Generate EC2 instance fix snippet
    fn generate_ec2_snippet(
        detection: &Detection,
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
        estimate: Option<&CostEstimate>,
    ) -> Option<FixSnippet> {
        // Check if this is an overprovisioned instance
        let is_overprovisioned = anti_patterns.iter()
            .any(|p| p.pattern_id == "OVERPROVISIONED_EC2");

        if !is_overprovisioned {
            return None;
        }

        let current_type = change.new_config
            .as_ref()
            .and_then(|c| c.get("instance_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Suggest smaller instance type
        let suggested_type = Self::suggest_smaller_instance(current_type);

        let snippet = format!(
            "resource \"aws_instance\" \"{}\" {{\n  instance_type = \"{}\"\n  # ... other attributes ...\n}}",
            change.resource_id.split('.').last().unwrap_or("example"),
            suggested_type
        );

        let before = format!("instance_type = \"{}\"", current_type);
        let after = format!("instance_type = \"{}\"", suggested_type);

        let estimated_savings = estimate
            .map(|e| e.monthly_cost * 0.4) // ~40% savings from right-sizing
            .unwrap_or(0.0);

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: format!(
                "Right-size EC2 instance from {} to {}. Start with a smaller instance and scale up based on actual utilization. \
                Use AWS Compute Optimizer to validate sizing decisions based on real workload patterns.",
                current_type, suggested_type
            ),
            before_after: BeforeAfter {
                before,
                after,
                change_description: format!("Downsize from {} to {}", current_type, suggested_type),
            },
            impact: format!("Estimated savings: ${:.2}/month", estimated_savings),
            deterministic: true,
            idempotent: true,
        })
    }

    /// Suggest smaller instance type
    fn suggest_smaller_instance(current: &str) -> &'static str {
        // Simple downsizing logic
        if current.contains("4xlarge") { "2xlarge" }
        else if current.contains("2xlarge") { "xlarge" }
        else if current.contains("xlarge") { "large" }
        else if current.contains("large") { "medium" }
        else if current.contains("medium") { "small" }
        else { "micro" }
    }

    /// Generate RDS instance fix snippet
    fn generate_rds_snippet(
        _detection: &Detection,
        change: &ResourceChange,
    ) -> Option<FixSnippet> {
        let current_class = change.new_config
            .as_ref()
            .and_then(|c| c.get("instance_class"))
            .and_then(|v| v.as_str())?;

        // Suggest Aurora Serverless for variable workloads
        let snippet = format!(
            "# Consider Aurora Serverless v2 for variable workloads\nresource \"aws_rds_cluster\" \"{}\" {{\n  engine_mode = \"provisioned\"\n  engine = \"aurora-mysql\"\n  \n  serverlessv2_scaling_configuration {{\n    min_capacity = 0.5\n    max_capacity = 1.0\n  }}\n}}",
            change.resource_id.split('.').last().unwrap_or("example")
        );

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: format!(
                "Consider Aurora Serverless v2 for variable workloads. It automatically scales capacity and you only pay for resources used. \
                For predictable workloads, review if {} is the optimal instance class.",
                current_class
            ),
            before_after: BeforeAfter {
                before: format!("instance_class = \"{}\"", current_class),
                after: "serverlessv2_scaling_configuration (dynamic)".to_string(),
                change_description: "Migrate to Aurora Serverless v2".to_string(),
            },
            impact: "Potential 50-90% cost reduction for variable workloads".to_string(),
            deterministic: true,
            idempotent: true,
        })
    }

    /// Generate Lambda function fix snippet
    fn generate_lambda_snippet(
        _detection: &Detection,
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Option<FixSnippet> {
        let has_unbounded = anti_patterns.iter()
            .any(|p| p.pattern_id == "UNBOUNDED_LAMBDA_CONCURRENCY");

        if !has_unbounded {
            return None;
        }

        let snippet = format!(
            "resource \"aws_lambda_function\" \"{}\" {{\n  # ... other attributes ...\n  \n  reserved_concurrent_executions = 10\n  \n  # Start conservative, monitor with CloudWatch, then adjust\n}}",
            change.resource_id.split('.').last().unwrap_or("example")
        );

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: "Set concurrency limit to prevent runaway costs from traffic spikes or bugs. \
                Start with a conservative limit (e.g., 10) and monitor CloudWatch metrics (ConcurrentExecutions, Throttles). \
                Gradually increase based on actual peak load patterns.".to_string(),
            before_after: BeforeAfter {
                before: "# No concurrency limit (unbounded)".to_string(),
                after: "reserved_concurrent_executions = 10".to_string(),
                change_description: "Add concurrency limit".to_string(),
            },
            impact: "Prevents unexpected cost spikes from runaway execution".to_string(),
            deterministic: true,
            idempotent: true,
        })
    }

    /// Generate S3 bucket fix snippet
    fn generate_s3_snippet(
        _detection: &Detection,
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Option<FixSnippet> {
        let missing_lifecycle = anti_patterns.iter()
            .any(|p| p.pattern_id == "S3_MISSING_LIFECYCLE");

        if !missing_lifecycle {
            return None;
        }

        let snippet = format!(
            "resource \"aws_s3_bucket_lifecycle_configuration\" \"{}\" {{\n  bucket = aws_s3_bucket.{}.id\n\n  rule {{\n    id     = \"intelligent-tiering\"\n    status = \"Enabled\"\n\n    transition {{\n      days          = 0\n      storage_class = \"INTELLIGENT_TIERING\"\n    }}\n  }}\n\n  rule {{\n    id     = \"archive-old-data\"\n    status = \"Enabled\"\n\n    transition {{\n      days          = 90\n      storage_class = \"GLACIER\"\n    }}\n\n    transition {{\n      days          = 365\n      storage_class = \"DEEP_ARCHIVE\"\n    }}\n  }}\n}}",
            change.resource_id.split('.').last().unwrap_or("example"),
            change.resource_id.split('.').last().unwrap_or("example")
        );

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: "Implement lifecycle policies to automatically transition objects to cheaper storage tiers. \
                Intelligent-Tiering automatically moves objects between access tiers based on usage patterns. \
                For data older than 90 days, archive to Glacier. For long-term retention (1+ year), use Deep Archive.".to_string(),
            before_after: BeforeAfter {
                before: "# No lifecycle rules".to_string(),
                after: "lifecycle_configuration with Intelligent-Tiering + Glacier".to_string(),
                change_description: "Add automated lifecycle management".to_string(),
            },
            impact: "Up to 95% storage cost reduction for infrequently accessed data".to_string(),
            deterministic: true,
            idempotent: true,
        })
    }

    /// Generate DynamoDB table fix snippet
    fn generate_dynamodb_snippet(
        _detection: &Detection,
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Option<FixSnippet> {
        let has_pay_per_request = anti_patterns.iter()
            .any(|p| p.pattern_id == "DYNAMODB_PAY_PER_REQUEST_DEFAULT");

        if !has_pay_per_request {
            return None;
        }

        let snippet = format!(
            "resource \"aws_dynamodb_table\" \"{}\" {{\n  # ... other attributes ...\n  \n  billing_mode = \"PROVISIONED\"\n  read_capacity  = 5\n  write_capacity = 5\n  \n  # Enable autoscaling\n}}\n\nresource \"aws_appautoscaling_target\" \"{}_read\" {{\n  max_capacity       = 100\n  min_capacity       = 5\n  resource_id        = \"table/${{aws_dynamodb_table.{}.name}}\"\n  scalable_dimension = \"dynamodb:table:ReadCapacityUnits\"\n  service_namespace  = \"dynamodb\"\n}}",
            change.resource_id.split('.').last().unwrap_or("example"),
            change.resource_id.split('.').last().unwrap_or("example"),
            change.resource_id.split('.').last().unwrap_or("example")
        );

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: "Switch to provisioned capacity with autoscaling for predictable workloads. \
                Provisioned mode is ~5x cheaper than on-demand for consistent traffic. \
                Autoscaling adjusts capacity based on actual usage, providing cost efficiency with performance.".to_string(),
            before_after: BeforeAfter {
                before: "billing_mode = \"PAY_PER_REQUEST\"".to_string(),
                after: "billing_mode = \"PROVISIONED\" with autoscaling".to_string(),
                change_description: "Switch to provisioned with autoscaling".to_string(),
            },
            impact: "Up to 80% cost reduction for tables with >1M requests/month".to_string(),
            deterministic: true,
            idempotent: true,
        })
    }

    /// Generate NAT Gateway fix snippet
    fn generate_nat_gateway_snippet(
        detection: &Detection,
        change: &ResourceChange,
    ) -> Option<FixSnippet> {
        let snippet = format!(
            "# Replace NAT Gateway with VPC Endpoints for AWS services\n\nresource \"aws_vpc_endpoint\" \"s3\" {{\n  vpc_id       = var.vpc_id\n  service_name = \"com.amazonaws.region.s3\"\n  route_table_ids = [aws_route_table.private.id]\n}}\n\nresource \"aws_vpc_endpoint\" \"dynamodb\" {{\n  vpc_id       = var.vpc_id\n  service_name = \"com.amazonaws.region.dynamodb\"\n  route_table_ids = [aws_route_table.private.id]\n}}\n\n# Remove or consolidate: {}\n# resource \"aws_nat_gateway\" ... {{}}",
            change.resource_id
        );

        let estimated_savings = detection.estimated_cost
            .as_ref()
            .map(|e| e.estimate * 0.7) // ~70% savings with VPC endpoints
            .unwrap_or(0.0);

        Some(FixSnippet {
            resource_id: change.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            snippet,
            format: SnippetFormat::Terraform,
            rationale: "Replace NAT Gateway with VPC Endpoints for AWS services (S3, DynamoDB). \
                VPC endpoints route traffic directly to AWS services without NAT Gateway data transfer costs. \
                If internet access is required, consolidate to single NAT Gateway if HA not critical.".to_string(),
            before_after: BeforeAfter {
                before: format!("NAT Gateway: ${:.2}/month", detection.estimated_cost.as_ref().map(|e| e.estimate).unwrap_or(0.0)),
                after: "VPC Endpoints: $0.01/GB (much cheaper)".to_string(),
                change_description: "Replace with VPC Endpoints".to_string(),
            },
            impact: format!("Estimated savings: ${:.2}/month", estimated_savings),
            deterministic: true,
            idempotent: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ChangeAction, RegressionType, Severity, CostEstimate};
    use std::collections::HashMap;
    use serde_json::json;

    #[test]
    fn test_ec2_snippet_generation() {
        let change = ResourceChange {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"instance_type": "m5.4xlarge"})),
            tags: HashMap::new(),
        };

        let detection = Detection {
            resource_id: "aws_instance.web".to_string(),
            resource_type: "aws_instance".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::High,
            estimated_cost: Some(CostEstimate {
                estimate: 560.0,
                lower: 420.0,
                upper: 700.0,
                confidence: 0.9,
            }),
            fix_snippet: None,
        };

        let anti_pattern = AntiPattern {
            pattern_id: "OVERPROVISIONED_EC2".to_string(),
            pattern_name: "Overprovisioned EC2".to_string(),
            description: "Large instance".to_string(),
            severity: "HIGH".to_string(),
            detected_in: "aws_instance.web".to_string(),
            evidence: vec![],
            suggested_fix: None,
            cost_impact: Some(560.0),
        };

        let snippet = SnippetGenerator::generate(&detection, &change, &[anti_pattern]);
        assert!(snippet.is_some());
        
        let snippet = snippet.unwrap();
        assert!(snippet.snippet.contains("2xlarge"));
        assert!(snippet.deterministic);
        assert!(snippet.idempotent);
        assert!(snippet.rationale.contains("Right-size"));
    }

    #[test]
    fn test_lambda_snippet_generation() {
        let change = ResourceChange {
            resource_id: "aws_lambda_function.api".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"memory_size": 256})),
            tags: HashMap::new(),
        };

        let detection = Detection {
            resource_id: "aws_lambda_function.api".to_string(),
            resource_type: "aws_lambda_function".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::Medium,
            estimated_cost: None,
            fix_snippet: None,
        };

        let anti_pattern = AntiPattern {
            pattern_id: "UNBOUNDED_LAMBDA_CONCURRENCY".to_string(),
            pattern_name: "Unbounded Concurrency".to_string(),
            description: "No limit".to_string(),
            severity: "HIGH".to_string(),
            detected_in: "aws_lambda_function.api".to_string(),
            evidence: vec![],
            suggested_fix: None,
            cost_impact: None,
        };

        let snippet = SnippetGenerator::generate(&detection, &change, &[anti_pattern]);
        assert!(snippet.is_some());
        
        let snippet = snippet.unwrap();
        assert!(snippet.snippet.contains("reserved_concurrent_executions"));
        assert!(snippet.deterministic);
        assert!(snippet.idempotent);
    }

    #[test]
    fn test_s3_snippet_generation() {
        let change = ResourceChange {
            resource_id: "aws_s3_bucket.data".to_string(),
            resource_type: "aws_s3_bucket".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(json!({"bucket": "my-data-bucket"})),
            tags: HashMap::new(),
        };

        let detection = Detection {
            resource_id: "aws_s3_bucket.data".to_string(),
            resource_type: "aws_s3_bucket".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::Medium,
            estimated_cost: None,
            fix_snippet: None,
        };

        let anti_pattern = AntiPattern {
            pattern_id: "S3_MISSING_LIFECYCLE".to_string(),
            pattern_name: "Missing Lifecycle".to_string(),
            description: "No rules".to_string(),
            severity: "MEDIUM".to_string(),
            detected_in: "aws_s3_bucket.data".to_string(),
            evidence: vec![],
            suggested_fix: None,
            cost_impact: None,
        };

        let snippet = SnippetGenerator::generate(&detection, &change, &[anti_pattern]);
        assert!(snippet.is_some());
        
        let snippet = snippet.unwrap();
        assert!(snippet.snippet.contains("INTELLIGENT_TIERING"));
        assert!(snippet.snippet.contains("GLACIER"));
        assert!(snippet.deterministic);
        assert!(snippet.idempotent);
    }

    #[test]
    fn test_suggest_smaller_instance() {
        assert_eq!(SnippetGenerator::suggest_smaller_instance("m5.4xlarge"), "2xlarge");
        assert_eq!(SnippetGenerator::suggest_smaller_instance("c5.2xlarge"), "xlarge");
        assert_eq!(SnippetGenerator::suggest_smaller_instance("r5.xlarge"), "large");
        assert_eq!(SnippetGenerator::suggest_smaller_instance("t3.large"), "medium");
    }
}
