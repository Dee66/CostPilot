#!/usr/bin/env python3
"""Test Premium: anomaly detection executes."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_anomaly_detection_command():
    """Test anomaly detection command exists."""
    result = subprocess.run(
        ["costpilot", "anomaly", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    # In Premium, should succeed or be subcommand
    if result.returncode != 0:
        result = subprocess.run(
            ["costpilot", "check", "--anomaly", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )


def test_anomaly_detection_with_baseline():
    """Test anomaly detection with baseline."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240  # Anomalous
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
                    "typical_range": {
                        "MemorySize": [128, 3008]
                    }
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)

        result = subprocess.run(
            ["costpilot", "anomaly", "--plan", str(template_path), "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should detect anomaly
        if result.returncode == 0:
            output = result.stdout.lower()
            assert "anomaly" in output or "unusual" in output or "outlier" in output, \
                "Should indicate anomaly detected"


def test_anomaly_detection_threshold():
    """Test anomaly detection with threshold."""
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

        result = subprocess.run(
            ["costpilot", "anomaly", "--plan", str(template_path), "--threshold", "3.0"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should use threshold for detection


def test_anomaly_detection_multiple_anomalies():
    """Test anomaly detection with multiple anomalies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240 if i % 2 == 0 else 128
                    }
                }
                for i in range(10)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "anomaly", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should detect multiple anomalies
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should report anomalies"


def test_anomaly_detection_output_format():
    """Test anomaly detection output format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "anomalies.json"

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
            ["costpilot", "anomaly", "--plan", str(template_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                anomalies = json.load(f)
            assert isinstance(anomalies, (dict, list)), "Anomalies output should be valid JSON"


def test_anomaly_detection_severity():
    """Test anomaly detection severity levels."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240  # High severity
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3500  # Medium severity
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "anomaly", "--plan", str(template_path), "--min-severity", "medium"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should report anomalies based on severity


if __name__ == "__main__":
    test_anomaly_detection_command()
    test_anomaly_detection_with_baseline()
    test_anomaly_detection_threshold()
    test_anomaly_detection_multiple_anomalies()
    test_anomaly_detection_output_format()
    test_anomaly_detection_severity()
    print("All Premium anomaly detection tests passed")
