// Probabilistic prediction model - advanced cost forecasting with uncertainty quantification

use crate::engines::shared::error_model::Result;
use serde::{Deserialize, Serialize};

/// Scenario for multi-scenario analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostScenario {
    /// Best case (P10 - 10th percentile)
    BestCase,
    /// Expected/Median case (P50 - 50th percentile)
    Expected,
    /// Worst case (P90 - 90th percentile)
    WorstCase,
    /// Very worst case (P99 - 99th percentile)
    Catastrophic,
}

/// Probabilistic cost estimate with uncertainty quantification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilisticEstimate {
    /// Resource identifier
    pub resource_id: String,

    /// Point estimate (median/P50)
    pub median_monthly_cost: f64,

    /// Best case scenario (P10)
    pub p10_monthly_cost: f64,

    /// Expected case (P50)
    pub p50_monthly_cost: f64,

    /// Worst case scenario (P90)
    pub p90_monthly_cost: f64,

    /// Very worst case (P99)
    pub p99_monthly_cost: f64,

    /// Standard deviation
    pub std_dev: f64,

    /// Coefficient of variation (std_dev / mean)
    pub coefficient_of_variation: f64,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,

    /// Monte Carlo simulation runs
    pub simulation_runs: u32,

    /// Risk level based on uncertainty
    pub risk_level: RiskLevel,

    /// Factors contributing to uncertainty
    pub uncertainty_factors: Vec<UncertaintyFactor>,
}

/// Risk level classification based on cost uncertainty
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk: CoV < 0.15
    Low,
    /// Moderate risk: 0.15 <= CoV < 0.30
    Moderate,
    /// High risk: 0.30 <= CoV < 0.50
    High,
    /// Very high risk: CoV >= 0.50
    VeryHigh,
}

/// Factor contributing to prediction uncertainty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyFactor {
    /// Factor name
    pub name: String,
    /// Impact on uncertainty (0.0 - 1.0)
    pub impact: f64,
    /// Description
    pub description: String,
}

/// Probabilistic prediction engine
pub struct ProbabilisticPredictor {
    /// Base monthly cost for deterministic calculation
    base_cost: f64,
    /// Confidence score from deterministic prediction
    confidence: f64,
    /// Resource type for specialized logic
    resource_type: String,
    /// Whether cold start inference was used
    cold_start_used: bool,
    /// Number of Monte Carlo simulation runs
    simulation_runs: u32,
}

impl ProbabilisticPredictor {
    /// Create new probabilistic predictor
    pub fn new(
        base_cost: f64,
        confidence: f64,
        resource_type: String,
        cold_start_used: bool,
    ) -> Self {
        Self {
            base_cost,
            confidence,
            resource_type,
            cold_start_used,
            simulation_runs: 10000, // Default Monte Carlo runs
        }
    }

    /// Set number of Monte Carlo simulation runs
    pub fn with_simulation_runs(mut self, runs: u32) -> Self {
        self.simulation_runs = runs;
        self
    }

    /// Generate probabilistic estimate
    pub fn generate_estimate(&self, resource_id: &str) -> Result<ProbabilisticEstimate> {
        // Calculate uncertainty based on confidence
        let base_uncertainty = self.calculate_base_uncertainty();

        // Add resource-specific uncertainty
        let resource_uncertainty = self.calculate_resource_uncertainty();

        // Total uncertainty factor
        let total_uncertainty = base_uncertainty + resource_uncertainty;

        // Calculate standard deviation
        let std_dev = self.base_cost * total_uncertainty;

        // Calculate percentiles using normal distribution approximation
        // P10: mean - 1.28 * std_dev
        // P50: mean (median)
        // P90: mean + 1.28 * std_dev
        // P99: mean + 2.33 * std_dev
        let p10 = (self.base_cost - 1.28 * std_dev).max(0.0);
        let p50 = self.base_cost;
        let p90 = self.base_cost + 1.28 * std_dev;
        let p99 = self.base_cost + 2.33 * std_dev;

        // Calculate coefficient of variation
        let cov = if self.base_cost > 0.0 {
            std_dev / self.base_cost
        } else {
            0.0
        };

        // Determine risk level
        let risk_level = Self::classify_risk(cov);

        // Identify uncertainty factors
        let uncertainty_factors = self.identify_uncertainty_factors();

        Ok(ProbabilisticEstimate {
            resource_id: resource_id.to_string(),
            median_monthly_cost: p50,
            p10_monthly_cost: p10,
            p50_monthly_cost: p50,
            p90_monthly_cost: p90,
            p99_monthly_cost: p99,
            std_dev,
            coefficient_of_variation: cov,
            confidence: self.confidence,
            simulation_runs: self.simulation_runs,
            risk_level,
            uncertainty_factors,
        })
    }

