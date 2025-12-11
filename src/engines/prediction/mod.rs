// Prediction engine module

pub mod calculation_steps;
pub mod cold_start;
pub mod confidence;
pub mod heuristics_loader;
pub mod minimal_heuristics;
pub mod monte_carlo;
pub mod prediction_engine;
pub mod probabilistic;
pub mod seasonality;

pub use crate::engines::shared::models::{CostEstimate, TotalCost};
pub use calculation_steps::{
    cold_start_step, confidence_step, document_calculation, dynamodb_calculation_step,
    ec2_calculation_step, interval_step, lambda_calculation_step, load_balancer_calculation_step,
    nat_gateway_calculation_step, rds_calculation_step, s3_calculation_step,
    storage_calculation_step, CalculationBreakdown, CalculationStep,
};
pub use cold_start::ColdStartInference;
pub use confidence::{calculate_confidence, calculate_interval_width};
pub use heuristics_loader::{HeuristicsLoader, HeuristicsStats};
pub use minimal_heuristics::MinimalHeuristics;
pub use monte_carlo::{
    CostDistribution, DistributionBin, DistributionShape, MonteCarloResult, MonteCarloSimulator,
    UncertaintyInput, UncertaintyType,
};
pub use prediction_engine::PredictionEngine;
pub use probabilistic::{
    CostScenario, ProbabilisticEstimate, ProbabilisticPredictor, RiskLevel, ScenarioAnalysis,
    ScenarioResult, UncertaintyFactor,
};
pub use seasonality::{
    CostDataPoint, PatternType, SeasonalAdjustedPrediction, SeasonalPattern, SeasonalityAnalysis,
    SeasonalityDetector,
};
