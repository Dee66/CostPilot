#!/usr/bin/env python3
"""Test UX Differentiation: Free disabled features show deterministic error."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_autofix_error_deterministic():
    """Test autofix error is deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        exit_codes = []
        error_messages = []

        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            exit_codes.append(result.returncode)
            error_messages.append(result.stdout + result.stderr)

        # All runs should have same exit code
        assert len(set(exit_codes)) == 1, "Exit code should be deterministic"
        assert exit_codes[0] != 0, "Should fail in Free"

        # Error messages should be identical
        assert len(set(error_messages)) == 1, "Error message should be deterministic"


def test_patch_error_deterministic():
    """Test patch error is deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        exit_codes = []
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "patch", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            exit_codes.append(result.returncode)

        # Should be consistent
        assert len(set(exit_codes)) == 1, "Patch error should be deterministic"


def test_slo_error_deterministic():
    """Test SLO error is deterministic."""
    exit_codes = []
    for _ in range(3):
        result = subprocess.run(
            ["costpilot", "slo", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        exit_codes.append(result.returncode)

    # Should be consistent
    assert len(set(exit_codes)) == 1, "SLO error should be deterministic"


def test_disabled_feature_error_message():
    """Test disabled feature error is clear."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        assert result.returncode != 0, "Should fail in Free"

        output = (result.stdout + result.stderr).lower()

        # Error should be clear
        # Might say "command not found", "premium feature", etc.
        assert len(output) > 0, "Should have error message"


def test_premium_flag_error_deterministic():
    """Test premium flag error is deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        exit_codes = []
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--bundle", "premium.bundle"],
                capture_output=True,
                text=True,
                timeout=10
            )
            exit_codes.append(result.returncode)

        # Should be consistent
        assert len(set(exit_codes)) == 1, "Flag error should be deterministic"


def test_error_exit_codes_documented():
    """Test error exit codes follow convention."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        # Try various disabled features
        commands = [
            ["costpilot", "autofix", "--plan", str(template_path)],
            ["costpilot", "patch", "--plan", str(template_path)],
            ["costpilot", "slo", "check"]
        ]

        for cmd in commands:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=10
            )

            # Exit code should be non-zero and reasonable
            assert result.returncode != 0, "Disabled feature should fail"
            assert result.returncode < 256, "Exit code should be valid"


if __name__ == "__main__":
    test_autofix_error_deterministic()
    test_patch_error_deterministic()
    test_slo_error_deterministic()
    test_disabled_feature_error_message()
    test_premium_flag_error_deterministic()
    test_error_exit_codes_documented()
    print("All UX Differentiation: disabled feature error tests passed")
