metadata:
  product_id: costpilot
  product_version: 1.0.2
  checklist_version: '2.1'
  source_spec_version: '2.1'
  created_at: '2025-12-07'
  last_reviewed: '2025-12-07'
  reviewer: System (AI Architect)
  auto_generated: false
  maintained_by: Deon Prinsloo
  target_environment: prod
  launch_tier: A
execution_context:
  automation_ready: true
  agent_directives:
    retry_on_fail: true
    parallelizable: true
    halt_on_critical_failure: true
linked_issues: []
structure:
  phases:
  - id: phase-01
    title: Project Initialization & Spec Wiring
    outcome: CostPilot project wired to v2.1 spec, zero-cost policy and folder layout.
    automation_expectation: full
  - id: phase-02
    title: 'Core Engines: Detect, Predict, Explain'
    outcome: Primary analysis engines implemented and deterministic.
    automation_expectation: partial
  - id: phase-03
    title: Zero-Cost Runtime & Deterministic WASM Pipeline
    outcome: Execution fully offline, zero-cost, and sandboxed with canonical outputs.
    automation_expectation: partial
  - id: phase-04
    title: 'Governance: Policy-as-Code & SLO Engine'
    outcome: Policies, exemptions and SLOs enforced with clear reports.
    automation_expectation: partial
  - id: phase-05
    title: Mapping, Baselines, Trend & Attribution
    outcome: Insight engines implemented with reproducible outputs.
    automation_expectation: partial
  - id: phase-06
    title: Testing, CI, Release & Final Invariants
    outcome: Test suites, CI pipelines and invariants validated for release.
    automation_expectation: partial
## 📊 Overall Progress

<div role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="100" style="width:94%; background:#e6eef0; border-radius:8px; padding:6px; box-shadow: inset 0 1px 2px rgba(0,0,0,0.04);">
  <div style="width:100.0%; background:linear-gradient(90deg,#2e7d32,#4caf50,#66bb6a); color:#fff; padding:10px 12px; text-align:right; border-radius:6px; font-weight:700; transition:width 0.5s ease;">
    <span style="display:inline-block; background:rgba(0,0,0,0.12); padding:4px 8px; border-radius:999px; font-size:0.95em;">100% · 26/26</span>
  </div>
</div>

**Target:** All 26 checklist tasks completed and validated

---

## Tasks

- [x] **task-010** (phase-01): Create CostPilot product structure
  - Create base directory layout and config skeleton for CostPilot.
  - automation_level: full
  - severity: critical
  - retry_policy: safe
  - notes: ''
  - dependencies: []
  - status: complete
  - result_hash: null
  steps:
  - id: step-010-1
    action: create_structure
    description: Create base folders (src/, tests/, .costpilot/, docs/).
    acceptance_criteria:
    - id: ac-010-1
      type: file_exists
      target: products/costpilot/src
  - id: step-010-2
    action: create_hidden_dirs
    description: Create .costpilot/ for local state and artifacts.
    acceptance_criteria:
    - id: ac-010-2
      type: file_exists
      target: .costpilot
- id: task-020
  phase: phase-01
  title: Install product.yml v2.1
  description: Place the CostPilot v2.1 product spec in the correct location and validate
    basic schema.
  automation_level: partial
  severity: critical
  retry_policy: safe
  notes: ''
  dependencies:
  - task-010
  status: completed
  result_hash: null
  steps:
  - id: step-020-1
    action: copy_spec
    description: Place product.yml under products/costpilot/.
    acceptance_criteria:
    - id: ac-020-1
      type: file_exists
      target: products/costpilot/product.yml
  - id: step-020-2
    action: validate_spec_schema
    description: Validate product.yml against spec schema.
    acceptance_criteria:
    - id: ac-020-2
      type: command
      command: nox -s validate_spec
      expected_exit_code: 0
- id: task-030
  phase: phase-01
  title: Implement spec loader
  description: Implement loader that maps product.yml into internal runtime config
    structures.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-020
  status: pending
  result_hash: null
  steps:
  - id: step-030-1
    action: implement_loader
    description: Implement config loader for metadata, platform and x_capabilities.
    acceptance_criteria:
    - id: ac-030-1
      type: unit_test_pass
      test_id: test_spec_loader_roundtrip
  - id: step-030-2
    action: expose_zero_cost_policy
    description: Ensure zero_cost_policy fields are exposed to runtime.
    acceptance_criteria:
    - id: ac-030-2
      type: unit_test_pass
      test_id: test_zero_cost_policy_loaded

