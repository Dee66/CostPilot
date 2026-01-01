MODEL_STATE = mutable

CLAIM execution_scope = pull_request_boundary_only
VERIFY = human

CLAIM runtime_network_access = prohibited
VERIFY = mechanical
EXCEPT context = static_analysis_violation_detection

CLAIM telemetry = none
VERIFY = mechanical

CLAIM cloud_api_calls = none
VERIFY = mechanical

CLAIM determinism = strict
VERIFY = mechanical
EXCEPT module = license_issuer
EXCEPT module = escrow_release
EXCEPT module = metering_usage
EXCEPT module = audit_timestamping
EXCEPT module = deterministic_pseudorandom_with_fixed_seed

CLAIM blocking_behavior = advisory_by_default
VERIFY = human
EXCEPT condition = explicit_governance_block
EXCEPT condition = safety_or_integrity_violation

CLAIM licensing_validation = offline_only
VERIFY = mechanical

CLAIM wasm_execution = sandboxed
VERIFY = mechanical

CLAIM host_system_access_from_wasm = prohibited
VERIFY = mechanical

CLAIM mutable_external_state = prohibited
VERIFY = mechanical
