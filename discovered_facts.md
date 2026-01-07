# Discovered Raw Facts

## Section: 1. Project Identity
**Fact:** Binary targets: costpilot, license-issuer
**Evidence:** Cargo.toml contains binary targets: ['costpilot', 'license-issuer']

## Section: 3. Execution Model
**Fact:** Supported IaC formats: CloudFormation, Terraform
**Evidence:** Code references found for: {'CloudFormation', 'Terraform'}

## Section: 6. Security Boundary
**Fact:** Network libraries found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/zero_cost_guard.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/messages.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/trend/svg_generator.rs']
**Evidence:** Network libraries found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/zero_cost_guard.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/messages.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/trend/svg_generator.rs']

## Section: 6. Security Boundary
**Fact:** ZeroCostGuard references found in source code
**Evidence:** ZeroCostGuard references found in source code

## Section: 6. Security Boundary
**Fact:** WASM references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/config.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs']
**Evidence:** WASM references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/config.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs']

## Section: 7. Edition & Licensing Model
**Fact:** Edition references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/mod.rs']
**Evidence:** Edition references found in: ['/home/dee/workspace/AI/GuardSuite/CostPilot/src/lib.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/pro_handle.rs', '/home/dee/workspace/AI/GuardSuite/CostPilot/src/edition/mod.rs']

## Section: 8. Known Volatility
**Fact:** Found 53 command variants in main.rs
**Evidence:** Found 53 command variants in main.rs
