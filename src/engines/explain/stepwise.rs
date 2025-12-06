// Stepwise reasoning for cost predictions and detections

use serde::{Deserialize, Serialize};
use crate::engines::shared::models::{ResourceChange, CostEstimate};
use crate::engines::prediction::prediction_engine::CostHeuristics;

/// A single reasoning step in the explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_number: usize,
    pub category: ReasoningCategory,
    pub title: String,
    pub description: String,
    pub input_values: Vec<InputValue>,
    pub calculation: Option<String>,
    pub output_value: Option<OutputValue>,
    pub confidence_impact: Option<ConfidenceImpact>,
    pub assumptions: Vec<String>,
}

/// Category of reasoning step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasoningCategory {
    ResourceIdentification,
    ConfigurationExtraction,
    HeuristicLookup,
    ColdStartInference,
    Calculation,
    AdjustmentFactor,
    ConfidenceScoring,
    IntervalEstimation,
}

/// Input value for a reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValue {
    pub name: String,
    pub value: String,
    pub source: ValueSource,
}

/// Source of input value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueSource {
    TerraformPlan,
    CostHeuristics,
    ColdStartInference,
    DefaultValue,
    PreviousStep,
}

/// Output value from a reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputValue {
    pub name: String,
    pub value: String,
    pub unit: Option<String>,
}

/// Impact on confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceImpact {
    pub factor: String,
    pub impact: f64, // -1.0 to 1.0
    pub reasoning: String,
}

/// Complete reasoning chain for a prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub resource_id: String,
    pub resource_type: String,
    pub steps: Vec<ReasoningStep>,
    pub final_estimate: FinalEstimate,
    pub overall_confidence: f64,
    pub key_assumptions: Vec<String>,
}

/// Final estimate with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalEstimate {
    pub monthly_cost: f64,
    pub interval_low: f64,
    pub interval_high: f64,
    pub components: Vec<CostComponent>,
}

/// Component of final cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostComponent {
    pub name: String,
    pub cost: f64,
    pub percentage: f64,
}

impl ReasoningChain {
    /// Create a new empty reasoning chain
    pub fn new(resource_id: String, resource_type: String) -> Self {
        Self {
            resource_id,
            resource_type,
            steps: Vec::new(),
            final_estimate: FinalEstimate {
                monthly_cost: 0.0,
                interval_low: 0.0,
                interval_high: 0.0,
                components: Vec::new(),
            },
            overall_confidence: 0.0,
            key_assumptions: Vec::new(),
        }
    }

    /// Add a reasoning step
    pub fn add_step(&mut self, step: ReasoningStep) {
        self.steps.push(step);
    }

    /// Set final estimate
    pub fn set_final_estimate(&mut self, estimate: FinalEstimate) {
        self.final_estimate = estimate;
    }

    /// Set overall confidence
    pub fn set_overall_confidence(&mut self, confidence: f64) {
        self.overall_confidence = confidence;
    }

    /// Add key assumption
    pub fn add_assumption(&mut self, assumption: String) {
        self.key_assumptions.push(assumption);
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Format as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("🔍 Cost Analysis: {}\n", self.resource_id));
        output.push_str(&format!("Type: {}\n\n", self.resource_type));
        
        output.push_str("Reasoning Steps:\n");
        output.push_str("===============\n\n");
        
        for step in &self.steps {
            output.push_str(&format!("Step {}: {} ({})\n", 
                step.step_number, 
                step.title, 
                format_category(&step.category)
            ));
            output.push_str(&format!("  {}\n", step.description));
            
            if !step.input_values.is_empty() {
                output.push_str("  Inputs:\n");
                for input in &step.input_values {
                    output.push_str(&format!("    • {} = {} (from {})\n", 
                        input.name, input.value, format_source(&input.source)));
                }
            }
            
            if let Some(calc) = &step.calculation {
                output.push_str(&format!("  Calculation: {}\n", calc));
            }
            
            if let Some(out) = &step.output_value {
                let unit_str = out.unit.as_ref().map(|u| format!(" {}", u)).unwrap_or_default();
                output.push_str(&format!("  Output: {} = {}{}\n", out.name, out.value, unit_str));
            }
            
            if let Some(impact) = &step.confidence_impact {
                let sign = if impact.impact >= 0.0 { "+" } else { "" };
                output.push_str(&format!("  Confidence Impact: {}{:.1}% ({})\n", 
                    sign, impact.impact * 100.0, impact.reasoning));
            }
            
            if !step.assumptions.is_empty() {
                output.push_str("  Assumptions:\n");
                for assumption in &step.assumptions {
                    output.push_str(&format!("    - {}\n", assumption));
                }
            }
            
            output.push_str("\n");
        }
        
