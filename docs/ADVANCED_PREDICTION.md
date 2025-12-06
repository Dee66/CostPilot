# Advanced Prediction Model Documentation

## Overview

The Advanced Prediction Model enhances CostPilot's cost forecasting with probabilistic analysis, seasonality detection, and Monte Carlo simulation. While the basic prediction engine provides deterministic point estimates, the advanced model quantifies uncertainty and provides risk-aware cost projections.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Advanced Prediction Model                         │
│                                                                       │
│  ┌─────────────────┐  ┌──────────────────┐  ┌────────────────────┐│
│  │  Probabilistic  │  │   Seasonality    │  │   Monte Carlo      ││
│  │   Predictor     │  │    Detector      │  │   Simulator        ││
│  │                 │  │                  │  │                    ││
│  │ • Confidence    │  │ • Weekly patterns│  │ • 10,000 runs      ││
│  │   intervals     │  │ • Monthly cycles │  │ • Percentiles      ││
│  │ • Risk levels   │  │ • Adjustment     │  │ • VaR/CVaR         ││
│  │ • Scenarios     │  │   factors        │  │ • Distribution     ││
│  └─────────────────┘  └──────────────────┘  └────────────────────┘│
│           │                     │                      │            │
│           └─────────────────────┼──────────────────────┘            │
│                                 │                                   │
│                    ┌────────────▼────────────┐                      │
│                    │   Enhanced Estimates    │                      │
│                    │  • P10/P50/P90/P99      │                      │
│                    │  • Std deviation        │                      │
│                    │  • Uncertainty factors  │                      │
│                    │  • Risk classification  │                      │
│                    └─────────────────────────┘                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Components

### 1. Probabilistic Predictor

**Purpose**: Generate confidence intervals and multi-scenario cost estimates.

**Key Features**:
- **Percentile-based estimates**: P10 (best case), P50 (median), P90 (worst case), P99 (catastrophic)
- **Uncertainty quantification**: Standard deviation, coefficient of variation
- **Risk classification**: Low/Moderate/High/VeryHigh based on CoV thresholds
- **Uncertainty factors**: Identifies sources of prediction uncertainty

**Example**:
```rust
use costpilot::engines::prediction::{ProbabilisticPredictor, CostScenario};

let predictor = ProbabilisticPredictor::new(
    100.0,  // base monthly cost
    0.8,    // confidence (80%)
    "aws_instance".to_string(),
    false,  // no cold start
);

let estimate = predictor.generate_estimate("aws_instance.web")?;

println!("Median: ${:.2}/mo", estimate.median_monthly_cost);
println!("P90: ${:.2}/mo", estimate.p90_monthly_cost);
println!("Risk: {:?}", estimate.risk_level);
```

**Output**:
```json
{
  "resource_id": "aws_instance.web",
  "median_monthly_cost": 100.0,
  "p10_monthly_cost": 87.20,
  "p50_monthly_cost": 100.0,
  "p90_monthly_cost": 112.80,
  "p99_monthly_cost": 123.30,
  "std_dev": 8.80,
  "coefficient_of_variation": 0.088,
  "confidence": 0.8,
  "risk_level": "Low",
  "uncertainty_factors": [
    {
      "name": "low_confidence",
      "impact": 0.12,
      "description": "Prediction confidence is 80.0%, indicating uncertainty in inputs"
    }
  ]
}
```

### 2. Seasonality Detector

**Purpose**: Identify periodic cost patterns (weekly, monthly, quarterly, annual).

**Key Features**:
- **Pattern detection**: Automatic identification of weekly and monthly patterns
- **Strength measurement**: Quantifies seasonal variation (0.0 - 1.0)
- **Adjustment factors**: Calculates multipliers for current time period
- **Phase calculation**: Determines position within seasonal cycle

**Weekly Pattern Example**:
```rust
use costpilot::engines::prediction::{SeasonalityDetector, CostDataPoint};

// Historical cost data (60 days)
let data: Vec<CostDataPoint> = vec![/* ... */];

let detector = SeasonalityDetector::new()
    .with_data(data);

let analysis = detector.detect_seasonality()?;

if analysis.has_seasonality {
    for pattern in &analysis.patterns {
        println!("Pattern: {:?}", pattern.pattern_type);
        println!("Strength: {:.1}%", pattern.strength * 100.0);
        println!("Peak multiplier: {:.2}x", pattern.peak_multiplier);
    }
}
```

