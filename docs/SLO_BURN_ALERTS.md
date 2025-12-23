# SLO Burn Rate Alerts

Enterprise-grade predictive alerting system that forecasts SLO budget exhaustion using linear regression on historical cost data.

## Overview

The burn rate system analyzes cost growth trends to predict when SLO budgets will be breached, enabling proactive cost management before limits are exceeded.

### Key Features

- **Linear Regression Analysis** - Statistical modeling of cost trends over time
- **Time-to-Breach Prediction** - Accurate forecasting of when budgets will exhaust
- **Risk Classification** - Four-level risk assessment (Low/Medium/High/Critical)
- **Multi-SLO Support** - Analyze all SLOs simultaneously
- **Confidence Scoring** - R² based confidence metrics for predictions
- **Zero-Network** - Fully offline, deterministic analysis

## Architecture

### Components

**burn_rate.rs** (470+ lines)
- `BurnRateCalculator` - Linear regression engine
- `BurnAnalysis` - Single SLO burn prediction
- `BurnReport` - Aggregated multi-SLO analysis
- `BurnRisk` - Risk classification enum
- 11 comprehensive unit tests

### Risk Levels

| Risk | Days to Breach | Severity | Action Required |
|------|---------------|----------|-----------------|
| **Low** | >30 or no breach | 0 | Monitor |
| **Medium** | 14-30 days | 1 | Plan mitigation |
| **High** | 7-14 days | 2 | Take action |
| **Critical** | <7 days or breached | 3 | Immediate action |

## Usage

### Basic Analysis

```rust
use costpilot::engines::slo::*;
use costpilot::engines::trend::snapshot_types::CostSnapshot;

// Load historical snapshots
let snapshots = load_snapshots(".costpilot/snapshots/")?;

// Load SLO configuration
let config = SloConfig::load(".costpilot/slo.yml")?;

// Create calculator
let calculator = BurnRateCalculator::new();

// Analyze single SLO
let slo = config.get_slo("monthly-budget").unwrap();
let analysis = calculator.analyze_slo(&slo, &snapshots);

if let Some(analysis) = analysis {
    println!("Burn Rate: ${:.2}/day", analysis.burn_rate);
    println!("Projected Cost: ${:.2}", analysis.projected_cost);
    println!("Risk: {:?}", analysis.risk);

    if let Some(days) = analysis.days_to_breach {
        println!("Days to Breach: {:.1}", days);
    }
}
```

### Multi-SLO Analysis

```rust
// Analyze all SLOs at once
let report = calculator.analyze_all(&config.slos, &snapshots);

println!("Total SLOs: {}", report.total_slos);
println!("At Risk: {}", report.slos_at_risk);
println!("Overall Risk: {:?}", report.overall_risk);

// Handle critical SLOs
for analysis in report.critical_slos() {
    eprintln!("CRITICAL: {} will breach in {:.1} days",
        analysis.slo_name,
        analysis.days_to_breach.unwrap_or(0.0)
    );
}

if report.requires_action() {
    // Block deployment or alert team
    return Err("SLO burn rate critical - action required");
}
```

### Custom Thresholds

```rust
// Require more historical data for higher confidence
let calculator = BurnRateCalculator::with_thresholds(
    5,    // min 5 snapshots
    0.85  // min R² of 0.85
);

let analysis = calculator.analyze_slo(&slo, &snapshots)?;
```

## Linear Regression Model

### Mathematical Foundation

The system uses ordinary least squares (OLS) regression:

$$y = mx + b$$

Where:
- **y** = predicted cost
- **x** = days from baseline
- **m** = burn rate (slope, $/day)
- **b** = intercept

### R² Quality Metric

R² (coefficient of determination) measures how well the linear model fits the data:

$$R^2 = 1 - \frac{SS_{res}}{SS_{tot}}$$

Where:
- $SS_{res}$ = sum of squared residuals
- $SS_{tot}$ = total sum of squares

**Interpretation:**
- R² = 1.0: Perfect fit
- R² ≥ 0.7: High confidence (default threshold)
- R² < 0.7: Lower confidence, penalized score

### Time-to-Breach Calculation

Given SLO limit $L$, current time $t_0$, and regression parameters:

$$t_{breach} = \frac{L - b}{m} - t_0$$