        output.push_str("Final Estimate:\n");
        output.push_str("==============\n");
        output.push_str(&format!("Monthly Cost: ${:.2}\n", self.final_estimate.monthly_cost));
        output.push_str(&format!("Range: ${:.2} - ${:.2}\n", 
            self.final_estimate.interval_low, self.final_estimate.interval_high));
        output.push_str(&format!("Confidence: {:.0}%\n\n", self.overall_confidence * 100.0));
        
        if !self.final_estimate.components.is_empty() {
            output.push_str("Cost Breakdown:\n");
            for component in &self.final_estimate.components {
                output.push_str(&format!("  • {}: ${:.2} ({:.1}%)\n", 
                    component.name, component.cost, component.percentage));
            }
            output.push_str("\n");
        }
        
        if !self.key_assumptions.is_empty() {
            output.push_str("Key Assumptions:\n");
            for (idx, assumption) in self.key_assumptions.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", idx + 1, assumption));
            }
        }
        
        output
    }
}

fn format_category(category: &ReasoningCategory) -> &'static str {
    match category {
        ReasoningCategory::ResourceIdentification => "Resource ID",
        ReasoningCategory::ConfigurationExtraction => "Config Extract",
        ReasoningCategory::HeuristicLookup => "Heuristic Lookup",
        ReasoningCategory::ColdStartInference => "Cold Start",
        ReasoningCategory::Calculation => "Calculation",
        ReasoningCategory::AdjustmentFactor => "Adjustment",
        ReasoningCategory::ConfidenceScoring => "Confidence",
        ReasoningCategory::IntervalEstimation => "Interval",
    }
}

fn format_source(source: &ValueSource) -> &'static str {
    match source {
        ValueSource::TerraformPlan => "Terraform plan",
        ValueSource::CostHeuristics => "cost heuristics",
        ValueSource::ColdStartInference => "cold start inference",
        ValueSource::DefaultValue => "default value",
        ValueSource::PreviousStep => "previous step",
    }
}

/// Builder for reasoning chains
pub struct ReasoningChainBuilder {
    chain: ReasoningChain,
    step_counter: usize,
}

impl ReasoningChainBuilder {
    /// Create new builder
    pub fn new(resource_id: String, resource_type: String) -> Self {
        Self {
            chain: ReasoningChain::new(resource_id, resource_type),
            step_counter: 0,
        }
    }

