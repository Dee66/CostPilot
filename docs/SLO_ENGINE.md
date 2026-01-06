# SLO Engine V1

## Overview

The SLO (Service Level Objective) Engine provides automated cost governance through configurable budget limits, warning thresholds, and enforcement policies. It evaluates infrastructure costs against defined SLOs and can block deployments that violate critical budget constraints.

## Architecture

### Components

1. **slo_types.rs** (764 lines)
   - Core SLO data structures and evaluation logic
   - `Slo`: Individual SLO definition with threshold and enforcement
   - `SloType`: Budget types (Monthly/Module/Service/Resource/Growth)
   - `SloThreshold`: Threshold configuration with baseline integration
   - `EnforcementLevel`: Observe/Warn/Block/StrictBlock policies
   - `SloEvaluation`: Individual evaluation result
   - `SloReport`: Aggregated evaluation report with summary
   - 15 comprehensive unit tests

2. **slo_manager.rs** (487 lines)
   - SLO management and evaluation engine
   - `SloManager`: Load, validate, evaluate SLOs
   - Snapshot evaluation with module/service breakdown
   - Baseline-aware threshold calculation
   - Deployment blocking logic
   - 13 unit tests for validation and evaluation

3. **mod.rs** (7 lines)
   - Module exports and public API

## SLO Types

### 1. Monthly Budget
Total infrastructure spend limit for a time window.

```json
{
  "slo_type": "monthly_budget",
  "target": "global",
  "threshold": {
    "max_value": 10000.0,
    "warning_threshold_percent": 80.0
  }
}
```

### 2. Module Budget
Cost limit for a specific Terraform/IaC module.

```json
{
  "slo_type": "module_budget",
  "target": "module.vpc",
  "threshold": {
    "max_value": 2000.0,
    "warning_threshold_percent": 85.0
  }
}
```

### 3. Service Budget
Cost limit for a specific cloud service type.

```json
{
  "slo_type": "service_budget",
  "target": "NAT Gateway",
  "threshold": {
    "max_value": 450.0,
    "warning_threshold_percent": 90.0
  }
}
```

### 4. Resource Count
Limit total number of infrastructure resources.

```json
{
  "slo_type": "resource_count",
  "target": "global",
  "threshold": {
    "max_value": 500.0
  }
}
```

### 5. Cost Growth Rate
Limit month-over-month cost increase percentage.

```json
{
  "slo_type": "cost_growth_rate",
  "target": "global",
  "threshold": {
    "max_value": 15.0
  }
}
```

## Enforcement Levels

### Observe
Log SLO evaluations but take no action. Useful for establishing baselines.

```json
{
  "enforcement": "observe"
}
```

### Warn
Alert on violations but allow deployment to proceed.

```json
{
  "enforcement": "warn"
}
```

### Block
Prevent deployment on SLO violation.

```json
{
  "enforcement": "block"
}
```

### StrictBlock
Block deployment and require explicit approval override.

```json
{
  "enforcement": "strict_block"
}
```

## Baseline Integration

SLOs can use baselines as dynamic thresholds instead of static values.

### Example: Baseline-Aware SLO

```json
{
  "id": "slo-vpc-baseline",
  "name": "VPC Baseline Budget",
  "slo_type": "module_budget",
  "target": "module.vpc",
  "threshold": {
    "max_value": 1200.0,
    "use_baseline": true,
    "baseline_multiplier": 1.2
  },
  "enforcement": "warn"
}
```

This SLO:
1. Looks up `module.vpc` baseline (e.g., $1,000/month)
2. Multiplies by 1.2 → $1,200 threshold
3. Warns if actual cost exceeds $1,200
4. Updates automatically when baseline changes

### Benefits
- **Dynamic thresholds**: Adjust with infrastructure changes
- **Consistent governance**: Baselines + SLOs work together
- **Flexible enforcement**: Allow variance beyond baseline
- **Audit trail**: Baseline justifications carry through to SLO violations

## Usage

### Loading SLOs

```rust
use costpilot::engines::slo::SloManager;

// Load SLO configuration
let slo_manager = SloManager::load_from_file("examples/slo.json")?;

// Validate configuration
slo_manager.validate()?;
```

### Loading with Baselines

```rust
// Load SLOs with baseline integration
let slo_manager = SloManager::with_baselines(
    "examples/slo.json",
    "examples/baselines.json"
)?;
```

