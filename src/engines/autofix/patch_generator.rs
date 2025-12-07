// Patch generator - creates full unified diff patches for cost optimizations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::engines::shared::models::{Detection, ResourceChange, CostEstimate};
use crate::engines::explain::anti_patterns::AntiPattern;

/// A complete patch file with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFile {
    pub resource_id: String,
    pub resource_type: String,
    pub filename: String,
    pub hunks: Vec<PatchHunk>,
    pub metadata: PatchMetadata,
}

/// A single hunk in a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<PatchLine>,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// A line in a patch hunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchLine {
    pub line_type: PatchLineType,
    pub content: String,
    pub indent_level: usize,
}

/// Type of line in a patch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatchLineType {
    Context,  // Unchanged line (space prefix)
    Addition, // Added line (+ prefix)
    Deletion, // Removed line (- prefix)
}

/// Metadata about the patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMetadata {
    pub cost_before: f64,
    pub cost_after: f64,
    pub monthly_savings: f64,
    pub confidence: f64,
    pub anti_patterns: Vec<String>,
    pub rationale: String,
    pub simulation_required: bool,
    pub beta: bool,
}

/// Result of patch generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResult {
    pub patches: Vec<PatchFile>,
    pub total_savings: f64,
    pub total_changes: usize,
    pub warnings: Vec<String>,
}

pub struct PatchGenerator;

impl PatchGenerator {
    /// Generate patches for detections
    pub fn generate(
        detections: &[Detection],
        changes: &[ResourceChange],
        estimates: &[CostEstimate],
    ) -> PatchResult {
        let mut patches = Vec::new();
        let mut total_savings = 0.0;
        let mut total_changes = 0;
        let mut warnings = Vec::new();

        for detection in detections {
            let change = changes.iter()
                .find(|c| c.resource_id == detection.resource_id);
            
            let estimate = estimates.iter()
                .find(|e| e.resource_id == detection.resource_id);

            if let Some(change) = change {
                match Self::generate_patch(detection, change, estimate) {
                    Ok(patch) => {
                        total_savings += patch.metadata.monthly_savings;
                        total_changes += patch.hunks.len();
                        patches.push(patch);
                    }
                    Err(e) => {
                        warnings.push(format!(
                            "Failed to generate patch for {}: {}",
                            detection.resource_id, e
                        ));
                    }
                }
            } else {
                warnings.push(format!(
                    "Resource change not found: {}",
                    detection.resource_id
                ));
            }
        }

        PatchResult {
            patches,
            total_savings,
            total_changes,
            warnings,
        }
    }

    /// Generate patch for a single detection
    fn generate_patch(
        detection: &Detection,
        change: &ResourceChange,
        estimate: Option<&CostEstimate>,
    ) -> Result<PatchFile, String> {
        let anti_patterns = crate::engines::explain::anti_patterns::detect_anti_patterns(
            change,
            estimate,
        );

        if anti_patterns.is_empty() {
            return Err("No fixable anti-patterns detected".to_string());
        }

        // Determine filename from resource
        let filename = Self::infer_filename(&change.resource_id);

        // Generate hunks based on resource type and anti-patterns
        let hunks = Self::generate_hunks(change, &anti_patterns)?;

        if hunks.is_empty() {
            return Err("No changes generated".to_string());
        }

        let cost_before = estimate
            .map(|e| e.monthly_cost)
            .unwrap_or(0.0);

        let cost_after = Self::estimate_cost_after(&anti_patterns, cost_before);
        let monthly_savings = cost_before - cost_after;

        let metadata = PatchMetadata {
            cost_before,
            cost_after,
            monthly_savings,
            confidence: estimate
                .map(|e| e.confidence_score)
                .unwrap_or(0.5),
            anti_patterns: anti_patterns.iter().map(|ap| ap.pattern_name.clone()).collect(),
            rationale: Self::build_rationale(&anti_patterns, monthly_savings),
            simulation_required: true,
            beta: true,
        };

        Ok(PatchFile {
            resource_id: detection.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            filename,
            hunks,
            metadata,
        })
    }