**Output**:
```json
{
  "has_seasonality": true,
  "patterns": [
    {
      "pattern_type": "Weekly",
      "period_days": 7,
      "strength": 0.35,
      "peak_multiplier": 1.25,
      "trough_multiplier": 0.80,
      "description": "Higher costs on weekdays (business hours)"
    }
  ],
  "strength": 0.35,
  "adjustment_factor": 1.15
}
```

**Seasonally-Adjusted Prediction**:
```rust
use costpilot::engines::prediction::SeasonalAdjustedPrediction;

let base_cost = 100.0;
let seasonality = detector.detect_seasonality()?;

let prediction = SeasonalAdjustedPrediction::new(base_cost, seasonality);

println!("Base: ${:.2}", prediction.base_cost);
println!("Adjusted: ${:.2}", prediction.adjusted_cost);
println!("Factor: {:.2}x", prediction.adjustment_factor);
```

### 3. Monte Carlo Simulator

**Purpose**: Quantify cost uncertainty through stochastic simulation.

**Key Features**:
- **10,000 simulation runs** (configurable)
- **Multiple distributions**: Normal, Log-Normal, Uniform, Triangular
- **Percentile calculation**: P1, P5, P10, P25, P50, P75, P90, P95, P99
- **Risk metrics**: VaR (Value at Risk), CVaR (Conditional VaR)
- **Distribution analysis**: Histogram bins, shape classification
- **Deterministic**: Same seed produces identical results

**Example**:
```rust
use costpilot::engines::prediction::{
    MonteCarloSimulator, UncertaintyInput, UncertaintyType
};

let simulator = MonteCarloSimulator::new(10000)
    .with_seed(42);

let inputs = vec![
    UncertaintyInput {
        base_value: 100.0,
        uncertainty_type: UncertaintyType::Normal { std_dev_ratio: 0.2 },
        weight: 1.0,
    },
];

let result = simulator.simulate(&inputs)?;

println!("Mean: ${:.2}", result.mean_cost);
println!("Median: ${:.2}", result.median_cost);
println!("VaR 95%: ${:.2}", result.var_95);
println!("CVaR 95%: ${:.2}", result.cvar_95);
```

**Output**:
```json
{
  "num_simulations": 10000,
  "mean_cost": 100.25,
  "median_cost": 100.10,
  "std_dev": 20.15,
  "percentiles": {
    "1": 64.50,
    "5": 73.20,
    "10": 80.40,
    "25": 90.80,
    "50": 100.10,
    "75": 109.60,
    "90": 119.80,
    "95": 127.30,
    "99": 136.10
  },
  "var_95": 127.30,
  "cvar_95": 131.40,
  "distribution": {
    "bins": [/* 20 bins */],
    "min": 45.20,
    "max": 155.80,
    "shape": "Normal"
  }
}
```

## Workflow Integration

### 1. Enhanced CLI Command

```bash
# Basic prediction (point estimate)
costpilot scan --format json

# Advanced prediction with uncertainty
costpilot scan --advanced --format json

# With seasonality adjustment
costpilot scan --advanced --seasonality --format json

# Full Monte Carlo simulation
costpilot scan --monte-carlo --runs 50000 --format json
```

### 2. Integration with Base Prediction Engine

```rust
// In prediction_engine.rs
impl PredictionEngine {
    pub fn predict_with_uncertainty(&self, change: &ResourceChange) -> Result<ProbabilisticEstimate> {
        // 1. Get base deterministic estimate
        let base_estimate = self.predict_resource(change)?;
        
        // 2. Calculate confidence
        let confidence = calculate_confidence(change, false, &change.resource_type);
        
        // 3. Generate probabilistic estimate
        let predictor = ProbabilisticPredictor::new(
            base_estimate.monthly_cost,
            confidence,
            change.resource_type.clone(),
            false,
        );
        
        predictor.generate_estimate(&change.resource_id)
    }
}
```

### 3. Seasonality-Aware Predictions

```rust
// Historical data from trend engine snapshots
let historical_data = load_historical_costs()?;

let detector = SeasonalityDetector::new()
    .with_data(historical_data);

let seasonality = detector.detect_seasonality()?;

// Apply seasonal adjustment to current prediction
let adjusted = SeasonalAdjustedPrediction::new(base_cost, seasonality);
```

## Use Cases

### 1. Budget Planning

