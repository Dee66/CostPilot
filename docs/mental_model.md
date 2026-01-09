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
EXCEPT module = escrow
EXCEPT module = metering
EXCEPT module = audit_timestamping
EXCEPT module = monte_carlo
EXCEPT module = zero_network
EXCEPT module = crypto_tests

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

CLAIM supported_iac_formats = terraform
VERIFY = mechanical

CLAIM binary_artifacts = costpilot,license_issuer
VERIFY = mechanical

CLAIM command_surface_area = large
VERIFY = mechanical
NOTE = 53 command variants detected in main.rs

CLAIM test_license_infrastructure = real_ed25519_with_deterministic_keypair
VERIFY = mechanical

CLAIM test_license_issuer = "test-costpilot"
VERIFY = mechanical

CLAIM rate_limiting_integrity = hmac_sha256_tamper_detection
VERIFY = mechanical

CLAIM rate_limiting_home_isolation = respects_HOME_env_var
VERIFY = mechanical

CLAIM release_validation_tests = verify_distributed_binary_behavior
VERIFY = mechanical
NOTE = 53 command variants detected in main.rs
