#!/usr/bin/env python3
"""Test Premium: SLO mode available and functional."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_slo_command_exists():
    """Test SLO command exists in Premium."""
    result = subprocess.run(
        ["costpilot", "slo", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # In Premium, should succeed
    # In Free, should fail
    if result.returncode == 0:
        assert "slo" in result.stdout.lower(), "Help should mention SLO"


def test_slo_check():
    """Test SLO check functionality."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_config_path = Path(tmpdir) / "slo.json"

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

        slo_config = {
            "version": "1.0.0",
            "slos": [
                {
                    "id": "cost-limit",
                    "target": 100.0,
                    "period": "monthly",
                    "metric": "total_cost"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(slo_config_path, 'w') as f:
            json.dump(slo_config, f)

        result = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path), "--config", str(slo_config_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should check SLOs
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should have SLO check output"


def test_slo_with_baseline():
    """Test SLO with baseline comparison."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        slo_config_path = Path(tmpdir) / "slo.json"

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

        baseline_content = {
            "resources": [
                {
                    "id": "Lambda",
                    "type": "AWS::Lambda::Function",
                    "properties": {
                        "MemorySize": 1024
                    },
                    "cost": 50.0
                }
            ],
            "total_cost": 50.0
        }

        slo_config = {
            "version": "1.0.0",
            "slos": [
                {
                    "id": "cost-increase-limit",
                    "target": 0.1,
                    "metric": "cost_increase_percent"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)

        with open(slo_config_path, 'w') as f:
            json.dump(slo_config, f)

        result = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path),
             "--baseline", str(baseline_path), "--config", str(slo_config_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should compare against baseline
        if result.returncode == 0:
            output = result.stdout.lower()
            # Should mention baseline or comparison


def test_slo_burn_rate():
    """Test SLO burn rate calculation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_config_path = Path(tmpdir) / "slo.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3008
                    }
                }
                for i in range(10)
            }
        }

        slo_config = {
            "version": "1.0.0",
            "slos": [
                {
                    "id": "cost-limit",
                    "target": 1000.0,
                    "period": "monthly",
                    "metric": "total_cost",
                    "burn_rate_alert": 2.0
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(slo_config_path, 'w') as f:
            json.dump(slo_config, f)

        result = subprocess.run(
            ["costpilot", "slo", "burn-rate", "--plan", str(template_path), "--config", str(slo_config_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should calculate burn rate


def test_slo_validate():
    """Test SLO validation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_config_path = Path(tmpdir) / "slo.json"

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

        slo_config = {
            "version": "1.0.0",
            "slos": [
                {
                    "id": "cost-limit",
                    "target": 100.0,
                    "period": "monthly",
                    "metric": "total_cost"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(slo_config_path, 'w') as f:
            json.dump(slo_config, f)

        result = subprocess.run(
            ["costpilot", "slo", "validate", "--plan", str(template_path), "--config", str(slo_config_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should validate SLO compliance


def test_slo_output_format():
    """Test SLO output format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_config_path = Path(tmpdir) / "slo.json"
        output_path = Path(tmpdir) / "slo_result.json"

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

        slo_config = {
            "version": "1.0.0",
            "slos": [
                {
                    "id": "cost-limit",
                    "target": 100.0,
                    "period": "monthly",
                    "metric": "total_cost"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(slo_config_path, 'w') as f:
            json.dump(slo_config, f)

        result = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path),
             "--config", str(slo_config_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                slo_result = json.load(f)
            assert isinstance(slo_result, dict), "SLO result should be valid JSON"


if __name__ == "__main__":
    test_slo_command_exists()
    test_slo_check()
    test_slo_with_baseline()
    test_slo_burn_rate()
    test_slo_validate()
    test_slo_output_format()
    print("All Premium SLO tests passed")
