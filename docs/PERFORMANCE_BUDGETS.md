# Performance Budgets & Hard Limits

## Overview

CostPilot enforces strict performance budgets across all engines to ensure:
- **Predictable latency** for CI/CD pipelines
- **Resource efficiency** in containerized environments
- **SLA compliance** for enterprise deployments
- **Fail-fast behavior** to prevent resource exhaustion

## Performance Budgets

### Engine-Specific Budgets

| Engine | Max Latency | Max Memory | Max File Size | Timeout Action |
|--------|-------------|------------|---------------|----------------|
| **Prediction** | 300ms | 128MB | 10MB | Partial Results |
| **Mapping** | 500ms | 256MB | 20MB | Partial Results |
| **Autofix** | 400ms | 128MB | 10MB | Error |
| **SLO** | 150ms | 64MB | 5MB | Error |
| **Policy** | 200ms | 128MB | 10MB | Error |
| **Total Scan** | 2000ms | 512MB | 20MB | Error |

### WASM Sandbox Limits

| Resource | Limit |
|----------|-------|
| **Max Memory** | 256MB |
| **Max Execution Time** | 2000ms |
| **Max Stack Depth** | 1024 |
| **Max Bytecode Size** | 10MB |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│               Performance Enforcement System                  │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ Performance  │  │   Circuit    │  │   Memory     │       │
│  │   Tracker    │  │   Breaker    │  │   Tracker    │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                  │                  │               │
│         └──────────────────┼──────────────────┘               │
│                            │                                  │
│                  ┌─────────▼─────────┐                        │
│                  │  Budget Enforcer  │                        │
│                  └─────────┬─────────┘                        │
│                            │                                  │
│         ┌──────────────────┼──────────────────┐              │
│         │                  │                  │              │
│    ┌────▼────┐      ┌─────▼──────┐    ┌─────▼──────┐       │
│    │ Timeout │      │  Memory    │    │  Circuit   │       │
│    │Violation│      │ Violation  │    │   Open     │       │
│    └─────────┘      └────────────┘    └────────────┘       │
│                                                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           Performance Monitoring                      │   │
│  │  • Baseline tracking                                  │   │
│  │  • Regression detection                               │   │
│  │  • Historical analysis                                │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

## Components

### 1. Performance Tracker

Monitors execution time and enforces latency budgets.

**Features:**
- Real-time budget checking
- Warning threshold alerts (80% of budget)
- Automatic timeout enforcement
- Metrics collection

**Usage:**
```rust
use costpilot::engines::performance::{PerformanceTracker, EngineBudget};

// Create budget
let budget = EngineBudget {
    name: "Prediction".to_string(),
    max_latency_ms: 300,
    max_memory_mb: 128,
    max_file_size_mb: 10,
    timeout_action: TimeoutAction::PartialResults,
    warning_threshold: 0.8,
};

// Start tracking
let tracker = PerformanceTracker::new(budget);

// ... perform work ...

// Check budget periodically
tracker.check_budget()?;

// Complete and get metrics
let metrics = tracker.complete();
println!("Duration: {}ms ({}% of budget)",
    metrics.duration_ms, metrics.utilization as u64);
```

**Output:**
```
⚠️  Performance warning: Prediction at 250ms (83% of budget)
Duration: 280ms (93% of budget)
```

### 2. Circuit Breaker

Prevents cascade failures by opening circuit after repeated failures.

**States:**
- **Closed**: Normal operation, all requests allowed
- **Open**: Rejecting requests after failure threshold
- **Half-Open**: Testing recovery with limited requests

**Configuration:**
```rust
use costpilot::engines::performance::CircuitBreakerConfig;

let config = CircuitBreakerConfig {
    failure_threshold: 5,         // Open after 5 failures
    success_threshold: 3,         // Close after 3 successes
    timeout_seconds: 60,          // Retry after 60 seconds
    max_consecutive_failures: 3,  // Open immediately after 3 consecutive
};
```

**Behavior:**
```
Closed → [3 failures] → Open → [60s wait] → Half-Open → [3 successes] → Closed
```

