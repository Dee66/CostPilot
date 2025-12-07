# Baselines System V1

## Overview

The Baselines System enables cost governance by defining expected cost targets for infrastructure modules, services, and total spend. It integrates with the Trend Engine to detect deviations from baselines and trigger alerts when costs exceed acceptable variance thresholds.

## Architecture

### Components

1. **baseline_types.rs** (402 lines)
   - Core data structures for baselines configuration
   - `Baseline`: Individual baseline with cost expectations
   - `BaselinesConfig`: Container for all baselines (global, modules, services)
   - `BaselineStatus`: Variance check results
   - `BaselineViolation`: Detected violations for reporting

2. **baselines_manager.rs** (423 lines)
   - Management layer for baseline operations
   - `BaselinesManager`: Load, validate, compare, and save baselines
   - `BaselineComparisonResult`: Aggregated comparison results
   - File I/O with JSON serialization
   - Validation logic for baseline constraints

3. **mod.rs** (6 lines)
   - Module exports and public API

4. **Trend Engine Integration** (90 lines added)
   - `detect_baseline_violations()`: Compare snapshots against baselines
   - `create_snapshot_with_baselines()`: Create snapshot with baseline checks
   - `annotate_with_baselines()`: Add baseline violations to existing snapshots

## Usage

### Loading Baselines

```rust
use costpilot::engines::baselines::BaselinesManager;

// Load from JSON file
let baselines = BaselinesManager::load_from_file("examples/baselines.json")?;

// Validate configuration
baselines.validate()?;
```

### Comparing Against Baselines

```rust
use std::collections::HashMap;

// Extract module costs from snapshot or scan results
let mut module_costs = HashMap::new();
module_costs.insert("module.vpc".to_string(), 1200.0);
module_costs.insert("module.compute".to_string(), 2800.0);

// Compare against baselines
let result = baselines.compare_module_costs(&module_costs);

println!("Violations: {}", result.total_violations);
println!("Within baseline: {}", result.within_baseline_count);
println!("No baseline: {}", result.no_baseline_count);

// Filter critical violations
for violation in result.critical_violations() {
    println!("[{}] {}: ${:.2} vs ${:.2} expected",
        violation.severity,
        violation.name,
        violation.actual_cost,
        violation.expected_cost
    );
}
```

### Creating Snapshots with Baseline Checks

```rust
use costpilot::engines::trend::TrendEngine;

let trend_engine = TrendEngine::new(".costpilot/snapshots");
let baselines = BaselinesManager::load_from_file("baselines.json")?;

// Create snapshot with automatic baseline comparison
let snapshot = trend_engine.create_snapshot_with_baselines(
    &resource_changes,
    Some("abc123".to_string()),
    Some("main".to_string()),
    &baselines
)?;

// Regressions field now contains baseline violations
for regression in &snapshot.regressions {
    println!("{}: ${:.2} vs ${:.2} baseline",
        regression.affected,
        regression.current_cost,
        regression.baseline_cost
    );
}
```

### Global Budget Checks

```rust
// Check total cost against global baseline
let total_cost = 6000.0;
if let Some(violation) = baselines.compare_total_cost(total_cost) {
    println!("Global budget exceeded: ${:.2} vs ${:.2} baseline",
        violation.actual_cost,
        violation.expected_cost
    );
}
```

## Baseline Configuration Format

```json
{
  "version": "1.0",
  "global": {
    "name": "global",
    "expected_monthly_cost": 5000.0,
    "acceptable_variance_percent": 15.0,
    "last_updated": "2024-01-15T10:00:00Z",
    "justification": "Total monthly infrastructure budget",
    "owner": "platform-team@example.com",
    "reference": "BUDGET-2024-Q1",
    "tags": {
      "environment": "production"
    }
  },
  "modules": {
    "module.vpc": {
      "name": "module.vpc",
      "expected_monthly_cost": 1000.0,
      "acceptable_variance_percent": 10.0,
      "last_updated": "2024-01-15T10:00:00Z",
      "justification": "Production VPC with 3 NAT Gateways",
      "owner": "networking-team@example.com",
      "reference": "ARCH-123",
      "tags": {
        "service": "networking",
        "criticality": "high"
      }
    }
  },
  "services": {
    "NAT Gateway": {
      "name": "NAT Gateway",
      "expected_monthly_cost": 405.0,
      "acceptable_variance_percent": 5.0,
      "last_updated": "2024-01-15T10:00:00Z",
      "justification": "3x NAT Gateways at $135/month each",
      "owner": "networking-team@example.com"
    }
  },
  "metadata": {
    "last_reviewed": "2024-01-15T10:00:00Z",
    "review_cadence_days": 90,
    "owner_team": "platform-team@example.com"
  }
}
```

## Baseline Fields

### Required Fields