- [x] **task-020** (phase-01): Install product.yml v2.1
  - Place the CostPilot v2.1 product spec in the correct location and validate basic schema.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-010]
  - status: complete

- [x] **task-030** (phase-01): Implement spec loader
  - Implement loader that maps product.yml into internal runtime config structures.
  - automation_level: partial
  - severity: high
  - dependencies: [task-020]
  - status: complete

- [x] **task-040** (phase-01): Bootstrap .costpilot config files
  - Create initial .costpilot config files for policies, SLOs, baselines and trend.
  - automation_level: partial
  - severity: medium
  - dependencies: [task-010]
  - status: complete

- [x] **task-050** (phase-02): Implement Terraform plan parser
  - Implement parser for terraform_plan_json into an internal resource graph.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-030]
  - status: complete
  steps:
  - id: step-040-1
    action: create_policies_file
    description: Create .costpilot/policies.yml stub.
    acceptance_criteria:
    - id: ac-040-1
      type: file_exists
      target: .costpilot/policies.yml
  - id: step-040-2
    action: create_slo_file
    description: Create .costpilot/slo.yml stub.
    acceptance_criteria:
    - id: ac-040-2
      type: file_exists
      target: .costpilot/slo.yml
  - id: step-040-3
    action: create_baselines_file
    description: Create .costpilot/baselines.json stub.
    acceptance_criteria:
    - id: ac-040-3
      type: file_exists
      target: .costpilot/baselines.json
- id: task-050
  phase: phase-02
  title: Implement Terraform plan parser
  description: Implement parser for terraform_plan_json into an internal resource
    graph.
  automation_level: partial
  severity: critical
  retry_policy: safe
  notes: ''
  dependencies:
  - task-030
  status: pending
  result_hash: null
  steps:
  - id: step-050-1
    action: implement_parser
    description: Parse plan JSON into normalized resource structures.
    acceptance_criteria:
    - id: ac-050-1
      type: unit_test_pass
      test_id: test_parse_basic_terraform_plan
  - id: step-050-2
    action: handle_invalid_plans
    description: Handle malformed plans gracefully.
    acceptance_criteria:
    - id: ac-050-2
      type: unit_test_pass
      test_id: test_parse_invalid_terraform_plan_graceful

- [x] **task-060** (phase-02): Implement detect engine with builtin rules
  - Implement detection engine using x_capabilities.detect builtin rules.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-050]
  - status: complete

- [x] **task-070** (phase-02): Implement prediction engine
  - Implement deterministic cost prediction using cost_heuristics.json and cold-start defaults.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-050]
  - status: complete

- [x] **task-080** (phase-02): Implement regression classifier
  - Classify cost events into configuration, provisioning, scaling, traffic_inferred and indirect_cost.
  - automation_level: partial
  - severity: high
  - dependencies: [task-070]
  - status: complete

- [x] **task-090** (phase-02): Implement explain engine
  - Implement explain output referencing heuristics, baselines and assumptions.
  - automation_level: partial
  - severity: high
  - dependencies: [task-060, task-070, task-080]
  - status: complete

- [x] **task-100** (phase-03): Implement zero-cost guard module
  - Implement guard module enforcing zero_cost_policy invariants across CLI and engines.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-030]
  - status: complete

- [x] **task-110** (phase-03): Enforce offline, no-network execution
  - Ensure no runtime path can perform network egress or real cloud calls.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-100]
  - status: complete

- [x] **task-120** (phase-03): Implement deterministic pipeline ordering
  - Ensure evaluation order and aggregation are deterministic and canonical.
  - automation_level: partial
  - severity: high
  - dependencies: [task-060, task-090]
  - status: complete

- [x] **task-130** (phase-03): Integrate WASM runtime and limits
  - Configure WASM runtime with max_memory_mb and timeout_ms_per_scan and validate.
  - automation_level: partial
  - severity: high
  - dependencies: [task-120]
  - status: complete