**Usage:**
```rust
use costpilot::engines::performance::CircuitBreaker;

let mut breaker = CircuitBreaker::new(config);

// Check if request allowed
if breaker.allow_request().is_ok() {
    match perform_operation() {
        Ok(_) => breaker.record_success(),
        Err(_) => breaker.record_failure(),
    }
}
```

### 3. Memory Tracker

Monitors memory usage and enforces limits.

**Features:**
- Peak memory tracking
- Real-time limit checking
- Memory utilization calculation
- Leak detection (sustained high usage)

**Usage:**
```rust
use costpilot::engines::performance::MemoryTracker;

let mut tracker = MemoryTracker::new(256); // 256MB limit

// Periodically check limit
tracker.check_limit()?;

// Get statistics
let stats = tracker.get_stats();
println!("Memory: peak={}MB, current={}MB ({}% utilized)",
    stats.peak_mb, stats.current_mb, stats.utilization as u64);
```

**Output:**
```
Memory: peak=187MB, current=145MB (73% utilized)
```

### 4. Performance Monitor

Tracks performance over time and detects regressions.

**Features:**
- Performance baseline tracking
- Historical snapshots (last 100 executions)
- Regression detection (>20% degradation)
- Statistical analysis (min/max/avg)

**Setting Baseline:**
```rust
use costpilot::engines::performance::PerformanceMonitor;

let mut monitor = PerformanceMonitor::new();

// Set baseline from current execution
monitor.set_baseline(&performance_report);
monitor.save(&history_path)?;
```

**Detecting Regressions:**
```rust
let monitor = PerformanceMonitor::load(&history_path)?;

// Check new execution against baseline
let regressions = monitor.detect_regressions(&new_report);

for regression in regressions {
    println!("{}", regression.format_text());
}
```

**Output:**
```
⚠️  Prediction regression: 200ms → 450ms (2.3x slower) - Severe
⚠️  Total Scan regression: 1200ms → 2100ms (1.8x slower) - Moderate
```

**Regression Severity:**
- **Minor**: 1.2x - 1.5x slower
- **Moderate**: 1.5x - 2.0x slower
- **Severe**: 2.0x - 3.0x slower
- **Critical**: > 3.0x slower

## Timeout Actions

When engine exceeds budget, one of three actions is taken:

### 1. Partial Results
Return best-effort results with warning.

**Use Case:** Non-critical engines (Prediction, Mapping)

**Behavior:**
```rust
match tracker.check_budget() {
    Ok(_) => continue_processing(),
    Err(violation) if violation.action == TimeoutAction::PartialResults => {
        eprintln!("⚠️  {}", violation.format_error());
        return Ok(partial_results);
    }
    Err(violation) => return Err(violation.format_error()),
}
```

### 2. Error
Fail execution immediately.

**Use Case:** Critical engines (SLO, Policy, Autofix)

**Behavior:**
```rust
if tracker.check_budget().is_err() {
    return Err("Budget exceeded");
}
```

### 3. Circuit Break
Open circuit breaker and reject subsequent requests.

**Use Case:** Systemic issues requiring intervention

**Behavior:**
```rust
if let Err(violation) = tracker.check_budget() {
    circuit_breaker.record_failure();
    if circuit_breaker.state() == CircuitState::Open {
        // Reject all requests until timeout
        return Err("Circuit breaker open");
    }
}
```

## CLI Commands

### Show Budgets

```bash
costpilot performance budgets
```

**Output:**
```
⚡ Performance Budgets
=====================

Prediction Engine:
  Max Latency: 300ms
  Max Memory: 128MB
  Max File Size: 10MB

Mapping Engine:
  Max Latency: 500ms
  Max Memory: 256MB
  Max File Size: 20MB

Autofix Engine:
  Max Latency: 400ms
  Max Memory: 128MB
  Max File Size: 10MB

Total Scan:
  Max Latency: 2000ms
  Max Memory: 512MB
  Max File Size: 20MB

SLO Engine:
  Max Latency: 150ms
  Max Memory: 64MB

Policy Engine:
  Max Latency: 200ms
  Max Memory: 128MB

WASM Sandbox:
  Max Memory: 256MB
  Max Execution: 2000ms
  Max Stack Depth: 1024
  Max Bytecode: 10MB

Circuit Breaker:
  Failure Threshold: 5
  Success Threshold: 3
  Timeout: 60s
  Max Consecutive Failures: 3
```