    /// Generate hunks based on resource type
    fn generate_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        match change.resource_type.as_str() {
            "aws_instance" => Self::generate_ec2_hunks(change, anti_patterns),
            "aws_rds_instance" => Self::generate_rds_hunks(change, anti_patterns),
            "aws_lambda_function" => Self::generate_lambda_hunks(change, anti_patterns),
            "aws_dynamodb_table" => Self::generate_dynamodb_hunks(change, anti_patterns),
            "aws_s3_bucket" => Self::generate_s3_hunks(change, anti_patterns),
            "aws_nat_gateway" => Self::generate_nat_gateway_hunks(change, anti_patterns),
            _ => Err(format!("Patch generation not supported for {}", change.resource_type)),
        }
    }

    /// Generate EC2 patch hunks
    fn generate_ec2_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // Check for overprovisioned instance
        if anti_patterns.iter().any(|ap| ap.pattern_name == "Overprovisioned EC2 instance") {
            let old_instance = change.new_config.as_ref()
                .and_then(|c| c.get("instance_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("t3.large");

            let new_instance = Self::recommend_instance_downsize(old_instance);

            hunks.push(PatchHunk {
                old_start: 5,
                old_count: 3,
                new_start: 5,
                new_count: 3,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: format!("resource \"aws_instance\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Deletion,
                        content: format!("  instance_type = \"{}\"", old_instance),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: format!("  instance_type = \"{}\"", new_instance),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  ami           = var.ami_id".to_string(),
                        indent_level: 1,
                    },
                ],
                context_before: vec![
                    "# Web server instance".to_string(),
                ],
                context_after: vec![
                    "  tags = {".to_string(),
                    "    Name = \"web-server\"".to_string(),
                    "  }".to_string(),
                ],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for EC2".to_string());
        }

        Ok(hunks)
    }

    /// Generate RDS patch hunks
    fn generate_rds_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // Check for overprovisioned RDS
        if anti_patterns.iter().any(|ap| ap.pattern_name.contains("RDS") || ap.pattern_name.contains("oversized")) {
            let old_instance = change.new_config.as_ref()
                .and_then(|c| c.get("instance_class"))
                .and_then(|v| v.as_str())
                .unwrap_or("db.m5.large");

            let new_instance = Self::recommend_rds_downsize(old_instance);

            hunks.push(PatchHunk {
                old_start: 8,
                old_count: 3,
                new_start: 8,
                new_count: 3,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: format!("resource \"aws_rds_instance\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Deletion,
                        content: format!("  instance_class = \"{}\"", old_instance),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: format!("  instance_class = \"{}\"", new_instance),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  engine         = \"mysql\"".to_string(),
                        indent_level: 1,
                    },
                ],
                context_before: vec![],
                context_after: vec![
                    "  allocated_storage = 20".to_string(),
                ],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for RDS".to_string());
        }

        Ok(hunks)
    }

    /// Generate Lambda patch hunks
    fn generate_lambda_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // Check for unbounded concurrency
        if anti_patterns.iter().any(|ap| ap.pattern_name.contains("concurrency")) {
            hunks.push(PatchHunk {
                old_start: 12,
                old_count: 2,
                new_start: 12,
                new_count: 5,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: format!("resource \"aws_lambda_function\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  function_name = var.function_name".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  # Set concurrency limit to prevent cost explosions".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  reserved_concurrent_executions = 10".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  runtime       = \"python3.9\"".to_string(),
                        indent_level: 1,
                    },
                ],
                context_before: vec![],
                context_after: vec![
                    "  handler       = \"index.handler\"".to_string(),
                ],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for Lambda".to_string());
        }

        Ok(hunks)
    }

    /// Generate DynamoDB patch hunks
    fn generate_dynamodb_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // Check for pay-per-request to provisioned conversion
        if anti_patterns.iter().any(|ap| ap.pattern_name.contains("pay-per-request")) {
            hunks.push(PatchHunk {
                old_start: 3,
                old_count: 3,
                new_start: 3,
                new_count: 6,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: format!("resource \"aws_dynamodb_table\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Deletion,
                        content: "  billing_mode = \"PAY_PER_REQUEST\"".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  billing_mode   = \"PROVISIONED\"".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  read_capacity  = 5".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  write_capacity = 5".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  hash_key       = \"id\"".to_string(),
                        indent_level: 1,
                    },
                ],
                context_before: vec![],
                context_after: vec![
                    "  attribute {".to_string(),
                ],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for DynamoDB".to_string());
        }

        Ok(hunks)
    }

    /// Generate S3 patch hunks
    fn generate_s3_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // Check for missing lifecycle
        if anti_patterns.iter().any(|ap| ap.pattern_name.contains("lifecycle")) {
            hunks.push(PatchHunk {
                old_start: 10,
                old_count: 2,
                new_start: 10,
                new_count: 13,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: format!("resource \"aws_s3_bucket\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "  bucket = var.bucket_name".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  lifecycle_rule {".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "    enabled = true".to_string(),
                        indent_level: 2,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "    transition {".to_string(),
                        indent_level: 2,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "      days          = 30".to_string(),
                        indent_level: 3,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "      storage_class = \"STANDARD_IA\"".to_string(),
                        indent_level: 3,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "    }".to_string(),
                        indent_level: 2,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  }".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "}".to_string(),
                        indent_level: 0,
                    },
                ],
                context_before: vec![
                    "# Storage bucket".to_string(),
                ],
                context_after: vec![],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for S3".to_string());
        }

        Ok(hunks)
    }

    /// Generate NAT Gateway patch hunks
    fn generate_nat_gateway_hunks(
        change: &ResourceChange,
        anti_patterns: &[AntiPattern],
    ) -> Result<Vec<PatchHunk>, String> {
        let mut hunks = Vec::new();

        // NAT Gateway optimization: suggest VPC endpoints
        if anti_patterns.iter().any(|ap| ap.pattern_name.contains("NAT")) {
            hunks.push(PatchHunk {
                old_start: 15,
                old_count: 1,
                new_start: 15,
                new_count: 8,
                lines: vec![
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "# Network configuration".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "# Consider VPC endpoints for AWS services to reduce NAT Gateway usage".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "resource \"aws_vpc_endpoint\" \"s3\" {".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  vpc_id       = aws_vpc.main.id".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "  service_name = \"com.amazonaws.${var.region}.s3\"".to_string(),
                        indent_level: 1,
                    },
                    PatchLine {
                        line_type: PatchLineType::Addition,
                        content: "}".to_string(),
                        indent_level: 0,
                    },
                    PatchLine {
                        line_type: PatchLineType::Context,
                        content: "".to_string(),
                        indent_level: 0,
                    },
                ],
                context_before: vec![],
                context_after: vec![
                    format!("resource \"aws_nat_gateway\" \"{}\" {{", Self::extract_name(&change.resource_id)),
                ],
            });
        }

        if hunks.is_empty() {
            return Err("No applicable fixes for NAT Gateway".to_string());
        }

        Ok(hunks)
    }

    /// Recommend instance downsize
    fn recommend_instance_downsize(instance_type: &str) -> &str {
        match instance_type {
            "t3.2xlarge" => "t3.xlarge",
            "t3.xlarge" => "t3.large",
            "t3.large" => "t3.medium",
            "t3.medium" => "t3.small",
            "m5.2xlarge" => "m5.xlarge",
            "m5.xlarge" => "m5.large",
            "m5.large" => "m5.medium",
            "c5.2xlarge" => "c5.xlarge",
            "c5.xlarge" => "c5.large",
            "c5.large" => "c5.medium",
            _ => "t3.small", // Safe default
        }
    }

    /// Recommend RDS instance downsize
    fn recommend_rds_downsize(instance_class: &str) -> &str {
        match instance_class {
            "db.m5.2xlarge" => "db.m5.xlarge",
            "db.m5.xlarge" => "db.m5.large",
            "db.m5.large" => "db.t3.medium",
            "db.t3.large" => "db.t3.medium",
            "db.t3.medium" => "db.t3.small",
            _ => "db.t3.small", // Safe default
        }
    }

    /// Estimate cost after fixes
    fn estimate_cost_after(anti_patterns: &[AntiPattern], cost_before: f64) -> f64 {
        let mut reduction_factor = 1.0;

        for pattern in anti_patterns {
            // Estimate reduction based on anti-pattern type
            reduction_factor *= match pattern.severity.as_str() {
                "Critical" => 0.5,  // 50% reduction
                "High" => 0.7,      // 30% reduction
                "Medium" => 0.85,   // 15% reduction
                _ => 0.95,          // 5% reduction
            };
        }

        cost_before * reduction_factor
    }

    /// Build rationale for patch
    fn build_rationale(anti_patterns: &[AntiPattern], monthly_savings: f64) -> String {
        let pattern_names: Vec<String> = anti_patterns
            .iter()
            .map(|ap| ap.pattern_name.clone())
            .collect();

        format!(
            "This patch addresses {} cost optimization issue(s): {}. \
             Expected monthly savings: ${:.2}. \
             Review and test in non-production environment before applying.",
            anti_patterns.len(),
            pattern_names.join(", "),
            monthly_savings
        )
    }

    /// Infer filename from resource ID
    fn infer_filename(resource_id: &str) -> String {
        // Extract module path if present (e.g., module.web.aws_instance.server)
        let parts: Vec<&str> = resource_id.split('.').collect();
        
        if parts.len() >= 2 {
            let resource_type = parts[parts.len() - 2];
            match resource_type {
                "aws_instance" => "compute.tf".to_string(),
                "aws_rds_instance" => "database.tf".to_string(),
                "aws_lambda_function" => "lambda.tf".to_string(),
                "aws_dynamodb_table" => "database.tf".to_string(),
                "aws_s3_bucket" => "storage.tf".to_string(),
                "aws_nat_gateway" => "network.tf".to_string(),
                _ => "main.tf".to_string(),
            }
        } else {
            "main.tf".to_string()
        }
    }

    /// Extract resource name from ID
    fn extract_name(resource_id: &str) -> String {
        resource_id
            .split('.')
            .last()
            .unwrap_or("resource")
            .to_string()
    }
}

