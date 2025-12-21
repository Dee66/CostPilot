# CostPilot Testing Strategy

**Target:** 100% Code Coverage | ~2,500 Tests | Zero Flakiness | WASM-Safe

---

## 📊 Testing Pyramid

```
                    E2E (50 tests, 2%)
                  ┌─────────────────┐
                  │  CLI Workflows  │
                  │  CI Integration │
                  └─────────────────┘
                         │
              Integration (200 tests, 8%)
            ┌───────────────────────────┐
            │   Engine Combinations     │
            │   File I/O & Parsing      │
            │   Multi-Engine Pipelines  │
            └───────────────────────────┘
                         │
              Unit (2,250 tests, 90%)
    ┌─────────────────────────────────────────┐
    │  Function-level Tests                    │
    │  Property-Based Tests                    │
    │  Snapshot Tests                          │
    │  Fuzz Tests                              │
    │  WASM Tests                              │
    └─────────────────────────────────────────┘
```

---

## 🎯 Test Distribution (2,500 Total)

### By Category

| Category | Count | % | Coverage Target |
|----------|-------|---|-----------------|
| **Unit Tests** | 2,250 | 90% | 95%+ per module |
| **Integration Tests** | 200 | 8% | 90%+ cross-module |
| **E2E Tests** | 50 | 2% | 100% CLI commands |
| **Total** | **2,500** | **100%** | **100% overall** |

### By Engine (Unit Tests Breakdown)

| Engine | Tests | Rationale |
|--------|-------|-----------|
| **Detection** | 350 | Parse 3 IaC formats, normalization, 50+ resource types |
| **Prediction** | 400 | Heuristics (50+ services), confidence, cold-start, edge cases |
| **Explain** | 300 | Reasoning chains, 8 step types, 50+ resource explanations |
| **Autofix** | 250 | Snippet/patch modes, drift-safe, rollback, 6 resource types |
| **Policy** | 300 | DSL parsing, 10 condition types, metadata, lifecycle |
| **Mapping** | 200 | Graph building, cycle detection, 4 export formats |
| **Grouping** | 250 | Module/service/environment grouping, attribution |
| **SLO** | 150 | 5 SLO types, burn alerts, enforcement levels |
| **Trend** | 100 | Snapshots, SVG/HTML generation, annotations |
| **WASM Runtime** | 150 | Sandbox limits, memory tracking, validation |
| **CLI** | 200 | Argument parsing, output formatting, error handling |
| **Utils** | 100 | Heuristics loading, file I/O, serialization |
| **Total** | **2,750** | *(buffer for growth)* |

---

## 🧪 Test Types & Tools

### 1. Unit Tests (2,250 tests)

**Framework:** `cargo test` (Rust built-in)

**Coverage per module:**
- Prediction: 400 tests
  - `test_heuristics_loader.rs` (50 tests)
  - `test_prediction_engine.rs` (100 tests)
  - `test_cold_start.rs` (80 tests)
  - `test_confidence_intervals.rs` (70 tests)
  - `test_probabilistic.rs` (50 tests)
  - `test_seasonality.rs` (30 tests)
  - `test_monte_carlo.rs` (20 tests)

- Detection: 350 tests
  - `test_terraform_parser.rs` (80 tests)
  - `test_cdk_parser.rs` (60 tests)
  - `test_normalizer.rs` (70 tests)
  - `test_resource_detector.rs` (80 tests)

- Policy: 300 tests
  - `test_rule_parser.rs` (80 tests)
  - `test_rule_evaluator.rs` (100 tests)
  - `test_metadata_engine.rs` (60 tests)
  - `test_lifecycle.rs` (60 tests)

**Patterns:**
```rust
// Standard unit test
#[test]
fn test_ec2_prediction_with_instance_type() {
    let config = ResourceConfig::ec2("t3.medium");
    let result = predict_cost(&config);
    assert_eq!(result.monthly_cost, 30.368);
}

// Parameterized tests
#[rstest]
#[case("t3.micro", 7.592)]
#[case("t3.medium", 30.368)]
#[case("m5.large", 70.08)]
fn test_ec2_instance_costs(#[case] instance_type: &str, #[case] expected: f64) {
    // ...
}

// Error cases
#[test]
#[should_panic(expected = "Invalid instance type")]
fn test_invalid_instance_type() {
    predict_cost(&ResourceConfig::ec2("invalid"));
}
```

---

### 2. Property-Based Tests (150 tests)

