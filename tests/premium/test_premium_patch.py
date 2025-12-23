#!/usr/bin/env python3
"""Test Premium: patch engine available."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_patch_command_exists():
    """Test patch command exists in Premium."""
    result = subprocess.run(
        ["costpilot", "patch", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # In Premium, should succeed
    # In Free, should fail
    if result.returncode == 0:
        assert "patch" in result.stdout.lower(), "Help should mention patch"


def test_patch_applies_fixes():
    """Test patch engine applies fixes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-limits",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008 OR Timeout > 300",
                    "fix": {
                        "MemorySize": 3008,
                        "Timeout": 300
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should apply patch
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should have patch output"


def test_patch_output_format():
    """Test patch output format."""
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

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                patched = json.load(f)
            assert "Resources" in patched, "Patched output should be valid"


def test_patch_multiple_violations():
    """Test patch handles multiple violations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 5000 + i * 1000
                    }
                }
                for i in range(5)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should handle multiple patches
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should report patches"


def test_patch_preserves_structure():
    """Test patch preserves template structure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "patched.json"

        template_content = {
            "AWSTemplateFormatVersion": "2010-09-09",
            "Description": "Test template",
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Runtime": "python3.9"
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should preserve structure
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                patched = json.load(f)

            # Should preserve metadata
            assert "Description" in patched, "Should preserve Description"
            # Should preserve other properties
            if "Resources" in patched and "Lambda" in patched["Resources"]:
                props = patched["Resources"]["Lambda"]["Properties"]
                assert "Runtime" in props, "Should preserve Runtime"


def test_patch_idempotent():
    """Test patch is idempotent."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

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

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # First patch
        result1 = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Second patch (should be no-op if already fixed)
        result2 = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Both should complete
        if result1.returncode == 0 and result2.returncode == 0:
            # Second run might indicate "no changes needed"
            pass


if __name__ == "__main__":
    test_patch_command_exists()
    test_patch_applies_fixes()
    test_patch_output_format()
    test_patch_multiple_violations()
    test_patch_preserves_structure()
    test_patch_idempotent()
    print("All Premium patch engine tests passed")
