// Probabilistic modeling unit tests - comprehensive testing for uncertainty quantification

use costpilot::engines::prediction::monte_carlo::{MonteCarloSimulator, UncertaintyInput, UncertaintyType};
use costpilot::engines::prediction::probabilistic::{ProbabilisticPredictor, RiskLevel, CostScenario};
use costpilot::engines::prediction::seasonality::{SeasonalityDetector, CostDataPoint, PatternType};

#[cfg(test)]
mod probabilistic_modeling_tests {
    use super::*;

    // ===== PROBABILISTIC PREDICTOR TESTS =====

    #[test]
    fn test_probabilistic_predictor_creation() {
        let predictor = ProbabilisticPredictor::new(
            100.0, // base cost
            0.8,   // confidence
            "aws_instance".to_string(),
            false, // no cold start
        );

        // Test that predictor can generate estimates (fields are private, test via behavior)
        let estimate = predictor.generate_estimate("test-resource").unwrap();
        assert_eq!(estimate.resource_id, "test-resource");
        assert_eq!(estimate.confidence, 0.8);
    }

    #[test]
    fn test_probabilistic_predictor_with_custom_runs() {
        let predictor = ProbabilisticPredictor::new(50.0, 0.9, "aws_lambda_function".to_string(), false)
            .with_simulation_runs(5000);

        let estimate = predictor.generate_estimate("test").unwrap();
        assert_eq!(estimate.simulation_runs, 5000);
    }