**Framework:** `proptest` or `quickcheck`

**Use cases:**
- Prediction intervals never inverted
- Cost deltas always valid
- Graph cycles always detected
- Policy evaluation deterministic
- JSON parsing round-trips

**Example:**
```rust
proptest! {
    #[test]
    fn prediction_interval_ordering(
        low in 0.0..1000.0,
        high in 1000.0..10000.0
    ) {
        let result = predict_with_confidence(low, high);
        assert!(result.p10 <= result.p50);
        assert!(result.p50 <= result.p90);
        assert!(result.p90 <= result.p99);
    }
}
```

---

### 3. Snapshot Tests (200 tests)

**Framework:** `insta` crate

**Coverage:**
- Terraform plan variants (50)
- CDK diff outputs (40)
- Explain output formatting (30)
- Mapping graph outputs (20)
- Policy evaluation reports (20)

**Example:**
```rust
#[test]
fn test_ec2_prediction_snapshot() {
    let plan = load_fixture("fixtures/ec2_create.json");
    let result = scan_plan(&plan);
    insta::assert_json_snapshot!(result);
}
```

**Storage:** `tests/snapshots/*.snap` (versioned)

---

### 4. Fuzz Tests (100 tests)

**Framework:** `cargo-fuzz` with libFuzzer

**Targets:**
- Terraform JSON parsing (no panics on malformed input)
- Policy DSL parsing (graceful error handling)
- Heuristics JSON parsing
- Mapping graph cycles (adversarial graphs)
- WASM input validation

**Example:**
```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_terraform_plan(s); // Should never panic
    }
});
```

**Continuous:** Run nightly, 100M+ inputs per target

---

### 5. Golden File Tests (150 tests)

**Framework:** Custom (compare against `.golden` files)

**Coverage:**
- Prediction output format (50)
- Explain reasoning chains (40)
- Mapping graph exports (30)
- Autofix patches (30)

**Directory:** `tests/golden/`
```
tests/golden/
├── prediction/
│   ├── ec2_t3_medium.golden
│   ├── rds_mysql_db_r5_large.golden
│   └── ...
├── explain/
│   ├── nat_gateway_reasoning.golden
│   └── ...
└── mapping/
    ├── microservices_graph.mermaid.golden
    └── ...
```

---

### 6. Baseline Tests (50 tests)

**Framework:** `cargo test` with baseline files

**Coverage:**
- Baseline file missing (graceful degradation)
- Baseline override scenarios
- Regression classifier integration
- SLO baseline evaluation

---

### 7. Integration Tests (200 tests)

**Location:** `tests/integration/`

**Scenarios:**
- Multi-engine pipelines (detect → predict → explain → autofix)
- File I/O workflows (read plan → write report)
- Policy + SLO enforcement chains
- Graph mapping + grouping combinations
- WASM runtime + all engines

**Example:**
```rust
#[test]
fn test_full_scan_pipeline() {
    let plan = load_fixture("integration/full_app.json");

    // Detection
    let detection = detect_changes(&plan);
    assert_eq!(detection.resources.len(), 15);

    // Prediction
    let prediction = predict_costs(&detection);
    assert!(prediction.monthly_delta > 0.0);

    // Explain
    let explanation = explain_prediction(&prediction);
    assert!(!explanation.reasoning_chain.is_empty());

    // Autofix
    let autofix = generate_autofix(&detection, &prediction);
    assert!(autofix.has_recommendations());
}
```

---

### 8. E2E Tests (50 tests)

**Framework:** `assert_cmd` + temp directories

**Coverage:**
- All CLI commands (15 commands × 3-4 scenarios each)
- Exit codes and error messages
- File output validation
- CI integration workflows

**Example:**
```rust
#[test]
fn test_cli_scan_command() {
    let temp = TempDir::new().unwrap();
    let plan = temp.child("plan.json");
    plan.write_str(SAMPLE_PLAN).unwrap();

    Command::cargo_bin("costpilot")
        .unwrap()
        .arg("scan")
        .arg(plan.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Monthly delta:"));
}
```

---

### 9. WASM-Specific Tests (150 tests)

**Frameworks:**
- Native: `cargo test`
- Browser: `wasm-bindgen-test`
- Determinism: Custom harness

**Coverage:**
- Memory limits (15 tests)
- Execution timeouts (10 tests)
- Determinism (12 tests - already implemented)
- Size limits (10 tests - already implemented)
- Sandbox violations (20 tests)
- Cross-engine execution (83 tests)

