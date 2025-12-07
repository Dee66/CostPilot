// Explain engine - provides root cause analysis and stepwise reasoning

use crate::engines::shared::models::{Detection, ResourceChange, CostEstimate};
use crate::engines::prediction::calculation_steps::{CalculationStep, CalculationBreakdown};
use crate::engines::explain::anti_patterns::{AntiPattern, detect_anti_patterns};
use crate::engines::explain::root_cause::RootCauseAnalysis;
use serde::{Deserialize, Serialize};

/// Full explanation for a detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    pub resource_id: String,
    pub resource_type: String,
    pub summary: String,
    pub root_cause: RootCauseAnalysis,
    pub prediction_steps: Option<CalculationBreakdown>,
    pub detection_reasoning: DetectionReasoning,
    pub anti_patterns: Vec<AntiPattern>,
    pub recommendations: Vec<String>,
    pub assumptions: Vec<String>,
}

/// Detection reasoning breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionReasoning {
    pub regression_type: String,
    pub severity_score: i32,
    pub severity_factors: Vec<SeverityFactor>,
    pub confidence: f64,
}

/// Factor contributing to severity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityFactor {
    pub name: String,
    pub value: f64,
    pub weight: f64,
    pub contribution: i32,
    pub reasoning: String,
}

pub struct ExplainEngine;

impl ExplainEngine {
    /// Generate full explanation for a detection
    pub fn explain(
        detection: &Detection,
        change: &ResourceChange,
        estimate: Option<&CostEstimate>,
        calculation_steps: Option<CalculationBreakdown>,
    ) -> Explanation {
        let anti_patterns = detect_anti_patterns(change, estimate);
        let root_cause = RootCauseAnalysis::analyze(change, detection, &anti_patterns);
        let detection_reasoning = Self::build_detection_reasoning(detection, change, estimate);
        let recommendations = Self::generate_recommendations(change, detection, &anti_patterns);
        let assumptions = Self::extract_assumptions(change, estimate, &calculation_steps);
        let summary = Self::build_summary(detection, &root_cause, &anti_patterns);

        Explanation {
            resource_id: detection.resource_id.clone(),
            resource_type: change.resource_type.clone(),
            summary,
            root_cause,
            prediction_steps: calculation_steps,
            detection_reasoning,
            anti_patterns,
            recommendations,
            assumptions,
        }
    }

    /// Build detection reasoning with severity factors
    fn build_detection_reasoning(detection: &Detection, change: &ResourceChange, estimate: Option<&CostEstimate>) -> DetectionReasoning {
        let mut factors = Vec::new();

        // Extract magnitude factor (45% weight)
        if let Some(est) = estimate {
            let magnitude_value = est.monthly_cost / 100.0; // Normalize
            factors.push(SeverityFactor {
                name: "Cost Magnitude".to_string(),
                value: magnitude_value,
                weight: 0.45,
                contribution: (magnitude_value * 0.45 * 100.0) as i32,
                reasoning: format!("Estimated monthly cost: ${:.2}", est.monthly_cost),
            });
        }

        // Confidence factor (25% weight)
        if let Some(est) = estimate {
            factors.push(SeverityFactor {
                name: "Confidence".to_string(),
                value: est.confidence_score,
                weight: 0.25,
                contribution: (est.confidence_score * 0.25 * 100.0) as i32,
                reasoning: format!("Prediction confidence: {:.0}%", est.confidence_score * 100.0),
            });
        }

        // Resource importance (20% weight) - inferred from resource type
        let importance = Self::resource_importance(&change.resource_type);
        factors.push(SeverityFactor {
            name: "Resource Importance".to_string(),
            value: importance,
            weight: 0.20,
            contribution: (importance * 0.20 * 100.0) as i32,
            reasoning: format!("{} is a {} importance resource type", change.resource_type, Self::importance_label(importance)),
        });

        // Blast radius (10% weight) - based on tags and module depth
        let blast_radius = 0.5; // TODO: Calculate based on tags
        factors.push(SeverityFactor {
            name: "Blast Radius".to_string(),
            value: blast_radius,
            weight: 0.10,
            contribution: (blast_radius * 0.10 * 100.0) as i32,
            reasoning: "Impact scope based on module structure and tags".to_string(),
        });

        DetectionReasoning {
            regression_type: format!("{:?}", detection.regression_type),
            severity_score: detection.severity_score as i32,
            severity_factors: factors,
            confidence: estimate.map(|e| e.confidence_score).unwrap_or(0.5),
        }
    }