**Scenario**: Finance team needs to budget for Q4 cloud costs.

**Solution**:
```rust
// Generate probabilistic estimates for all resources
let estimates: Vec<ProbabilisticEstimate> = resources
    .iter()
    .map(|r| predictor.predict_with_uncertainty(r))
    .collect::<Result<_>>()?;

// Use P75 for conservative budget
let budget = estimates.iter()
    .map(|e| e.p90_monthly_cost * 0.875) // P75 approximation
    .sum::<f64>();

println!("Recommended budget: ${:.2}/mo", budget);
```

**Output**:
```
Recommended budget: $12,450.00/mo
├─ Expected (P50): $10,800.00/mo
├─ Conservative (P75): $12,450.00/mo
└─ Worst case (P90): $14,200.00/mo
```

### 2. Risk Assessment

**Scenario**: Evaluate cost risk before deploying new infrastructure.

**Solution**:
```rust
let estimate = predictor.generate_estimate("aws_eks_cluster.prod")?;

if estimate.is_high_risk() {
    println!("⚠️  HIGH RISK DEPLOYMENT");
    println!("Cost at Risk: ${:.2}", estimate.p90_monthly_cost - estimate.median_monthly_cost);
    
    for factor in &estimate.uncertainty_factors {
        println!("  - {}: {:.0}% impact", factor.name, factor.impact * 100.0);
    }
}
```

**Output**:
```
⚠️  HIGH RISK DEPLOYMENT
Cost at Risk: $8,450.00
  - scaling_behavior: 30% impact
  - usage_dependent: 25% impact
```

### 3. Seasonal Cost Forecasting

**Scenario**: Predict Black Friday traffic spike costs.

**Solution**:
```rust
// Load historical data including previous Black Friday
let historical = load_cost_data_for_last_year()?;

let detector = SeasonalityDetector::new()
    .with_data(historical);

let seasonality = detector.detect_seasonality()?;

// Current estimates
let base_cost = 5000.0;
let adjusted = SeasonalAdjustedPrediction::new(base_cost, seasonality);

println!("{}", adjusted.explanation());
```

**Output**:
```
Seasonal adjustment: 45.0% above from baseline
  - Weekly pattern: 35% variation
  - Monthly pattern: 25% variation

Adjusted cost: $7,250.00/mo (was $5,000.00)
```

### 4. Multi-Scenario Analysis

**Scenario**: Present cost scenarios to stakeholders.

**Solution**:
```rust
let estimate = predictor.generate_estimate("aws_instance.app")?;
let scenarios = estimate.to_scenario_analysis();

for scenario_result in &scenarios.scenarios {
    println!("{:?}: ${:.2}/mo ({})", 
        scenario_result.scenario,
        scenario_result.monthly_cost,
        scenario_result.description
    );
}

println!("\nRecommended: {:?}", scenarios.recommended_scenario);
```

**Output**:
```
BestCase: $85.00/mo (Best case - low usage, optimal conditions)
Expected: $100.00/mo (Expected case - typical usage patterns)
WorstCase: $125.00/mo (Worst case - high usage, peak conditions)
Catastrophic: $145.00/mo (Catastrophic case - extreme usage spike)

Recommended: Expected
```

## Configuration

### Simulation Parameters

```yaml
# costpilot.yaml
advanced_prediction:
  enabled: true
  
  probabilistic:
    default_confidence: 0.8
    simulation_runs: 10000
  
  seasonality:
    enabled: true
    min_data_points: 30
    significance_threshold: 0.15
  
  monte_carlo:
    enabled: false  # Expensive, on-demand only
    default_runs: 10000
    histogram_bins: 20
    
  risk_thresholds:
    low: 0.15      # CoV < 0.15
    moderate: 0.30  # 0.15 <= CoV < 0.30
    high: 0.50     # 0.30 <= CoV < 0.50
```

## Performance Considerations

### Determinism

All advanced prediction components are **deterministic**:
- Same inputs → same outputs
- Seeded random number generation for Monte Carlo
- No network calls
- WASM-safe

### Latency Budgets

| Component | Target Latency | Notes |
|-----------|---------------|-------|
| Probabilistic | < 50ms | Per resource |
| Seasonality | < 200ms | Per analysis (cached) |
| Monte Carlo | < 2s | 10K runs, on-demand only |

### Memory Usage