**Example:**
```rust
#[wasm_bindgen_test]
fn test_prediction_in_browser() {
    let plan = js_sys::JSON::parse(SAMPLE_PLAN).unwrap();
    let result = wasm_predict_costs(plan);
    assert!(result.is_ok());
}
```

---

### 10. Performance Tests (100 tests)

**Framework:** `criterion` for benchmarks

**Coverage:**
- Engine latency budgets (6 engines)
- Memory usage tracking
- Regression detection
- Large plan handling (10k resources)

**Example:**
```rust
fn benchmark_prediction(c: &mut Criterion) {
    let plan = generate_plan(100); // 100 resources
    c.bench_function("predict_100_resources", |b| {
        b.iter(|| predict_costs(black_box(&plan)))
    });
}
```

---

## 📁 Directory Structure

```
tests/
├── unit/                          # 2,250 tests
│   ├── detection/                 # 350 tests
│   │   ├── test_terraform_parser.rs
│   │   ├── test_cdk_parser.rs
│   │   ├── test_normalizer.rs
│   │   └── test_resource_detector.rs
│   ├── prediction/                # 400 tests
│   │   ├── test_heuristics_loader.rs
│   │   ├── test_prediction_engine.rs
│   │   ├── test_cold_start.rs
│   │   ├── test_confidence.rs
│   │   ├── test_probabilistic.rs
│   │   └── test_monte_carlo.rs
│   ├── explain/                   # 300 tests
│   │   ├── test_reasoning_chain.rs
│   │   ├── test_prediction_explainer.rs
│   │   └── test_formatters.rs
│   ├── autofix/                   # 250 tests
│   │   ├── test_snippet_generator.rs
│   │   ├── test_patch_generator.rs
│   │   ├── test_drift_safe.rs
│   │   └── test_rollback.rs
│   ├── policy/                    # 300 tests
│   │   ├── test_rule_parser.rs
│   │   ├── test_rule_evaluator.rs
│   │   ├── test_metadata_engine.rs
│   │   └── test_lifecycle.rs
│   ├── mapping/                   # 200 tests
│   │   ├── test_graph_builder.rs
│   │   ├── test_cycle_detection.rs
│   │   ├── test_mermaid_export.rs
│   │   ├── test_graphviz_export.rs
│   │   └── test_json_export.rs
│   ├── grouping/                  # 250 tests
│   │   ├── test_by_module.rs
│   │   ├── test_by_service.rs
│   │   ├── test_by_environment.rs
│   │   └── test_attribution.rs
│   ├── slo/                       # 150 tests
│   │   ├── test_slo_types.rs
│   │   ├── test_burn_alerts.rs
│   │   └── test_enforcement.rs
│   ├── trend/                     # 100 tests
│   │   ├── test_snapshot.rs
│   │   ├── test_svg_generation.rs
│   │   └── test_html_generation.rs
│   ├── wasm/                      # 150 tests (37 already implemented)
│   │   ├── test_runtime.rs
│   │   ├── test_sandbox_limits.rs
│   │   ├── wasm_determinism_tests.rs (✓ existing)
│   │   ├── wasm_memory_tests.rs (✓ existing)
│   │   └── wasm_size_tests.rs (✓ existing)
│   ├── cli/                       # 200 tests
│   │   ├── test_parser.rs
│   │   ├── test_commands.rs
│   │   └── test_output.rs
│   └── utils/                     # 100 tests
│       ├── test_heuristics_loader.rs
│       ├── test_file_io.rs
│       └── test_serialization.rs
│
├── integration/                   # 200 tests
│   ├── test_detect_predict_explain.rs
│   ├── test_policy_slo_enforcement.rs
│   ├── test_mapping_grouping.rs
│   ├── test_wasm_engines.rs
│   └── test_file_workflows.rs
│
├── e2e/                          # 50 tests
│   ├── test_cli_scan.rs
│   ├── test_cli_autofix.rs
│   ├── test_cli_mapping.rs
│   ├── test_cli_grouping.rs
│   ├── test_cli_policy.rs
│   └── test_ci_integration.rs
│
├── property/                     # 150 tests
│   ├── prop_prediction_invariants.rs
│   ├── prop_graph_properties.rs
│   ├── prop_policy_determinism.rs
│   └── prop_json_roundtrips.rs
│
├── snapshot/                     # 200 tests
│   ├── snap_terraform_plans.rs
│   ├── snap_cdk_diffs.rs
│   ├── snap_explain_output.rs
│   └── snap_mapping_graphs.rs
│
├── fuzz/                         # 100 tests
│   ├── fuzz_terraform_parser/
│   ├── fuzz_policy_dsl/
│   ├── fuzz_heuristics_json/
│   └── fuzz_graph_cycles/
│
├── golden/                       # 150 tests
│   ├── prediction/
│   ├── explain/
│   ├── mapping/
│   └── autofix/
│
├── baseline/                     # 50 tests
│   └── test_baseline_scenarios.rs
│
├── performance/                  # 100 tests (criterion benches)
│   ├── bench_prediction.rs
│   ├── bench_detection.rs
│   ├── bench_policy.rs
│   └── bench_mapping.rs
│
├── fixtures/                     # Test data
│   ├── terraform/
│   │   ├── ec2_create.json
│   │   ├── rds_modify.json
│   │   └── ...
│   ├── cdk/
│   ├── policies/
│   └── baselines/
│
└── helpers/                      # Test utilities
    ├── mod.rs
    ├── fixtures.rs
    ├── assertions.rs
    └── generators.rs
```

