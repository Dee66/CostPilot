// Detection engine - main orchestrator

use crate::engines::detection::classifier::RegressionClassifier;
use crate::engines::detection::severity::{calculate_severity_score, score_to_severity};
use crate::engines::detection::terraform::{convert_to_resource_changes, parse_terraform_plan};
use crate::engines::explain::anti_patterns;
use crate::engines::shared::error_model::{CostPilotError, ErrorCategory, Result};
use crate::engines::shared::models::{
    CostEstimate, Detection, RegressionType, ResourceChange, Severity,
};
use std::collections::HashMap;
use std::path::Path;

/// Main detection engine
pub struct DetectionEngine {
    /// Enable verbose logging
    verbose: bool,
    /// Enable advanced optimization detection
    enable_advanced_detection: bool,
}

impl DetectionEngine {
    /// Create a new detection engine
    pub fn new() -> Self {
        Self {
            verbose: false,
            enable_advanced_detection: true, // Enable by default
        }
    }

    /// Enable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable or disable advanced optimization detection
    pub fn with_advanced_detection(mut self, enable: bool) -> Self {
        self.enable_advanced_detection = enable;
        self
    }

    /// Detect cost issues from Terraform plan JSON file
    pub fn detect_from_terraform_plan(&self, plan_path: &Path) -> Result<Vec<ResourceChange>> {
        // Read the plan file
        let content = std::fs::read_to_string(plan_path).map_err(|e| {
            CostPilotError::new(
                "DETECT_001",
                ErrorCategory::FileSystemError,
                format!("Failed to read Terraform plan file: {}", e),
            )
            .with_hint(format!(
                "Ensure the file exists and is readable: {}",
                plan_path.display()
            ))
        })?;

        self.detect_from_terraform_json(&content)
    }

    /// Detect cost issues from Terraform plan JSON string
    pub fn detect_from_terraform_json(&self, json_content: &str) -> Result<Vec<ResourceChange>> {
        if self.verbose {
            println!("Parsing Terraform plan JSON...");
        }

        // Parse the Terraform plan
        let plan = parse_terraform_plan(json_content)?;

        if self.verbose {
            println!("Terraform version: {:?}", plan.terraform_version);
            println!("Format version: {}", plan.format_version);
        }

        // Convert to canonical format
        let changes = convert_to_resource_changes(&plan)?;

        if self.verbose {
            println!("Detected {} resource changes", changes.len());
        }

        Ok(changes)
    }

    /// Detect cost issues from CDK diff JSON file
    pub fn detect_from_cdk_diff(&self, diff_path: &Path) -> Result<Vec<ResourceChange>> {
        // Read the diff file
        let content = std::fs::read_to_string(diff_path).map_err(|e| {
            CostPilotError::new(
                "DETECT_004",
                ErrorCategory::FileSystemError,
                format!("Failed to read CDK diff file: {}", e),
            )
            .with_hint(format!(
                "Ensure the file exists and is readable: {}",
                diff_path.display()
            ))
        })?;

        self.detect_from_cdk_diff_json(&content)
    }

    /// Detect cost issues from CDK diff JSON string
    pub fn detect_from_cdk_diff_json(&self, json_content: &str) -> Result<Vec<ResourceChange>> {
        if self.verbose {
            println!("Parsing CDK diff JSON...");
        }

        // Parse the CDK diff
        let diff = crate::engines::detection::cdk::parser::parse_cdk_diff(json_content)?;

        if self.verbose {
            println!("CDK diff success: {}", diff.success);
            println!("Stacks: {}", diff.stacks.len());
        }

        // Convert to canonical format
        let changes = crate::engines::detection::cdk::parser::cdk_diff_to_resource_changes(&diff);

        if self.verbose {
            println!("Detected {} resource changes", changes.len());
        }

        Ok(changes)
    }