| Component | Memory | Notes |
|-----------|--------|-------|
| Probabilistic | ~1 KB | Per estimate |
| Seasonality | ~10 KB | Includes historical data |
| Monte Carlo | ~500 KB | 10K simulations |

## Testing

### Unit Tests

All modules include comprehensive unit tests:

```bash
# Test probabilistic predictions
cargo test --test probabilistic

# Test seasonality detection
cargo test --test seasonality

# Test Monte Carlo simulation
cargo test --test monte_carlo
```

### Integration Tests

```rust
#[test]
fn test_full_advanced_prediction_workflow() {
    // 1. Base prediction
    let engine = PredictionEngine::new();
    let base = engine.predict(&changes)?;
    
    // 2. Probabilistic enhancement
    let prob = ProbabilisticPredictor::new(base.monthly_cost, 0.85, "aws_instance".to_string(), false);
    let estimate = prob.generate_estimate("test")?;
    
    // 3. Seasonality adjustment
    let detector = SeasonalityDetector::new().with_data(historical);
    let seasonality = detector.detect_seasonality()?;
    let adjusted = SeasonalAdjustedPrediction::new(estimate.median_monthly_cost, seasonality);
    
    // 4. Monte Carlo validation
    let simulator = MonteCarloSimulator::new(1000);
    let mc_result = simulator.simulate(&uncertainty_inputs)?;
    
    // Verify consistency
    assert!((mc_result.median_cost - adjusted.adjusted_cost).abs() < 10.0);
}
```

## Future Enhancements

### 1. Machine Learning Integration
- Train models on historical prediction accuracy
- Adaptive confidence scoring based on past performance
- Resource-specific prediction models

### 2. Advanced Seasonality
- Quarterly patterns detection
- Annual cycles (holiday seasons)
- Multi-dimensional seasonality (time + region)

### 3. Correlation Analysis
- Cross-resource cost correlation
- Service dependency cost propagation
- Cascading cost impact modeling

### 4. What-If Scenarios
- Interactive scenario builder
- Compare multiple infrastructure configurations
- Sensitivity analysis (which parameters matter most)

### 5. Bayesian Updating
- Continuous improvement from actual costs
- Prior distribution refinement
- Posterior probability updates

## API Reference

### ProbabilisticPredictor

```rust
pub struct ProbabilisticPredictor {
    base_cost: f64,
    confidence: f64,
    resource_type: String,
    cold_start_used: bool,
    simulation_runs: u32,
}

impl ProbabilisticPredictor {
    pub fn new(base_cost: f64, confidence: f64, resource_type: String, cold_start_used: bool) -> Self;
    pub fn with_simulation_runs(mut self, runs: u32) -> Self;
    pub fn generate_estimate(&self, resource_id: &str) -> Result<ProbabilisticEstimate>;
}
```

### SeasonalityDetector

```rust
pub struct SeasonalityDetector {
    data_points: Vec<CostDataPoint>,
    min_data_points: usize,
    significance_threshold: f64,
}

impl SeasonalityDetector {
    pub fn new() -> Self;
    pub fn with_data(mut self, data: Vec<CostDataPoint>) -> Self;
    pub fn with_min_data_points(mut self, min: usize) -> Self;
    pub fn detect_seasonality(&self) -> Result<SeasonalityAnalysis>;
}
```

### MonteCarloSimulator

```rust
pub struct MonteCarloSimulator {
    num_simulations: u32,
    seed: u64,
    num_bins: usize,
}

impl MonteCarloSimulator {
    pub fn new(num_simulations: u32) -> Self;
    pub fn with_seed(mut self, seed: u64) -> Self;
    pub fn with_bins(mut self, num_bins: usize) -> Self;
    pub fn simulate(&self, inputs: &[UncertaintyInput]) -> Result<MonteCarloResult>;
}
```

## Conclusion

The Advanced Prediction Model transforms CostPilot from a deterministic cost calculator into a sophisticated risk analysis tool. By quantifying uncertainty, detecting patterns, and providing probabilistic forecasts, it enables:

- **Better budgeting**: Use P75/P90 for conservative planning
- **Risk management**: Identify high-uncertainty deployments
- **Informed decisions**: Understand cost ranges, not just points
- **Seasonal awareness**: Account for usage pattern variations
- **Stakeholder communication**: Present multiple scenarios

All while maintaining CostPilot's core principles: **deterministic**, **zero-IAM**, **offline-capable**, and **WASM-safe**.
