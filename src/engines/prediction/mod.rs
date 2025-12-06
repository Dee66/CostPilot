// Prediction engine module

pub mod prediction_engine;
pub mod cold_start;
pub mod confidence;
pub mod calculation_steps;
pub mod probabilistic;
pub mod seasonality;
pub mod monte_carlo;
pub mod heuristics_loader;

pub use prediction_engine::PredictionEngine;
pub use cold_start::ColdStartInference;
pub use confidence::{calculate_confidence, calculate_interval_width};
pub use crate::engines::shared::models::CostEstimate;
pub use calculation_steps::{
    CalculationStep, CalculationBreakdown, document_calculation,
    ec2_calculation_step, rds_calculation_step, storage_calculation_step,
    dynamodb_calculation_step, lambda_calculation_step, nat_gateway_calculation_step,
    load_balancer_calculation_step, s3_calculation_step, cold_start_step,
    confidence_step, interval_step,
};
pub use probabilistic::{
    ProbabilisticEstimate, ProbabilisticPredictor, CostScenario,
    ScenarioAnalysis, ScenarioResult, RiskLevel, UncertaintyFactor,
};
pub use seasonality::{
    SeasonalityAnalysis, SeasonalityDetector, SeasonalPattern, PatternType,
    CostDataPoint, SeasonalAdjustedPrediction,
};
pub use monte_carlo::{
    MonteCarloSimulator, MonteCarloResult, UncertaintyInput, UncertaintyType,
    CostDistribution, DistributionBin, DistributionShape,
};
pub use heuristics_loader::{HeuristicsLoader, HeuristicsStats};
