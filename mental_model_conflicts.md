# Mental Model Conflicts Report

## Summary: 17 conflicts found

## Conflict 1
**Description:** Non-deterministic UUID generation in production code
**Mental Model Claim:** Non-deterministic behavior is a defect
**Codebase Evidence:** Pattern 'uuid::' found in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/metering/usage_meter.rs

## Conflict 2
**Description:** Non-deterministic Random number generation in production code
**Mental Model Claim:** Non-deterministic behavior is a defect
**Codebase Evidence:** Pattern 'rand::' found in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/policy/zero_network.rs

## Conflict 3
**Description:** Non-deterministic UUID generation in production code
**Mental Model Claim:** Non-deterministic behavior is a defect
**Codebase Evidence:** Pattern 'uuid::' found in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/escrow/release.rs

## Conflict 4
**Description:** Non-deterministic Random number generation in production code
**Mental Model Claim:** Non-deterministic behavior is a defect
**Codebase Evidence:** Pattern 'rand::' found in /home/dee/workspace/AI/GuardSuite/CostPilot/src/bin/license_issuer.rs

## Conflict 5
**Description:** ZeroCostGuard not found in main execution path
**Mental Model Claim:** ZeroCostGuard executes before command execution
**Codebase Evidence:** ZeroCostGuard not referenced in main.rs

## Conflict 6
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/prediction/cold_start.rs

## Conflict 7
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/explain/explain_engine.rs

## Conflict 8
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/policy/exemption_types.rs

## Conflict 9
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'http.*get' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/policy/zero_network.rs

## Conflict 10
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'api\.' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/policy/zero_network.rs

## Conflict 11
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/autofix/snippet_generator.rs

## Conflict 12
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/autofix/patch_generator.rs

## Conflict 13
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/autofix/drift_safety/critical_drift.rs

## Conflict 14
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/mapping/mermaid_generator.rs

## Conflict 15
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/mapping/mod.rs

## Conflict 16
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'server' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/engines/mapping/graphviz_generator.rs

## Conflict 17
**Description:** Potential server-side validation detected
**Mental Model Claim:** No server-side license validation occurs at runtime
**Codebase Evidence:** Server pattern 'api\.' in /home/dee/workspace/AI/GuardSuite/CostPilot/src/security/validator.rs