    /// Get resource importance score
    fn resource_importance(resource_type: &str) -> f64 {
        match resource_type {
            "aws_instance" | "aws_rds_instance" | "aws_eks_cluster" => 0.9,
            "aws_nat_gateway" | "aws_lb" | "aws_dynamodb_table" => 0.7,
            "aws_lambda_function" | "aws_s3_bucket" => 0.5,
            _ => 0.4,
        }
    }

    /// Get importance label
    fn importance_label(score: f64) -> &'static str {
        if score >= 0.8 { "high" } 
        else if score >= 0.6 { "medium-high" }
        else if score >= 0.4 { "medium" }
        else { "low" }
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        change: &ResourceChange,
        detection: &Detection,
        anti_patterns: &[AntiPattern],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Add anti-pattern specific recommendations
        for pattern in anti_patterns {
            if let Some(fix) = &pattern.suggested_fix {
                recommendations.push(fix.clone());
            }
        }

        // Add general recommendations based on resource type
        match change.resource_type.as_str() {
            "aws_instance" => {
                recommendations.push("Consider using Reserved Instances or Savings Plans for long-running workloads".to_string());
                recommendations.push("Evaluate right-sizing with AWS Compute Optimizer".to_string());
            }
            "aws_rds_instance" => {
                recommendations.push("Consider Aurora Serverless for variable workloads".to_string());
                recommendations.push("Review storage autoscaling settings".to_string());
            }
            "aws_lambda_function" => {
                recommendations.push("Review memory allocation - higher memory = faster execution = potentially lower cost".to_string());
                recommendations.push("Consider Provisioned Concurrency for predictable traffic".to_string());
            }
            "aws_nat_gateway" => {
                recommendations.push("Consider VPC endpoints to reduce NAT Gateway data transfer".to_string());
                recommendations.push("Consolidate NAT Gateways across availability zones if high availability not required".to_string());
            }
            "aws_s3_bucket" => {
                recommendations.push("Implement Intelligent-Tiering for automatic cost optimization".to_string());
                recommendations.push("Review storage analytics to optimize lifecycle policies".to_string());
            }
            _ => {}
        }