### Set Baseline

```bash
# Set baseline from current execution
costpilot performance set-baseline --from-file performance-report.json
```

**Output:**
```
✅ Performance baseline set from: performance-report.json
```

### Show Statistics

```bash
costpilot performance stats
```

**Output:**
```
📊 Performance Statistics
========================

Total Samples: 47

Engine Statistics:
  Prediction: avg=245ms, min=198ms, max=289ms (47 samples)
  Mapping: avg=387ms, min=312ms, max=456ms (47 samples)
  Autofix: avg=278ms, min=201ms, max=356ms (47 samples)
  SLO: avg=98ms, min=67ms, max=134ms (47 samples)
  Policy: avg=156ms, min=123ms, max=189ms (47 samples)
```

### Check Regressions

```bash
costpilot performance check-regressions --report performance-report.json
```

**Output (No Regressions):**
```
✅ No performance regressions detected
```

**Output (Regressions Found):**
```
⚠️  Performance Regressions Detected
==================================

⚠️  Prediction regression: 245ms → 567ms (2.3x slower) - Severe
⚠️  Mapping regression: 387ms → 623ms (1.6x slower) - Moderate
```

### Show History

```bash
# All engines
costpilot performance history

# Specific engine
costpilot performance history --engine Prediction

# Limited samples
costpilot performance history --limit 10
```

## Integration

### In Engine Code

```rust
use costpilot::engines::performance::{PerformanceTracker, EngineBudget, PerformanceBudgets};

pub fn run_prediction_engine(input: &Input) -> Result<Output, String> {
    // Load budgets
    let budgets = PerformanceBudgets::default();

    // Start tracking
    let tracker = PerformanceTracker::new(budgets.prediction);

    // Perform work
    let mut result = initialize_result();

    for item in &input.items {
        // Check budget periodically
        if let Err(violation) = tracker.check_budget() {
            eprintln!("{}", violation.format_error());

            match violation.action {
                TimeoutAction::PartialResults => break,
                TimeoutAction::Error => return Err(violation.format_error()),
                TimeoutAction::CircuitBreak => {
                    // Would record in circuit breaker
                    return Err(violation.format_error());
                }
            }
        }

        result.add(process_item(item)?);
    }

    // Complete tracking
    let metrics = tracker.complete();

    // Log metrics
    println!("{}", metrics.format_text());

    Ok(result)
}
```

### In CI/CD

```yaml
name: Performance Regression Check

on:
  pull_request:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run CostPilot with Performance Monitoring
        run: |
          costpilot scan --performance-report performance.json

      - name: Check for Regressions
        run: |
          costpilot performance check-regressions \
            --report performance.json \
            --fail-on-severity moderate

      - name: Upload Performance Report
        uses: actions/upload-artifact@v3
        with:
          name: performance-report
          path: performance.json
```

## Best Practices

### 1. Set Realistic Budgets

Base budgets on actual usage patterns:

```bash
# Run baseline measurement
for i in {1..10}; do
  costpilot scan --performance-report perf-$i.json
done

# Calculate p95 and set as budget
costpilot performance stats --calculate-percentiles
```

### 2. Monitor Continuously

Track performance over time:

```bash
# In CI pipeline
costpilot scan --performance-report performance.json
costpilot performance check-regressions --report performance.json
```

### 3. Handle Partial Results

Gracefully handle timeout with partial results:

```rust
match engine.run(&input) {
    Ok(result) => process_result(result),
    Err(e) if e.contains("exceeded time budget") => {
        eprintln!("⚠️  Using partial results due to timeout");
        process_partial_result(engine.get_partial_results())
    }
    Err(e) => return Err(e),
}
```

### 4. Configure Circuit Breakers

Tune for your failure tolerance:

```rust
// Aggressive (fail fast)
CircuitBreakerConfig {
    failure_threshold: 3,
    max_consecutive_failures: 2,
    timeout_seconds: 30,
    success_threshold: 5,
}

// Conservative (more resilient)
CircuitBreakerConfig {
    failure_threshold: 10,
    max_consecutive_failures: 5,
    timeout_seconds: 120,
    success_threshold: 2,
}
```

