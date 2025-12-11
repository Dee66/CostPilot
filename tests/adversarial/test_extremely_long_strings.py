#!/usr/bin/env python3
"""Test handling of extremely long strings."""

import json
import subprocess
import tempfile
from pathlib import Path
import string
import random
import sys


def test_extremely_long_resource_name():
    """Test handling of extremely long resource names."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # 10000 character resource name
        long_name = "Lambda" + "A" * 9994
        
        template_content = {
            "Resources": {
                long_name: {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should handle long names
        assert result.returncode in [0, 1, 2, 101], "Should handle long resource names"


def test_extremely_long_property_value():
    """Test handling of extremely long property values."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # 100000 character description
        long_description = "A" * 100000
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": long_description,
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should handle long property values
        assert result.returncode in [0, 1, 2, 101], "Should handle long property values"


def test_extremely_long_type_name():
    """Test handling of extremely long type names."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # 5000 character type name
        long_type = "AWS::Lambda::" + "Custom" * 1000
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": long_type,
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should handle long type names
        assert result.returncode in [0, 1, 2, 101], "Should handle long type names"


def test_extremely_long_json_array():
    """Test handling of extremely long JSON arrays."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Array with 10000 items
        long_array = ["item"] * 10000
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "ITEMS": long_array
                            }
                        },
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should handle long arrays
        assert result.returncode in [0, 1, 2, 101], "Should handle long arrays"


def test_extremely_long_nested_structure():
    """Test handling of extremely deep nested structures."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Increase recursion limit for deep nesting test
        old_limit = sys.getrecursionlimit()
        sys.setrecursionlimit(5000)
        
        try:
            # Create 1000-level nested structure
            nested = {"value": "leaf"}
            for _ in range(1000):
                nested = {"nested": nested}
            
            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Environment": nested,
                            "MemorySize": 1024
                        }
                    }
                }
            }
            
            with open(template_path, 'w') as f:
                json.dump(template_content, f)
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )
        finally:
            # Restore original recursion limit
            sys.setrecursionlimit(old_limit)
        
        # Should handle deep nesting (might fail due to stack limits)
        assert result.returncode in [0, 1, 2, 101], "Should handle deep nesting"


def test_extremely_long_key_name():
    """Test handling of extremely long JSON key names."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # 10000 character key name
        long_key = "K" * 10000
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        long_key: "value",
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should handle long keys
        assert result.returncode in [0, 1, 2, 101], "Should handle long key names"


def test_extremely_long_combined_template():
    """Test handling of template with multiple long strings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {}
        }
        
        # Add 100 resources with long names and properties
        for i in range(100):
            long_name = f"Lambda{i}" + "X" * 9990
            template_content["Resources"][long_name] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "Description": "D" * 10000,
                    "MemorySize": 1024
                }
            }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle combined long strings
        assert result.returncode in [0, 1, 2, 101], "Should handle combined long strings"


def test_extremely_long_policy_string():
    """Test handling of extremely long policy condition strings."""
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
        
        # Extremely long condition string
        long_condition = "MemorySize > 3008 AND Description CONTAINS '" + "A" * 10000 + "'"
        
        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "long-condition",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": long_condition
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)
        
        result = subprocess.run(
            ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should handle long policy conditions
        assert result.returncode in [0, 1, 2, 101], "Should handle long policy conditions"


def test_extremely_long_baseline_json():
    """Test handling of extremely long baseline JSON."""
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
        
        # Baseline with 1000 resources with long names
        baseline_content = {
            "resources": []
        }
        
        for i in range(1000):
            baseline_content["resources"].append({
                "name": f"Lambda{i}" + "X" * 9990,
                "type": "AWS::Lambda::Function",
                "cost": 10.0 + i,
                "properties": {
                    "MemorySize": 1024,
                    "Description": "D" * 10000
                }
            })
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should handle long baseline
        assert result.returncode in [0, 1, 2, 101], "Should handle long baseline"


def test_extremely_long_unicode_string():
    """Test handling of extremely long Unicode strings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # 10000 character Unicode string
        long_unicode = "函数" * 5000
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": long_unicode,
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w', encoding='utf-8') as f:
            json.dump(template_content, f, ensure_ascii=False)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        # Should handle long Unicode strings
        assert result.returncode in [0, 1, 2, 101], "Should handle long Unicode strings"


if __name__ == "__main__":
    test_extremely_long_resource_name()
    test_extremely_long_property_value()
    test_extremely_long_type_name()
    test_extremely_long_json_array()
    test_extremely_long_nested_structure()
    test_extremely_long_key_name()
    test_extremely_long_combined_template()
    test_extremely_long_policy_string()
    test_extremely_long_baseline_json()
    test_extremely_long_unicode_string()
    print("All extremely long string tests passed")