- [x] **task-140** (phase-04): Implement policy-as-code engine
  - Implement policy-as-code engine loading policies from .costpilot/policies.yml.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-040, task-070]
  - status: complete

- [x] **task-150** (phase-04): Implement exemption workflow
  - Implement exemption workflow with issue reference validation and expiry.
  - automation_level: partial
  - severity: high
  - dependencies: [task-140]
  - status: complete

- [x] **task-160** (phase-04): Implement SLO evaluation engine
  - Implement SLO evaluation using burn rate prediction from slo.yml.
  - automation_level: partial
  - severity: high
  - dependencies: [task-040, task-070]
  - status: complete

- [x] **task-170** (phase-04): Implement SLO burn prediction
  - Implement burn rate forecasting and alert thresholds.
  - automation_level: partial
  - severity: medium
  - dependencies: [task-160]
  - status: complete

- [x] **task-180** (phase-05): Implement mapping engine
  - Implement dependency graph construction and cost propagation.
  - automation_level: partial
  - severity: high
  - dependencies: [task-050, task-070]
  - status: complete

- [x] **task-190** (phase-05): Implement baselines engine
  - Load baselines from baselines.json and stabilize predictions.
  - automation_level: partial
  - severity: medium
  - dependencies: [task-040, task-070]
  - status: complete

- [x] **task-200** (phase-05): Implement trend engine
  - Track cost evolution and detect anomalies over time.
  - automation_level: partial
  - severity: medium
  - dependencies: [task-070, task-190]
  - status: complete

- [x] **task-210** (phase-05): Implement attribution engine
  - Group costs by tags and compute tag-based budgets.
  - automation_level: partial
  - severity: medium
  - dependencies: [task-070]
  - status: complete

- [x] **task-220** (phase-06): Write unit tests for all engines
  - Write comprehensive unit tests achieving >85% coverage.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-060, task-070, task-080, task-090, task-140, task-160, task-180, task-190, task-200, task-210]
  - status: complete

- [x] **task-230** (phase-06): Write integration tests
  - Write end-to-end integration tests for full pipeline workflows.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-220]
  - status: complete

- [x] **task-240** (phase-06): Write snapshot tests
  - Write snapshot tests for deterministic output validation.
  - automation_level: partial
  - severity: high
  - dependencies: [task-120, task-220]
  - status: complete

- [x] **task-250** (phase-06): Configure CI pipeline
  - Configure .github/workflows/costpilot.yml for required jobs.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-230, task-240]
  - status: complete

- [x] **task-260** (phase-06): Run full suite and verify invariants
  - Run full suite and confirm zero-cost, determinism and policy invariants.
  - automation_level: partial
  - severity: critical
  - dependencies: [task-100, task-120, task-130, task-180, task-200]
  - status: complete

---

---

## Detailed Task Breakdown

### task-010
  steps:
  - id: step-010-1
    action: create_structure
    description: Create base folders (src/, tests/, .costpilot/, docs/).
    acceptance_criteria:
    - id: ac-010-1
      type: file_exists
      target: products/costpilot/src
  - id: step-010-2
    action: create_hidden_dirs
    description: Create .costpilot/ for local state and artifacts.
    acceptance_criteria:
    - id: ac-010-2
      type: file_exists
      target: .costpilot

### task-020
  steps:
  - id: step-020-1
    action: copy_spec
    description: Place product.yml under products/costpilot/.
    acceptance_criteria:
    - id: ac-020-1
      type: file_exists
      target: products/costpilot/product.yml
  - id: step-020-2
    action: validate_spec_schema
    description: Validate product.yml against spec schema.
    acceptance_criteria:
    - id: ac-020-2
      type: command
      command: nox -s validate_spec
      expected_exit_code: 0

### task-030
  steps:
  - id: step-030-1
    action: implement_loader
    description: Implement config loader for metadata, platform and x_capabilities.
    acceptance_criteria:
    - id: ac-030-1
      type: unit_test_pass
      test_id: test_spec_loader_roundtrip
  - id: step-030-2
    action: expose_zero_cost_policy
    description: Ensure zero_cost_policy fields are exposed to runtime.
    acceptance_criteria:
    - id: ac-030-2
      type: unit_test_pass
      test_id: test_zero_cost_policy_loaded

