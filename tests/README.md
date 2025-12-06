# CostPilot Test Suite

**Comprehensive testing infrastructure targeting 100% code coverage and ~2,500 tests**

---

## 📊 Current Status

- **Total Tests:** ~40 (WASM tests implemented)
- **Target Tests:** 2,500+
- **Code Coverage:** TBD (Target: 95%+)
- **Test Infrastructure:** ✅ Complete

---

## 📁 Directory Structure

```
tests/
├── unit/                  # Unit tests (2,250 tests planned)
│   ├── test_detection_engine.rs
│   ├── test_prediction_engine.rs
│   ├── test_explain_engine.rs
│   ├── test_autofix_engine.rs
│   ├── test_policy_engine.rs
│   ├── test_mapping_engine.rs
│   ├── test_grouping_engine.rs
│   └── test_slo_engine.rs
│
├── integration/           # Integration tests (200 tests planned)
│   └── test_pipelines.rs
│
├── e2e/                   # End-to-end tests (50 tests planned)
│   └── test_cli.rs
│
├── property/              # Property-based tests (150 tests planned)
├── snapshot/              # Snapshot tests (200 tests planned)
├── fuzz/                  # Fuzz tests (100 tests planned)
├── golden/                # Golden file tests (150 tests planned)
├── baseline/              # Baseline tests (50 tests planned)
│
├── fixtures/              # Test data
│   ├── terraform/
│   ├── cdk/
│   ├── cloudformation/
│   ├── policies/
│   └── baselines/
│
├── helpers/               # Test utilities
│   ├── mod.rs
│   ├── fixtures.rs        # Pre-built test data
│   ├── assertions.rs      # Custom assertions
│   └── generators.rs      # Test data generators
│
├── snapshots/             # Snapshot baselines (insta)
└── wasm_*.rs             # WASM tests (37 implemented ✓)
```

---

## 🚀 Quick Start

### Run All Tests
```bash
cargo test --all-features
```

### Generate Coverage Report
```bash
cargo tarpaulin --out Html --output-dir coverage/
open coverage/index.html
```

### Run Benchmarks
```bash
cargo bench
open target/criterion/report/index.html
```

### Generate New Test
```bash
python3 scripts/generate_test.py unit prediction test_new_feature
```

---

## 📋 Test Distribution (2,500 Total)

| Category | Tests | % | Status |
|----------|-------|---|--------|
| **Unit** | 2,250 | 90% | Templates ready |
| **Integration** | 200 | 8% | Templates ready |
| **E2E** | 50 | 2% | Templates ready |
| **Property** | 150 | - | Infrastructure ready |
| **Snapshot** | 200 | - | Infrastructure ready |
| **Fuzz** | 100 | - | Infrastructure ready |

---

## 🧪 Test Types

### 1. Unit Tests (tests/unit/)
**Coverage:** Function-level testing of individual components

**Examples:**
- `test_prediction_engine.rs` - 400 tests planned
- `test_detection_engine.rs` - 350 tests planned
- `test_policy_engine.rs` - 300 tests planned

**Run:**
```bash
cargo test --lib
```

### 2. Integration Tests (tests/integration/)
**Coverage:** Multi-engine pipelines and workflows

**Examples:**
- Detect → Predict → Explain → Autofix
- Policy + SLO enforcement
- Mapping + Grouping combinations

**Run:**
```bash
cargo test --test 'integration_*'
```

### 3. E2E Tests (tests/e2e/)
**Coverage:** CLI commands with real file I/O

**Examples:**
- `costpilot scan plan.json`
- `costpilot autofix patch plan.json`
- `costpilot map --format=mermaid`

**Run:**
```bash
cargo test --test 'e2e_*'
```

### 4. Property-Based Tests
**Coverage:** Invariant validation across input ranges

**Framework:** proptest, quickcheck

**Examples:**
- Prediction intervals never inverted
- Cost deltas always valid
- Graph cycles always detected

**Run:**
```bash
cargo test prop_
```

### 5. Snapshot Tests
**Coverage:** Output format regression detection

**Framework:** insta

**Examples:**
- Terraform plan parsing output
- Explain reasoning chain format
- Mapping graph exports

**Run:**
```bash
cargo test snap_
cargo insta review
```

### 6. Fuzz Tests
**Coverage:** Malformed input resilience

**Framework:** cargo-fuzz

**Examples:**
- Terraform JSON parsing
- Policy DSL parsing
- Graph cycle detection

**Run:**
```bash
cargo fuzz run fuzz_terraform_parser
```

---

## 🎯 Coverage Targets

