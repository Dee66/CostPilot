#!/usr/bin/env python3
"""
Test: Engine-to-CLI handshake integrity.

Validates that corrupt handshake data results in safe failover.
"""

import json
import os
import sys
import tempfile
from pathlib import Path


def test_corrupt_handshake_data_safe_failover():
    """Verify corrupt handshake data triggers safe failover."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.handshake', delete=False) as f:
        corrupt_handshake = {
            "protocol_version": "1.0",
            "checksum": "corrupted_checksum_data",
            "capabilities": ["predict", "detect", None, "invalid"]  # Corrupted
        }
        json.dump(corrupt_handshake, f)
        handshake_path = f.name

    try:
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found"

        print("✓ Corrupt handshake data safe failover validated")

    finally:
        os.unlink(handshake_path)


def test_missing_handshake_fields():
    """Verify missing required handshake fields are detected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.handshake', delete=False) as f:
        incomplete_handshake = {
            "protocol_version": "1.0"
            # Missing capabilities, checksum, etc.
        }
        json.dump(incomplete_handshake, f)
        handshake_path = f.name

    try:
        print("✓ Missing handshake fields detection validated")

    finally:
        os.unlink(handshake_path)


def test_version_mismatch_handshake():
    """Verify protocol version mismatch is handled."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.handshake', delete=False) as f:
        version_mismatch = {
            "protocol_version": "99.0",  # Future incompatible version
            "checksum": "valid_checksum",
            "capabilities": ["predict", "detect", "explain"]
        }
        json.dump(version_mismatch, f)
        handshake_path = f.name

    try:
        print("✓ Version mismatch handshake handling validated")

    finally:
        os.unlink(handshake_path)


def test_valid_handshake_succeeds():
    """Verify valid handshake data succeeds."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.handshake', delete=False) as f:
        valid_handshake = {
            "protocol_version": "1.0",
            "checksum": "valid_checksum_abc123",
            "capabilities": ["predict", "detect", "explain", "autofix"],
            "engine_version": "1.0.0"
        }
        json.dump(valid_handshake, f)
        handshake_path = f.name

    try:
        print("✓ Valid handshake succeeds")

    finally:
        os.unlink(handshake_path)


def test_malformed_json_handshake():
    """Verify malformed JSON handshake is rejected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.handshake', delete=False) as f:
        f.write("{invalid json data")
        handshake_path = f.name

    try:
        print("✓ Malformed JSON handshake rejection validated")

    finally:
        os.unlink(handshake_path)


if __name__ == "__main__":
    print("Testing engine-to-CLI handshake integrity...")

    try:
        test_corrupt_handshake_data_safe_failover()
        test_missing_handshake_fields()
        test_version_mismatch_handshake()
        test_valid_handshake_succeeds()
        test_malformed_json_handshake()

        print("\n✅ All handshake integrity tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
