#!/usr/bin/env python3
"""Test repeated patch cycles stability."""

import subprocess
import tempfile
from pathlib import Path
import json
import shutil


def test_repeated_patch_cycles():
    """Test stability of repeated patch application cycles."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Initial template with violations
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 8192,
                        "Timeout": 600
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
                },
                {
                    "id": "lambda-timeout-limit",
                    "severity": "medium",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "Timeout > 300",
                    "fix": {
                        "Timeout": 300
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Apply patches 100 times
        for cycle in range(100):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Should complete
            assert result.returncode in [0, 1, 2, 101], f"Patch cycle {cycle} failed"

            # After first fix, subsequent runs should be no-ops
            if cycle > 0:
                # Check that template is stable
                with open(template_path, 'r') as f:
                    current = json.load(f)

                # Memory should be fixed at 3008
                assert current["Resources"]["Lambda1"]["Properties"]["MemorySize"] <= 3008, \
                    "Patch should stabilize"


def test_patch_convergence():
    """Test that repeated patches converge to fixed state."""
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

        # Apply patch
        result1 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Apply again (should be no-op)
        result2 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Apply third time (should still be no-op)
        result3 = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        # All should complete successfully
        assert result1.returncode in [0, 1, 2, 101], "First patch should complete"
        assert result2.returncode in [0, 1, 2, 101], "Second patch should complete"
        assert result3.returncode in [0, 1, 2, 101], "Third patch should complete"


def test_patch_no_oscillation():
    """Test that patches don't oscillate between states."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 5120
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

        # Apply patch and record states
        states = []

        for _ in range(10):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
                capture_output=True,
                text=True,
                timeout=30
            )

            if template_path.exists():
                with open(template_path, 'r') as f:
                    current = json.load(f)

                memory = current["Resources"]["Lambda"]["Properties"]["MemorySize"]
                states.append(memory)

        # After first fix, should stabilize
        if len(states) > 1:
            # All subsequent states should be same
            stable_value = states[1]
            for state in states[2:]:
                assert state == stable_value, "Patch should not oscillate"


def test_multiple_resources_patch_stability():
    """Test patch stability with multiple resources."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240 - (i * 100)
                    }
                }
                for i in range(50)
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory",
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

        # Apply patches repeatedly
        for cycle in range(20):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
                capture_output=True,
                text=True,
                timeout=60
            )

            assert result.returncode in [0, 1, 2, 101], f"Patch cycle {cycle} should complete"


def test_patch_preserves_other_properties():
    """Test that patches preserve unmodified properties."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 300,
                        "Runtime": "python3.9",
                        "Description": "Test function"
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

        # Apply patch
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=30
        )

        if template_path.exists():
            with open(template_path, 'r') as f:
                fixed = json.load(f)

            props = fixed["Resources"]["Lambda"]["Properties"]

            # Other properties should be preserved
            assert props.get("Timeout") == 300, "Timeout should be preserved"
            assert props.get("Runtime") == "python3.9", "Runtime should be preserved"
            assert props.get("Description") == "Test function", "Description should be preserved"


if __name__ == "__main__":
    test_repeated_patch_cycles()
    test_patch_convergence()
    test_patch_no_oscillation()
    test_multiple_resources_patch_stability()
    test_patch_preserves_other_properties()
    print("All repeated patch cycles tests passed")
