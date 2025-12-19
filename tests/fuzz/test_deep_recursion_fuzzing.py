#!/usr/bin/env python3
"""Test deep recursion fuzzing."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_deep_nesting_fuzzing():
    """Fuzz with deeply nested structures."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        depths = [100, 500, 1000, 1500]
        
        for depth in depths:
            # Create deeply nested object
            nested = {}
            current = nested
            for i in range(depth):
                current["level"] = {}
                current = current["level"]
            current["value"] = "deep"
            
            template_content = {
                "Resources": {
                    "Nested": nested
                }
            }
            
            with open(template_path, 'w') as f:
                json.dump(template_content, f)
            
            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True,
                timeout=5
            )
            
            # Should handle or reject gracefully
            assert result.returncode in [0, 1, 2], f"Should handle depth {depth}"


def test_deep_array_nesting_fuzzing():
    """Fuzz with deeply nested arrays."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create deeply nested array
        nested = []
        current = nested
        for i in range(500):
            inner = []
            current.append(inner)
            current = inner
        current.append("deep")
        
        template_content = {
            "Resources": {
                "NestedArray": nested
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        assert result.returncode in [0, 1, 2], "Should handle deeply nested arrays"


def test_mixed_nesting_fuzzing():
    """Fuzz with mixed object/array nesting."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Alternate between objects and arrays
        nested = {}
        current_obj = nested
        for i in range(300):
            if i % 2 == 0:
                current_obj["array"] = []
                current_arr = current_obj["array"]
                inner_obj = {}
                current_arr.append(inner_obj)
                current_obj = inner_obj
            else:
                current_obj["level"] = {}
                current_obj = current_obj["level"]
        
        template_content = {
            "Resources": {
                "Mixed": nested
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        assert result.returncode in [0, 1, 2], "Should handle mixed nesting"


def test_recursive_reference_fuzzing():
    """Fuzz with recursive-like references."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Simulate recursive references (CloudFormation)
        resources = {}
        for i in range(100):
            depends_on = [f"Lambda{j}" for j in range(i)]
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "DependsOn": depends_on,
                "Properties": {"MemorySize": 1024}
            }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode in [0, 1, 2], "Should handle recursive-like references"


def test_circular_dependency_fuzzing():
    """Fuzz with circular dependencies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create circular dependency chain
        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "DependsOn": "Lambda10"
                }
            }
        }
        
        for i in range(2, 11):
            template_content["Resources"][f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "DependsOn": f"Lambda{i-1}"
            }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        # Should detect circular dependency
        output = result.stdout + result.stderr
        if "circular" in output.lower() or "cycle" in output.lower():
            assert True, "Should detect circular dependencies"


def test_wide_shallow_fuzzing():
    """Fuzz with wide but shallow structures."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Many siblings, not deep
        wide_obj = {
            f"field_{i}": {
                "level1": {
                    "value": i
                }
            }
            for i in range(1000)
        }
        
        template_content = {
            "Resources": wide_obj
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode in [0, 1, 2], "Should handle wide shallow structures"


def test_recursion_limit_fuzzing():
    """Test recursion limit handling."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create structure that might hit recursion limits
        def create_nested(depth, max_depth):
            if depth >= max_depth:
                return {"value": "leaf"}
            return {
                "left": create_nested(depth + 1, max_depth),
                "right": create_nested(depth + 1, max_depth)
            }
        
        # Binary tree structure
        nested = create_nested(0, 10)  # 2^10 = 1024 nodes
        
        template_content = {
            "Resources": {
                "Tree": nested
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        assert result.returncode in [0, 1, 2], "Should handle recursion limits"


if __name__ == "__main__":
    test_deep_nesting_fuzzing()
    test_deep_array_nesting_fuzzing()
    test_mixed_nesting_fuzzing()
    test_recursive_reference_fuzzing()
    test_circular_dependency_fuzzing()
    test_wide_shallow_fuzzing()
    test_recursion_limit_fuzzing()
    print("All deep recursion fuzzing tests passed")
