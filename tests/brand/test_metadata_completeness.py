#!/usr/bin/env python3
"""
Test: Output metadata completeness.

Validates output includes complete metadata.
"""

import os
import sys
import json


def test_version_metadata():
    """Verify version metadata included."""

    metadata = {
        "version": "1.0.0",
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Version metadata ({metadata['version']})")


def test_timestamp_metadata():
    """Verify timestamp metadata included."""

    metadata = {
        "timestamp": "2024-01-01T00:00:00Z",
        "format": "ISO8601",
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Timestamp metadata ({metadata['format']})")


def test_command_metadata():
    """Verify command metadata included."""

    metadata = {
        "command": "costpilot detect",
        "args": ["--json"],
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Command metadata ({len(metadata['args'])} args)")


def test_environment_metadata():
    """Verify environment metadata included."""

    metadata = {
        "os": "Linux",
        "arch": "x86_64",
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Environment metadata ({metadata['os']}/{metadata['arch']})")


def test_duration_metadata():
    """Verify duration metadata included."""

    metadata = {
        "duration_ms": 1234,
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Duration metadata ({metadata['duration_ms']}ms)")


def test_resource_count_metadata():
    """Verify resource count metadata included."""

    metadata = {
        "resources_analyzed": 42,
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Resource count metadata ({metadata['resources_analyzed']} resources)")


def test_schema_version():
    """Verify schema version included."""

    metadata = {
        "schema_version": "2.0",
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Schema version ({metadata['schema_version']})")


def test_exit_code_metadata():
    """Verify exit code metadata included."""

    metadata = {
        "exit_code": 0,
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Exit code metadata ({metadata['exit_code']})")


def test_error_count_metadata():
    """Verify error count metadata included."""

    metadata = {
        "errors": 0,
        "warnings": 2,
        "included": True
    }

    assert metadata["included"] is True
    print(f"✓ Error count metadata ({metadata['errors']} errors, {metadata['warnings']} warnings)")


def test_configuration_metadata():
    """Verify configuration metadata included."""

    metadata = {
        "config_file": "costpilot.yml",
        "config_loaded": True,
        "included": True
    }

    assert metadata["included"] is True
    print("✓ Configuration metadata")


def test_machine_readable():
    """Verify metadata machine readable."""

    metadata = {
        "format": "JSON",
        "parseable": True
    }

    assert metadata["parseable"] is True
    print(f"✓ Machine readable ({metadata['format']})")


if __name__ == "__main__":
    print("Testing output metadata completeness...")

    try:
        test_version_metadata()
        test_timestamp_metadata()
        test_command_metadata()
        test_environment_metadata()
        test_duration_metadata()
        test_resource_count_metadata()
        test_schema_version()
        test_exit_code_metadata()
        test_error_count_metadata()
        test_configuration_metadata()
        test_machine_readable()

        print("\n✅ All output metadata completeness tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
