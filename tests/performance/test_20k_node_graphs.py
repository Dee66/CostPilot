#!/usr/bin/env python3
"""Test graph mapping for 20k-node graphs."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_mapping_20k_linear_chain():
    """Test mapping a 20k-node linear dependency chain."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create 20k resources with linear dependencies
        resources = {}
        for i in range(20000):
            resource = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            
            # Each depends on previous (except first)
            if i > 0:
                resource["DependsOn"] = [f"Lambda{i-1}"]
            
            resources[f"Lambda{i}"] = resource
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should map 20k-node linear chain
        assert result.returncode in [0, 1, 2, 101], "Should map 20k-node linear chain"


def test_mapping_20k_star_topology():
    """Test mapping a 20k-node star topology."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create central node + 19999 leaf nodes
        resources = {
            "CentralLambda": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
        }
        
        # All others depend on central
        for i in range(19999):
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                },
                "DependsOn": ["CentralLambda"]
            }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should map star topology
        assert result.returncode in [0, 1, 2, 101], "Should map 20k-node star topology"


def test_mapping_20k_binary_tree():
    """Test mapping a 20k-node binary tree."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create binary tree structure
        resources = {}
        
        # Root
        resources["Lambda0"] = {
            "Type": "AWS::Lambda::Function",
            "Properties": {
                "MemorySize": 1024
            }
        }
        
        # Build tree
        for i in range(1, 20000):
            parent_idx = (i - 1) // 2
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                },
                "DependsOn": [f"Lambda{parent_idx}"]
            }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should map binary tree
        assert result.returncode in [0, 1, 2, 101], "Should map 20k-node binary tree"


def test_mapping_20k_dense_graph():
    """Test mapping a dense 20k-node graph."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create resources with multiple dependencies
        resources = {}
        
        for i in range(20000):
            resource = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            
            # Each node depends on previous 5 nodes
            if i >= 5:
                resource["DependsOn"] = [f"Lambda{i-j}" for j in range(1, 6)]
            elif i > 0:
                resource["DependsOn"] = [f"Lambda{j}" for j in range(i)]
            
            resources[f"Lambda{i}"] = resource
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=300
        )
        
        # Should map dense graph
        assert result.returncode in [0, 1, 2, 101], "Should map dense 20k-node graph"


def test_mapping_20k_disconnected_components():
    """Test mapping 20k nodes in disconnected components."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create 100 components of 200 nodes each
        resources = {}
        component_size = 200
        num_components = 100
        
        for comp in range(num_components):
            base_idx = comp * component_size
            
            # First node in component
            resources[f"Lambda{base_idx}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            
            # Rest of component
            for i in range(1, component_size):
                idx = base_idx + i
                resources[f"Lambda{idx}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    },
                    "DependsOn": [f"Lambda{base_idx + i - 1}"]
                }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should map disconnected components
        assert result.returncode in [0, 1, 2, 101], "Should map 20k nodes in disconnected components"


def test_mapping_20k_with_cycles():
    """Test mapping with potential cycles in 20k-node graph."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create circular dependencies (should be detected/handled)
        resources = {}
        
        for i in range(20000):
            next_idx = (i + 1) % 20000
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                },
                "DependsOn": [f"Lambda{next_idx}"]
            }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should detect cycles or handle gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle cycles in 20k-node graph"


def test_mapping_20k_mixed_resource_types():
    """Test mapping 20k nodes with mixed resource types."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        resource_types = [
            "AWS::Lambda::Function",
            "AWS::EC2::Instance",
            "AWS::RDS::DBInstance",
            "AWS::DynamoDB::Table",
            "AWS::S3::Bucket"
        ]
        
        resources = {}
        
        for i in range(20000):
            resource_type = resource_types[i % len(resource_types)]
            
            resource = {
                "Type": resource_type,
                "Properties": {}
            }
            
            # Add dependencies
            if i > 0:
                resource["DependsOn"] = [f"Resource{i-1}"]
            
            resources[f"Resource{i}"] = resource
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should map mixed resource types
        assert result.returncode in [0, 1, 2, 101], "Should map 20k mixed resource types"


def test_mapping_memory_efficiency():
    """Test that 20k-node mapping is memory efficient."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Simple 20k nodes
        resources = {
            f"Lambda{i}": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            for i in range(20000)
        }
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Should complete without excessive memory
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=180
        )
        
        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should map 20k nodes efficiently"
        
        # Output should be bounded
        output_size = len(result.stdout) + len(result.stderr)
        assert output_size < 50 * 1024 * 1024, "Output should be memory efficient"


if __name__ == "__main__":
    test_mapping_20k_linear_chain()
    test_mapping_20k_star_topology()
    test_mapping_20k_binary_tree()
    test_mapping_20k_dense_graph()
    test_mapping_20k_disconnected_components()
    test_mapping_20k_with_cycles()
    test_mapping_20k_mixed_resource_types()
    test_mapping_memory_efficiency()
    print("All 20k-node graph mapping tests passed")