    /// Detect cost issues from CDK synthesized template directory
    pub fn detect_from_cdk_synthesized(
        &self,
        cdk_out_path: &Path,
        stack_name: &str,
    ) -> Result<Vec<ResourceChange>> {
        let template_path = cdk_out_path.join(format!("{}.template.json", stack_name));

        if !template_path.exists() {
            return Err(CostPilotError::new(
                "DETECT_005",
                ErrorCategory::FileSystemError,
                format!(
                    "CDK synthesized template not found: {}",
                    template_path.display()
                ),
            )
            .with_hint(format!(
                "Run 'cdk synth' first to generate synthesized templates in {}",
                cdk_out_path.display()
            )));
        }

        // Read the template file
        let content = std::fs::read_to_string(&template_path).map_err(|e| {
            CostPilotError::new(
                "DETECT_006",
                ErrorCategory::FileSystemError,
                format!("Failed to read CDK synthesized template: {}", e),
            )
        })?;

        self.detect_from_cdk_template_json(&content, stack_name)
    }

    /// Detect cost issues from CDK synthesized template JSON string
    pub fn detect_from_cdk_template_json(
        &self,
        json_content: &str,
        stack_name: &str,
    ) -> Result<Vec<ResourceChange>> {
        if self.verbose {
            println!("Parsing CDK synthesized template JSON...");
        }

        // Parse the CDK template
        let template = crate::engines::detection::cdk::parser::parse_cdk_template(json_content)?;

        if self.verbose {
            let resource_count = template.resources.as_ref().map(|r| r.len()).unwrap_or(0);
            println!("Template resources: {}", resource_count);
        }

        // Convert to canonical format
        let changes = crate::engines::detection::cdk::parser::cdk_template_to_resource_changes(
            &template, stack_name,
        );

        if self.verbose {
            println!("Detected {} resource changes", changes.len());
        }

        Ok(changes)
    }

    /// Analyze resource changes and generate detections
    pub fn analyze_changes(
        &self,
        changes: &[ResourceChange],
        cost_estimates: &[(String, f64, f64)], // (resource_id, cost, confidence)
    ) -> Result<Vec<Detection>> {
        let mut detections = Vec::new();

        // Build estimates map for batch detection
        let estimates_map: HashMap<String, CostEstimate> = cost_estimates
            .iter()
            .map(|(id, cost, conf)| {
                (
                    id.clone(),
                    CostEstimate::builder()
                        .resource_id(id.clone())
                        .monthly_cost(*cost)
                        .confidence_score(*conf)
                        .prediction_interval_low(cost * 0.8)
                        .prediction_interval_high(cost * 1.2)
                        .build(),
                )
            })
            .collect();

        // Run advanced batch detection if enabled
        if self.enable_advanced_detection {
            if self.verbose {
                println!("Running advanced optimization detection...");
            }
            let advanced_patterns =
                anti_patterns::detect_anti_patterns_batch(changes, &estimates_map);

            if self.verbose {
                println!(
                    "Detected {} advanced optimization opportunities",
                    advanced_patterns.len()
                );
            }

            // Convert anti-patterns to detections
            for pattern in advanced_patterns {
                let severity = match pattern.severity.as_str() {
                    "HIGH" => Severity::High,
                    "MEDIUM" => Severity::Medium,
                    "LOW" => Severity::Low,
                    "CRITICAL" => Severity::Critical,
                    _ => Severity::Medium,
                };

                detections.push(Detection {
                    rule_id: pattern.pattern_id.clone(),
                    severity: severity.clone(),
                    resource_id: pattern.detected_in.clone(),
                    regression_type: RegressionType::Configuration, // Advanced patterns are config-based
                    severity_score: match severity {
                        Severity::Critical => 90,
                        Severity::High => 70,
                        Severity::Medium => 45,
                        Severity::Low => 20,
                    },
                    message: format!("{}: {}", pattern.pattern_name, pattern.description),
                    fix_snippet: pattern.suggested_fix.clone(),
                    estimated_cost: pattern.cost_impact,
                });
            }
        }

        // Original per-resource detection for baseline anti-patterns
        for change in changes {
            // Find cost estimate for this resource
            let (cost_delta, confidence) = cost_estimates
                .iter()
                .find(|(id, _, _)| id == &change.resource_id)
                .map(|(_, cost, conf)| (*cost, *conf))
                .unwrap_or((0.0, 0.5));

            // Classify the regression
            let regression_type = RegressionClassifier::classify(change);

            // Calculate severity
            let severity_score =
                calculate_severity_score(change, cost_delta, &regression_type, confidence);
            let severity = score_to_severity(severity_score);

            // Detect specific anti-patterns (legacy per-resource)
            if let Some(detection) = self.detect_anti_patterns(
                change,
                &regression_type,
                severity,
                severity_score,
                cost_delta,
            ) {
                detections.push(detection);
            }
        }

        Ok(detections)
    }

