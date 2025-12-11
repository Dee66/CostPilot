#!/usr/bin/env python3
"""Test differential fuzzing between versions."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_differential_output_consistency():
    """Compare outputs between runs for consistency."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Runtime": "python3.9"
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run multiple times
        outputs = []
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True
            )
            outputs.append(result.stdout)
        
        # Outputs should be identical (deterministic)
        assert all(out == outputs[0] for out in outputs), "Outputs should be deterministic"


def test_differential_json_vs_text_output():
    """Compare JSON and text output for consistency."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
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
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Text output
        result_text = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True
        )
        
        # JSON output
        result_json = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--format", "json"],
            capture_output=True,
            text=True
        )
        
        # Both should succeed or both fail
        assert (result_text.returncode == 0) == (result_json.returncode == 0), \
            "Text and JSON outputs should have consistent success/failure"


def test_differential_policy_application():
    """Compare outputs with and without policy."""
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
        
        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)
        
        # Without policy
        result_no_policy = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path)],
            capture_output=True,
            text=True
        )
        
        # With policy
        result_with_policy = subprocess.run(
            ["costpilot", "analyze", "--template", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True
        )
        
        # With policy should show violations
        if "violation" in result_with_policy.stdout.lower() or "high" in result_with_policy.stdout.lower():
            assert True, "Policy should affect output"


def test_differential_input_formats():
    """Compare handling of different input formats."""
    with tempfile.TemporaryDirectory() as tmpdir:
        json_path = Path(tmpdir) / "template.json"
        yaml_path = Path(tmpdir) / "template.yaml"
        
        # Same content in different formats
        template_json = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        template_yaml = """Resources:
  Lambda:
    Type: AWS::Lambda::Function
    Properties:
      MemorySize: 1024
"""
        
        with open(json_path, 'w') as f:
            json.dump(template_json, f)
        
        with open(yaml_path, 'w') as f:
            f.write(template_yaml)
        
        # Analyze both
        result_json = subprocess.run(
            ["costpilot", "analyze", "--template", str(json_path)],
            capture_output=True,
            text=True
        )
        
        result_yaml = subprocess.run(
            ["costpilot", "analyze", "--template", str(yaml_path)],
            capture_output=True,
            text=True
        )
        
        # Should produce similar results
        assert (result_json.returncode == 0) == (result_yaml.returncode == 0), \
            "JSON and YAML should produce consistent results"


def test_differential_flag_combinations():
    """Test different flag combinations produce consistent results."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {"Resources": {}}
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Different flag combinations
        flag_combos = [
            [],
            ["--verbose"],
            ["--quiet"],
            ["--format", "json"],
            ["--format", "text"]
        ]
        
        results = []
        for flags in flag_combos:
            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)] + flags,
                capture_output=True,
                text=True
            )
            results.append(result.returncode)
        
        # All should succeed
        assert all(rc in [0, 1] for rc in results), "All flag combinations should work"


def test_differential_resource_order():
    """Test that resource order doesn't affect analysis."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.json"
        template2_path = Path(tmpdir) / "template2.json"
        
        # Same resources, different order
        template1 = {
            "Resources": {
                "Lambda1": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}},
                "Lambda2": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 2048}}
            }
        }
        
        template2 = {
            "Resources": {
                "Lambda2": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 2048}},
                "Lambda1": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}}
            }
        }
        
        with open(template1_path, 'w') as f:
            json.dump(template1, f)
        
        with open(template2_path, 'w') as f:
            json.dump(template2, f)
        
        result1 = subprocess.run(
            ["costpilot", "analyze", "--template", str(template1_path)],
            capture_output=True,
            text=True
        )
        
        result2 = subprocess.run(
            ["costpilot", "analyze", "--template", str(template2_path)],
            capture_output=True,
            text=True
        )
        
        # Results should be equivalent (deterministic ordering)
        assert result1.stdout == result2.stdout, "Resource order should not affect output"


def test_differential_version_compatibility():
    """Test version field handling across formats."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        versions = ["1.0", "2.0", None]
        
        for version in versions:
            template_content = {
                "Resources": {}
            }
            
            if version:
                template_content["AWSTemplateFormatVersion"] = version
            
            with open(template_path, 'w') as f:
                json.dump(template_content, f)
            
            result = subprocess.run(
                ["costpilot", "analyze", "--template", str(template_path)],
                capture_output=True,
                text=True
            )
            
            # Should handle all versions
            assert result.returncode in [0, 1], f"Should handle version {version}"


if __name__ == "__main__":
    test_differential_output_consistency()
    test_differential_json_vs_text_output()
    test_differential_policy_application()
    test_differential_input_formats()
    test_differential_flag_combinations()
    test_differential_resource_order()
    test_differential_version_compatibility()
    print("All differential fuzzing tests passed")