### Evaluating Snapshots

```rust
use costpilot::engines::trend::TrendEngine;

// Create snapshot
let trend_engine = TrendEngine::new(".costpilot/snapshots");
let snapshot = trend_engine.create_snapshot(&changes, None, None)?;

// Evaluate against SLOs
let report = slo_manager.evaluate_snapshot(&snapshot);

// Check results
println!("{}", report.format());

if report.has_violations() {
    println!("⚠️  {} SLO violations detected", report.summary.violation_count);
}

// Block deployment if needed
if slo_manager.should_block_deployment(&report) {
    eprintln!("❌ Deployment blocked due to SLO violations");
    std::process::exit(1);
}
```

### Evaluating Module Costs

```rust
use std::collections::HashMap;

let mut module_costs = HashMap::new();
module_costs.insert("module.vpc".to_string(), 1500.0);
module_costs.insert("module.compute".to_string(), 3200.0);

let report = slo_manager.evaluate_module_costs(&module_costs);

for eval in &report.evaluations {
    match eval.status {
        SloStatus::Pass => println!("✅ {}: {}", eval.slo_name, eval.message),
        SloStatus::Warning => println!("⚠️  {}: {}", eval.slo_name, eval.message),
        SloStatus::Violation => println!("❌ {}: {}", eval.slo_name, eval.message),
        SloStatus::NoData => println!("❓ {}: {}", eval.slo_name, eval.message),
    }
}
```

### Getting Blocking Violations

```rust
// Get violations that block deployment
let blocking = slo_manager.get_blocking_violations(&report);

if !blocking.is_empty() {
    eprintln!("🚫 The following SLOs block deployment:");
    for violation in blocking {
        eprintln!("  - {}: {}", violation.slo_name, violation.message);
    }
}
```

## SLO Configuration Format

### Complete Example

```json
{
  "version": "1.0",
  "slos": [
    {
      "id": "slo-global-budget",
      "name": "Global Monthly Budget",
      "description": "Total infrastructure spend limit",
      "slo_type": "monthly_budget",
      "target": "global",
      "threshold": {
        "max_value": 10000.0,
        "min_value": null,
        "warning_threshold_percent": 80.0,
        "time_window": "30d",
        "use_baseline": false,
        "baseline_multiplier": null
      },
      "enforcement": "block",
      "owner": "finance@example.com",
      "created_at": "2024-01-15T10:00:00Z",
      "updated_at": null,
      "tags": {
        "priority": "critical"
      }
    }
  ],
  "config": {
    "default_enforcement": "warn",
    "enable_inheritance": false,
    "baseline_file": "examples/baselines.json"
  }
}
```

### Required Fields

- **id**: Unique identifier
- **name**: Human-readable name
- **description**: Purpose of this SLO
- **slo_type**: Budget type (see SLO Types)
- **target**: Entity to evaluate (module name, service, or "global")
- **threshold**: Threshold configuration (see below)
- **enforcement**: Enforcement level (observe/warn/block/strict_block)
- **owner**: Email or team responsible

### Threshold Configuration

- **max_value**: Maximum allowed value
- **warning_threshold_percent**: Percentage of max for warnings (default: 80%)
- **time_window**: Evaluation window (e.g., "30d", default: "30d")
- **use_baseline**: Use baseline as threshold source (default: false)
- **baseline_multiplier**: Multiplier for baseline value (default: 1.0)

## Evaluation Status

### Pass (✅)
Cost is within acceptable limits (below warning threshold).

### Warning (⚠️)
Cost is between warning threshold and max threshold.
- Warning threshold = max_value × (warning_threshold_percent / 100)
- Example: 80% of $10,000 = $8,000 warning, $10,000 max

### Violation (❌)
Cost exceeds max threshold. May block deployment depending on enforcement level.

### NoData (❓)
Insufficient data to evaluate (module/service not found in snapshot).

## SLO Report

### Structure

```rust
pub struct SloReport {
    pub generated_at: String,
    pub evaluations: Vec<SloEvaluation>,
    pub summary: SloSummary,
    pub metadata: Option<HashMap<String, String>>,
}

pub struct SloSummary {
    pub total_slos: usize,
    pub pass_count: usize,
    pub warning_count: usize,
    pub violation_count: usize,
    pub no_data_count: usize,
    pub overall_status: SloStatus,
}
```

