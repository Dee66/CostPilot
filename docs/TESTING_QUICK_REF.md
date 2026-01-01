# Testing Quick Reference

**Quick commands for developers working on CostPilot tests**

---

## 🚀 Running Tests

### All Tests
```bash
cargo test --all-features
```

### Unit Tests Only
```bash
cargo test --lib
```

### Specific Module
```bash
cargo test --test test_detection_engine
cargo test --test test_prediction_engine
```

### Integration Tests
```bash
cargo test --test 'integration_*'
```

### E2E Tests
```bash
cargo test --test 'e2e_*'
```

### With Output
```bash
cargo test -- --nocapture
```

### Single Test
```bash
cargo test test_ec2_prediction_with_instance_type
```

---

## 📊 Code Coverage

### Generate HTML Report
```bash
cargo tarpaulin --out Html --output-dir coverage/
open coverage/index.html
```

### Check Threshold
```bash
cargo tarpaulin --config tarpaulin.toml
```

### Quick Coverage
```bash
cargo tarpaulin --lib
```

---

## 📸 Snapshot Tests

### Accept New Snapshots
```bash
cargo test -- --nocapture
cargo insta review
```

### Update All Snapshots
```bash
cargo insta accept
```

---

## 🎲 Property-Based Tests

### Run with More Cases
```bash
PROPTEST_CASES=10000 cargo test prop_
```

---

## 🔥 Fuzz Tests

### List Fuzz Targets
```bash
cargo fuzz list
```

### Run Fuzz Target
```bash
cargo fuzz run fuzz_terraform_parser
```

### With Time Limit
```bash
cargo fuzz run fuzz_terraform_parser -- -max_total_time=300
```

---

## 📈 Benchmarks

### Run All Benchmarks
```bash
cargo bench
```

### Specific Benchmark
```bash
cargo bench predict
```

### Open Report
```bash
open target/criterion/report/index.html
```

---

## 🧬 Mutation Testing

### Install cargo-mutants
```bash
cargo install cargo-mutants
```

### Run Mutation Tests
```bash
cargo mutants
```

### Quick Check
```bash
cargo mutants --timeout 60 --jobs 4
```

---

## 🔍 Debugging Tests

### Show Test Output
```bash
cargo test -- --nocapture
```

### Run One Test with Backtrace
```bash
RUST_BACKTRACE=1 cargo test test_name -- --exact --nocapture
```

### Test-Specific Log Level
```bash
RUST_LOG=debug cargo test
```

---

## ✅ Pre-Commit Checks

### Fast Checks
```bash
cargo test --lib
cargo fmt --check
cargo clippy -- -D warnings
```

### Full PR Validation
```bash
cargo test --all-features
cargo tarpaulin --config tarpaulin.toml
cargo bench --no-run
```

---

## 📝 Writing Tests

### Unit Test Template
```rust
#[test]
fn test_component_scenario_expected() {
    // Arrange
    let input = setup_test_data();

    // Act
    let result = function_under_test(&input);

    // Assert
    assert_eq!(result.value, expected_value);
}
```

### Property Test Template
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_name(value in 0.0..1000.0) {
        let result = function(value);
        prop_assert!(result >= 0.0);
    }
}
```

### Snapshot Test Template
```rust
#[test]
fn test_output_snapshot() {
    let result = generate_output();
    insta::assert_json_snapshot!(result);
}
```

### Parameterized Test Template
```rust
use rstest::rstest;

#[rstest]
#[case("t3.micro", 7.592)]
#[case("t3.medium", 30.368)]
fn test_costs(#[case] instance: &str, #[case] cost: f64) {
    assert_eq!(predict(instance), cost);
}
```

---

## 🎯 Test Coverage Goals

| Module | Target | Priority |
|--------|--------|----------|
| detection/ | 98% | Critical |
| prediction/ | 98% | Critical |
| policy/ | 98% | Critical |
| wasm/ | 98% | Critical |
| autofix/ | 95% | High |
| mapping/ | 95% | High |
| Overall | **95%+** | - |

---

## 🛠️ Helper Usage

### Using Test Fixtures
```rust
use crate::helpers::*;

#[test]
fn test_with_fixture() {
    let plan = terraform_plan_with_ec2("t3.medium");
    let result = parse_plan(&plan);
    assert!(result.is_ok());
}
```

### Using Custom Assertions
```rust
use crate::helpers::*;

#[test]
fn test_prediction() {
    let pred = predict_cost(&config);
    assert_prediction_valid(
        pred.low,
        pred.estimate,
        pred.high,
        pred.confidence
    );
}
```

### Using Generators
```rust
use crate::helpers::*;

#[test]
fn test_large_plan() {
    let plan = generate_large_terraform_plan(1000);
    let result = scan_plan(&plan);
    assert!(result.is_ok());
}
```

---

## 📦 Test Data Location

```
tests/
├── fixtures/           # Static test data files
│   ├── terraform/
├── snapshots/          # Snapshot test baselines
├── golden/            # Golden file references
└── helpers/           # Test utilities
```

---

## 🚨 Common Issues

### Tests Fail with "file not found"
```bash
# Make sure you're in the project root
cd /path/to/CostPilot
cargo test
```

### Coverage Report Empty
```bash
# Install tarpaulin first
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Snapshot Mismatch
```bash
# Review and accept changes
cargo insta review
# Or reject all
cargo insta reject
```

### Benchmark Won't Run
```bash
# Benchmarks require release mode
cargo bench
# Not: cargo test --benches
```

---

## 📚 Resources

- **Main Strategy:** [TESTING_STRATEGY.md](TESTING_STRATEGY.md)
- **Fixtures:** [tests/helpers/fixtures.rs](tests/helpers/fixtures.rs)
- **Assertions:** [tests/helpers/assertions.rs](tests/helpers/assertions.rs)
- **Criterion Docs:** https://bheisler.github.io/criterion.rs/
- **Proptest Guide:** https://proptest-rs.github.io/proptest/
- **Insta Docs:** https://insta.rs/

---

**Last Updated:** 2025-12-06
**Version:** 1.0.0
