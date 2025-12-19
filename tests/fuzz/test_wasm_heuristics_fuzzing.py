#!/usr/bin/env python3
"""Test WASM heuristics fuzzing."""

import json
import random
import string
import subprocess
import tempfile
from pathlib import Path


def generate_random_heuristics(num_resources=10):
    """Generate random heuristics file."""
    resource_types = [
        "AWS::Lambda::Function",
        "AWS::EC2::Instance",
        "AWS::RDS::DBInstance",
        "AWS::DynamoDB::Table",
        "AWS::S3::Bucket"
    ]
    
    heuristics = {}
    
    for _ in range(num_resources):
        resource_type = random.choice(resource_types)
        
        heuristics[resource_type] = {
            "base_cost": random.uniform(0.0001, 100.0),
            "cost_per_unit": random.uniform(0.0001, 10.0),
            "unit": random.choice(["hour", "GB", "request", "GB-hour"]),
            "multipliers": {
                random.choice(["MemorySize", "InstanceType", "StorageType"]): random.uniform(0.5, 5.0)
            }
        }
    
    return heuristics


def test_wasm_random_heuristics():
    """Test WASM with random heuristics."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
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
        
        heuristics_content = generate_random_heuristics(20)
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle random heuristics gracefully
        assert result.returncode in [0, 1, 2], "Should handle random heuristics"


def test_wasm_malformed_heuristics_keys():
    """Test WASM with malformed heuristics keys."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {"Resources": {}}
        
        # Random invalid keys
        heuristics_content = {
            "".join(random.choices(string.ascii_letters + string.digits + string.punctuation, k=50)): {
                "base_cost": 0.1
            }
            for _ in range(10)
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject or handle malformed keys
        assert result.returncode in [0, 1, 2], "Should handle malformed keys"


def test_wasm_heuristics_extreme_values():
    """Test WASM with extreme heuristics values."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {
            "Resources": {
                "Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}}
            }
        }
        
        # Extreme values
        heuristics_content = {
            "AWS::Lambda::Function": {
                "base_cost": 1e308,  # Near max float
                "cost_per_unit": -1e308,  # Negative extreme
                "unit": "hour",
                "multipliers": {
                    "MemorySize": 1e100
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle extreme values
        assert result.returncode in [0, 1, 2], "Should handle extreme values"


def test_wasm_heuristics_type_mismatches():
    """Test WASM with heuristics type mismatches."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {"Resources": {}}
        
        # Type mismatches
        heuristics_content = {
            "AWS::Lambda::Function": {
                "base_cost": "not a number",  # String instead of number
                "cost_per_unit": [1, 2, 3],  # Array instead of number
                "unit": 123,  # Number instead of string
                "multipliers": "not an object"  # String instead of object
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject type mismatches
        assert result.returncode in [1, 2], "Should reject type mismatches"


def test_wasm_heuristics_missing_required_fields():
    """Test WASM with missing required heuristics fields."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {
            "Resources": {
                "Lambda": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}}
            }
        }
        
        # Missing required fields
        heuristics_content = {
            "AWS::Lambda::Function": {
                # Missing base_cost
                "cost_per_unit": 0.01
                # Missing unit
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle missing fields
        assert result.returncode in [0, 1, 2], "Should handle missing fields"


def test_wasm_heuristics_large_multipliers():
    """Test WASM with large multipliers object."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {"Resources": {}}
        
        # Large multipliers object
        multipliers = {
            f"Property{i}": random.uniform(0.1, 10.0)
            for i in range(1000)
        }
        
        heuristics_content = {
            "AWS::Lambda::Function": {
                "base_cost": 0.01,
                "cost_per_unit": 0.001,
                "unit": "hour",
                "multipliers": multipliers
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle large multipliers
        assert result.returncode in [0, 1, 2], "Should handle large multipliers"


def test_wasm_heuristics_unicode_keys():
    """Test WASM with Unicode heuristics keys."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {"Resources": {}}
        
        # Unicode keys
        heuristics_content = {
            "AWS::Lambda::函数": {
                "base_cost": 0.01,
                "cost_per_unit": 0.001,
                "unit": "時間",
                "multipliers": {
                    "メモリサイズ": 1.5
                }
            },
            "AWS::EC2::实例": {
                "base_cost": 0.1,
                "cost_per_unit": 0.01,
                "unit": "hour",
                "multipliers": {}
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w', encoding='utf-8') as f:
            json.dump(heuristics_content, f, ensure_ascii=False)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle Unicode
        assert result.returncode in [0, 1, 2], "Should handle Unicode keys"


def test_wasm_heuristics_nested_multipliers():
    """Test WASM with deeply nested multipliers."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.json"
        
        template_content = {"Resources": {}}
        
        # Nested multipliers (if supported)
        nested = {"level1": {"level2": {"level3": 1.5}}}
        
        heuristics_content = {
            "AWS::Lambda::Function": {
                "base_cost": 0.01,
                "cost_per_unit": 0.001,
                "unit": "hour",
                "multipliers": nested
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(heuristics_path, 'w') as f:
            json.dump(heuristics_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle nested structures
        assert result.returncode in [0, 1, 2], "Should handle nested multipliers"


if __name__ == "__main__":
    test_wasm_random_heuristics()
    test_wasm_malformed_heuristics_keys()
    test_wasm_heuristics_extreme_values()
    test_wasm_heuristics_type_mismatches()
    test_wasm_heuristics_missing_required_fields()
    test_wasm_heuristics_large_multipliers()
    test_wasm_heuristics_unicode_keys()
    test_wasm_heuristics_nested_multipliers()
    print("All WASM heuristics fuzzing tests passed")
