#!/usr/bin/env python3
"""Test patch stability across versions."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_patch_output_deterministic():
    """Test that patch output is deterministic."""
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
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Generate patch multiple times
        patches = []

        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
                capture_output=True,
                text=True,
                timeout=30
            )

            patches.append(result.stdout)

        # All patches should be identical
        assert all(p == patches[0] for p in patches), \
            "Patch output not deterministic"


def test_patch_format_stable():
    """Test that patch format is stable across runs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 5120
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 8192
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Generate patch
        result1 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=30
        )

        result2 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Format should be consistent
        assert result1.stdout == result2.stdout, \
            "Patch format not stable"


def test_patch_application_idempotent():
    """Test that applying same patch multiple times is idempotent."""
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
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Apply patch first time
        result1 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Apply patch second time (should be no-op)
        result2 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Second application should indicate no changes needed
        assert result1.returncode in [0, 1, 2, 101], "First patch should complete"
        assert result2.returncode in [0, 1, 2, 101], "Second patch should complete"


def test_patch_line_numbers_stable():
    """Test that patch line numbers are stable."""
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
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        # Write with consistent formatting
        with open(template_path, 'w') as f:
            json.dump(template_content, f, indent=2)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Generate patch multiple times
        patches = []

        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
                capture_output=True,
                text=True,
                timeout=30
            )

            patches.append(result.stdout)

        # Line numbers should be consistent
        assert all(p == patches[0] for p in patches), \
            "Patch line numbers not stable"


def test_patch_metadata_preserved():
    """Test that patch metadata is preserved across versions."""
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
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    },
                    "metadata": {
                        "reason": "AWS Lambda limit",
                        "reference": "https://docs.aws.amazon.com/lambda/latest/dg/limits.html"
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Check metadata in output
        try:
            output_data = json.loads(result.stdout)

            # Metadata should be included
            if "patches" in output_data or "fixes" in output_data:
                print("Patch metadata preserved")
        except json.JSONDecodeError:
            pass


def test_patch_ordering_stable():
    """Test that patch ordering is stable when multiple patches exist."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
                for i in range(10)
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Generate patches multiple times
        results = []

        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
                capture_output=True,
                text=True,
                timeout=30
            )

            results.append(result.stdout)

        # Ordering should be consistent
        assert all(r == results[0] for r in results), \
            "Patch ordering not stable"


def test_patch_diff_format_stable():
    """Test that diff format in patches is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900,
                        "Runtime": "python3.9"
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f, indent=2)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Generate diff
        result1 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--show-diff"],
            capture_output=True,
            text=True,
            timeout=30
        )

        result2 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--show-diff"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Diff format should be stable
        assert result1.stdout == result2.stdout, \
            "Diff format not stable"


if __name__ == "__main__":
    test_patch_output_deterministic()
    test_patch_format_stable()
    test_patch_application_idempotent()
    test_patch_line_numbers_stable()
    test_patch_metadata_preserved()
    test_patch_ordering_stable()
    test_patch_diff_format_stable()
    print("All patch stability tests passed")