---

## 🎯 Coverage Targets by Module

| Module | Target | Priority | Tests |
|--------|--------|----------|-------|
| `detection/` | 98% | Critical | 350 |
| `prediction/` | 98% | Critical | 400 |
| `explain/` | 95% | High | 300 |
| `autofix/` | 95% | High | 250 |
| `policy/` | 98% | Critical | 300 |
| `mapping/` | 95% | High | 200 |
| `grouping/` | 90% | Medium | 250 |
| `slo/` | 95% | High | 150 |
| `trend/` | 90% | Medium | 100 |
| `wasm/` | 98% | Critical | 150 |
| `cli/` | 95% | High | 200 |
| `utils/` | 95% | High | 100 |
| **Overall** | **95%+** | - | **2,750** |

---

## 🔧 Tools & Infrastructure

### Code Coverage

**Tool:** `cargo-tarpaulin` or `cargo-llvm-cov`

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# Target: 95%+ overall, 98%+ for critical paths
```

**CI Integration:**
- Coverage report on every PR
- Block merge if coverage drops below 90%
- Generate badges for README

### Mutation Testing

**Tool:** `cargo-mutants`

```bash
# Find untested code paths
cargo mutants
```

**Target:** Catch 95%+ of injected mutations

### Test Data Generators

**Tool:** `fake` + `quickcheck`

```rust
// Generate realistic test data
fn generate_terraform_plan(resources: usize) -> TerraformPlan {
    let mut plan = TerraformPlan::default();
    for i in 0..resources {
        plan.resources.push(fake_ec2_resource(i));
    }
    plan
}
```

### CI/CD Integration

**GitHub Actions:**
```yaml
test:
  - unit-tests:        cargo test --all
  - integration-tests: cargo test --test integration_*
  - e2e-tests:        cargo test --test e2e_*
  - fuzz-tests:       cargo fuzz run --runs=1000000 (nightly)
  - coverage:         cargo tarpaulin --out Xml
  - mutation:         cargo mutants (weekly)
```

---

## 📈 Testing Phases (Implementation Plan)

### Phase 1: Foundation (Week 1) - 500 tests
- [x] WASM tests (37 existing)
- [ ] Core unit tests for Detection (100)
- [ ] Core unit tests for Prediction (120)
- [ ] Core unit tests for Policy (80)
- [ ] Basic integration tests (20)
- [ ] E2E smoke tests (10)
- [ ] Coverage infrastructure setup

### Phase 2: Core Engines (Week 2) - 800 tests
- [ ] Complete Detection tests (250 remaining)
- [ ] Complete Prediction tests (280 remaining)
- [ ] Explain engine tests (300)
- [ ] Autofix engine tests (150)
- [ ] Snapshot tests setup (100)

### Phase 3: Advanced Features (Week 3) - 600 tests
- [ ] Policy engine complete (220 remaining)
- [ ] Mapping tests (200)
- [ ] Grouping tests (250)
- [ ] SLO tests (150)

### Phase 4: Quality Assurance (Week 4) - 600 tests
- [ ] Property-based tests (150)
- [ ] Fuzz tests (100)
- [ ] Golden file tests (150)
- [ ] Performance tests (100)
- [ ] Integration tests complete (180 remaining)
- [ ] E2E tests complete (40 remaining)

**Total:** 2,500+ tests across 4 weeks

---

## ✅ Test Quality Standards

### Every test must:
1. **Be deterministic** - Same input = Same output
2. **Be isolated** - No shared state between tests
3. **Be fast** - Unit tests < 50ms, Integration < 500ms
4. **Be clear** - Obvious what's being tested
5. **Be WASM-safe** - No network, filesystem, or non-deterministic ops

### Naming Convention:
```rust
#[test]
fn test_<component>_<scenario>_<expected_outcome>() {
    // Example:
    // test_prediction_ec2_t3_medium_returns_correct_cost()
    // test_policy_block_action_prevents_deployment()
    // test_mapping_cycle_detection_finds_circular_deps()
}
```

### Assertion Patterns:
```rust
// Use descriptive messages
assert_eq!(result.monthly_cost, 30.368,
    "EC2 t3.medium should cost $30.368/month");