    /// Add resource identification step
    pub fn add_resource_identification(&mut self, resource_id: &str, resource_type: &str) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::ResourceIdentification,
            title: "Identify Resource".to_string(),
            description: format!("Detected {} resource requiring cost analysis", resource_type),
            input_values: vec![
                InputValue {
                    name: "resource_id".to_string(),
                    value: resource_id.to_string(),
                    source: ValueSource::TerraformPlan,
                },
                InputValue {
                    name: "resource_type".to_string(),
                    value: resource_type.to_string(),
                    source: ValueSource::TerraformPlan,
                },
            ],
            calculation: None,
            output_value: None,
            confidence_impact: Some(ConfidenceImpact {
                factor: "Resource Type Known".to_string(),
                impact: 0.1,
                reasoning: "CostPilot has heuristics for this resource type".to_string(),
            }),
            assumptions: vec![],
        });
        
        self
    }

    /// Add configuration extraction step
    pub fn add_configuration_extraction(&mut self, config_key: &str, config_value: &str, from_plan: bool) -> &mut Self {
        self.step_counter += 1;
        
        let source = if from_plan { ValueSource::TerraformPlan } else { ValueSource::DefaultValue };
        let confidence_impact = if from_plan {
            Some(ConfidenceImpact {
                factor: "Configuration Explicit".to_string(),
                impact: 0.05,
                reasoning: "Value explicitly defined in Terraform".to_string(),
            })
        } else {
            Some(ConfidenceImpact {
                factor: "Configuration Assumed".to_string(),
                impact: -0.1,
                reasoning: "Using default value, actual may differ".to_string(),
            })
        };
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::ConfigurationExtraction,
            title: format!("Extract {}", config_key),
            description: format!("Retrieved {} from resource configuration", config_key),
            input_values: vec![
                InputValue {
                    name: config_key.to_string(),
                    value: config_value.to_string(),
                    source,
                },
            ],
            calculation: None,
            output_value: Some(OutputValue {
                name: config_key.to_string(),
                value: config_value.to_string(),
                unit: None,
            }),
            confidence_impact,
            assumptions: if from_plan { 
                vec![] 
            } else { 
                vec![format!("{} not specified, using AWS default: {}", config_key, config_value)] 
            },
        });
        
        self
    }

    /// Add heuristic lookup step
    pub fn add_heuristic_lookup(&mut self, key: &str, value: f64, unit: &str, heuristics_version: &str) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::HeuristicLookup,
            title: format!("Lookup {} Price", key),
            description: format!("Retrieved pricing from cost heuristics database (v{})", heuristics_version),
            input_values: vec![
                InputValue {
                    name: "lookup_key".to_string(),
                    value: key.to_string(),
                    source: ValueSource::PreviousStep,
                },
            ],
            calculation: None,
            output_value: Some(OutputValue {
                name: format!("{}_price", key),
                value: format!("{:.6}", value),
                unit: Some(unit.to_string()),
            }),
            confidence_impact: Some(ConfidenceImpact {
                factor: "Heuristic Available".to_string(),
                impact: 0.15,
                reasoning: "AWS pricing data available in heuristics".to_string(),
            }),
            assumptions: vec![
                format!("Pricing from heuristics v{} (last updated from AWS pricing API)", heuristics_version),
                "Pricing for us-east-1 region".to_string(),
            ],
        });
        
        self
    }

    /// Add cold start inference step
    pub fn add_cold_start_inference(&mut self, what: &str, inferred_value: &str, reasoning: &str) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::ColdStartInference,
            title: format!("Infer {}", what),
            description: reasoning.to_string(),
            input_values: vec![],
            calculation: Some(reasoning.to_string()),
            output_value: Some(OutputValue {
                name: what.to_string(),
                value: inferred_value.to_string(),
                unit: None,
            }),
            confidence_impact: Some(ConfidenceImpact {
                factor: "Cold Start Inference".to_string(),
                impact: -0.2,
                reasoning: "Estimated value, not from heuristics".to_string(),
            }),
            assumptions: vec![
                format!("{} not found in heuristics, using inference model", what),
            ],
        });
        
        self.chain.add_assumption(format!("{} inferred using cold-start logic: {}", what, reasoning));
        
        self
    }

    /// Add calculation step
    pub fn add_calculation(&mut self, operation: &str, formula: &str, result: f64, unit: &str) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::Calculation,
            title: operation.to_string(),
            description: format!("Applying formula: {}", formula),
            input_values: vec![],
            calculation: Some(formula.to_string()),
            output_value: Some(OutputValue {
                name: operation.to_string(),
                value: format!("{:.2}", result),
                unit: Some(unit.to_string()),
            }),
            confidence_impact: None,
            assumptions: vec![],
        });
        
        self
    }

    /// Add adjustment factor step
    pub fn add_adjustment(&mut self, factor_name: &str, multiplier: f64, reasoning: &str) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::AdjustmentFactor,
            title: format!("Apply {}", factor_name),
            description: reasoning.to_string(),
            input_values: vec![
                InputValue {
                    name: "multiplier".to_string(),
                    value: format!("{:.2}", multiplier),
                    source: ValueSource::CostHeuristics,
                },
            ],
            calculation: Some(format!("cost × {:.2}", multiplier)),
            output_value: None,
            confidence_impact: None,
            assumptions: vec![reasoning.to_string()],
        });
        
        self
    }

    /// Add confidence scoring step
    pub fn add_confidence_scoring(&mut self, confidence: f64, factors: Vec<String>) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::ConfidenceScoring,
            title: "Calculate Confidence".to_string(),
            description: "Aggregate confidence from all factors".to_string(),
            input_values: factors.iter().map(|f| InputValue {
                name: "factor".to_string(),
                value: f.clone(),
                source: ValueSource::PreviousStep,
            }).collect(),
            calculation: Some("Base 0.5 + sum of all confidence impacts".to_string()),
            output_value: Some(OutputValue {
                name: "confidence_score".to_string(),
                value: format!("{:.0}%", confidence * 100.0),
                unit: None,
            }),
            confidence_impact: None,
            assumptions: vec![],
        });
        
        self.chain.set_overall_confidence(confidence);
        
        self
    }

    /// Add interval estimation step
    pub fn add_interval_estimation(&mut self, range_factor: f64, low: f64, high: f64) -> &mut Self {
        self.step_counter += 1;
        
        self.chain.add_step(ReasoningStep {
            step_number: self.step_counter,
            category: ReasoningCategory::IntervalEstimation,
            title: "Calculate Prediction Interval".to_string(),
            description: format!("Apply ±{:.0}% range to account for uncertainty", range_factor * 100.0),
            input_values: vec![
                InputValue {
                    name: "range_factor".to_string(),
                    value: format!("{:.0}%", range_factor * 100.0),
                    source: ValueSource::CostHeuristics,
                },
            ],
            calculation: Some(format!("[estimate × (1 - {:.2}), estimate × (1 + {:.2})]", range_factor, range_factor)),
            output_value: Some(OutputValue {
                name: "interval".to_string(),
                value: format!("${:.2} - ${:.2}", low, high),
                unit: Some("monthly".to_string()),
            }),
            confidence_impact: None,
            assumptions: vec![
                format!("Prediction interval accounts for ±{:.0}% variance", range_factor * 100.0),
            ],
        });
        
        self
    }

    /// Set final estimate
    pub fn set_final_estimate(&mut self, monthly_cost: f64, low: f64, high: f64, components: Vec<CostComponent>) -> &mut Self {
        self.chain.set_final_estimate(FinalEstimate {
            monthly_cost,
            interval_low: low,
            interval_high: high,
            components,
        });
        
        self
    }

    /// Build the chain
    pub fn build(self) -> ReasoningChain {
        self.chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_chain_builder() {
        let mut builder = ReasoningChainBuilder::new(
            "aws_instance.web".to_string(),
            "aws_instance".to_string(),
        );
        
        builder
            .add_resource_identification("aws_instance.web", "aws_instance")
            .add_configuration_extraction("instance_type", "t3.micro", true)
            .add_heuristic_lookup("t3.micro", 0.0104, "$/hour", "1.0.0")
            .add_calculation("Monthly Cost", "0.0104 × 730 hours", 7.6, "$/month")
            .add_confidence_scoring(0.85, vec!["Known resource".to_string(), "Explicit config".to_string()])
            .add_interval_estimation(0.25, 5.7, 9.5)
            .set_final_estimate(7.6, 5.7, 9.5, vec![
                CostComponent {
                    name: "Compute".to_string(),
                    cost: 7.6,
                    percentage: 100.0,
                },
            ]);
        
        let chain = builder.build();
        
        assert_eq!(chain.step_count(), 6);
        assert_eq!(chain.overall_confidence, 0.85);
        assert_eq!(chain.final_estimate.monthly_cost, 7.6);
    }

    #[test]
    fn test_reasoning_chain_format() {
        let mut builder = ReasoningChainBuilder::new(
            "aws_instance.test".to_string(),
            "aws_instance".to_string(),
        );
        
        builder.add_resource_identification("aws_instance.test", "aws_instance");
        
        let chain = builder.build();
        let formatted = chain.format_text();
        
        assert!(formatted.contains("Cost Analysis"));
        assert!(formatted.contains("aws_instance.test"));
        assert!(formatted.contains("Reasoning Steps"));
    }
}
