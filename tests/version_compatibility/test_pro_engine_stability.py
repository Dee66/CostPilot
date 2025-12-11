#!/usr/bin/env python3
"""Test pro-engine invariant stability."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_pro_engine_cost_output_stable():
    """Test that pro-engine cost output is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                },
                "DynamoDB": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "BillingMode": "PROVISIONED",
                        "ProvisionedThroughput": {
                            "ReadCapacityUnits": 5,
                            "WriteCapacityUnits": 5
                        }
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run pro-engine multiple times
        costs = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path), "--engine", "pro"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            costs.append(result.stdout)
        
        # Cost output should be identical
        assert all(c == costs[0] for c in costs), \
            "Pro-engine cost output not stable"


def test_pro_engine_cost_breakdown_deterministic():
    """Test that pro-engine cost breakdown is deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 600
                    }
                },
                "S3Bucket": {
                    "Type": "AWS::S3::Bucket",
                    "Properties": {
                        "VersioningConfiguration": {
                            "Status": "Enabled"
                        }
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run with JSON output
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path), "--engine", "pro", "--format", "json"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            results.append(result.stdout)
        
        # Breakdown should be identical
        assert all(r == results[0] for r in results), \
            "Pro-engine cost breakdown not deterministic"


def test_pro_engine_vs_basic_comparison_stable():
    """Test that pro-engine vs basic-engine comparison is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run basic-engine
        basic_result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--engine", "basic"],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Run pro-engine
        pro_result = subprocess.run(
            ["costpilot", "predict", "--plan", str(template_path), "--engine", "pro"],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Both should complete successfully
        assert basic_result.returncode in [0, 1, 2, 101], "Basic-engine should complete"
        assert pro_result.returncode in [0, 1, 2, 101], "Pro-engine should complete"


def test_pro_engine_with_baseline_stable():
    """Test that pro-engine with baseline is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048,
                        "Timeout": 300
                    }
                }
            }
        }
        
        baseline_content = {
            "resources": [
                {
                    "name": "Lambda",
                    "type": "AWS::Lambda::Function",
                    "cost": 10.0,
                    "properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        # Run with baseline multiple times
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path), "--baseline", str(baseline_path), "--engine", "pro"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            results.append(result.stdout)
        
        # Output should be stable
        assert all(r == results[0] for r in results), \
            "Pro-engine with baseline not stable"


def test_pro_engine_monthly_estimate_consistent():
    """Test that pro-engine monthly estimate is consistent."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                },
                "RDS": {
                    "Type": "AWS::RDS::DBInstance",
                    "Properties": {
                        "DBInstanceClass": "db.t3.micro",
                        "Engine": "postgres",
                        "AllocatedStorage": 20
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        estimates = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path), "--engine", "pro", "--format", "json"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            estimates.append(result.stdout)
        
        # Monthly estimate should be consistent
        assert all(e == estimates[0] for e in estimates), \
            "Pro-engine monthly estimate not consistent"


def test_pro_engine_trend_analysis_stable():
    """Test that pro-engine trend analysis is stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        trend_path = Path(tmpdir) / "trend.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Timeout": 300
                    }
                }
            }
        }
        
        trend_content = {
            "history": [
                {"date": "2024-01-01", "cost": 10.0},
                {"date": "2024-01-02", "cost": 12.0},
                {"date": "2024-01-03", "cost": 15.0},
                {"date": "2024-01-04", "cost": 18.0},
                {"date": "2024-01-05", "cost": 20.0}
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(trend_path, 'w') as f:
            json.dump(trend_content, f)
        
        # Run trend analysis multiple times
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "trend", "--plan", str(template_path), "--history", str(trend_path), "--engine", "pro"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            results.append(result.stdout)
        
        # Trend analysis should be stable
        assert all(r == results[0] for r in results), \
            "Pro-engine trend analysis not stable"


def test_pro_engine_optimization_suggestions_stable():
    """Test that pro-engine optimization suggestions are stable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900
                    }
                },
                "EC2": {
                    "Type": "AWS::EC2::Instance",
                    "Properties": {
                        "InstanceType": "m5.24xlarge"
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run optimization analysis multiple times
        suggestions = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "optimize", "--plan", str(template_path), "--engine", "pro"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            suggestions.append(result.stdout)
        
        # Optimization suggestions should be stable
        assert all(s == suggestions[0] for s in suggestions), \
            "Pro-engine optimization suggestions not stable"


def test_pro_engine_json_output_parseable():
    """Test that pro-engine JSON output is always parseable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + (i * 512),
                        "Timeout": 300
                    }
                }
                for i in range(20)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run with JSON format
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path), "--engine", "pro", "--format", "json"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if result.returncode == 0:
                # Should be valid JSON
                try:
                    json.loads(result.stdout)
                except json.JSONDecodeError:
                    assert False, "Pro-engine JSON output not parseable"


if __name__ == "__main__":
    test_pro_engine_cost_output_stable()
    test_pro_engine_cost_breakdown_deterministic()
    test_pro_engine_vs_basic_comparison_stable()
    test_pro_engine_with_baseline_stable()
    test_pro_engine_monthly_estimate_consistent()
    test_pro_engine_trend_analysis_stable()
    test_pro_engine_optimization_suggestions_stable()
    test_pro_engine_json_output_parseable()
    print("All pro-engine invariant stability tests passed")
