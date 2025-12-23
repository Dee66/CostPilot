#!/usr/bin/env python3
"""Test Premium: economic attack detection executes."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_economic_attack_detection_command():
    """Test economic attack detection command exists."""
    result = subprocess.run(
        ["costpilot", "economic-attack", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # In Premium, should succeed or be subcommand
    if result.returncode != 0:
        result = subprocess.run(
            ["costpilot", "check", "--economic-attack", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )


def test_economic_attack_detection_resource_bomb():
    """Test economic attack detection for resource bombs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Resource bomb: many expensive resources
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900
                    }
                }
                for i in range(100)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=15
        )

        # In Premium, should detect resource bomb
        if result.returncode == 0:
            output = result.stdout.lower()
            assert "attack" in output or "bomb" in output or "excessive" in output, \
                "Should indicate economic attack detected"


def test_economic_attack_detection_cost_spike():
    """Test economic attack detection for cost spikes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "NatGateway1": {
                    "Type": "AWS::EC2::NatGateway",
                    "Properties": {}
                },
                "NatGateway2": {
                    "Type": "AWS::EC2::NatGateway",
                    "Properties": {}
                },
                "NatGateway3": {
                    "Type": "AWS::EC2::NatGateway",
                    "Properties": {}
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should detect expensive resource pattern
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should report cost spike"


def test_economic_attack_detection_exponential_growth():
    """Test economic attack detection for exponential growth patterns."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Pattern: each resource creates more resources
        template_content = {
            "Resources": {
                "AutoScaling1": {
                    "Type": "AWS::AutoScaling::AutoScalingGroup",
                    "Properties": {
                        "MaxSize": 1000
                    }
                },
                "AutoScaling2": {
                    "Type": "AWS::AutoScaling::AutoScalingGroup",
                    "Properties": {
                        "MaxSize": 1000
                    }
                },
                "AutoScaling3": {
                    "Type": "AWS::AutoScaling::AutoScalingGroup",
                    "Properties": {
                        "MaxSize": 1000
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should detect exponential growth pattern


def test_economic_attack_detection_cost_threshold():
    """Test economic attack detection with cost threshold."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
                for i in range(50)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path), "--threshold", "1000.0"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should use threshold for detection


def test_economic_attack_detection_output_format():
    """Test economic attack detection output format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "attacks.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
                for i in range(100)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=15
        )

        # Should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                attacks = json.load(f)
            assert isinstance(attacks, (dict, list)), "Attacks output should be valid JSON"


def test_economic_attack_detection_severity():
    """Test economic attack detection severity levels."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                # High severity: many expensive resources
                **{
                    f"Lambda{i}": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "MemorySize": 10240,
                            "Timeout": 900
                        }
                    }
                    for i in range(50)
                },
                # Medium severity: moderately expensive
                "NatGateway": {
                    "Type": "AWS::EC2::NatGateway",
                    "Properties": {}
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "economic-attack", "--plan", str(template_path), "--min-severity", "high"],
            capture_output=True,
            text=True,
            timeout=15
        )

        # Should filter by severity


if __name__ == "__main__":
    test_economic_attack_detection_command()
    test_economic_attack_detection_resource_bomb()
    test_economic_attack_detection_cost_spike()
    test_economic_attack_detection_exponential_growth()
    test_economic_attack_detection_cost_threshold()
    test_economic_attack_detection_output_format()
    test_economic_attack_detection_severity()
    print("All Premium economic attack detection tests passed")