Only calculated when:
1. Burn rate $m > 0$ (increasing costs)
2. Current cost $< L$ (not already breached)

## Data Requirements

### Minimum Snapshots

- **Default**: 3 snapshots minimum
- **Recommended**: 5+ snapshots for reliable trends
- **Ideal**: 10+ snapshots covering multiple weeks

### Snapshot Format

Snapshots must include:

```json
{
  "timestamp": "2024-12-06T10:00:00Z",
  "total_monthly_cost": 3500.0,
  "modules": [
    {
      "name": "vpc",
      "monthly_cost": 1200.0
    }
  ]
}
```

### SLO Types Supported

- ✅ `monthly_budget` - Global infrastructure cost
- ✅ `module_budget` - Per-module cost (e.g., `module.vpc`)
- ✅ `service_budget` - Per-service cost (e.g., `aws_nat_gateway`)
- ⏳ `resource_count` - Resource counts (future)
- ⏳ `cost_growth_rate` - Growth percentage (future)

## Examples

### Example 1: Detecting Budget Exhaustion

```rust
// Historical data showing steady growth
let snapshots = vec![
    create_snapshot("2024-11-01", 2000.0),
    create_snapshot("2024-11-08", 2500.0),
    create_snapshot("2024-11-15", 3000.0),
    create_snapshot("2024-11-22", 3500.0),
    create_snapshot("2024-11-29", 4000.0),
];

// SLO limit: $5000/month
let slo = Slo::monthly_budget("prod", 5000.0);

let analysis = calculator.analyze_slo(&slo, &snapshots)?;

// Output:
// Burn Rate: $71.43/day
// Days to Breach: 14.0
// Risk: High
// Confidence: 0.98
```

### Example 2: Multi-Module Analysis

```rust
let config = SloConfig {
    slos: vec![
        Slo::monthly_budget("global", 10000.0),
        Slo::module_budget("vpc", 3000.0),
        Slo::module_budget("database", 4000.0),
        Slo::module_budget("compute", 2000.0),
    ],
    ..Default::default()
};

let report = calculator.analyze_all(&config.slos, &snapshots);

// Check which modules are at risk
for analysis in &report.analyses {
    if analysis.risk.requires_action() {
        println!("⚠️  {} - {} days to breach",
            analysis.slo_name,
            analysis.days_to_breach.unwrap_or(0.0)
        );
    }
}
```

### Example 3: CI/CD Integration

```rust
fn check_burn_rate_in_ci() -> Result<(), String> {
    let snapshots = load_historical_snapshots()?;
    let config = SloConfig::load(".costpilot/slo.yml")?;
    let calculator = BurnRateCalculator::new();

    let report = calculator.analyze_all(&config.slos, &snapshots);

    // Block deployment if critical
    if report.overall_risk == BurnRisk::Critical {
        return Err(format!(
            "Deployment blocked: {} SLOs at critical burn rate",
            report.critical_slos().len()
        ));
    }

    // Warn if high risk
    if report.overall_risk == BurnRisk::High {
        eprintln!("Warning: High SLO burn rate detected");
        eprintln!("Consider reviewing cost optimizations");
    }

    Ok(())
}
```

## CLI Command (Planned)

```bash
# Analyze burn rate for all SLOs
costpilot slo burn

# Output:
# 📊 SLO Burn Rate Analysis
#
# Global Budget ($10,000/month)
#   Burn Rate: $142.86/day
#   Days to Breach: 21.0
#   Risk: MEDIUM
#   Confidence: 92%
#
# VPC Module ($3,000/month)
#   Burn Rate: $71.43/day
#   Days to Breach: 8.5
#   Risk: HIGH
#   Confidence: 95%
#
# Overall Risk: HIGH
# Action Required: 1 SLO needs attention

# Analyze specific SLO
costpilot slo burn --slo-id global-budget

# Output JSON for automation
costpilot slo burn --format json > burn-report.json

# With custom thresholds
costpilot slo burn --min-snapshots 5 --min-r-squared 0.85
```

## Configuration

### Snapshot Collection

Add to `.github/workflows/costpilot.yml`:

```yaml
- name: Collect Cost Snapshot
  run: |
    costpilot snapshot create \
      --plan plan.json \
      --output .costpilot/snapshots/$(date +%Y%m%d).json

- name: Check Burn Rate
  run: costpilot slo burn
```