- **name**: Identifier for the baseline (module name, service name, or "global")
- **expected_monthly_cost**: Target monthly cost in USD
- **last_updated**: ISO 8601 timestamp when baseline was set
- **justification**: Explanation of why this cost is expected
- **owner**: Email or team responsible for this baseline

### Optional Fields

- **acceptable_variance_percent**: Allowed deviation (default: 10%)
- **reference**: Link to ticket, doc, or decision record
- **tags**: Key-value metadata for categorization

## Variance Checking

### Status Types

1. **Within**: Actual cost within acceptable variance
2. **Exceeded**: Cost exceeds upper bound (expected + variance)
3. **Below**: Cost below lower bound (expected - variance)
4. **NoBaseline**: No baseline defined for this entity

### Severity Calculation

- **Critical**: >50% variance
- **High**: 25-50% variance
- **Medium**: 10-25% variance
- **Low**: <10% variance (but still outside acceptable range)

### Bounds

```rust
let baseline = Baseline::new("test".to_string(), 1000.0, "...".to_string(), "...".to_string());
// With 10% variance:
baseline.upper_bound(); // 1100.0
baseline.lower_bound(); // 900.0

// Check status
baseline.check_variance(1050.0); // Within
baseline.check_variance(1200.0); // Exceeded
baseline.check_variance(850.0);  // Below
```

## Stale Baseline Detection

Baselines should be reviewed periodically. The system can detect stale baselines:

```rust
// Get baselines older than review cadence
let stale = baselines.get_stale_baselines();

for (name, baseline) in stale {
    println!("{} baseline is stale (last updated: {})",
        name,
        baseline.last_updated
    );
}
```

Default review cadence: 90 days

## Integration with Trend Engine

The Trend Engine now supports baseline-aware snapshot creation:

### Traditional Regression Detection
Compares current snapshot against previous snapshot to detect cost increases.

### Baseline-Aware Detection
Compares current snapshot against expected baselines to detect budget violations.

### Combined Approach
Use both methods:
1. Baseline violations → immediate alerts for governance
2. Regression detection → trends over time for optimization

## Validation

The BaselinesManager performs validation:

```rust
match baselines.validate() {
    Ok(_) => println!("Baselines valid"),
    Err(errors) => {
        for error in errors {
            eprintln!("Validation error: {}", error);
        }
    }
}
```

### Validation Rules

- Expected costs must be non-negative
- Variance must be 0-100%
- Justification must not be empty
- Owner must be specified
- Timestamps must be valid ISO 8601

## Test Coverage

**baseline_types.rs**: 19 tests
- Baseline creation and serialization
- Variance checking (within, exceeded, below)
- Bounds calculation
- Config management (global, modules, services)
- Stale detection
- JSON serialization/deserialization

**baselines_manager.rs**: 10 tests
- File load/save operations
- Validation (negative costs, invalid variance, missing fields)
- Module cost comparison
- Global cost comparison
- Severity calculation
- Violation filtering
- Result formatting

**Trend Engine Integration**: 3 tests
- Baseline violation detection
- Snapshot annotation with baselines
- Combined regression + baseline checks

**Total**: 32 tests

## Example Workflow

### 1. Define Baselines

Create `baselines.json` with expected costs for all critical modules.

### 2. Integrate into CI/CD

```bash
# In your CI pipeline
costpilot scan --plan terraform.tfplan --format json > scan.json
costpilot snapshot create --with-baselines baselines.json
```

### 3. Review Violations

```rust
let snapshot = trend_engine.create_snapshot_with_baselines(
    &changes, None, None, &baselines
)?;

if !snapshot.regressions.is_empty() {
    eprintln!("❌ Baseline violations detected:");
    for regression in &snapshot.regressions {
        eprintln!("  {} - ${:.2} (expected ${:.2})",
            regression.affected,
            regression.current_cost,
            regression.baseline_cost
        );
    }
    std::process::exit(1);
}
```

### 4. Update Baselines Quarterly

Review and update baselines every 90 days based on:
- Infrastructure changes
- Traffic growth
- Business requirements
- Optimization efforts

## Future Enhancements

### Planned for V2
- Baseline drift tracking over time
- Automatic baseline suggestions from historical data
- Multi-environment baselines (dev, staging, prod)
- Baseline approval workflow
- Integration with SLO engine for automated enforcement

### Planned for V3
- Machine learning-based baseline recommendations
- Seasonal/cyclical baseline patterns
- Predictive baseline violations
- Cost attribution to business metrics

## Related Documentation

- [TREND_ENGINE.md](TREND_ENGINE.md) - Snapshot and trend tracking
- [SLO_ENGINE.md](SLO_ENGINE.md) - Service-level objectives (coming soon)
- [examples/baselines.json](../examples/baselines.json) - Real-world example

## API Reference

See inline documentation in source files:
- `src/engines/baselines/baseline_types.rs`
- `src/engines/baselines/baselines_manager.rs`
- `src/engines/trend/mod.rs` (baseline integration methods)