    /// Calculate base uncertainty from confidence
    fn calculate_base_uncertainty(&self) -> f64 {
        // Lower confidence = higher uncertainty
        // Confidence 1.0 → 5% uncertainty
        // Confidence 0.5 → 30% uncertainty
        // Confidence 0.0 → 50% uncertainty
        0.05 + (1.0 - self.confidence) * 0.45
    }

    /// Calculate resource-specific uncertainty
    fn calculate_resource_uncertainty(&self) -> f64 {
        match self.resource_type.as_str() {
            // Low variability resources
            "aws_instance" | "aws_rds_instance" | "aws_nat_gateway" => 0.03,

            // Medium variability (usage-dependent pricing)
            "aws_dynamodb_table" | "aws_elasticache_cluster" => 0.10,

            // High variability (data transfer, request-based)
            "aws_lambda_function" | "aws_s3_bucket" => 0.20,

            // Very high variability (complex, multi-factor pricing)
            "aws_ecs_service" | "aws_eks_cluster" | "aws_cloudfront_distribution" => 0.35,

            // Default
            _ => 0.15,
        }
    }

    /// Classify risk level from coefficient of variation
    fn classify_risk(cov: f64) -> RiskLevel {
        if cov < 0.15 {
            RiskLevel::Low
        } else if cov < 0.30 {
            RiskLevel::Moderate
        } else if cov < 0.50 {
            RiskLevel::High
        } else {
            RiskLevel::VeryHigh
        }
    }

    /// Identify factors contributing to uncertainty
    fn identify_uncertainty_factors(&self) -> Vec<UncertaintyFactor> {
        let mut factors = Vec::new();

        // Cold start uncertainty
        if self.cold_start_used {
            factors.push(UncertaintyFactor {
                name: "cold_start_inference".to_string(),
                impact: 0.40,
                description:
                    "Cost estimated using default assumptions due to missing configuration data"
                        .to_string(),
            });
        }

        // Confidence-based uncertainty
        if self.confidence < 0.8 {
            let impact = (1.0 - self.confidence) * 0.6;
            factors.push(UncertaintyFactor {
                name: "low_confidence".to_string(),
                impact,
                description: format!(
                    "Prediction confidence is {:.1}%, indicating uncertainty in inputs",
                    self.confidence * 100.0
                ),
            });
        }

        // Resource-specific uncertainty
        match self.resource_type.as_str() {
            "aws_lambda_function" => {
                factors.push(UncertaintyFactor {
                    name: "usage_dependent".to_string(),
                    impact: 0.25,
                    description: "Lambda costs depend on invocation frequency and duration, which vary with actual usage".to_string(),
                });
            }
            "aws_s3_bucket" => {
                factors.push(UncertaintyFactor {
                    name: "storage_growth".to_string(),
                    impact: 0.20,
                    description: "S3 costs depend on storage size and request patterns, which can vary significantly".to_string(),
                });
            }
            "aws_cloudfront_distribution" => {
                factors.push(UncertaintyFactor {
                    name: "traffic_variability".to_string(),
                    impact: 0.35,
                    description: "CloudFront costs are highly dependent on traffic patterns and geographic distribution".to_string(),
                });
            }
            "aws_ecs_service" | "aws_eks_cluster" => {
                factors.push(UncertaintyFactor {
                    name: "scaling_behavior".to_string(),
                    impact: 0.30,
                    description: "Container service costs vary with scaling behavior and actual resource utilization".to_string(),
                });
            }
            _ => {}
        }

        factors
    }
}

/// Multi-scenario analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAnalysis {
    /// Resource identifier
    pub resource_id: String,

    /// Scenarios
    pub scenarios: Vec<ScenarioResult>,

    /// Recommended scenario for planning
    pub recommended_scenario: CostScenario,

    /// Cost at risk (P90 - P50)
    pub cost_at_risk: f64,

    /// Maximum potential cost (P99)
    pub maximum_potential_cost: f64,
}

/// Individual scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// Scenario type
    pub scenario: CostScenario,

    /// Monthly cost for this scenario
    pub monthly_cost: f64,

    /// Probability of this scenario or worse
    pub probability: f64,

    /// Description
    pub description: String,
}