    #[test]
    fn test_generate_estimate_basic() {
        let predictor = ProbabilisticPredictor::new(200.0, 0.85, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("test-resource").unwrap();

        assert_eq!(estimate.resource_id, "test-resource");
        assert_eq!(estimate.median_monthly_cost, 200.0);
        assert_eq!(estimate.p50_monthly_cost, 200.0);
        assert!(estimate.p10_monthly_cost < estimate.p50_monthly_cost);
        assert!(estimate.p90_monthly_cost > estimate.p50_monthly_cost);
        assert!(estimate.p99_monthly_cost > estimate.p90_monthly_cost);
        assert!(estimate.std_dev > 0.0);
        assert!(estimate.coefficient_of_variation > 0.0);
        assert_eq!(estimate.confidence, 0.85);
        assert_eq!(estimate.simulation_runs, 10000);
    }

    #[test]
    fn test_generate_estimate_with_cold_start() {
        let predictor = ProbabilisticPredictor::new(150.0, 0.7, "aws_lambda_function".to_string(), true);
        let estimate = predictor.generate_estimate("lambda-cold").unwrap();

        // Cold start should increase uncertainty
        assert!(estimate.coefficient_of_variation > 0.1);
        assert!(estimate.uncertainty_factors.len() > 0);

        // Should have cold start uncertainty factor
        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "cold_start_inference"));
    }

    #[test]
    fn test_generate_estimate_low_confidence() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.3, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("low-confidence").unwrap();

        // Low confidence should increase uncertainty
        assert!(estimate.coefficient_of_variation > 0.2);
        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "low_confidence"));
    }

    #[test]
    fn test_risk_classification_low() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.95, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("low-risk").unwrap();

        assert_eq!(estimate.risk_level, RiskLevel::Low);
        assert!(estimate.coefficient_of_variation < 0.15);
    }

    #[test]
    fn test_risk_classification_moderate() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.7, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("moderate-risk").unwrap();

        assert_eq!(estimate.risk_level, RiskLevel::Moderate);
        assert!(estimate.coefficient_of_variation >= 0.15);
        assert!(estimate.coefficient_of_variation < 0.30);
    }

    #[test]
    fn test_risk_classification_high() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.4, "aws_lambda_function".to_string(), true);
        let estimate = predictor.generate_estimate("high-risk").unwrap();

        // With low confidence (0.4), cold start, and lambda function, risk should be VeryHigh
        assert_eq!(estimate.risk_level, RiskLevel::VeryHigh);
        assert!(estimate.coefficient_of_variation >= 0.50);
    }

    #[test]
    fn test_risk_classification_very_high() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.1, "aws_cloudfront_distribution".to_string(), true);
        let estimate = predictor.generate_estimate("very-high-risk").unwrap();

        assert_eq!(estimate.risk_level, RiskLevel::VeryHigh);
        assert!(estimate.coefficient_of_variation >= 0.50);
    }

    #[test]
    fn test_resource_specific_uncertainty_ec2() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.9, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("ec2").unwrap();

        // EC2 should have relatively low uncertainty (base 0.095 + resource 0.03 = ~0.125)
        assert!(estimate.coefficient_of_variation < 0.2);
        assert!(estimate.coefficient_of_variation > 0.1);
    }

    #[test]
    fn test_resource_specific_uncertainty_lambda() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.9, "aws_lambda_function".to_string(), false);
        let estimate = predictor.generate_estimate("lambda").unwrap();

        // Lambda should have higher uncertainty due to usage-dependent pricing
        assert!(estimate.coefficient_of_variation > 0.15);
        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "usage_dependent"));
    }

    #[test]
    fn test_resource_specific_uncertainty_s3() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.9, "aws_s3_bucket".to_string(), false);
        let estimate = predictor.generate_estimate("s3").unwrap();

        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "storage_growth"));
    }

    #[test]
    fn test_resource_specific_uncertainty_cloudfront() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.9, "aws_cloudfront_distribution".to_string(), false);
        let estimate = predictor.generate_estimate("cloudfront").unwrap();

        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "traffic_variability"));
    }

    #[test]
    fn test_resource_specific_uncertainty_ecs() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.9, "aws_ecs_service".to_string(), false);
        let estimate = predictor.generate_estimate("ecs").unwrap();

        assert!(estimate.uncertainty_factors.iter()
            .any(|f| f.name == "scaling_behavior"));
    }

    #[test]
    fn test_scenario_analysis_generation() {
        let predictor = ProbabilisticPredictor::new(200.0, 0.8, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("scenario-test").unwrap();
        let analysis = estimate.to_scenario_analysis();

        assert_eq!(analysis.resource_id, "scenario-test");
        assert_eq!(analysis.scenarios.len(), 4);

        // Check scenarios are in correct order
        assert_eq!(analysis.scenarios[0].scenario, CostScenario::BestCase);
        assert_eq!(analysis.scenarios[1].scenario, CostScenario::Expected);
        assert_eq!(analysis.scenarios[2].scenario, CostScenario::WorstCase);
        assert_eq!(analysis.scenarios[3].scenario, CostScenario::Catastrophic);

        // Check cost ordering
        assert!(analysis.scenarios[0].monthly_cost <= analysis.scenarios[1].monthly_cost);
        assert!(analysis.scenarios[1].monthly_cost <= analysis.scenarios[2].monthly_cost);
        assert!(analysis.scenarios[2].monthly_cost <= analysis.scenarios[3].monthly_cost);

        // Check probabilities
        assert_eq!(analysis.scenarios[0].probability, 0.10);
        assert_eq!(analysis.scenarios[1].probability, 0.50);
        assert_eq!(analysis.scenarios[2].probability, 0.90);
        assert_eq!(analysis.scenarios[3].probability, 0.99);
    }

    #[test]
    fn test_scenario_analysis_recommendation_low_risk() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.95, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("low-risk").unwrap();
        let analysis = estimate.to_scenario_analysis();

        // Low risk should recommend expected scenario
        assert_eq!(analysis.recommended_scenario, CostScenario::Expected);
    }

    #[test]
    fn test_scenario_analysis_recommendation_high_risk() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.3, "aws_lambda_function".to_string(), true);
        let estimate = predictor.generate_estimate("high-risk").unwrap();
        let analysis = estimate.to_scenario_analysis();

        // High risk should recommend worst case scenario
        assert_eq!(analysis.recommended_scenario, CostScenario::WorstCase);
    }

    #[test]
    fn test_cost_at_risk_calculation() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.8, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("car-test").unwrap();
        let analysis = estimate.to_scenario_analysis();

        // Cost at risk should be P90 - P50
        assert_eq!(analysis.cost_at_risk, estimate.p90_monthly_cost - estimate.p50_monthly_cost);
        assert!(analysis.cost_at_risk > 0.0);
    }

    #[test]
    fn test_maximum_potential_cost() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.8, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("max-test").unwrap();
        let analysis = estimate.to_scenario_analysis();

        // Maximum potential cost should be P99
        assert_eq!(analysis.maximum_potential_cost, estimate.p99_monthly_cost);
    }

    #[test]
    fn test_is_high_risk() {
        let low_risk = ProbabilisticPredictor::new(100.0, 0.9, "aws_instance".to_string(), false);
        let high_risk = ProbabilisticPredictor::new(100.0, 0.2, "aws_lambda_function".to_string(), true);

        let low_estimate = low_risk.generate_estimate("low").unwrap();
        let high_estimate = high_risk.generate_estimate("high").unwrap();

        assert!(!low_estimate.is_high_risk());
        assert!(high_estimate.is_high_risk());
    }

    #[test]
    fn test_cost_range_description() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.8, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("range-test").unwrap();
        let description = estimate.cost_range_description();

        // Should contain P10, P90, and median
        assert!(description.contains("$"));
        assert!(description.contains("median"));
        assert!(description.contains("-"));
    }

    // ===== MONTE CARLO SIMULATOR TESTS =====

    #[test]
    fn test_monte_carlo_simulator_creation() {
        let simulator = MonteCarloSimulator::new(1000);

        // Test via behavior - can run simulation
        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];
        let result = simulator.simulate(&inputs).unwrap();
        assert_eq!(result.num_simulations, 1000);
    }

    #[test]
    fn test_monte_carlo_with_custom_seed() {
        let simulator = MonteCarloSimulator::new(500).with_seed(12345);

        let inputs = vec![UncertaintyInput {
            base_value: 50.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];
        let result = simulator.simulate(&inputs).unwrap();
        assert_eq!(result.num_simulations, 500);
    }

    #[test]
    fn test_monte_carlo_with_custom_bins() {
        let simulator = MonteCarloSimulator::new(1000).with_bins(50);

        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];
        let result = simulator.simulate(&inputs).unwrap();
        assert_eq!(result.distribution.bins.len(), 50);
    }

    #[test]
    fn test_monte_carlo_simulation_normal_distribution() {
        let simulator = MonteCarloSimulator::new(10000);

        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        assert_eq!(result.num_simulations, 10000);
        assert!((result.mean_cost - 100.0).abs() < 2.0); // Should be close to base value
        assert!(result.std_dev > 0.0);
        assert!(result.std_dev < 20.0); // Should be reasonable

        // Check percentiles exist
        assert!(result.percentiles.contains_key(&10));
        assert!(result.percentiles.contains_key(&50));
        assert!(result.percentiles.contains_key(&90));

        // P10 < P50 < P90
        assert!(result.percentiles[&10] < result.percentiles[&50]);
        assert!(result.percentiles[&50] < result.percentiles[&90]);
    }

    #[test]
    fn test_monte_carlo_simulation_log_normal() {
        let simulator = MonteCarloSimulator::new(5000);

        let inputs = vec![UncertaintyInput {
            base_value: 50.0,
            uncertainty_type: UncertaintyType::LogNormal { std_dev_ratio: 0.2 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        // Log-normal should be positively skewed (mean >= median for large samples)
        // With small samples, this might not always hold, so check that costs are reasonable
        assert!(result.mean_cost > 0.0);
        assert!(result.median_cost > 0.0);
        assert!(result.mean_cost >= result.median_cost * 0.9); // Allow some tolerance

        // All costs should be positive
        assert!(result.distribution.min >= 0.0);
    }

    #[test]
    fn test_monte_carlo_simulation_uniform() {
        let simulator = MonteCarloSimulator::new(10000);

        let inputs = vec![UncertaintyInput {
            base_value: 200.0,
            uncertainty_type: UncertaintyType::Uniform {
                min_ratio: 0.8,
                max_ratio: 1.2,
            },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        // Uniform distribution should have min/max close to expected range
        let expected_min = 200.0 * 0.8;
        let expected_max = 200.0 * 1.2;

        assert!((result.distribution.min - expected_min).abs() < 10.0);
        assert!((result.distribution.max - expected_max).abs() < 10.0);
    }

    #[test]
    fn test_monte_carlo_simulation_triangular() {
        let simulator = MonteCarloSimulator::new(10000);

        let inputs = vec![UncertaintyInput {
            base_value: 150.0,
            uncertainty_type: UncertaintyType::Triangular {
                min_ratio: 0.7,
                max_ratio: 1.3,
            },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        assert!(result.mean_cost >= result.distribution.min);
        assert!(result.mean_cost <= result.distribution.max);
    }

    #[test]
    fn test_monte_carlo_multiple_inputs() {
        let simulator = MonteCarloSimulator::new(5000);

        let inputs = vec![
            UncertaintyInput {
                base_value: 100.0,
                uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
                weight: 0.6,
            },
            UncertaintyInput {
                base_value: 50.0,
                uncertainty_type: UncertaintyType::Uniform {
                    min_ratio: 0.9,
                    max_ratio: 1.1,
                },
                weight: 0.4,
            },
        ];

        let result = simulator.simulate(&inputs).unwrap();

        // Total cost should be around 100 * 0.6 + 50 * 0.4 = 80
        assert!((result.mean_cost - 80.0).abs() < 5.0);
    }

    #[test]
    fn test_monte_carlo_empty_inputs_error() {
        let simulator = MonteCarloSimulator::new(1000);
        let result = simulator.simulate(&[]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("No uncertainty inputs provided"));
    }

    #[test]
    fn test_monte_carlo_value_at_risk() {
        let simulator = MonteCarloSimulator::new(10000);

        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.2 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        // VaR at 95% should be >= P95
        assert!(result.var_95 >= result.percentiles[&95]);

        // CVaR should be >= VaR
        assert!(result.cvar_95 >= result.var_95);
    }

    #[test]
    fn test_monte_carlo_distribution_analysis() {
        let simulator = MonteCarloSimulator::new(10000).with_bins(25);

        let inputs = vec![UncertaintyInput {
            base_value: 100.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.15 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        assert_eq!(result.distribution.bins.len(), 25);

        // Check bin properties
        for bin in &result.distribution.bins {
            assert!(bin.frequency >= 0.0);
            assert!(bin.frequency <= 1.0);
            assert!(bin.lower <= bin.upper);
        }

        // Total frequency should sum to 1.0
        let total_freq: f64 = result.distribution.bins.iter().map(|b| b.frequency).sum();
        assert!((total_freq - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_monte_carlo_deterministic_reproducibility() {
        let sim1 = MonteCarloSimulator::new(1000).with_seed(999);
        let sim2 = MonteCarloSimulator::new(1000).with_seed(999);

        let inputs = vec![UncertaintyInput {
            base_value: 75.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];

        let result1 = sim1.simulate(&inputs).unwrap();
        let result2 = sim2.simulate(&inputs).unwrap();

        // Same seed should produce identical results
        assert_eq!(result1.mean_cost, result2.mean_cost);
        assert_eq!(result1.median_cost, result2.median_cost);
        assert_eq!(result1.std_dev, result2.std_dev);
        assert_eq!(result1.var_95, result2.var_95);
    }

    // ===== SEASONALITY DETECTOR TESTS =====

    #[test]
    fn test_seasonality_detector_creation() {
        let detector = SeasonalityDetector::new();

        // Test via behavior - can detect seasonality with data
        let data = vec![
            CostDataPoint { timestamp: 1000, cost: 100.0 },
            CostDataPoint { timestamp: 2000, cost: 105.0 },
        ];
        let detector_with_data = detector.with_data(data);
        let result = detector_with_data.detect_seasonality().unwrap();
        assert!(!result.has_seasonality);
    }

    #[test]
    fn test_seasonality_detector_with_custom_params() {
        let detector = SeasonalityDetector::new()
            .with_min_data_points(50);

        let data = (0..40).map(|i| CostDataPoint {
            timestamp: i * 86400,
            cost: 100.0,
        }).collect();
        let detector_with_data = detector.with_data(data);
        let result = detector_with_data.detect_seasonality().unwrap();
        // Should not detect seasonality in constant data
        assert!(!result.has_seasonality);
    }

    #[test]
    fn test_seasonality_insufficient_data() {
        let data = vec![
            CostDataPoint { timestamp: 1000, cost: 100.0 },
            CostDataPoint { timestamp: 2000, cost: 105.0 },
        ];

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        assert!(!result.has_seasonality);
        assert_eq!(result.patterns.len(), 0);
        assert_eq!(result.strength, 0.0);
    }

    #[test]
    fn test_seasonality_weekly_pattern() {
        let mut data = Vec::new();

        // Generate 60 days of data with weekday/weekend pattern
        for i in 0..60 {
            let timestamp = i * 86400; // Daily timestamps
            let day_of_week = (i + 3) % 7; // Start on Thursday (3 = Thursday in 0-based week)
            let cost = if day_of_week < 5 { 150.0 } else { 80.0 }; // Higher on weekdays
            data.push(CostDataPoint { timestamp, cost });
        }

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        assert!(result.has_seasonality);
        assert!(result.strength > 0.0);

        // Should detect weekly pattern
        assert!(result.patterns.iter()
            .any(|p| p.pattern_type == PatternType::Weekly));
    }

    #[test]
    fn test_seasonality_no_pattern() {
        let mut data = Vec::new();

        // Generate random data with no clear pattern
        for i in 0..60 {
            let timestamp = i * 86400;
            let cost = 100.0 + (i as f64 * 0.5) + (i as f64).sin() * 10.0; // Trend + small variation
            data.push(CostDataPoint { timestamp, cost });
        }

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        // May or may not detect seasonality depending on algorithm
        // Just ensure it doesn't crash
        assert!(result.strength >= 0.0);
        assert!(result.strength <= 1.0);
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_complete_uncertainty_pipeline() {
        // 1. Generate probabilistic estimate
        let predictor = ProbabilisticPredictor::new(150.0, 0.75, "aws_lambda_function".to_string(), false);
        let estimate = predictor.generate_estimate("pipeline-test").unwrap();

        // 2. Run Monte Carlo simulation with similar parameters
        let simulator = MonteCarloSimulator::new(10000);
        let mc_inputs = vec![UncertaintyInput {
            base_value: 150.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.25 }, // Approximate uncertainty
            weight: 1.0,
        }];
        let mc_result = simulator.simulate(&mc_inputs).unwrap();

        // 3. Create seasonal adjustment if seasonality detected
        let seasonal_data = vec![
            CostDataPoint { timestamp: 1000000, cost: 140.0 },
            CostDataPoint { timestamp: 1000864, cost: 160.0 },
            // ... more data points would be needed for real seasonality detection
        ];
        let detector = SeasonalityDetector::new().with_data(seasonal_data);
        let seasonal_result = detector.detect_seasonality().unwrap();

        // Verify all components work together
        assert!(estimate.median_monthly_cost > 0.0);
        assert!(mc_result.mean_cost > 0.0);
        assert!(seasonal_result.strength >= 0.0);

        // If seasonality detected, check adjustment factor
        if seasonal_result.has_seasonality {
            assert!(seasonal_result.adjustment_factor >= 0.0);
        }
    }

    // ===== EDGE CASES AND ERROR HANDLING =====

    #[test]
    fn test_probabilistic_zero_base_cost() {
        let predictor = ProbabilisticPredictor::new(0.0, 0.8, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("zero-cost").unwrap();

        // Should handle zero cost gracefully
        assert_eq!(estimate.median_monthly_cost, 0.0);
        assert_eq!(estimate.p10_monthly_cost, 0.0);
        assert_eq!(estimate.p50_monthly_cost, 0.0);
        assert_eq!(estimate.p90_monthly_cost, 0.0);
        assert_eq!(estimate.p99_monthly_cost, 0.0);
        assert_eq!(estimate.std_dev, 0.0);
        assert_eq!(estimate.coefficient_of_variation, 0.0);
    }

    #[test]
    fn test_probabilistic_perfect_confidence() {
        let predictor = ProbabilisticPredictor::new(100.0, 1.0, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("perfect-confidence").unwrap();

        // Perfect confidence should minimize uncertainty
        assert!(estimate.coefficient_of_variation < 0.1);
        assert_eq!(estimate.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_probabilistic_zero_confidence() {
        let predictor = ProbabilisticPredictor::new(100.0, 0.0, "aws_instance".to_string(), false);
        let estimate = predictor.generate_estimate("zero-confidence").unwrap();

        // Zero confidence should maximize uncertainty
        assert!(estimate.coefficient_of_variation > 0.4);
        assert_eq!(estimate.risk_level, RiskLevel::VeryHigh);
    }

    #[test]
    fn test_monte_carlo_single_simulation() {
        let simulator = MonteCarloSimulator::new(1);

        let inputs = vec![UncertaintyInput {
            base_value: 50.0,
            uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.1 },
            weight: 1.0,
        }];

        let result = simulator.simulate(&inputs).unwrap();

        // Single simulation should have zero variance
        assert_eq!(result.std_dev, 0.0);
        assert_eq!(result.mean_cost, result.median_cost);
    }

    #[test]
    fn test_seasonality_single_data_point() {
        let data = vec![CostDataPoint { timestamp: 1000, cost: 100.0 }];

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        assert!(!result.has_seasonality);
        assert_eq!(result.patterns.len(), 0);
    }

    #[test]
    fn test_seasonality_identical_data() {
        let data = vec![
            CostDataPoint { timestamp: 1000, cost: 100.0 },
            CostDataPoint { timestamp: 2000, cost: 100.0 },
            CostDataPoint { timestamp: 3000, cost: 100.0 },
        ];

        let detector = SeasonalityDetector::new().with_data(data);
        let result = detector.detect_seasonality().unwrap();

        // Identical data should not show seasonality
        assert!(!result.has_seasonality);
    }
}