    /// Detect specific cost anti-patterns
    fn detect_anti_patterns(
        &self,
        change: &ResourceChange,
        regression_type: &RegressionType,
        severity: Severity,
        severity_score: u32,
        cost_delta: f64,
    ) -> Option<Detection> {
        // NAT Gateway overuse
        if change.resource_type == "aws_nat_gateway" && cost_delta > 100.0 {
            return Some(Detection {
                rule_id: "NAT_GATEWAY_COST".to_string(),
                severity: severity.clone(),
                resource_id: change.resource_id.clone(),
                regression_type: regression_type.clone(),
                severity_score,
                message: format!(
                    "NAT Gateway cost increase of ${:.2}/month. Consider using VPC endpoints or reducing NAT gateway count.",
                    cost_delta
                ),
                fix_snippet: None,
                estimated_cost: None,
            });
        }

        // Overprovisioned EC2
        if change.resource_type == "aws_instance" {
            if let Some(config) = &change.new_config {
                if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
                    if instance_type.contains("xlarge") && cost_delta > 200.0 {
                        return Some(Detection {
                            rule_id: "OVERPROVISIONED_EC2".to_string(),
                            severity: severity.clone(),
                            resource_id: change.resource_id.clone(),
                            regression_type: regression_type.clone(),
                            severity_score,
                            message: format!(
                                "Large EC2 instance type '{}' with ${:.2}/month cost. Consider rightsizing.",
                                instance_type, cost_delta
                            ),
                            fix_snippet: None,
                            estimated_cost: None,
                        });
                    }
                }
            }
        }

        // S3 missing lifecycle
        if change.resource_type == "aws_s3_bucket" {
            if let Some(config) = &change.new_config {
                if config.get("lifecycle_rule").is_none() && cost_delta > 50.0 {
                    return Some(Detection {
                        rule_id: "S3_MISSING_LIFECYCLE".to_string(),
                        severity: Severity::Medium,
                        resource_id: change.resource_id.clone(),
                        regression_type: regression_type.clone(),
                        severity_score,
                        message: "S3 bucket without lifecycle rules. Consider adding policies to transition old data to cheaper storage.".to_string(),
                        fix_snippet: None,
                        estimated_cost: None,
                    });
                }
            }
        }

        // Default: generic high-cost detection
        if cost_delta > 300.0 {
            return Some(Detection {
                rule_id: "HIGH_COST_CHANGE".to_string(),
                severity,
                resource_id: change.resource_id.clone(),
                regression_type: regression_type.clone(),
                severity_score,
                message: format!(
                    "Significant cost increase of ${:.2}/month detected for {}",
                    cost_delta, change.resource_type
                ),
                fix_snippet: None,
                estimated_cost: None,
            });
        }

        None
    }

    /// Detect cost issues from resource changes (convenience method)
    pub fn detect(&self, changes: &[ResourceChange]) -> Result<Vec<Detection>> {
        // For now, analyze without cost estimates (use defaults)
        let cost_estimates: Vec<(String, f64, f64)> = changes
            .iter()
            .map(|c| (c.resource_id.clone(), 0.0, 0.5))
            .collect();

        self.analyze_changes(changes, &cost_estimates)
    }

    /// Detect cost issues from a file (convenience method)
    pub fn detect_from_file(&self, plan_path: &Path) -> Result<Vec<ResourceChange>> {
        self.detect_from_terraform_plan(plan_path)
    }
}

impl Default for DetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_engine_creation() {
        let engine = DetectionEngine::new();
        assert!(!engine.verbose);

        let engine = DetectionEngine::new().with_verbose(true);
        assert!(engine.verbose);
    }

    #[test]
    fn test_detect_from_json() {
        let json = r#"{
            "format_version": "1.2",
            "resource_changes": [
                {
                    "address": "aws_instance.test",
                    "type": "aws_instance",
                    "name": "test",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {"instance_type": "t3.micro"}
                    }
                }
            ]
        }"#;

        let engine = DetectionEngine::new();
        let result = engine.detect_from_terraform_json(json);
        assert!(result.is_ok());

        let changes = result.unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].resource_type, "aws_instance");
    }
}
