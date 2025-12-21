#!/usr/bin/env python3
"""Test Free Edition: patch command not present."""

import subprocess
import tempfile
from pathlib import Path
import json
import os

COSTPILOT_PATH = os.path.join(os.path.dirname(__file__), '..', '..', 'target', 'debug', 'costpilot')


def test_patch_command_not_present():
    """Test patch command not available in Free Edition."""
    result = subprocess.run(
        [COSTPILOT_PATH, "autofix-patch", "--help"],
        capture_output=True,
        text=True,
        timeout=10,
        check=False
    )

    # Should fail - patch not available in Free
    assert result.returncode != 0, "patch command should not exist in Free Edition"

    # Check error message
    error = result.stderr.lower()
    assert "not found" in error or "unknown" in error or "free" in error or "premium" in error, \
        "Should indicate command not available"


def test_patch_with_template_rejected():
    """Test patch with template is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w', encoding='utf-8') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w', encoding='utf-8') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "autofix-patch", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=10,
            check=False
        )

        # Should fail
        assert result.returncode != 0, "patch should be rejected"


def test_patch_not_in_help():
    """Test patch not listed in help."""
    result = subprocess.run(
        [COSTPILOT_PATH, "--help"],
        capture_output=True,
        text=True,
        timeout=10,
        check=False
    )

    # patch should not appear in help (unless as part of another word)
    help_text = result.stdout.lower()
    # Allow "dispatch" but not "patch" as standalone command
    if "patch" in help_text:
        # More strict check: should not be a subcommand
        assert "costpilot autofix-patch" not in help_text, "autofix-patch should not be a subcommand in help"


def test_patch_subcommand_rejected():
    """Test patch subcommand variations rejected."""
    commands = [
        [COSTPILOT_PATH, "autofix-patch"],
        [COSTPILOT_PATH, "apply-patch"],
    ]

    for cmd in commands:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=10,
            check=False
        )

        # Should fail
        assert result.returncode != 0, f"Command {cmd} should be rejected"


def test_patch_with_output_rejected():
    """Test patch with --output flag rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "patched.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }

        with open(template_path, 'w', encoding='utf-8') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            [COSTPILOT_PATH, "autofix-patch", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10,
            check=False
        )

        # Should fail
        assert result.returncode != 0, "patch --output should be rejected"


if __name__ == "__main__":
    test_patch_command_not_present()
    test_patch_with_template_rejected()
    test_patch_not_in_help()
    test_patch_subcommand_rejected()
    test_patch_with_output_rejected()
    print("All Free Edition patch gating tests passed")
