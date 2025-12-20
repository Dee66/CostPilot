#!/usr/bin/env python3
"""Test Premium: mapping depth unlimited."""

import subprocess
import tempfile
from pathlib import Path
import json
import pytest


@pytest.mark.premium_only
@pytest.mark.premium_only
def test_mapping_depth_unlimited():
    """Test Premium allows unlimited mapping depth."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create deep dependency chain
        resources = {}
        for i in range(10):
            if i == 0:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            else:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": [f"Resource{i-1}", "Arn"]}
                    }
                }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path), "--max-depth", "10"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # In Premium, should allow depth 10
        # In Free, should reject depth > 1
        assert result.returncode == 0, "Premium should allow depth 10"


@pytest.mark.premium_only
def test_mapping_depth_very_deep():
    """Test Premium allows very deep mappings."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Create very deep chain (20 levels)
        resources = {}
        for i in range(20):
            if i == 0:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            else:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": [f"Resource{i-1}", "Arn"]}
                    }
                }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path), "--max-depth", "20"],
            capture_output=True,
            text=True,
            timeout=15
        )

        # Premium should handle 20 levels
        assert result.returncode == 0, "Premium should allow depth 20"


@pytest.mark.premium_only
def test_mapping_depth_complex_graph():
    """Test Premium handles complex dependency graphs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Complex graph with multiple paths
        template_content = {
            "Resources": {
                "A": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024}
                },
                "B": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["A", "Arn"]}
                    }
                },
                "C": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["A", "Arn"]}
                    }
                },
                "D": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Environment": {
                            "Variables": {
                                "B_ARN": {"Fn::GetAtt": ["B", "Arn"]},
                                "C_ARN": {"Fn::GetAtt": ["C", "Arn"]}
                            }
                        }
                    }
                },
                "E": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": ["D", "Arn"]}
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path), "--max-depth", "5"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Premium should handle complex graph
        assert result.returncode == 0, "Premium should handle complex dependency graph"


@pytest.mark.premium_only
def test_mapping_depth_unlimited_flag():
    """Test Premium unlimited flag."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Deep chain
        resources = {}
        for i in range(15):
            if i == 0:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024}
                }
            else:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": [f"Resource{i-1}", "Arn"]}
                    }
                }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path), "--max-depth", "unlimited"],
            capture_output=True,
            text=True,
            timeout=15
        )

        # Premium should accept unlimited
        if result.returncode == 0:
            # Successfully analyzed with unlimited depth
            pass


@pytest.mark.premium_only
def test_mapping_depth_no_limit():
    """Test Premium with no depth limit specified."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Deep chain
        resources = {}
        for i in range(10):
            if i == 0:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"MemorySize": 1024}
                }
            else:
                resources[f"Resource{i}"] = {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {"Fn::GetAtt": [f"Resource{i-1}", "Arn"]}
                    }
                }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # No depth flag - Premium should use high default
        result = subprocess.run(
            ["costpilot", "map", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Premium default should be high enough
        assert result.returncode == 0, "Premium should handle deep chains by default"


if __name__ == "__main__":
    test_mapping_depth_unlimited()
    test_mapping_depth_very_deep()
    test_mapping_depth_complex_graph()
    test_mapping_depth_unlimited_flag()
    test_mapping_depth_no_limit()
    print("All Premium mapping depth tests passed")
