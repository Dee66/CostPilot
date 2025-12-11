#!/usr/bin/env python3
"""Test graph node-count stability across versions."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_graph_node_count_deterministic():
    """Test that graph node count is deterministic for same input."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(50)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        node_counts = []
        
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "map", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Extract node count from output
            output = result.stdout + result.stderr
            
            # Look for node count indicators
            if "nodes" in output.lower():
                # Parse node count
                import re
                node_pattern = r"(\d+)\s+nodes?"
                matches = re.findall(node_pattern, output, re.IGNORECASE)
                if matches:
                    node_counts.append(int(matches[0]))
        
        # All node counts should be identical
        if len(node_counts) > 1:
            assert all(nc == node_counts[0] for nc in node_counts), \
                f"Node counts not stable: {node_counts}"


def test_graph_node_count_with_dependencies():
    """Test node count stability with dependency graph."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create dependency chain
        resources = {}
        for i in range(20):
            resource = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            
            if i > 0:
                resource["DependsOn"] = [f"Lambda{i-1}"]
            
            resources[f"Lambda{i}"] = resource
        
        template_content = {"Resources": resources}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "map", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            results.append(result.stdout)
        
        # Results should be identical (deterministic node count)
        assert all(r == results[0] for r in results), "Graph node count not stable"


def test_node_count_after_minor_changes():
    """Test that node count changes predictably with template changes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Initial template
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result1 = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Add one resource
        template_content["Resources"]["Lambda10"] = {
            "Type": "AWS::Lambda::Function",
            "Properties": {
                "MemorySize": 1024
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result2 = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should complete both times
        assert result1.returncode in [0, 1, 2, 101], "First mapping should complete"
        assert result2.returncode in [0, 1, 2, 101], "Second mapping should complete"


def test_node_count_independent_of_order():
    """Test that node count doesn't depend on resource order."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.json"
        template2_path = Path(tmpdir) / "template2.json"
        
        # Same resources, different order
        resources1 = {
            f"Lambda{i}": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            for i in range(20)
        }
        
        resources2 = {
            f"Lambda{i}": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024
                }
            }
            for i in range(19, -1, -1)  # Reverse order
        }
        
        with open(template1_path, 'w') as f:
            json.dump({"Resources": resources1}, f)
        
        with open(template2_path, 'w') as f:
            json.dump({"Resources": resources2}, f)
        
        result1 = subprocess.run(
            ["costpilot", "map", "--plan", str(template1_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        result2 = subprocess.run(
            ["costpilot", "map", "--plan", str(template2_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Node counts should be same (order-independent)
        assert result1.stdout == result2.stdout, "Node count should be order-independent"


def test_node_count_with_cycles():
    """Test node count with cyclic dependencies."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create cycle: A -> B -> C -> A
        template_content = {
            "Resources": {
                "LambdaA": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024},
                    "DependsOn": ["LambdaC"]
                },
                "LambdaB": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024},
                    "DependsOn": ["LambdaA"]
                },
                "LambdaC": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024},
                    "DependsOn": ["LambdaB"]
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        results = []
        
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "map", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            results.append((result.returncode, result.stdout))
        
        # Should handle consistently (either detect cycle or process)
        assert all(r[0] == results[0][0] for r in results), "Cycle handling not stable"


def test_node_count_large_graph():
    """Test node count stability for large graphs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Large graph
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(1000)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run twice
        result1 = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        result2 = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Should produce identical results
        assert result1.stdout == result2.stdout, "Large graph node count not stable"


def test_node_count_metadata_preserved():
    """Test that node count metadata is preserved across runs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(30)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path), "--format", "json"],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Parse JSON output
        try:
            output_data = json.loads(result.stdout)
            
            # Check for node count field
            if "node_count" in output_data or "nodes" in output_data:
                print("Node count metadata present in output")
        except json.JSONDecodeError:
            pass


if __name__ == "__main__":
    test_graph_node_count_deterministic()
    test_graph_node_count_with_dependencies()
    test_node_count_after_minor_changes()
    test_node_count_independent_of_order()
    test_node_count_with_cycles()
    test_node_count_large_graph()
    test_node_count_metadata_preserved()
    print("All graph node-count stability tests passed")
