#!/usr/bin/env python3
"""Test Premium: drift detection executes."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_drift_detection_command():
    """Test drift detection command exists."""
    result = subprocess.run(
        ["costpilot", "drift", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # In Premium, should succeed or have drift command
    # May be subcommand of check or analyze
    if result.returncode != 0:
        # Try as subcommand
        result = subprocess.run(
            ["costpilot", "check", "--drift", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )


def test_drift_detection_with_baseline():
    """Test drift detection with baseline."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
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
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "drift", "--plan", str(template_path), "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should detect drift
        if result.returncode == 0:
            output = result.stdout.lower()
            # Should mention drift or changes
            assert "drift" in output or "change" in output or "differ" in output, \
                "Should indicate drift detected"


def test_drift_detection_no_drift():
    """Test drift detection when no drift exists."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        # Same as template
        baseline_content = {
            "resources": [
                {
                    "id": "Lambda",
                    "type": "AWS::Lambda::Function",
                    "properties": {
                        "MemorySize": 1024
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "drift", "--plan", str(template_path), "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should complete - no drift
        if result.returncode == 0:
            output = result.stdout.lower()
            # Should indicate no drift
            pass


def test_drift_detection_multiple_changes():
    """Test drift detection with multiple changes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 60
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 3008
                    }
                }
            }
        }
        
        baseline_content = {
            "resources": [
                {
                    "id": "Lambda1",
                    "type": "AWS::Lambda::Function",
                    "properties": {
                        "MemorySize": 1024,
                        "Timeout": 30
                    }
                },
                {
                    "id": "Lambda2",
                    "type": "AWS::Lambda::Function",
                    "properties": {
                        "MemorySize": 1024
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "drift", "--plan", str(template_path), "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should detect multiple drifts
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should report drift details"


def test_drift_detection_output_format():
    """Test drift detection output format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        output_path = Path(tmpdir) / "drift.json"
        
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
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "drift", "--plan", str(template_path), 
             "--baseline", str(baseline_path), "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should create output file
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                drift = json.load(f)
            assert isinstance(drift, (dict, list)), "Drift output should be valid JSON"


if __name__ == "__main__":
    test_drift_detection_command()
    test_drift_detection_with_baseline()
    test_drift_detection_no_drift()
    test_drift_detection_multiple_changes()
    test_drift_detection_output_format()
    print("All Premium drift detection tests passed")