### task-040
  steps:
  - id: step-040-1
    action: create_policies_file
    description: Create .costpilot/policies.yml stub.
    acceptance_criteria:
    - id: ac-040-1
      type: file_exists
      target: .costpilot/policies.yml
  - id: step-040-2
    action: create_slo_file
    description: Create .costpilot/slo.yml stub.
    acceptance_criteria:
    - id: ac-040-2
      type: file_exists
      target: .costpilot/slo.yml
  - id: step-040-3
    action: create_baselines_file
    description: Create .costpilot/baselines.json stub.
    acceptance_criteria:
    - id: ac-040-3
      type: file_exists
      target: .costpilot/baselines.json

### task-050
  steps:
  - id: step-050-1
    action: implement_parser
    description: Parse plan JSON into normalized resource structures.
    acceptance_criteria:
    - id: ac-050-1
      type: unit_test_pass
      test_id: test_parse_basic_terraform_plan
  - id: step-050-2
    action: handle_invalid_plans
    description: Handle malformed plans gracefully.
    acceptance_criteria:
    - id: ac-050-2
      type: unit_test_pass
      test_id: test_parse_invalid_terraform_plan_graceful

### task-060
  steps:
### task-060
  steps:
  - id: step-060-1
    action: implement_builtin_rules
    description: Implement nat_gateway, s3_missing_lifecycle, compute_overprovision rules.
    acceptance_criteria:
    - id: ac-060-1
      type: unit_test_pass
      test_id: test_builtin_detection_rules
  - id: step-060-2
    action: compute_severity_score
    description: Compute deterministic severity_score per finding.
    acceptance_criteria:
    - id: ac-060-2
      type: unit_test_pass
      test_id: test_severity_scoring_deterministic

### task-070
  steps:
  - id: step-070-1
    action: load_heuristics_file
    description: Load and validate cost_heuristics.json.
    acceptance_criteria:
    - id: ac-070-1
      type: unit_test_pass
      test_id: test_load_cost_heuristics
  - id: step-070-2
    action: apply_cold_start_defaults
    description: Apply cold-start defaults for missing usage.
    acceptance_criteria:
    - id: ac-070-2
      type: unit_test_pass
      test_id: test_cold_start_defaults_applied
  - id: step-070-3
    action: compute_prediction_interval
    description: Compute prediction intervals using range_factor.
    acceptance_criteria:
    - id: ac-070-3
      type: unit_test_pass
      test_id: test_prediction_interval_bounds

### task-080
  steps:
  - id: step-080-1
    action: map_regression_types
    description: Map events to classifier types deterministically.
    acceptance_criteria:
    - id: ac-080-1
      type: unit_test_pass
      test_id: test_regression_classifier_mapping
  - id: step-080-2
    action: snapshot_classifier_output
    description: Ensure classifier output is deterministic via snapshot.
    acceptance_criteria:
    - id: ac-080-2
      type: snapshot_hash
      file: tests/snapshots/classifier_output.json

### task-090
  steps:
  - id: step-090-1
    action: generate_markdown_explain
    description: Generate markdown_explanation for top N high-cost patterns.
    acceptance_criteria:
    - id: ac-090-1
      type: unit_test_pass
      test_id: test_explain_markdown_contains_references
  - id: step-090-2
    action: reference_heuristics_version
    description: Ensure explain output references heuristics version and assumptions.
    acceptance_criteria:
    - id: ac-090-2
      type: unit_test_pass
      test_id: test_explain_references_heuristics_version

### task-100
  steps:
  - id: step-100-1
    action: implement_guard_module
    description: Implement central guard enforcing no terraform apply/plan or cloud SDK calls.
    acceptance_criteria:
    - id: ac-100-1
      type: code_search_absent
      target: products/costpilot/src
      pattern: terraform apply
  - id: step-100-2
    action: wire_cli_to_guard
    description: Ensure all CLI commands call zero-cost guard before work.
    acceptance_criteria:
    - id: ac-100-2
      type: unit_test_pass
      test_id: test_zero_cost_enforced_for_all_commands

### task-110
  steps:
  - id: step-110-1
    action: scan_for_network_calls
    description: Scan for HTTP client usage or SDK calls and remove/guard.
    acceptance_criteria:
    - id: ac-110-1
      type: code_search_absent
      target: products/costpilot/src
      pattern: http