// Use helper assertions
assert_prediction_valid(&result);
assert_no_policy_violations(&evaluation);
assert_graph_acyclic(&graph);
```

---

## 🚨 Critical Path Testing

### Zero-IAM Guarantees (Must have 100% coverage):
- [ ] No network calls in any code path
- [ ] No AWS SDK imports
- [ ] WASM sandbox enforced
- [ ] Secrets/tokens redacted
- [ ] File size limits enforced
- [ ] Memory limits enforced
- [ ] Timeout limits enforced

### Determinism Guarantees (Must have 100% coverage):
- [x] Identical inputs → identical outputs
- [x] No random number generation
- [x] No system time dependencies
- [x] No filesystem side effects
- [ ] Stable error messages
- [ ] Consistent ordering (BTreeMap, not HashMap)

### Safety Guarantees (Must have 100% coverage):
- [ ] No panics on malformed input
- [ ] Graceful error handling
- [ ] Prediction intervals never inverted
- [ ] Cost deltas never negative
- [ ] Graph validation always runs

---

## 🔄 Continuous Testing

### Pre-commit Hooks:
```bash
# Run fast tests
cargo test --lib
cargo fmt --check
cargo clippy -- -D warnings
```

### PR Checks:
```bash
# Full test suite
cargo test --all
cargo tarpaulin --out Xml
cargo mutants --check
```

### Nightly:
```bash
# Extensive fuzz testing
cargo fuzz run --runs=100000000
```

### Weekly:
```bash
# Performance regression
cargo bench
# Compare against baseline
```

---

## 📊 Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| **Total Tests** | 2,500+ | ~40 |
| **Code Coverage** | 95%+ | TBD |
| **Critical Path Coverage** | 100% | ~90% |
| **Mutation Score** | 95%+ | TBD |
| **Test Execution Time** | < 2 min | < 5 sec |
| **Flaky Tests** | 0 | 0 |
| **Test Debt** | < 5% | TBD |

---

## 🎓 Testing Best Practices

### 1. **Table-Driven Tests**
```rust
#[rstest]
#[case("t3.micro", 7.592)]
#[case("t3.small", 15.184)]
#[case("t3.medium", 30.368)]
fn test_ec2_costs(#[case] instance: &str, #[case] cost: f64) {
    assert_eq!(predict_ec2_cost(instance), cost);
}
```

### 2. **Test Fixtures**
```rust
// tests/helpers/fixtures.rs
pub fn sample_terraform_plan() -> TerraformPlan { /* ... */ }
pub fn sample_ec2_resource() -> Resource { /* ... */ }
```

### 3. **Custom Assertions**
```rust
// tests/helpers/assertions.rs
pub fn assert_prediction_valid(pred: &Prediction) {
    assert!(pred.low <= pred.estimate);
    assert!(pred.estimate <= pred.high);
    assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
}
```

### 4. **Error Testing**
```rust
#[test]
fn test_invalid_heuristics_returns_error() {
    let result = load_heuristics("invalid.json");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "HEURISTICS_INVALID");
}
```

---

## 🔮 Future Enhancements

- **Chaos Engineering:** Random failure injection in integration tests
- **Load Testing:** 10k+ resource plans
- **Stress Testing:** Memory pressure scenarios
- **Soak Testing:** 24-hour continuous runs
- **Regression Testing:** Compare against previous versions
- **Visual Regression:** Screenshot diffs for HTML/SVG outputs

---

**Last Updated:** 2025-12-06
**Version:** 1.0.0
**Status:** Foundation Phase Active