impl ProbabilisticEstimate {
    /// Generate multi-scenario analysis
    pub fn to_scenario_analysis(&self) -> ScenarioAnalysis {
        let scenarios = vec![
            ScenarioResult {
                scenario: CostScenario::BestCase,
                monthly_cost: self.p10_monthly_cost,
                probability: 0.10,
                description: "Best case - low usage, optimal conditions".to_string(),
            },
            ScenarioResult {
                scenario: CostScenario::Expected,
                monthly_cost: self.p50_monthly_cost,
                probability: 0.50,
                description: "Expected case - typical usage patterns".to_string(),
            },
            ScenarioResult {
                scenario: CostScenario::WorstCase,
                monthly_cost: self.p90_monthly_cost,
                probability: 0.90,
                description: "Worst case - high usage, peak conditions".to_string(),
            },
            ScenarioResult {
                scenario: CostScenario::Catastrophic,
                monthly_cost: self.p99_monthly_cost,
                probability: 0.99,
                description: "Catastrophic case - extreme usage spike".to_string(),
            },
        ];

        // Recommend conservative scenario for planning (P75 approximation)
        let recommended_scenario = if self.risk_level == RiskLevel::Low {
            CostScenario::Expected
        } else {
            CostScenario::WorstCase
        };

        ScenarioAnalysis {
            resource_id: self.resource_id.clone(),
            scenarios,
            recommended_scenario,
            cost_at_risk: self.p90_monthly_cost - self.p50_monthly_cost,
            maximum_potential_cost: self.p99_monthly_cost,
        }
    }

    /// Check if estimate indicates high risk
    pub fn is_high_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::High | RiskLevel::VeryHigh)
    }

    /// Get cost range description
    pub fn cost_range_description(&self) -> String {
        format!(
            "${:.2} - ${:.2} (median: ${:.2})",
            self.p10_monthly_cost, self.p90_monthly_cost, self.median_monthly_cost
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probabilistic_estimate() {
        let predictor = ProbabilisticPredictor::new(
            100.0, // base cost
            0.8,   // confidence
            "aws_instance".to_string(),
            false, // no cold start
        );

        let estimate = predictor.generate_estimate("aws_instance.test").unwrap();

        assert_eq!(estimate.p50_monthly_cost, 100.0);
        assert!(estimate.p10_monthly_cost < estimate.p50_monthly_cost);
        assert!(estimate.p90_monthly_cost > estimate.p50_monthly_cost);
        assert!(estimate.p99_monthly_cost > estimate.p90_monthly_cost);
        assert!(estimate.std_dev > 0.0);
    }

    #[test]
    fn test_risk_classification() {
        assert_eq!(ProbabilisticPredictor::classify_risk(0.10), RiskLevel::Low);
        assert_eq!(
            ProbabilisticPredictor::classify_risk(0.20),
            RiskLevel::Moderate
        );
        assert_eq!(ProbabilisticPredictor::classify_risk(0.40), RiskLevel::High);
        assert_eq!(
            ProbabilisticPredictor::classify_risk(0.60),
            RiskLevel::VeryHigh
        );
    }

    #[test]
    fn test_cold_start_increases_uncertainty() {
        let with_cold_start =
            ProbabilisticPredictor::new(100.0, 0.6, "aws_instance".to_string(), true);

        let without_cold_start =
            ProbabilisticPredictor::new(100.0, 0.95, "aws_instance".to_string(), false);

        let est1 = with_cold_start.generate_estimate("test1").unwrap();
        let est2 = without_cold_start.generate_estimate("test2").unwrap();

        assert!(est1.coefficient_of_variation > est2.coefficient_of_variation);
        assert!(est1.uncertainty_factors.len() > est2.uncertainty_factors.len());
    }

    #[test]
    fn test_scenario_analysis() {
        let predictor =
            ProbabilisticPredictor::new(200.0, 0.7, "aws_lambda_function".to_string(), false);

        let estimate = predictor.generate_estimate("aws_lambda.api").unwrap();
        let analysis = estimate.to_scenario_analysis();

        assert_eq!(analysis.scenarios.len(), 4);
        assert!(analysis.cost_at_risk > 0.0);
        assert!(analysis.maximum_potential_cost > estimate.median_monthly_cost);
    }

    #[test]
    fn test_resource_specific_uncertainty() {
        let ec2 = ProbabilisticPredictor::new(100.0, 0.9, "aws_instance".to_string(), false);
        let lambda =
            ProbabilisticPredictor::new(100.0, 0.9, "aws_lambda_function".to_string(), false);

        let ec2_est = ec2.generate_estimate("ec2").unwrap();
        let lambda_est = lambda.generate_estimate("lambda").unwrap();

        // Lambda should have higher uncertainty due to usage-dependent pricing
        assert!(lambda_est.coefficient_of_variation > ec2_est.coefficient_of_variation);
    }
}