### task-120
  steps:
  - id: step-120-1
    action: fix_iteration_order
    description: Enforce stable ordering for collections and outputs.
    acceptance_criteria:
    - id: ac-120-1
      type: snapshot_hash
      file: tests/snapshots/full_scan_output.json
  - id: step-120-2
    action: canonical_json_output
    description: Use canonical JSON serialization with stable keys.
    acceptance_criteria:
    - id: ac-120-2
      type: unit_test_pass
      test_id: test_canonical_serialization

### task-130
  steps:
  - id: step-130-1
    action: configure_wasm_runtime
    description: Configure WASM runtime limits (max_memory_mb, timeout_ms_per_scan).
    acceptance_criteria:
    - id: ac-130-1
      type: unit_test_pass
      test_id: test_wasm_config_enforcement
  - id: step-130-2
    action: test_wasm_build
    description: Validate WASM build produces valid .wasm artifact.
    acceptance_criteria:
    - id: ac-130-2
      type: command
      command: cargo build --target wasm32-wasi
      expected_exit_code: 0

### task-140
  steps:
  - id: step-140-1
    action: load_policies_file
    description: Load .costpilot/policies.yml and parse enforcement rules.
    acceptance_criteria:
    - id: ac-140-1
      type: unit_test_pass
      test_id: test_load_policies_yml
  - id: step-140-2
    action: evaluate_policy_rules
    description: Evaluate rules against predictions and emit violations.
    acceptance_criteria:
    - id: ac-140-2
      type: unit_test_pass
      test_id: test_policy_evaluation

### task-150
  steps:
  - id: step-150-1
    action: parse_exemption_entries
    description: Parse exemption entries with issue reference validation.
    acceptance_criteria:
    - id: ac-150-1
      type: unit_test_pass
      test_id: test_exemption_parsing
  - id: step-150-2
    action: apply_exemptions_to_scan
    description: Apply exemptions and skip matching violations.
    acceptance_criteria:
    - id: ac-150-2
      type: unit_test_pass
      test_id: test_exemption_application

### task-160
  steps:
  - id: step-160-1
    action: load_slo_file
    description: Load .costpilot/slo.yml and compute SLO compliance.
    acceptance_criteria:
    - id: ac-160-1
      type: unit_test_pass
      test_id: test_load_slo_yml
  - id: step-160-2
    action: compute_slo_compliance
    description: Compute SLO compliance and emit status.
    acceptance_criteria:
    - id: ac-160-2
      type: unit_test_pass
      test_id: test_slo_compliance

### task-170
  steps:
  - id: step-170-1
    action: implement_burn_rate_forecast
    description: Implement burn rate forecasting based on historical data.
    acceptance_criteria:
    - id: ac-170-1
      type: unit_test_pass
      test_id: test_burn_rate_forecast
  - id: step-170-2
    action: emit_slo_alerts
    description: Emit SLO alert notifications when thresholds exceeded.
    acceptance_criteria:
    - id: ac-170-2
      type: unit_test_pass
      test_id: test_slo_alerts

### task-180
  steps:
  - id: step-180-1
    action: construct_dependency_graph
    description: Build internal dependency graph from IaC resources.
    acceptance_criteria:
    - id: ac-180-1
      type: unit_test_pass
      test_id: test_mapping_graph_construction
  - id: step-180-2
    action: export_mermaid_and_graphviz
    description: Export mermaid_diagram.md and graphviz.dot.
    acceptance_criteria:
    - id: ac-180-2
      type: file_exists
      target: .costpilot/mermaid_diagram.md

### task-190
  steps:
  - id: step-190-1
    action: load_baselines_file
    description: Load .costpilot/baselines.json and index by resource_id.
    acceptance_criteria:
    - id: ac-190-1
      type: unit_test_pass
      test_id: test_baselines_load_and_map
  - id: step-190-2
    action: apply_baselines_to_predictions
    description: Override small deltas based on baselines.
    acceptance_criteria:
    - id: ac-190-2
      type: unit_test_pass
      test_id: test_prediction_with_baselines