### Example Output

```
📊 SLO Evaluation Report
Generated: 2024-01-15T14:30:00Z

Summary:
  Total SLOs: 5
  ✅ Pass: 3
  ⚠️  Warning: 1
  ❌ Violation: 1

Overall Status: Violation

Evaluations:
✅ Global Monthly Budget: Within SLO: $4500.00 of $10000.00 (45.0%)
⚠️  VPC Module Budget: Approaching limit: $950.00 of $1000.00 (95.0%)
❌ Database Module Budget: SLO violated: $1200.00 exceeds $1100.00 (109.1%)
✅ NAT Gateway Service Budget: Within SLO: $405.00 of $450.00 (90.0%)
✅ Resource Count: Within SLO: 87.00 of 500.00 (17.4%)
```

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Check Cost SLOs
  run: |
    costpilot scan --plan terraform.tfplan --format json > scan.json
    costpilot slo check --config slo.json --baselines baselines.json
  continue-on-error: false
```

### Exit Codes

- **0**: All SLOs pass
- **1**: SLO violations detected (with blocking enforcement)
- **2**: Configuration or validation errors

## Validation

The SloManager validates configurations:

```rust
match slo_manager.validate() {
    Ok(_) => println!("✅ SLO configuration valid"),
    Err(errors) => {
        eprintln!("❌ SLO configuration errors:");
        for error in errors {
            eprintln!("  - {}", error);
        }
    }
}
```

### Validation Rules

- Non-empty SLO list
- Unique SLO IDs
- Non-negative thresholds
- Valid warning_threshold_percent (0-100%)
- Baseline configuration consistency
- Non-empty names and owners
- Valid min/max value relationships

## Test Coverage

**slo_types.rs**: 15 tests
- SLO creation and configuration
- Warning threshold calculation
- Violation detection
- Evaluation logic (pass/warn/violation)
- Report generation and formatting
- Enforcement level checks
- SLO config management

**slo_manager.rs**: 13 tests
- Manager creation and configuration
- Validation (negative thresholds, empty config, duplicates)
- Snapshot evaluation (pass/warning/violation)
- Module budget evaluation
- Blocking deployment logic
- Enforcement level handling

**Total**: 28 tests

## Integration with Other Systems

### Baselines System
SLOs can use baselines as dynamic thresholds with multipliers.

### Trend Engine
Evaluate snapshots against SLOs for automated governance.

### Policy Engine
SLOs complement static policy checks with dynamic budget enforcement.

### Exemption Workflow
Future: SLO violations can be exempted with justification and expiration.

## Future Enhancements

### Planned for V2
- **SLO Inheritance**: Child modules inherit parent SLO limits
- **Multi-SLO Composition**: Combine multiple SLOs (e.g., global + module)
- **Historical Trend Analysis**: Evaluate growth rate SLOs with snapshot history
- **Burn Rate Alerts**: Linear regression for time-to-breach prediction
- **Per-Resource Budgets**: Fine-grained cost control
- **Conditional SLOs**: Environment-specific limits (dev/staging/prod)

### Planned for Enterprise
- **Approval Workflow**: Override strict-block SLOs with multi-party approval
- **Teams Integration**: Real-time SLO violation notifications
- **Custom Webhooks**: Trigger external systems on violations
- **Cost Attribution**: Link SLOs to business units or projects
- **Forecasting**: Predict future SLO violations based on trends

## Examples

See `examples/slo.json` for a complete real-world configuration with:
- Global budget SLO ($6,000/month with baseline multiplier)
- Module-level SLOs (VPC, Compute, Database)
- Service-level SLO (NAT Gateway)
- Resource count limit
- Cost growth rate limit

All SLOs include baseline integration, appropriate enforcement levels, and comprehensive metadata.

## Related Documentation

- [BASELINES_SYSTEM.md](BASELINES_SYSTEM.md) - Baseline cost expectations
- [TREND_ENGINE.md](TREND_ENGINE.md) - Snapshot and trend tracking
- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Static policy enforcement
- [examples/slo.json](../examples/slo.json) - Real-world SLO configuration
- [examples/baselines.json](../examples/baselines.json) - Baseline configuration

## API Reference

See inline documentation in source files:
- `src/engines/slo/slo_types.rs`
- `src/engines/slo/slo_manager.rs`
