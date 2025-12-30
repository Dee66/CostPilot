# Proposed Mental Model Deltas

## Section: 1. Project Identity
**Proposed Addition:** Binary targets: costpilot, license-issuer
**Evidence:** Cargo.toml contains binary targets: ['costpilot', 'license-issuer']
**Confidence:** 100.0%

## Section: 3. Execution Model
**Proposed Addition:** Supported IaC formats: CloudFormation, Terraform
**Evidence:** Code references found for: {'Terraform', 'CloudFormation'}
**Confidence:** 80.0%

## Section: 6. Security Boundary
**Proposed Addition:** Runtime network access may be permitted in some contexts
**Evidence:** Network libraries found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/zero_cost_guard.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/messages.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/trend/svg_generator.rs']
**Confidence:** 90.0%

## Section: 6. Security Boundary
**Proposed Addition:** ZeroCostGuard executes before command execution
**Evidence:** ZeroCostGuard references found in source code
**Confidence:** 100.0%

## Section: 6. Security Boundary
**Proposed Addition:** WASM execution is used and sandboxed
**Evidence:** WASM references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/config.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs']
**Confidence:** 80.0%

## Section: 7. Edition & Licensing Model
**Proposed Addition:** Multiple editions exist with feature gating
**Evidence:** Edition references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/mod.rs']
**Confidence:** 90.0%

## Section: 8. Known Volatility
**Proposed Addition:** Command surface area is extensive
**Evidence:** Found 41 command variants in main.rs
**Confidence:** 70.0%