### task-200
  steps:
  - id: step-200-1
    action: append_trend_entries
    description: Append trend entries with UTC timestamps.
    acceptance_criteria:
    - id: ac-200-1
      type: unit_test_pass
      test_id: test_trend_append_only
  - id: step-200-2
    action: render_trend_reports
    description: Generate trend_report.html and trend_graph.svg.
    acceptance_criteria:
    - id: ac-200-2
      type: file_exists
      target: .costpilot/trend_report.html

### task-210
  steps:
  - id: step-210-1
    action: resolve_tags_to_groups
    description: Resolve tags into module/service/environment groupings.
    acceptance_criteria:
    - id: ac-210-1
      type: unit_test_pass
      test_id: test_tag_resolution
  - id: step-210-2
    action: emit_attribution_outputs
    description: Emit free/pro/enterprise attribution outputs.
    acceptance_criteria:
    - id: ac-210-2
      type: unit_test_pass
      test_id: test_attribution_outputs

### task-220
  steps:
  phase: phase-03
  title: Integrate WASM runtime and limits
  description: Configure WASM runtime with max_memory_mb and timeout_ms_per_scan and
    validate.
  automation_level: partial
  severity: high
  retry_policy: manual
  notes: ''
  dependencies:
  - task-120
  status: pending
  result_hash: null
  steps:
  - id: step-130-1
    action: configure_wasm_limits
    description: Configure memory/time limits per spec.
    acceptance_criteria:
    - id: ac-130-1
      type: unit_test_pass
      test_id: test_wasm_limits_configured
  - id: step-130-2
    action: run_stress_test
    description: Run load tests to verify limits enforced.
    acceptance_criteria:
    - id: ac-130-2
      type: perf_test_pass
      test_id: test_wasm_limits_under_load
- id: task-140
  phase: phase-04
  title: Implement policy loader and schema validation
  description: Implement loader for .costpilot/policies.yml with schema validation.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-040
  status: pending
  result_hash: null
  steps:
  - id: step-140-1
    action: validate_policy_schema
    description: Validate policy file against schema.
    acceptance_criteria:
    - id: ac-140-1
      type: unit_test_pass
      test_id: test_policy_schema_validation
- id: task-150
  phase: phase-04
  title: Implement policy enforcement engine
  description: Evaluate policies in warn/block modes and emit enforcement reports.
  automation_level: partial
  severity: critical
  retry_policy: manual
  notes: ''
  dependencies:
  - task-140
  status: pending
  result_hash: null
  steps:
  - id: step-150-1
    action: implement_warn_mode
    description: Implement warn mode behavior for violations.
    acceptance_criteria:
    - id: ac-150-1
      type: unit_test_pass
      test_id: test_policy_warn_mode
  - id: step-150-2
    action: implement_block_mode
    description: Block CI when blocking policies fail.
    acceptance_criteria:
    - id: ac-150-2
      type: unit_test_pass
      test_id: test_policy_block_mode
- id: task-160
  phase: phase-04
  title: Implement exemptions workflow
  description: Implement exemptions in .costpilot/exemptions.yml with expiry enforcement.
  automation_level: partial
  severity: high
  retry_policy: manual
  notes: ''
  dependencies:
  - task-150
  status: pending
  result_hash: null
  steps:
  - id: step-160-1
    action: load_exemptions_file
    description: Load exemptions and associate with policy rules.
    acceptance_criteria:
    - id: ac-160-1
      type: unit_test_pass
      test_id: test_exemptions_load_and_match
  - id: step-160-2
    action: enforce_expired_exemptions
    description: Ensure expired exemptions fail CI.
    acceptance_criteria:
    - id: ac-160-2
      type: unit_test_pass
      test_id: test_exemptions_expiry_enforced
- id: task-170
  phase: phase-04
  title: Implement SLO evaluation engine
  description: Implement evaluation of SLOs defined in .costpilot/slo.yml using prediction
    outputs.
  automation_level: partial
  severity: critical
  retry_policy: manual
  notes: ''
  dependencies:
  - task-070
  status: pending
  result_hash: null
  steps:
  - id: step-170-1
    action: validate_slo_config
    description: Load and validate SLO config schema.
    acceptance_criteria:
    - id: ac-170-1
      type: unit_test_pass
      test_id: test_slo_config_validation
  - id: step-170-2
    action: evaluate_slo_rules
    description: Evaluate monthly_cost_slo and composed SLOs.
    acceptance_criteria:
    - id: ac-170-2
      type: unit_test_pass
      test_id: test_slo_evaluation_deterministic
