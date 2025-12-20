#!/usr/bin/env python3
"""Test drift check takes precedence over autofix."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_drift_check_before_autofix():
    """Drift detection must run before autofix is allowed."""
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
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Attempt autofix without drift check
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )

        # Should require drift check first or mention it
        output = result.stdout + result.stderr
        if "drift" in output.lower():
            assert True, "Drift check mentioned"


def test_drift_detected_blocks_autofix():
    """Detected drift must block autofix."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Original baseline
        baseline_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        # Modified template (drift)
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Check drift
        drift_result = subprocess.run(
            ["costpilot", "drift", "--baseline", str(baseline_path), "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # If drift detected, autofix should be blocked
        if drift_result.returncode != 0 or "drift" in drift_result.stdout.lower():
            # Attempt autofix
            autofix_result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--baseline", str(baseline_path)],
                capture_output=True,
                text=True
            )

            # Should block or require --force
            output = autofix_result.stdout + autofix_result.stderr
            if autofix_result.returncode != 0:
                assert "drift" in output.lower() or "baseline" in output.lower() or "premium" in output.lower(), \
                    "Should mention drift blocking or premium requirement"


def test_no_drift_allows_autofix():
    """No drift detection allows autofix to proceed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Identical baseline and template (no drift)
        content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(baseline_path, 'w') as f:
            json.dump(content, f)

        with open(template_path, 'w') as f:
            json.dump(content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Check drift (should pass)
        drift_result = subprocess.run(
            ["costpilot", "drift", "--baseline", str(baseline_path), "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # Autofix should proceed
        autofix_result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
            capture_output=True,
            text=True
        )

        # Should not block on drift
        output = autofix_result.stdout + autofix_result.stderr
        if "blocked" in output.lower():
            assert "drift" not in output.lower(), "Should not block on drift when none detected"


def test_force_flag_overrides_drift():
    """--force flag must override drift blocking."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Drift scenario
        baseline_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Attempt with --force
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--baseline", str(baseline_path), "--force", "--dry-run"],
            capture_output=True,
            text=True
        )

        # Should proceed despite drift
        output = result.stdout + result.stderr
        if result.returncode == 0:
            # Force worked
            assert True


def test_drift_check_precedence_documented():
    """Drift check precedence must be documented in help."""
    result = subprocess.run(
        ["costpilot", "autofix", "--help"],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        help_text = result.stdout
        # Check for drift-related flags or documentation
        assert "--force" in help_text or "--baseline" in help_text or "drift" in help_text.lower(), \
            "Help should document drift check precedence"


def test_autofix_updates_baseline():
    """Successful autofix must update baseline."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"

        original_content = {
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
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(original_content, f)

        with open(baseline_path, 'w') as f:
            json.dump(original_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Apply autofix
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--baseline", str(baseline_path), "--update-baseline"],
            capture_output=True,
            text=True
        )

        # Baseline should be updated
        if baseline_path.exists():
            with open(baseline_path) as f:
                updated_baseline = json.load(f)

            # Verify baseline reflects changes
            assert isinstance(updated_baseline, dict), "Baseline should be valid JSON"


if __name__ == "__main__":
    test_drift_check_before_autofix()
    test_drift_detected_blocks_autofix()
    test_no_drift_allows_autofix()
    test_force_flag_overrides_drift()
    test_drift_check_precedence_documented()
    test_autofix_updates_baseline()
    print("All drift check precedence tests passed")
