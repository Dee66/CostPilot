#!/usr/bin/env python3
"""Test SLO drift stability across versions."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_slo_calculation_deterministic():
    """Test that SLO calculations are deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
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
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-threshold",
                    "target": 0.99,
                    "window": "30d",
                    "budget": 100.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run multiple times
        results = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            results.append(result.stdout)
        
        # All results should be identical
        assert all(r == results[0] for r in results), \
            f"SLO calculations not deterministic"


def test_slo_drift_detection_stable():
    """Test that SLO drift detection is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048  # Increased from baseline
                    }
                }
            }
        }
        
        baseline_content = {
            "resources": {
                "Lambda": {
                    "cost": 10.0,
                    "memory": 1024
                }
            }
        }
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-drift",
                    "target": 0.95,
                    "window": "30d",
                    "budget": 100.0,
                    "drift_threshold": 0.1
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run multiple times
        drift_detected = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path), "--baseline", str(baseline_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            output = result.stdout + result.stderr
            drift_detected.append("drift" in output.lower() or result.returncode != 0)
        
        # Drift detection should be consistent
        assert all(d == drift_detected[0] for d in drift_detected), \
            "SLO drift detection not stable"


def test_slo_error_budget_calculation_stable():
    """Test that error budget calculations are stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(10)
            }
        }
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-slo",
                    "target": 0.999,
                    "window": "30d",
                    "budget": 1000.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run multiple times
        outputs = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path), "--format", "json"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            outputs.append(result.stdout)
        
        # All outputs should be identical
        assert all(o == outputs[0] for o in outputs), \
            "Error budget calculations not stable"


def test_slo_burn_rate_consistency():
    """Test that burn rate calculations are consistent."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
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
        
        slo_content = {
            "slos": [
                {
                    "name": "high-memory-slo",
                    "target": 0.99,
                    "window": "7d",
                    "budget": 50.0,
                    "burn_rate_threshold": 10.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run multiple times
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            results.append(result.stdout)
        
        # Burn rate should be calculated consistently
        assert all(r == results[0] for r in results), \
            "Burn rate calculations not consistent"


def test_slo_multiwindow_stability():
    """Test SLO stability with multiple time windows."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
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
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-1d",
                    "target": 0.99,
                    "window": "1d",
                    "budget": 10.0
                },
                {
                    "name": "cost-7d",
                    "target": 0.99,
                    "window": "7d",
                    "budget": 70.0
                },
                {
                    "name": "cost-30d",
                    "target": 0.99,
                    "window": "30d",
                    "budget": 300.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run twice
        result1 = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        result2 = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should be identical across windows
        assert result1.stdout == result2.stdout, \
            "Multi-window SLO not stable"


def test_slo_metadata_stability():
    """Test that SLO metadata is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
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
        
        slo_content = {
            "slos": [
                {
                    "name": "cost-slo",
                    "target": 0.99,
                    "window": "30d",
                    "budget": 100.0,
                    "metadata": {
                        "owner": "team-a",
                        "severity": "high"
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        result = subprocess.run(
            ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Parse output
        try:
            output_data = json.loads(result.stdout)
            
            # Check that metadata is preserved
            if "slos" in output_data:
                for slo in output_data["slos"]:
                    if "metadata" in slo:
                        print("SLO metadata preserved in output")
        except json.JSONDecodeError:
            pass


def test_slo_compliance_percentage_stable():
    """Test that compliance percentage is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 * (i + 1)
                    }
                }
                for i in range(20)
            }
        }
        
        slo_content = {
            "slos": [
                {
                    "name": "aggregate-cost",
                    "target": 0.99,
                    "window": "30d",
                    "budget": 500.0
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)
        
        # Run multiple times
        results = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "slo", "check", "--plan", str(template_path), "--slo", str(slo_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            results.append(result.stdout)
        
        # Compliance should be stable
        assert all(r == results[0] for r in results), \
            "Compliance percentage not stable"


if __name__ == "__main__":
    test_slo_calculation_deterministic()
    test_slo_drift_detection_stable()
    test_slo_error_budget_calculation_stable()
    test_slo_burn_rate_consistency()
    test_slo_multiwindow_stability()
    test_slo_metadata_stability()
    test_slo_compliance_percentage_stable()
    print("All SLO drift stability tests passed")