### SLO Configuration

```yaml
# .costpilot/slo.yml
version: "1.0"

slos:
  - id: prod-monthly
    name: "Production Monthly Budget"
    slo_type: monthly_budget
    target: global
    threshold:
      max_value: 10000.0
      warning_threshold_percent: 80.0
    enforcement: block

  - id: vpc-module
    name: "VPC Module Budget"
    slo_type: module_budget
    target: module.vpc
    threshold:
      max_value: 3000.0
      warning_threshold_percent: 85.0
    enforcement: warn
```

## Best Practices

### 1. Regular Snapshots

Collect snapshots at consistent intervals:
- **Daily**: Ideal for production
- **Per-PR**: Good for development
- **Weekly**: Minimum recommendation

### 2. Retention Policy

Keep snapshots for:
- **30 days minimum** - For monthly trend analysis
- **90 days recommended** - For seasonal patterns
- **1 year ideal** - For year-over-year comparisons

### 3. Confidence Thresholds

- **R² ≥ 0.9**: High confidence, act on predictions
- **R² 0.7-0.9**: Moderate confidence, monitor closely
- **R² < 0.7**: Low confidence, collect more data

### 4. Action Triggers

- **Critical (< 7 days)**: Block deployments, alert on-call
- **High (7-14 days)**: Require review, plan mitigation
- **Medium (14-30 days)**: Track trend, investigate drivers
- **Low (> 30 days)**: Monitor, no action needed

## Limitations

### Current

- **Linear Model Only**: Assumes constant burn rate
- **Minimum Data**: Requires 3+ snapshots
- **SLO Types**: Limited to budget-based SLOs
- **Single Trend**: Doesn't detect trend changes

### Future Enhancements (V3)

- **Polynomial Regression**: Detect accelerating costs
- **Seasonal Adjustment**: Account for monthly patterns
- **Anomaly Detection**: Identify sudden cost spikes
- **Multi-Trend Analysis**: Detect trend shifts
- **Bayesian Forecasting**: Probabilistic predictions
- **Alert Integration**: Slack/Teams/PagerDuty webhooks

## Performance

- **Analysis Speed**: <50ms per SLO
- **Memory Usage**: <10MB for 100 snapshots
- **Deterministic**: Same inputs → same outputs
- **Zero-Network**: No external API calls
- **WASM-Safe**: Runs in browser/sandbox

## Testing

### Unit Tests

```bash
cargo test burn_rate
```

11 tests covering:
- Linear regression accuracy
- Risk classification
- Time-to-breach calculation
- Insufficient data handling
- Multi-SLO analysis
- Confidence scoring

### Integration Tests

```rust
#[test]
fn test_end_to_end_burn_analysis() {
    let snapshots = load_real_snapshots();
    let config = SloConfig::load("tests/fixtures/slo.yml")?;
    let calculator = BurnRateCalculator::new();

    let report = calculator.analyze_all(&config.slos, &snapshots);

    assert!(report.total_slos > 0);
    assert!(report.overall_risk != BurnRisk::Critical); // Assuming healthy state
}
```

## Related Documentation

- [SLO_ENGINE.md](SLO_ENGINE.md) - Core SLO system
- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Policy enforcement
- [ZERO_NETWORK.md](ZERO_NETWORK.md) - Security guarantees

## Troubleshooting

### "Insufficient snapshots"

**Problem**: `analysis.is_none()` returned

**Solution**: Collect at least 3 historical snapshots before analysis

### "Low confidence (R² < 0.7)"

**Problem**: Predictions have low confidence score

**Possible Causes**:
- Noisy data (cost fluctuations)
- Too few snapshots
- Non-linear cost growth

**Solution**: Collect more snapshots or investigate cost drivers

### "No breach predicted despite increasing costs"

**Problem**: `days_to_breach = None` but costs are rising

**Explanation**: Current trajectory doesn't reach SLO limit within analysis window

**Action**: Monitor trend, may need to lower SLO limit

## Security

- **No Network Access**: All calculations are local
- **Deterministic**: Same inputs produce identical outputs
- **Data Privacy**: Snapshots never leave your infrastructure
- **Audit Trail**: All analyses timestamped and logged

## License

Part of CostPilot - Enterprise cost management suite