impl PatchFile {
    /// Format as unified diff
    pub fn to_unified_diff(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("--- a/{}\n", self.filename));
        output.push_str(&format!("+++ b/{}\n", self.filename));
        output.push_str(&format!("# Resource: {}\n", self.resource_id));
        output.push_str(&format!("# Monthly Savings: ${:.2}\n", self.metadata.monthly_savings));
        output.push_str(&format!("# Confidence: {:.0}%\n", self.metadata.confidence * 100.0));
        output.push_str("\n");

        // Hunks
        for hunk in &self.hunks {
            output.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.old_start, hunk.old_count,
                hunk.new_start, hunk.new_count
            ));

            for line in &hunk.lines {
                let prefix = match line.line_type {
                    PatchLineType::Context => " ",
                    PatchLineType::Addition => "+",
                    PatchLineType::Deletion => "-",
                };
                output.push_str(&format!("{}{}\n", prefix, line.content));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_downsize() {
        assert_eq!(PatchGenerator::recommend_instance_downsize("t3.large"), "t3.medium");
        assert_eq!(PatchGenerator::recommend_instance_downsize("m5.xlarge"), "m5.large");
    }

    #[test]
    fn test_infer_filename() {
        assert_eq!(PatchGenerator::infer_filename("aws_instance.web"), "compute.tf");
        assert_eq!(PatchGenerator::infer_filename("aws_rds_instance.db"), "database.tf");
        assert_eq!(PatchGenerator::infer_filename("aws_lambda_function.api"), "lambda.tf");
    }

    #[test]
    fn test_extract_name() {
        assert_eq!(PatchGenerator::extract_name("aws_instance.web"), "web");
        assert_eq!(PatchGenerator::extract_name("module.app.aws_instance.server"), "server");
    }
}