### 5. Memory-Bounded Processing

Process large inputs in chunks:

```rust
let mut tracker = MemoryTracker::new(256);

for chunk in input.chunks(1000) {
    // Check memory before processing chunk
    tracker.check_limit()?;

    process_chunk(chunk)?;
}
```

## Failure Scenarios

### Scenario 1: Single Engine Timeout

**Problem:** Prediction engine exceeds 300ms budget

**Detection:**
```
⏱️  Prediction exceeded time budget: 345ms (limit: 300ms)
```

**Action:** Return partial results, log warning

**Prevention:**
- Optimize prediction algorithm
- Reduce input complexity
- Increase budget if acceptable

### Scenario 2: Total Scan Timeout

**Problem:** Total scan exceeds 2000ms budget

**Detection:**
```
⏱️  Total Scan exceeded time budget: 2340ms (limit: 2000ms)
```

**Action:** Fail execution, block deployment

**Prevention:**
- Identify slow engine via per-engine metrics
- Optimize bottleneck
- Process in parallel where possible

### Scenario 3: Memory Exhaustion

**Problem:** Engine exceeds memory limit

**Detection:**
```
💾 Mapping exceeded memory budget: 312MB (limit: 256MB)
```

**Action:** Terminate execution, prevent OOM

**Prevention:**
- Process in chunks
- Implement streaming where possible
- Reduce memory footprint

### Scenario 4: Circuit Breaker Opens

**Problem:** Multiple consecutive failures

**Detection:**
```
🔌 Prediction circuit breaker is open (too many failures)
```

**Action:** Reject all requests for 60 seconds

**Prevention:**
- Fix underlying issue (bug, infrastructure)
- Review recent changes
- Check external dependencies

## Performance Report Format

```json
{
  "metrics": [
    {
      "engine": "Prediction",
      "duration_ms": 245,
      "budget_ms": 300,
      "within_budget": true,
      "utilization": 81.7,
      "circuit_breaker_stats": {
        "state": "Closed",
        "failure_count": 0,
        "success_count": 23,
        "consecutive_failures": 0
      }
    },
    {
      "engine": "Mapping",
      "duration_ms": 387,
      "budget_ms": 500,
      "within_budget": true,
      "utilization": 77.4,
      "circuit_breaker_stats": null
    }
  ],
  "total_duration_ms": 1456,
  "all_within_budget": true
}
```

## Troubleshooting

### High Latency

**Symptoms:**
- Engines approaching or exceeding budgets
- Warning messages in logs
- Partial results returned

**Diagnosis:**
```bash
# Check statistics
costpilot performance stats

# Identify slow engines
costpilot performance history --engine Prediction
```

**Solutions:**
1. Optimize engine algorithm
2. Reduce input complexity
3. Increase budget (if acceptable for SLA)
4. Enable partial results for non-critical engines

### Memory Issues

**Symptoms:**
- Memory limit exceeded errors
- OOM kills in containers

**Diagnosis:**
```bash
# Review memory usage patterns
grep "Memory:" costpilot.log
```

**Solutions:**
1. Process in smaller chunks
2. Implement streaming processing
3. Increase memory limit
4. Fix memory leaks

### Circuit Breaker Trips

**Symptoms:**
- "Circuit breaker is open" errors
- All requests rejected

**Diagnosis:**
```bash
# Check circuit breaker stats
costpilot performance stats | grep -A5 "circuit_breaker"
```

**Solutions:**
1. Wait for timeout (60 seconds default)
2. Fix underlying issue causing failures
3. Restart service to reset circuit breaker
4. Tune circuit breaker thresholds

## Conclusion

Performance budgets transform CostPilot from a best-effort tool into a reliable, predictable enterprise platform with:

- ✅ **Guaranteed latency** for CI/CD pipelines
- ✅ **Resource limits** preventing exhaustion
- ✅ **Fail-fast behavior** with circuit breakers
- ✅ **Regression detection** maintaining quality
- ✅ **Observability** via metrics and monitoring

All while maintaining CostPilot's core principles: **deterministic**, **zero-IAM**, and **offline-capable**.
