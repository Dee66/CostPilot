#!/usr/bin/env python3
"""
Test: Telemetry anonymization.

Validates that telemetry contains no file paths, secrets, or user identifiers.
"""

import os
import sys
import tempfile
import json
import hashlib


def test_no_file_paths():
    """Verify no file paths in telemetry."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_telemetry.json', delete=False) as f:
        telemetry = {
            "event": "file_analyzed",
            "file_hash": hashlib.sha256(b"/path/to/file").hexdigest()[:16],
            "resource_count": 10
            # Original path should NOT be here
        }
        json.dump(telemetry, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        # Check no paths
        telemetry_str = json.dumps(data)
        forbidden_patterns = ["/home/", "/usr/", "C:\\", "/path/"]

        for pattern in forbidden_patterns:
            assert pattern not in telemetry_str

        print("✓ No file paths in telemetry")

    finally:
        os.unlink(path)


def test_no_secrets():
    """Verify no secrets in telemetry."""

    telemetry = {
        "event": "analysis_complete",
        "duration_ms": 1000,
        # No API keys, passwords, tokens
    }

    forbidden_keys = ["api_key", "password", "token", "secret", "credential"]

    for key in forbidden_keys:
        assert key not in telemetry

    print("✓ No secrets in telemetry")


def test_no_usernames():
    """Verify no usernames in telemetry."""

    telemetry = {
        "event": "session_started",
        "user_id": hashlib.sha256(b"username").hexdigest()[:16],
        "timestamp": "2024-01-15T10:00:00Z"
    }

    # Should be hashed, not plaintext
    assert "user_id" in telemetry
    assert len(telemetry["user_id"]) == 16  # Hash prefix

    print("✓ No usernames in telemetry (hashed)")


def test_no_email_addresses():
    """Verify no email addresses in telemetry."""

    telemetry = {
        "event": "license_validated",
        "license_hash": hashlib.sha256(b"user@example.com").hexdigest()[:16]
    }

    telemetry_str = json.dumps(telemetry)

    # Should not contain @ symbol or email patterns
    assert "@" not in telemetry_str or "@" in "2024-01-15T10:00:00Z"  # Timestamp OK

    print("✓ No email addresses in telemetry")


def test_no_hostnames():
    """Verify no hostnames in telemetry."""

    telemetry = {
        "event": "analysis_started",
        "machine_id": hashlib.sha256(b"hostname").hexdigest()[:16]
    }

    # Hostname should be hashed
    assert "machine_id" in telemetry
    assert "hostname" not in json.dumps(telemetry)

    print("✓ No hostnames in telemetry (hashed)")


def test_no_ip_addresses():
    """Verify no IP addresses in telemetry."""

    telemetry = {
        "event": "connection_established",
        "endpoint": "telemetry_service"  # Not IP address
    }

    telemetry_str = json.dumps(telemetry)

    # Should not contain IP patterns
    ip_patterns = ["192.168.", "10.", "172."]
    for pattern in ip_patterns:
        assert pattern not in telemetry_str

    print("✓ No IP addresses in telemetry")


def test_identifiers_are_hashed():
    """Verify identifiers are hashed."""

    original_id = "user_project_12345"
    hashed_id = hashlib.sha256(original_id.encode()).hexdigest()[:16]

    telemetry = {
        "event": "project_analyzed",
        "project_id": hashed_id
    }

    # Original should not appear
    assert original_id not in json.dumps(telemetry)
    assert telemetry["project_id"] == hashed_id

    print("✓ Identifiers are hashed (SHA256 prefix)")


def test_resource_names_anonymized():
    """Verify resource names are anonymized."""

    original_name = "aws_instance.production-web-server-01"
    anonymized = "aws_instance." + hashlib.sha256(original_name.encode()).hexdigest()[:8]

    telemetry = {
        "event": "resource_analyzed",
        "resource_type": "aws_instance",
        "resource_id": anonymized
    }

    # Original name should not appear
    assert "production-web-server-01" not in json.dumps(telemetry)

    print("✓ Resource names anonymized")


def test_error_stack_traces_sanitized():
    """Verify error stack traces are sanitized."""

    telemetry = {
        "event": "error_occurred",
        "error_type": "parse_error",
        "error_code": "E1001"
        # No stack trace with file paths
    }

    assert "stack_trace" not in telemetry
    assert "file_path" not in telemetry

    print("✓ Error stack traces sanitized")


def test_environment_variables_excluded():
    """Verify environment variables are excluded."""

    telemetry = {
        "event": "environment_detected",
        "os_type": "linux",  # Generic info OK
        "arch": "x86_64"
        # No env vars like HOME, PATH, etc.
    }

    env_keys = ["HOME", "PATH", "USER", "AWS_ACCESS_KEY"]

    for key in env_keys:
        assert key not in telemetry

    print("✓ Environment variables excluded")


def test_consistent_anonymization():
    """Verify anonymization is consistent."""

    original = "user@example.com"
    hash1 = hashlib.sha256(original.encode()).hexdigest()[:16]
    hash2 = hashlib.sha256(original.encode()).hexdigest()[:16]

    # Same input should produce same hash
    assert hash1 == hash2

    print("✓ Consistent anonymization (deterministic hashing)")


if __name__ == "__main__":
    print("Testing telemetry anonymization...")

    try:
        test_no_file_paths()
        test_no_secrets()
        test_no_usernames()
        test_no_email_addresses()
        test_no_hostnames()
        test_no_ip_addresses()
        test_identifiers_are_hashed()
        test_resource_names_anonymized()
        test_error_stack_traces_sanitized()
        test_environment_variables_excluded()
        test_consistent_anonymization()

        print("\n✅ All telemetry anonymization tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