        recommendations
    }

    /// Extract assumptions from calculation steps
    fn extract_assumptions(
        change: &ResourceChange,
        estimate: Option<&CostEstimate>,
        calculation_steps: &Option<CalculationBreakdown>,
    ) -> Vec<String> {
        let mut assumptions = Vec::new();

        // Check for unknown values
        if let Some(config) = &change.new_config {
            if config.is_null() || (config.is_object() && config.as_object().unwrap().values().any(|v| v.is_null())) {
                assumptions.push("Some configuration values were unknown at plan time - cold start inference was used".to_string());
            }
        }

        // Check if cold start was used
        if let Some(breakdown) = calculation_steps {
            if breakdown.cold_start_used {
                assumptions.push("Cost estimate used default values for unknown configuration".to_string());
            }
        }

        // Add default assumptions
        assumptions.push("Pricing based on us-east-1 (adjustments may be needed for other regions)".to_string());
        assumptions.push("Estimates assume 730 hours/month (30.4 days) for hourly resources".to_string());

        // Resource-specific assumptions
        match change.resource_type.as_str() {
            "aws_lambda_function" => {
                assumptions.push("Lambda cost estimate assumes 10,000 invocations/month (adjust based on actual traffic)".to_string());
            }
            "aws_nat_gateway" => {
                assumptions.push("NAT Gateway data transfer estimate assumes 10 GB/month (adjust based on actual usage)".to_string());
            }
            "aws_s3_bucket" => {
                assumptions.push("S3 storage estimate assumes 50 GB (adjust based on actual usage)".to_string());
            }
            "aws_dynamodb_table" => {
                assumptions.push("DynamoDB provisioned capacity assumes minimal read/write units if not specified".to_string());
            }
            _ => {}
        }

        assumptions
    }

    /// Build summary text
    fn build_summary(detection: &Detection, root_cause: &RootCauseAnalysis, anti_patterns: &[AntiPattern]) -> String {
        let mut summary = format!(
            "Detected {} cost regression in {}",
            format!("{:?}", detection.regression_type).to_lowercase().replace('_', " "),
            detection.resource_type
        );

        if let Some(estimate) = &detection.estimated_cost {
            summary.push_str(&format!(
                " with estimated impact of ${:.2}/month (confidence: {:.0}%)",
                estimate.estimate,
                estimate.confidence_score * 100.0
            ));
        }

        if !anti_patterns.is_empty() {
            summary.push_str(&format!(". {} anti-pattern(s) detected", anti_patterns.len()));
        }

        summary.push('.');
        summary
    }

    /// Generate top 5 patterns explanation (MVP)
    pub fn explain_top_patterns(detections: &[Detection]) -> Vec<String> {
        let mut patterns = Vec::new();

        for detection in detections.iter().take(5) {
            let pattern = format!(
                "• {} in {}: {} severity, ${:.2}/month estimated",
                format!("{:?}", detection.regression_type),
                detection.resource_type,
                format!("{:?}", detection.severity),
                detection.estimated_cost.as_ref().map(|e| e.estimate).unwrap_or(0.0)
            );
            patterns.push(pattern);
        }

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ChangeAction, RegressionType, Severity};
    use std::collections::HashMap;

    #[test]
    fn test_explain_generation() {
        let change = ResourceChange {
            resource_id: "aws_instance.test".to_string(),
            resource_type: "aws_instance".to_string(),
            action: ChangeAction::Create,
            module_path: None,
            old_config: None,
            new_config: Some(serde_json::json!({"instance_type": "t3.micro"})),
            tags: HashMap::new(),
        };

        let estimate = CostEstimate {
            estimate: 7.59,
            lower: 5.69,
            upper: 9.49,
            confidence: 0.95,
        };

        let detection = Detection {
            resource_id: "aws_instance.test".to_string(),
            resource_type: "aws_instance".to_string(),
            regression_type: RegressionType::Configuration,
            severity: Severity::Low,
            estimated_cost: Some(estimate.clone()),
            fix_snippet: None,
        };

        let explanation = ExplainEngine::explain(&detection, &change, Some(&estimate), None);

        assert_eq!(explanation.resource_id, "aws_instance.test");
        assert!(!explanation.recommendations.is_empty());
        assert!(!explanation.assumptions.is_empty());
        assert!(explanation.summary.contains("configuration"));
    }

    #[test]
    fn test_resource_importance() {
        assert_eq!(ExplainEngine::resource_importance("aws_instance"), 0.9);
        assert_eq!(ExplainEngine::resource_importance("aws_lambda_function"), 0.5);
    }

    #[test]
    fn test_top_patterns() {
        let detections = vec![
            Detection {
                resource_id: "test1".to_string(),
                resource_type: "aws_instance".to_string(),
                regression_type: RegressionType::Scaling,
                severity: Severity::High,
                estimated_cost: Some(CostEstimate {
                    estimate: 100.0,
                    lower: 75.0,
                    upper: 125.0,
                    confidence: 0.9,
                }),
                fix_snippet: None,
            },
            Detection {
                resource_id: "test2".to_string(),
                resource_type: "aws_rds_instance".to_string(),
                regression_type: RegressionType::Configuration,
                severity: Severity::Medium,
                estimated_cost: Some(CostEstimate {
                    estimate: 50.0,
                    lower: 37.5,
                    upper: 62.5,
                    confidence: 0.85,
                }),
                fix_snippet: None,
            },
        ];

        let patterns = ExplainEngine::explain_top_patterns(&detections);
        assert_eq!(patterns.len(), 2);
        assert!(patterns[0].contains("Scaling"));
        assert!(patterns[1].contains("Configuration"));
    }
}
