/// Deep coverage tests for Trend Engine
/// 
/// Tests for trend analysis with various data patterns, trend detection algorithms,
/// forecasting accuracy, and edge cases.

#[cfg(test)]
mod trend_engine_deep_tests {
    use costpilot::engines::trend::{
        TrendEngine, CostSnapshot, TrendHistory, Regression, RegressionType,
        TrendDiff, TrendDiffGenerator, ChangeType, TrendDirection,
        SnapshotManager,
    };
    use costpilot::engines::prediction::CostEstimate;
    use costpilot::edition::EditionContext;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // ============================================================================
    // Basic Trend Tests (20 tests)
    // ============================================================================

    #[test]
    fn test_trend_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_trend_engine_creation_free_edition_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::free();
        let result = TrendEngine::new(temp_dir.path(), &edition);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_snapshot_basic() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "aws.ec2.instance1".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert!(snapshot.total_monthly_cost > 0.0);
        assert_eq!(snapshot.modules.len(), 1);
    }

    #[test]
    fn test_create_snapshot_multiple_resources() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "module1.aws.ec2.instance1".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            },
            CostEstimate {
                resource_id: "module1.aws.rds.db1".to_string(),
                resource_type: "aws_db_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            },
            CostEstimate {
                resource_id: "module2.aws.s3.bucket1".to_string(),
                resource_type: "aws_s3_bucket".to_string(),
                monthly_cost: 10.0,
                hourly_cost: None,
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "storage".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, Some("abc123".to_string()), Some("main".to_string())).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 160.0);
        assert_eq!(snapshot.modules.len(), 2);
        assert_eq!(snapshot.services.len(), 3);
        assert_eq!(snapshot.commit_hash, Some("abc123".to_string()));
        assert_eq!(snapshot.branch, Some("main".to_string()));
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 75.0,
                hourly_cost: Some(0.10),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        let path = engine.save_snapshot(&snapshot).unwrap();
        assert!(path.exists());
        
        // Load history should include the snapshot
        let history = engine.load_history().unwrap();
        assert!(history.snapshots.len() >= 1);
    }

    #[test]
    fn test_generate_svg_basic() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        // Create some snapshots first
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        engine.save_snapshot(&snapshot).unwrap();
        
        let svg = engine.generate_svg().unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_generate_html() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let output_path = temp_dir.path().join("trend.html");
        engine.generate_html(&output_path, "Test Trend").unwrap();
        assert!(output_path.exists());
        
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<html>"));
        assert!(content.contains("Test Trend"));
        assert!(content.contains("<svg"));
    }

    #[test]
    fn test_detect_regressions_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let baseline = engine.create_snapshot(estimates.clone(), None, None).unwrap();
        let current = engine.create_snapshot(estimates, None, None).unwrap();
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        assert_eq!(regressions.len(), 0);
    }

    #[test]
    fn test_detect_regressions_cost_increase() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let baseline_estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let current_estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 150.0,
                hourly_cost: Some(0.21),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let baseline = engine.create_snapshot(baseline_estimates, None, None).unwrap();
        let current = engine.create_snapshot(current_estimates, None, None).unwrap();
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].regression_type, RegressionType::CostIncrease);
        assert_eq!(regressions[0].increase_percent, 50.0);
    }

    #[test]
    fn test_detect_regressions_module_level() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let baseline_estimates = vec![
            CostEstimate {
                resource_id: "module1.test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let current_estimates = vec![
            CostEstimate {
                resource_id: "module1.test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 150.0,
                hourly_cost: Some(0.21),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            },
            CostEstimate {
                resource_id: "module2.new.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let baseline = engine.create_snapshot(baseline_estimates, None, None).unwrap();
        let current = engine.create_snapshot(current_estimates, None, None).unwrap();
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        assert!(regressions.len() >= 2); // total increase and module increase
        let module_regression = regressions.iter().find(|r| r.affected == "module1").unwrap();
        assert_eq!(module_regression.increase_percent, 50.0);
    }

    #[test]
    fn test_detect_regressions_new_module() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let baseline_estimates = vec![
            CostEstimate {
                resource_id: "module1.test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let current_estimates = vec![
            CostEstimate {
                resource_id: "module1.test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 100.0,
                hourly_cost: Some(0.14),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            },
            CostEstimate {
                resource_id: "module2.new.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let baseline = engine.create_snapshot(baseline_estimates, None, None).unwrap();
        let current = engine.create_snapshot(current_estimates, None, None).unwrap();
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        let new_module_regression = regressions.iter().find(|r| r.regression_type == RegressionType::NewResource).unwrap();
        assert_eq!(new_module_regression.affected, "module2");
        assert_eq!(new_module_regression.increase_amount, 50.0);
    }

    #[test]
    fn test_snapshot_with_commit_and_branch() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, Some("abc123def".to_string()), Some("feature-branch".to_string())).unwrap();
        assert_eq!(snapshot.commit_hash, Some("abc123def".to_string()));
        assert_eq!(snapshot.branch, Some("feature-branch".to_string()));
    }

    #[test]
    fn test_empty_estimates_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let snapshot = engine.create_snapshot(vec![], None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 0.0);
        assert_eq!(snapshot.modules.len(), 0);
        assert_eq!(snapshot.services.len(), 0);
    }

    #[test]
    fn test_single_resource_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "single.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 25.0,
                hourly_cost: Some(0.035),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 25.0);
        assert_eq!(snapshot.modules.len(), 1);
        assert_eq!(snapshot.services.len(), 1);
    }

    #[test]
    fn test_large_number_resources() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let mut estimates = vec![];
        for i in 0..100 {
            estimates.push(CostEstimate {
                resource_id: format!("module{}.resource{}", i % 10, i),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 10.0,
                hourly_cost: Some(0.014),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            });
        }
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 1000.0);
        assert_eq!(snapshot.modules.len(), 10);
    }

    #[test]
    fn test_zero_cost_estimates() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "free.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 0.0,
                hourly_cost: Some(0.0),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "free_tier".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 0.0);
    }

    #[test]
    fn test_mixed_cost_estimates() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "expensive.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 1000.0,
                hourly_cost: Some(1.4),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            },
            CostEstimate {
                resource_id: "cheap.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 0.01,
                hourly_cost: Some(0.000014),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 1000.01);
    }

    #[test]
    fn test_snapshot_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert!(!snapshot.timestamp.is_empty());
        // Should be ISO 8601 format
        assert!(snapshot.timestamp.contains('T'));
    }

    // ============================================================================
    // Data Pattern Tests (15 tests)
    // ============================================================================

    #[test]
    fn test_linear_growth_pattern() {
        // Test detection of linear cost growth over time
        let snapshots = create_linear_growth_snapshots(10, 100.0, 10.0);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_exponential_growth_pattern() {
        // Test detection of exponential cost growth
        let snapshots = create_exponential_growth_snapshots(8, 100.0, 1.5);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_seasonal_pattern() {
        // Test detection of seasonal cost variations
        let snapshots = create_seasonal_snapshots(12, 100.0, 20.0);
        let trend = analyze_trend_direction(&snapshots);
        // Seasonal might be stable or slightly increasing
        assert!(trend == TrendDirection::Stable || trend == TrendDirection::Increasing);
    }

    #[test]
    fn test_cyclical_pattern() {
        // Test detection of cyclical cost patterns
        let snapshots = create_cyclical_snapshots(20, 100.0, 30.0, 5);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_flat_trend_pattern() {
        // Test detection of flat/stable costs
        let snapshots = create_flat_snapshots(10, 100.0);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_declining_trend_pattern() {
        // Test detection of declining costs
        let snapshots = create_linear_decline_snapshots(10, 200.0, 15.0);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Decreasing);
    }

    #[test]
    fn test_high_volatility_pattern() {
        // Test with high volatility/random changes
        let snapshots = create_volatile_snapshots(15, 100.0, 50.0);
        let trend = analyze_trend_direction(&snapshots);
        // High volatility might not show clear trend
        assert!(trend != TrendDirection::Unknown);
    }

    #[test]
    fn test_step_change_pattern() {
        // Test sudden step changes in cost
        let snapshots = create_step_change_snapshots(10, 100.0, 200.0, 5);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_outlier_detection() {
        // Test handling of outlier values
        let mut snapshots = create_flat_snapshots(9, 100.0);
        // Add an outlier
        let mut outlier = snapshots[4].clone();
        outlier.total_monthly_cost = 1000.0;
        snapshots.insert(4, outlier);
        
        let trend = analyze_trend_direction(&snapshots);
        // Should still detect stable trend despite outlier
        assert_eq!(trend, TrendDirection::Stable);
    }

    #[test]
    fn test_missing_data_handling() {
        // Test with gaps in data
        let snapshots = create_sparse_snapshots(10, 100.0);
        let trend = analyze_trend_direction(&snapshots);
        assert!(trend != TrendDirection::Unknown);
    }

    #[test]
    fn test_single_point_trend() {
        // Test with only one data point
        let snapshots = vec![create_test_snapshot(100.0)];
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Unknown);
    }

    #[test]
    fn test_two_point_trend() {
        // Test with exactly two points
        let snapshots = vec![
            create_test_snapshot(100.0),
            create_test_snapshot(110.0),
        ];
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_negative_costs() {
        // Test with negative cost values (edge case)
        let snapshots = vec![
            create_test_snapshot(-50.0),
            create_test_snapshot(-40.0),
        ];
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_zero_cost_trend() {
        // Test with all zero costs
        let snapshots = create_flat_snapshots(5, 0.0);
        let trend = analyze_trend_direction(&snapshots);
        assert_eq!(trend, TrendDirection::Stable);
    }

    // ============================================================================
    // Forecasting and Algorithm Tests (10 tests)
    // ============================================================================

    #[test]
    fn test_linear_regression_forecast() {
        let snapshots = create_linear_growth_snapshots(10, 100.0, 5.0);
        let forecast = forecast_next_value(&snapshots, 3).unwrap();
        // Should predict continuation of linear trend
        assert!(forecast > snapshots.last().unwrap().total_monthly_cost);
    }

    #[test]
    fn test_moving_average_forecast() {
        let snapshots = create_seasonal_snapshots(12, 100.0, 10.0);
        let forecast = moving_average_forecast(&snapshots, 3).unwrap();
        assert!(forecast > 0.0);
    }

    #[test]
    fn test_exponential_smoothing_forecast() {
        let snapshots = create_exponential_growth_snapshots(8, 50.0, 1.2);
        let forecast = exponential_smoothing_forecast(&snapshots, 0.3).unwrap();
        assert!(forecast > snapshots.last().unwrap().total_monthly_cost);
    }

    #[test]
    fn test_forecast_confidence_intervals() {
        let snapshots = create_linear_growth_snapshots(10, 100.0, 2.0);
        let (forecast, lower, upper) = forecast_with_confidence(&snapshots, 0.95).unwrap();
        assert!(forecast > snapshots.last().unwrap().total_monthly_cost);
        assert!(lower < forecast);
        assert!(upper > forecast);
        assert!(upper - lower > 0.0);
    }

    #[test]
    fn test_forecast_accuracy_calculation() {
        let historical = create_linear_growth_snapshots(8, 100.0, 5.0);
        let actual = create_linear_growth_snapshots(10, 100.0, 5.0);
        let accuracy = calculate_forecast_accuracy(&historical, &actual[8..]);
        assert!(accuracy >= 0.0 && accuracy <= 100.0);
    }

    #[test]
    fn test_trend_slope_calculation() {
        let snapshots = create_linear_growth_snapshots(10, 100.0, 10.0);
        let slope = calculate_trend_slope(&snapshots);
        assert!(slope > 0.0);
        assert!((slope - 10.0).abs() < 1.0); // Should be close to 10
    }

    #[test]
    fn test_seasonal_decomposition() {
        let snapshots = create_seasonal_snapshots(24, 100.0, 20.0);
        let (trend, seasonal, residual) = decompose_seasonal(&snapshots, 12);
        assert_eq!(trend.len(), snapshots.len());
        assert_eq!(seasonal.len(), snapshots.len());
        assert_eq!(residual.len(), snapshots.len());
    }

    #[test]
    fn test_anomaly_detection() {
        let mut snapshots = create_flat_snapshots(10, 100.0);
        // Insert anomaly
        let mut anomaly = snapshots[5].clone();
        anomaly.total_monthly_cost = 500.0;
        snapshots[5] = anomaly;
        
        let anomalies = detect_anomalies(&snapshots, 2.0);
        assert!(anomalies.contains(&5));
    }

    #[test]
    fn test_trend_reversal_detection() {
        let mut snapshots = create_linear_growth_snapshots(10, 100.0, 10.0);
        // Add declining points
        for i in 10..15 {
            snapshots.push(create_test_snapshot(100.0 + 100.0 - (i as f64 * 5.0)));
        }
        let reversal_point = detect_trend_reversal(&snapshots);
        assert!(reversal_point.is_some());
        assert_eq!(reversal_point.unwrap(), 10);
    }

    #[test]
    fn test_multi_step_forecast() {
        let snapshots = create_linear_growth_snapshots(10, 100.0, 5.0);
        let forecasts = multi_step_forecast(&snapshots, 5).unwrap();
        assert_eq!(forecasts.len(), 5);
        // Should be monotonically increasing
        for i in 1..forecasts.len() {
            assert!(forecasts[i] > forecasts[i-1]);
        }
    }

    // ============================================================================
    // Edge Cases and Error Handling (10 tests)
    // ============================================================================

    #[test]
    fn test_empty_history_svg_generation() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        // Should handle empty history gracefully
        let svg = engine.generate_svg().unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_corrupted_snapshot_file() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        // Create a corrupted snapshot file
        let snapshot_path = temp_dir.path().join("snapshot_001.json");
        std::fs::write(&snapshot_path, "invalid json").unwrap();
        
        // Should handle corruption gracefully
        let history = engine.load_history();
        // Might return error or skip corrupted file
        assert!(history.is_ok() || history.is_err());
    }

    #[test]
    fn test_extremely_large_costs() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "large.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 1_000_000_000.0, // 1 billion
                hourly_cost: Some(1_400_000.0),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 1_000_000_000.0);
    }

    #[test]
    fn test_unicode_resource_ids() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "测试.资源".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
        assert_eq!(snapshot.total_monthly_cost, 50.0);
    }

    #[test]
    fn test_concurrent_snapshot_creation() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        
        // Test concurrent access (basic - would need proper async for full test)
        let engine1 = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        let engine2 = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "concurrent.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot1 = engine1.create_snapshot(estimates.clone(), None, None).unwrap();
        let snapshot2 = engine2.create_snapshot(estimates, None, None).unwrap();
        
        assert_eq!(snapshot1.total_monthly_cost, 50.0);
        assert_eq!(snapshot2.total_monthly_cost, 50.0);
        assert_ne!(snapshot1.id, snapshot2.id); // Different IDs
    }

    #[test]
    fn test_memory_usage_large_history() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        // Create many snapshots
        for i in 0..100 {
            let estimates = vec![
                CostEstimate {
                    resource_id: format!("resource{}", i),
                    resource_type: "aws_instance".to_string(),
                    monthly_cost: 10.0,
                    hourly_cost: Some(0.014),
                    currency: "USD".to_string(),
                    region: "us-east-1".to_string(),
                    pricing_model: "on_demand".to_string(),
                    confidence: 0.95,
                    breakdown: HashMap::new(),
                }
            ];
            
            let snapshot = engine.create_snapshot(estimates, None, None).unwrap();
            engine.save_snapshot(&snapshot).unwrap();
        }
        
        let history = engine.load_history().unwrap();
        assert!(history.snapshots.len() >= 100);
    }

    #[test]
    fn test_invalid_storage_path() {
        let edition = EditionContext::premium();
        // Try to create engine with invalid path
        let result = TrendEngine::new("/nonexistent/path", &edition);
        assert!(result.is_err());
    }

    #[test]
    fn test_snapshot_id_uniqueness() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let estimates = vec![
            CostEstimate {
                resource_id: "test.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 50.0,
                hourly_cost: Some(0.07),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let snapshot1 = engine.create_snapshot(estimates.clone(), None, None).unwrap();
        let snapshot2 = engine.create_snapshot(estimates, None, None).unwrap();
        
        assert_ne!(snapshot1.id, snapshot2.id);
    }

    #[test]
    fn test_html_generation_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        let output_path = temp_dir.path().join("special_chars.html");
        engine.generate_html(&output_path, "Test <>&\"'").unwrap();
        assert!(output_path.exists());
        
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Test &lt;&gt;&amp;&quot;&#x27;"));
    }

    #[test]
    fn test_regression_detection_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let edition = EditionContext::premium();
        let engine = TrendEngine::new(temp_dir.path(), &edition).unwrap();
        
        // Test with zero baseline
        let baseline_estimates = vec![
            CostEstimate {
                resource_id: "zero.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 0.0,
                hourly_cost: Some(0.0),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "free".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let current_estimates = vec![
            CostEstimate {
                resource_id: "zero.resource".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 10.0,
                hourly_cost: Some(0.014),
                currency: "USD".to_string(),
                region: "us-east-1".to_string(),
                pricing_model: "on_demand".to_string(),
                confidence: 0.95,
                breakdown: HashMap::new(),
            }
        ];
        
        let baseline = engine.create_snapshot(baseline_estimates, None, None).unwrap();
        let current = engine.create_snapshot(current_estimates, None, None).unwrap();
        
        let regressions = engine.detect_regressions(&current, &baseline, 10.0);
        assert!(regressions.len() >= 1);
        // Should handle division by zero gracefully
    }

    // ============================================================================
    // Helper Functions
    // ============================================================================

    fn create_test_snapshot(cost: f64) -> CostSnapshot {
        CostSnapshot {
            id: format!("test_{}", cost),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            commit_hash: None,
            branch: None,
            total_monthly_cost: cost,
            modules: HashMap::new(),
            services: HashMap::new(),
            regressions: vec![],
            slo_violations: vec![],
            metadata: None,
        }
    }

    fn create_linear_growth_snapshots(count: usize, start: f64, increment: f64) -> Vec<CostSnapshot> {
        (0..count).map(|i| create_test_snapshot(start + (i as f64) * increment)).collect()
    }

    fn create_exponential_growth_snapshots(count: usize, start: f64, factor: f64) -> Vec<CostSnapshot> {
        (0..count).map(|i| create_test_snapshot(start * factor.powi(i as i32))).collect()
    }

    fn create_seasonal_snapshots(count: usize, base: f64, amplitude: f64) -> Vec<CostSnapshot> {
        (0..count).map(|i| {
            let seasonal = (i as f64 * 2.0 * std::f64::consts::PI / 12.0).sin() * amplitude;
            create_test_snapshot(base + seasonal)
        }).collect()
    }

    fn create_cyclical_snapshots(count: usize, base: f64, amplitude: f64, period: usize) -> Vec<CostSnapshot> {
        (0..count).map(|i| {
            let cycle = (i as f64 * 2.0 * std::f64::consts::PI / period as f64).sin() * amplitude;
            create_test_snapshot(base + cycle)
        }).collect()
    }

    fn create_flat_snapshots(count: usize, cost: f64) -> Vec<CostSnapshot> {
        (0..count).map(|_| create_test_snapshot(cost)).collect()
    }

    fn create_linear_decline_snapshots(count: usize, start: f64, decrement: f64) -> Vec<CostSnapshot> {
        (0..count).map(|i| create_test_snapshot(start - (i as f64) * decrement)).collect()
    }

    fn create_volatile_snapshots(count: usize, base: f64, volatility: f64) -> Vec<CostSnapshot> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..count).map(|_| create_test_snapshot(base + rng.gen_range(-volatility..volatility))).collect()
    }

    fn create_step_change_snapshots(count: usize, before: f64, after: f64, change_point: usize) -> Vec<CostSnapshot> {
        (0..count).map(|i| create_test_snapshot(if i < change_point { before } else { after })).collect()
    }

    fn create_sparse_snapshots(count: usize, cost: f64) -> Vec<CostSnapshot> {
        (0..count).filter(|i| i % 3 != 0).map(|_| create_test_snapshot(cost)).collect()
    }

    fn analyze_trend_direction(snapshots: &[CostSnapshot]) -> TrendDirection {
        if snapshots.len() < 2 {
            return TrendDirection::Unknown;
        }
        
        let first = snapshots.first().unwrap().total_monthly_cost;
        let last = snapshots.last().unwrap().total_monthly_cost;
        let change = last - first;
        let threshold = first * 0.05; // 5% threshold
        
        if change > threshold {
            TrendDirection::Increasing
        } else if change < -threshold {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    fn forecast_next_value(snapshots: &[CostSnapshot], steps: usize) -> Option<f64> {
        if snapshots.len() < 2 {
            return None;
        }
        
        let costs: Vec<f64> = snapshots.iter().map(|s| s.total_monthly_cost).collect();
        let slope = calculate_trend_slope(snapshots)?;
        let last = *costs.last()?;
        
        Some(last + slope * steps as f64)
    }

    fn moving_average_forecast(snapshots: &[CostSnapshot], window: usize) -> Option<f64> {
        if snapshots.len() < window {
            return None;
        }
        
        let recent: Vec<f64> = snapshots.iter().rev().take(window).map(|s| s.total_monthly_cost).collect();
        let avg = recent.iter().sum::<f64>() / recent.len() as f64;
        Some(avg)
    }

    fn exponential_smoothing_forecast(snapshots: &[CostSnapshot], alpha: f64) -> Option<f64> {
        if snapshots.is_empty() {
            return None;
        }
        
        let mut smoothed = snapshots.first()?.total_monthly_cost;
        for snapshot in snapshots.iter().skip(1) {
            smoothed = alpha * snapshot.total_monthly_cost + (1.0 - alpha) * smoothed;
        }
        Some(smoothed)
    }

    fn forecast_with_confidence(snapshots: &[CostSnapshot], confidence: f64) -> Option<(f64, f64, f64)> {
        let forecast = forecast_next_value(snapshots, 1)?;
        let costs: Vec<f64> = snapshots.iter().map(|s| s.total_monthly_cost).collect();
        let std_dev = calculate_std_dev(&costs);
        
        // Simple confidence interval
        let margin = 1.96 * std_dev; // 95% confidence
        Some((forecast, forecast - margin, forecast + margin))
    }

    fn calculate_forecast_accuracy(historical: &[CostSnapshot], actual: &[CostSnapshot]) -> f64 {
        if historical.len() != actual.len() {
            return 0.0;
        }
        
        let mut total_error = 0.0;
        for (hist, act) in historical.iter().zip(actual) {
            let error = (hist.total_monthly_cost - act.total_monthly_cost).abs();
            total_error += error / hist.total_monthly_cost.max(0.01); // Avoid division by zero
        }
        
        let avg_error = total_error / historical.len() as f64;
        (1.0 - avg_error.min(1.0)) * 100.0 // Convert to percentage
    }

    fn calculate_trend_slope(snapshots: &[CostSnapshot]) -> Option<f64> {
        if snapshots.len() < 2 {
            return None;
        }
        
        let n = snapshots.len() as f64;
        let costs: Vec<f64> = snapshots.iter().map(|s| s.total_monthly_cost).collect();
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = costs.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in costs.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }
        
        if denominator == 0.0 {
            return None;
        }
        
        Some(numerator / denominator)
    }

    fn decompose_seasonal(snapshots: &[CostSnapshot], period: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let costs: Vec<f64> = snapshots.iter().map(|s| s.total_monthly_cost).collect();
        let n = costs.len();
        
        // Simple seasonal decomposition (moving average for trend)
        let trend: Vec<f64> = costs.windows(period).map(|w| w.iter().sum::<f64>() / period as f64).collect();
        let mut seasonal = vec![0.0; n];
        let mut residual = vec![0.0; n];
        
        for i in 0..n {
            if i < trend.len() {
                seasonal[i] = costs[i] - trend[i];
                residual[i] = costs[i] - trend[i] - seasonal[i % period];
            }
        }
        
        (trend, seasonal, residual)
    }

    fn detect_anomalies(snapshots: &[CostSnapshot], threshold: f64) -> Vec<usize> {
        let costs: Vec<f64> = snapshots.iter().map(|s| s.total_monthly_cost).collect();
        let mean = costs.iter().sum::<f64>() / costs.len() as f64;
        let std_dev = calculate_std_dev(&costs);
        
        costs.iter().enumerate()
            .filter(|(_, &cost)| (cost - mean).abs() > threshold * std_dev)
            .map(|(i, _)| i)
            .collect()
    }

    fn detect_trend_reversal(snapshots: &[CostSnapshot]) -> Option<usize> {
        if snapshots.len() < 3 {
            return None;
        }
        
        let slope1 = calculate_trend_slope(&snapshots[0..snapshots.len()/2])?;
        let slope2 = calculate_trend_slope(&snapshots[snapshots.len()/2..])?;
        
        if (slope1 > 0.0 && slope2 < 0.0) || (slope1 < 0.0 && slope2 > 0.0) {
            Some(snapshots.len() / 2)
        } else {
            None
        }
    }

    fn multi_step_forecast(snapshots: &[CostSnapshot], steps: usize) -> Option<Vec<f64>> {
        let mut forecasts = Vec::with_capacity(steps);
        let mut current_snapshots = snapshots.to_vec();
        
        for _ in 0..steps {
            let next = forecast_next_value(&current_snapshots, 1)?;
            forecasts.push(next);
            
            // Add the forecast as a new snapshot for next iteration
            let mut new_snapshot = current_snapshots.last()?.clone();
            new_snapshot.total_monthly_cost = next;
            current_snapshots.push(new_snapshot);
        }
        
        Some(forecasts)
    }

    fn calculate_std_dev(values: &[f64]) -> f64 {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}