- id: task-180
  phase: phase-04
  title: Implement SLO burn prediction
  description: Compute burn_risk and projected_cost_after_merge based on predicted
    deltas.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-170
  status: pending
  result_hash: null
  steps:
  - id: step-180-1
    action: compute_burn_risk_levels
    description: Compute burn risk levels (low, medium, high, critical).
    acceptance_criteria:
    - id: ac-180-1
      type: unit_test_pass
      test_id: test_slo_burn_risk_levels
- id: task-190
  phase: phase-05
  title: Implement mapping engine
  description: Implement mapping engine with dependency_graph.json and mermaid/graphviz
    outputs.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-050
  status: pending
  result_hash: null
  steps:
  - id: step-190-1
    action: construct_dependency_graph
    description: Build internal dependency graph from IaC resources.
    acceptance_criteria:
    - id: ac-190-1
      type: unit_test_pass
      test_id: test_mapping_graph_construction
  - id: step-190-2
    action: export_mermaid_and_graphviz
    description: Export mermaid_diagram.md and graphviz.dot.
    acceptance_criteria:
    - id: ac-190-2
      type: file_exists
      target: .costpilot/mermaid_diagram.md
- id: task-200
  phase: phase-05
  title: Implement baselines engine
  description: Implement baselines loader and integration with prediction engine.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-070
  status: pending
  result_hash: null
  steps:
  - id: step-200-1
    action: load_baselines_file
    description: Load .costpilot/baselines.json and index by resource_id.
    acceptance_criteria:
    - id: ac-200-1
      type: unit_test_pass
      test_id: test_baselines_load_and_map
  - id: step-200-2
    action: apply_baselines_to_predictions
    description: Override small deltas based on baselines.
    acceptance_criteria:
    - id: ac-200-2
      type: unit_test_pass
      test_id: test_prediction_with_baselines
- id: task-210
  phase: phase-05
  title: Implement trend engine
  description: Implement trend.json append-only log and HTML/SVG report generation.
  automation_level: partial
  severity: medium
  retry_policy: safe
  notes: ''
  dependencies:
  - task-200
  status: pending
  result_hash: null
  steps:
  - id: step-210-1
    action: append_trend_entries
    description: Append trend entries with UTC timestamps.
    acceptance_criteria:
    - id: ac-210-1
      type: unit_test_pass
      test_id: test_trend_append_only
  - id: step-210-2
    action: render_trend_reports
    description: Generate trend_report.html and trend_graph.svg.
    acceptance_criteria:
    - id: ac-210-2
      type: file_exists
      target: .costpilot/trend_report.html
- id: task-220
  phase: phase-05
  title: Implement attribution engine
  description: Implement tag-based attribution and grouping outputs per tier.
  automation_level: partial
  severity: medium
  retry_policy: safe
  notes: ''
  dependencies:
  - task-070
  status: pending
  result_hash: null
  steps:
  - id: step-220-1
    action: resolve_tags_to_groups
    description: Resolve tags into module/service/environment groupings.
    acceptance_criteria:
    - id: ac-220-1
      type: unit_test_pass
      test_id: test_tag_resolution
  - id: step-220-2
    action: emit_attribution_outputs
    description: Emit free/pro/enterprise attribution outputs.
    acceptance_criteria:
    - id: ac-220-2
      type: unit_test_pass
      test_id: test_attribution_outputs
### task-220
  steps:
  - id: step-220-1
    action: write_unit_tests_detect
    description: Write unit tests for detect engine.
    acceptance_criteria:
    - id: ac-220-1
      type: coverage_threshold
      target: products/costpilot/src/detect_engine.rs
      threshold: 85
  - id: step-220-2
    action: write_unit_tests_predict
    description: Write unit tests for predict engine.
    acceptance_criteria:
    - id: ac-220-2
      type: coverage_threshold
      target: products/costpilot/src/predict_engine.rs
      threshold: 85
  - id: step-220-3
    action: write_unit_tests_policy
    description: Write unit tests for policy engine.
    acceptance_criteria:
    - id: ac-220-3
      type: coverage_threshold
      target: products/costpilot/src/policy_engine.rs
      threshold: 85

