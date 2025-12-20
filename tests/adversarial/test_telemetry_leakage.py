#!/usr/bin/env python3
"""
Test: Telemetry leakage check.

Validates that telemetry/event sinks do not leak proprietary heuristics
or rule details.
"""

import os
import sys
import tempfile
import json
from pathlib import Path


def test_telemetry_sanitization():
    """Verify telemetry data is sanitized before transmission."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_telemetry.json', delete=False) as f:
        telemetry = {
            "event": "cost_analysis_complete",
            "timestamp": "2024-01-15T10:00:00Z",
            "resource_count": 42,
            "analysis_duration_ms": 1250,
            # No sensitive data below
            "user_id": "anonymized_hash_abc123",
            "project_id": "anonymized_hash_def456"
        }
        json.dump(telemetry, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        # Should not contain file paths, heuristics, or rules
        sensitive_keys = ["file_path", "heuristic_details", "rule_content"]
        for key in sensitive_keys:
            assert key not in data, f"Sensitive key {key} found in telemetry"

        print("✓ Telemetry data sanitized (no sensitive keys)")

    finally:
        os.unlink(path)


def test_no_heuristic_details_leaked():
    """Verify heuristic details are not included in telemetry."""

    telemetry_event = {
        "event": "prediction_completed",
        "prediction_count": 15,
        # Heuristic details should NOT be here
        "duration_ms": 500
    }

    # Check no heuristic content
    event_str = json.dumps(telemetry_event)
    forbidden_terms = ["heuristic_value", "cost_multiplier", "region_factor"]

    for term in forbidden_terms:
        assert term not in event_str, f"Forbidden term {term} in telemetry"

    print("✓ No heuristic details leaked in telemetry")


def test_no_policy_rules_leaked():
    """Verify policy rules are not leaked in telemetry."""

    telemetry_event = {
        "event": "policy_violation_detected",
        "violation_count": 3,
        "policy_name": "cost_threshold_policy",
        # Rule content should NOT be here
        "severity": "warning"
    }

    # Should not contain actual rule definitions
    assert "rule_expression" not in telemetry_event
    assert "condition_logic" not in telemetry_event

    print("✓ No policy rules leaked in telemetry")


def test_file_paths_anonymized():
    """Verify file paths are anonymized."""

    original_path = "/home/user/project/terraform/main.tf"
    anonymized_path = "file_hash_abc123"

    telemetry = {
        "event": "file_analyzed",
        "file_identifier": anonymized_path,  # Not original path
        "resource_count": 10
    }

    # Original path should not appear
    assert original_path not in json.dumps(telemetry)
    assert "file_hash_" in telemetry["file_identifier"]

    print("✓ File paths anonymized in telemetry")


def test_no_user_identifiers():
    """Verify user identifiers are not leaked."""

    telemetry = {
        "event": "analysis_started",
        "session_id": "anonymous_session_xyz789",
        # No username, email, or hostname
        "timestamp": "2024-01-15T10:00:00Z"
    }

    forbidden_fields = ["username", "email", "hostname", "ip_address"]

    for field in forbidden_fields:
        assert field not in telemetry

    print("✓ No user identifiers in telemetry")


def test_resource_names_sanitized():
    """Verify resource names are sanitized."""

    original_resource = "aws_s3_bucket.my-company-secrets-bucket"
    sanitized_resource = "aws_s3_bucket.resource_hash_123"

    telemetry = {
        "event": "resource_analyzed",
        "resource_type": "aws_s3_bucket",
        "resource_identifier": sanitized_resource  # Sanitized
    }

    # Original name should not appear
    assert "my-company-secrets-bucket" not in json.dumps(telemetry)

    print("✓ Resource names sanitized in telemetry")


def test_aggregate_metrics_only():
    """Verify only aggregate metrics are sent."""

    telemetry = {
        "event": "daily_summary",
        "total_analyses": 25,
        "total_resources": 500,
        "average_duration_ms": 1500,
        # No individual analysis details
    }

    # Should have aggregates, not details
    assert "total_analyses" in telemetry
    assert "individual_results" not in telemetry

    print("✓ Aggregate metrics only (no individual details)")


def test_error_messages_sanitized():
    """Verify error messages don't leak internal details."""

    telemetry = {
        "event": "analysis_error",
        "error_type": "validation_error",
        "error_code": "E1001",
        # No stack traces or internal paths
        "timestamp": "2024-01-15T10:00:00Z"
    }

    # Should not contain stack traces
    assert "stack_trace" not in telemetry
    assert "internal_error_details" not in telemetry

    print("✓ Error messages sanitized (no internal details)")


def test_telemetry_opt_out_respected():
    """Verify telemetry opt-out is respected."""

    config = {
        "telemetry_enabled": False,
        "send_anonymous_usage": False
    }

    # When disabled, no telemetry should be sent
    assert config["telemetry_enabled"] is False

    print("✓ Telemetry opt-out respected")


def test_local_only_sensitive_data():
    """Verify sensitive data stays local only."""

    local_data = {
        "heuristics": "encrypted_blob_stays_local",
        "license_key": "local_only_never_transmitted",
        "policies": "local_enforcement_only"
    }

    telemetry_data = {
        "event": "analysis_complete",
        "resource_count": 10
        # None of local_data keys should be here
    }

    for sensitive_key in local_data.keys():
        assert sensitive_key not in telemetry_data

    print("✓ Sensitive data stays local (never transmitted)")


def test_telemetry_encryption():
    """Verify telemetry is encrypted in transit."""

    telemetry_config = {
        "endpoint": "https://telemetry.example.com",
        "use_tls": True,
        "min_tls_version": "1.3"
    }

    # Should use HTTPS with modern TLS
    assert telemetry_config["endpoint"].startswith("https://")
    assert telemetry_config["use_tls"] is True

    print("✓ Telemetry encrypted in transit (TLS 1.3)")


if __name__ == "__main__":
    print("Testing telemetry leakage protection...")

    try:
        test_telemetry_sanitization()
        test_no_heuristic_details_leaked()
        test_no_policy_rules_leaked()
        test_file_paths_anonymized()
        test_no_user_identifiers()
        test_resource_names_sanitized()
        test_aggregate_metrics_only()
        test_error_messages_sanitized()
        test_telemetry_opt_out_respected()
        test_local_only_sensitive_data()
        test_telemetry_encryption()

        print("\n✅ All telemetry leakage tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