| Module | Target | Priority |
|--------|--------|----------|
| detection/ | 98% | Critical |
| prediction/ | 98% | Critical |
| policy/ | 98% | Critical |
| wasm/ | 98% | Critical |
| autofix/ | 95% | High |
| explain/ | 95% | High |
| mapping/ | 95% | High |
| slo/ | 95% | High |
| grouping/ | 90% | Medium |
| **Overall** | **95%+** | - |

---

## 🛠️ Helper Functions

### Test Fixtures
Pre-built test data for common scenarios:

```rust
use crate::helpers::*;

let plan = terraform_plan_with_ec2("t3.medium");
let plan = terraform_plan_with_rds("db.r5.large", "mysql", 100);
let policy = policy_with_nat_limit();
let baseline = baseline_with_module("root.vpc", 1000.0);
```

### Custom Assertions
Domain-specific validation:

```rust
use crate::helpers::*;

assert_prediction_valid(low, estimate, high, confidence);
assert_interval_not_inverted(p10, p50, p90, p99);
assert_cost_delta_valid(old, new, delta, percentage);
assert_graph_acyclic(has_cycles, cycle_nodes);
```

### Test Generators
Create test data dynamically:

```rust
use crate::helpers::*;

let plan = generate_terraform_plan_with_n_ec2(100);
let plan = generate_mixed_terraform_plan(10, 5, 20);
let plan = generate_large_terraform_plan(1000);
let policy = generate_policy_with_n_rules(50);
```

---

## 📈 Performance Budgets

Tests enforce performance budgets for critical paths:

| Engine | Budget | Test |
|--------|--------|------|
| Prediction | 300ms | bench_prediction_single |
| Detection | 400ms | bench_detection_parse |
| Policy | 200ms | bench_policy_evaluation |
| Mapping | 500ms | bench_mapping_build_graph |
| Total Scan | 2000ms | bench_full_scan_pipeline |

---

## 🔧 Tools & Dependencies

### Test Frameworks
- **cargo test** - Built-in unit/integration tests
- **proptest** - Property-based testing
- **quickcheck** - Alternative property testing
- **insta** - Snapshot testing
- **criterion** - Benchmarking

### E2E Testing
- **assert_cmd** - CLI testing
- **predicates** - Assertion helpers
- **tempfile** - Temporary directories

### Utilities
- **rstest** - Parameterized tests
- **test-case** - Test cases
- **fake** - Fake data generation
- **pretty_assertions** - Better diffs

### Coverage
- **cargo-tarpaulin** - Code coverage
- **cargo-llvm-cov** - Alternative coverage

### Quality
- **cargo-mutants** - Mutation testing
- **cargo-fuzz** - Fuzz testing

---

## 📝 Writing Tests

### Best Practices

1. **Arrange-Act-Assert Pattern**
```rust
#[test]
fn test_feature() {
    // Arrange
    let input = setup_test_data();
    
    // Act
    let result = function_under_test(&input);
    
    // Assert
    assert_eq!(result.value, expected_value);
}
```

2. **Descriptive Names**
```rust
#[test]
fn test_prediction_ec2_t3_medium_returns_correct_cost() { }
```

3. **Test One Thing**
```rust
#[test]
fn test_prediction_positive() { }

#[test]
fn test_prediction_interval_ordered() { }
```

4. **Use Helpers**
```rust
let plan = terraform_plan_with_ec2("t3.medium");
assert_prediction_valid(pred.low, pred.estimate, pred.high, pred.confidence);
```

---

## 🔄 CI/CD Integration

Tests run automatically on:
- **Every push** to main/develop
- **Every pull request**
- **Nightly** (fuzz tests)
- **Weekly** (mutation tests)

**GitHub Actions:**
- ✅ Unit tests
- ✅ Integration tests
- ✅ E2E tests
- ✅ Code coverage (90% threshold)
- ✅ Property tests
- ✅ Snapshot tests
- ✅ Fuzz tests
- ✅ Performance benchmarks
- ✅ Lint & format

---

## 📚 Documentation

- **Main Strategy:** [../TESTING_STRATEGY.md](../TESTING_STRATEGY.md)
- **Quick Reference:** [../TESTING_QUICK_REF.md](../TESTING_QUICK_REF.md)
- **Test Generator:** [../scripts/generate_test.py](../scripts/generate_test.py)

---

## 🚨 Common Issues

### Coverage report empty?
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Snapshot mismatch?
```bash
cargo insta review
cargo insta accept  # or reject
```

### Benchmark won't run?
```bash
cargo bench  # Not: cargo test --benches
```

---

## 🎓 Resources

- **Criterion:** https://bheisler.github.io/criterion.rs/
- **Proptest:** https://proptest-rs.github.io/proptest/
- **Insta:** https://insta.rs/
- **Rust Book - Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html

---

**Target:** 2,500+ tests | 95%+ coverage | Zero flakiness  
**Status:** Infrastructure complete, implementation in progress  
**Last Updated:** 2025-12-06