### task-230
  steps:
  - id: step-230-1
    action: write_integration_tests
    description: Write end-to-end integration tests for full pipeline.
    acceptance_criteria:
    - id: ac-230-1
      type: unit_test_pass
      test_id: test_full_pipeline_integration
  - id: step-230-2
    action: test_cli_exit_codes
    description: Test all CLI commands with correct exit codes.
    acceptance_criteria:
    - id: ac-230-2
      type: unit_test_pass
      test_id: test_cli_exit_codes

### task-240
  steps:
  - id: step-240-1
    action: create_snapshot_tests
    description: Create snapshot tests for deterministic outputs.
    acceptance_criteria:
    - id: ac-240-1
      type: snapshot_hash
      file: tests/snapshots/full_scan_output.json
  - id: step-240-2
    action: verify_deterministic_output
    description: Verify identical output across multiple runs.
    acceptance_criteria:
    - id: ac-240-2
      type: unit_test_pass
      test_id: test_deterministic_output

### task-250
  steps:
  - id: step-250-1
    action: create_ci_workflow_file
    description: Define validate_schema, unit, integration, snapshot, wasm, self_test, verify, perf_regression jobs.
    acceptance_criteria:
    - id: ac-250-1
      type: file_exists
      target: .github/workflows/costpilot.yml

### task-260
  steps:
  - id: step-260-1
    action: run_full_test_suite
    description: Run full test/CI suite locally.
    acceptance_criteria:
    - id: ac-260-1
      type: command
      command: nox -s full
      expected_exit_code: 0
  - id: step-260-2
    action: manual_invariant_review
    description: Manual review of invariants checklist (zero-cost, determinism, governance).
    acceptance_criteria:
    - id: ac-260-2
      type: manual_review
      reviewer: Deon Prinsloo
  description: Implement core unit tests for detect, predict, explain, policy and
    SLO engines.
  automation_level: partial
  severity: critical
  retry_policy: safe
  notes: ''
  dependencies:
  - task-060
  - task-070
  - task-090
  - task-150
  - task-170
  status: pending
  result_hash: null
  steps:
  - id: step-230-1
    action: create_core_unit_tests
    description: Create focused tests for key engine behaviors.
    acceptance_criteria:
    - id: ac-230-1
      type: coverage
      minimum_percent: 80
- id: task-240
  phase: phase-06
  title: Implement snapshot determinism tests
  description: Implement snapshot tests to ensure deterministic full scan outputs.
  automation_level: partial
  severity: high
  retry_policy: safe
  notes: ''
  dependencies:
  - task-120
  status: pending
  result_hash: null
  steps:
  - id: step-240-1
    action: create_full_scan_snapshots
    description: Create representative plan snapshots and hash them.
    acceptance_criteria:
    - id: ac-240-1
      type: snapshot_hash
      file: tests/snapshots/full_scan_output.json
- id: task-250
  phase: phase-06
  title: Configure CI workflow
  description: Configure .github/workflows/costpilot.yml for required jobs.
  automation_level: partial
  severity: critical
  retry_policy: safe
  notes: ''
  dependencies:
  - task-230
  - task-240
  status: pending
  result_hash: null
  steps:
  - id: step-250-1
    action: create_ci_workflow_file
    description: Define validate_schema, unit, integration, snapshot, wasm, self_test,
      verify, perf_regression jobs.
    acceptance_criteria:
    - id: ac-250-1
      type: file_exists
      target: .github/workflows/costpilot.yml
- id: task-260
  phase: phase-06
  title: Run full suite and verify invariants
  description: Run full suite and confirm zero-cost, determinism and policy invariants.
  automation_level: partial
  severity: critical
  retry_policy: manual
  notes: ''
  dependencies:
  - task-100
  - task-120
  - task-130
  - task-180
  - task-200
  status: completed
  result_hash: null
  steps:
  - id: step-260-1
    action: run_full_test_suite
    description: Run full test/CI suite locally.
    acceptance_criteria:
    - id: ac-260-1
      type: command
      command: nox -s full
      expected_exit_code: 0
  - id: step-260-2
    action: manual_invariant_review
    description: Manual review of invariants checklist (zero-cost, determinism, governance).
    acceptance_criteria:
    - id: ac-260-2
      type: manual_review
      reviewer: Deon Prinsloo
