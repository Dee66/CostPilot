# CostPilot Coverage Regression Monitoring Report

**Generated:** 2025-12-20 12:04:12

## Coverage Monitoring Results

### Unit Coverage Coverage

Running enforce_unit_coverage.sh...
[1;33m⚠️  SAFETY NOTICE: This system analyzes test coverage only.[0m
[1;33m⚠️  NO actual deployments or infrastructure changes are made.[0m

[0;34mStarting Unit Test Coverage Enforcement System...[0m
Analyzing codebase for coverage estimation...
Estimating coverage from codebase structure...
[0;34m📊 Estimated coverage from codebase analysis:[0m
  Critical modules: 50.0% (71 test lines / 10257 code lines)
  Core engines: 85.0% (20627 test lines / 38741 code lines)
  Utilities: 95.0% (341243 test lines / 55572 code lines)
  Overall: 95.0% (361941 test lines / 104570 code lines)
--- Exit code: 2 ---
❌ FAILED (exit code: 2)

### Integration Coverage Coverage

Running enforce_integration_coverage.sh...
[1;33m⚠️  SAFETY NOTICE: This system analyzes integration test coverage only.[0m
[1;33m⚠️  NO actual deployments or infrastructure changes are made.[0m

[0;34mStarting Integration Coverage Enforcement System...[0m
Analyzing API endpoints...
Found 2031 API endpoints, 507 tested
Analyzing data flows...
Found 99 data flows, 99 tested
Analyzing error paths...
Found 115 error paths, 115 tested
[0;32m✅ Integration coverage report generated: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/integration_coverage/reports/integration_coverage_report_20251220_120418.md[0m
[0;34mQuality gate created: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/integration_coverage/quality_gates/integration_coverage_gate_20251220_120418.json[0m
[0;31m⚠️  1 integration coverage targets not met[0m
[1;33mReview integration coverage report for improvement recommendations[0m
--- Exit code: 1 ---
❌ FAILED (exit code: 1)

### E2e Coverage Coverage

Running enforce_e2e_coverage.sh...
⚠️  SAFETY NOTICE: This system analyzes E2E test coverage only.
⚠️  NO actual deployments or infrastructure changes are made.

[0;34mStarting E2E Coverage Enforcement System...[0m
Analyzing user workflows...
Found 136 user workflows, 62 tested
Analyzing failure scenarios...
Found 73 failure scenarios, 73 tested
Analyzing platform matrix...
Found 80 platform matrix combinations, 80 tested
[0;32m✅ E2E coverage report generated: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/e2e_coverage/reports/e2e_coverage_report_20251220_120425.md[0m
[0;34mQuality gate created: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/e2e_coverage/quality_gates/e2e_coverage_gate_20251220_120425.json[0m
[0;31m⚠️  1 E2E coverage targets not met[0m
[1;33mReview E2E coverage report for improvement recommendations[0m
--- Exit code: 1 ---
❌ FAILED (exit code: 1)

### Property Coverage Coverage

Running enforce_property_coverage.sh...
⚠️  SAFETY NOTICE: This system analyzes property-based test coverage only.
⚠️  NO actual deployments or infrastructure changes are made.

[0;34mStarting Property-Based Coverage Enforcement System...[0m
Analyzing invariants...
Found 134 invariants, 84 tested
Analyzing edge cases...
Found 200 edge cases, 36 tested
[0;32m✅ Property-based coverage report generated: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/property_coverage/reports/property_coverage_report_20251220_120436.md[0m
[0;34mQuality gate created: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/property_coverage/quality_gates/property_coverage_gate_20251220_120436.json[0m
[0;31m⚠️  2 property-based coverage targets not met[0m
[1;33mReview property-based coverage report for improvement recommendations[0m
--- Exit code: 2 ---
❌ FAILED (exit code: 2)

### Security Coverage Coverage

Running enforce_security_coverage.sh...
⚠️  SAFETY NOTICE: This system analyzes security test coverage only.
⚠️  NO actual deployments or infrastructure changes are made.

[0;34mStarting Security Coverage Enforcement System...[0m
Analyzing input validation...
Found 154 input validation points, 48 tested
Analyzing authentication...
Found 29 authentication points, 12 tested
Analyzing authorization...
Found 56 authorization points, 24 tested
Analyzing data protection...
Found 82 data protection points, 82 tested
[0;32m✅ Security coverage report generated: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/security_coverage/reports/security_coverage_report_20251220_120452.md[0m
[0;34mQuality gate created: /home/dee/workspace/AI/GuardSuite/CostPilot/tests/security_coverage/quality_gates/security_coverage_gate_20251220_120452.json[0m
[0;31m⚠️  3 security coverage targets not met[0m
[1;33mReview security coverage report for improvement recommendations[0m
--- Exit code: 3 ---
❌ FAILED (exit code: 3)

## Regression Analysis

Regression Analysis:
===================
[0;34m➡️  unit coverage STABLE: 0%[0m
[0;34m➡️  integration coverage STABLE: 0%[0m
[0;34m➡️  e2e coverage STABLE: 0%[0m
[0;34m➡️  property coverage STABLE: 0%[0m
[0;34m➡️  security coverage STABLE: 0%[0m


## Summary

- **Total Coverage Violations:** 9
- **Coverage Regressions:** _REGRESSIONS_COUNT_MARKER_

⚠️  **9 coverage violations detected.** Review individual coverage reports for details.
