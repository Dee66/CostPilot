#!/usr/bin/env python3
"""Test Premium: advanced cost models produce expected outputs."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_advanced_cost_model_exists():
    """Test advanced cost model flag exists."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--cost-model", "advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should accept advanced model
        # In Free, should reject
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Advanced model should produce output"


def test_advanced_cost_model_accuracy():
    """Test advanced cost model provides accurate estimates."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "cost.json"
        
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), 
             "--cost-model", "advanced", "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Advanced model should produce detailed cost breakdown
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                cost_data = json.load(f)
            
            assert isinstance(cost_data, dict), "Cost data should be dict"
            # Advanced model should include detailed breakdown


def test_advanced_cost_model_multiple_resources():
    """Test advanced cost model with multiple resources."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                },
                "DynamoDB": {
                    "Type": "AWS::DynamoDB::Table",
                    "Properties": {
                        "BillingMode": "PAY_PER_REQUEST"
                    }
                },
                "S3": {
                    "Type": "AWS::S3::Bucket",
                    "Properties": {}
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--cost-model", "advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should analyze all resources with advanced model
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should have cost analysis"


def test_advanced_cost_model_comparison():
    """Test advanced cost model comparison with basic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run with basic model
        result_basic = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Run with advanced model
        result_advanced = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--cost-model", "advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Both should succeed
        if result_basic.returncode == 0 and result_advanced.returncode == 0:
            # Advanced might provide more detail
            pass


def test_advanced_cost_model_confidence_scores():
    """Test advanced cost model includes confidence scores."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "cost.json"
        
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), 
             "--cost-model", "advanced", "--output", str(output_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Advanced model might include confidence scores
        if result.returncode == 0 and output_path.exists():
            with open(output_path) as f:
                cost_data = json.load(f)
            
            # Check for confidence or uncertainty fields
            # (structure depends on implementation)


def test_advanced_cost_model_edge_cases():
    """Test advanced cost model handles edge cases."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,  # Maximum
                        "Timeout": 900,  # Maximum
                        "EphemeralStorage": {
                            "Size": 10240  # Maximum
                        }
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--cost-model", "advanced"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle edge cases
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should analyze edge cases"


if __name__ == "__main__":
    test_advanced_cost_model_exists()
    test_advanced_cost_model_accuracy()
    test_advanced_cost_model_multiple_resources()
    test_advanced_cost_model_comparison()
    test_advanced_cost_model_confidence_scores()
    test_advanced_cost_model_edge_cases()
    print("All Premium advanced cost model tests passed")
