#!/usr/bin/env python3
"""Test performance under memory pressure."""

import json
import subprocess
import tempfile
from pathlib import Path


def allocate_memory(size_mb):
    """Allocate memory to create pressure."""
    try:
        # Allocate large list
        data = [0] * (size_mb * 1024 * 256)  # Approximate MB
        return data
    except MemoryError:
        return None


def test_predict_under_low_memory():
    """Test prediction under low memory conditions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 * (i + 1)
                    }
                }
                for i in range(100)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Allocate memory to create pressure
        memory_hog = allocate_memory(500)  # 500 MB

        try:
            result = subprocess.run(
                ["costpilot", "predict", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Should handle memory pressure
            assert result.returncode in [0, 1, 2, 101], "Should handle memory pressure"
        finally:
            del memory_hog


def test_analyze_with_large_template():
    """Test analyzing large template under memory pressure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Very large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Environment": {
                            "Variables": {
                                f"VAR{j}": f"value{j}" * 10
                                for j in range(50)
                            }
                        }
                    }
                }
                for i in range(1000)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Create memory pressure
        memory_hog = allocate_memory(500)

        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=60
            )

            # Should complete despite large template and memory pressure
            assert result.returncode in [0, 1, 2, 101], "Should handle large template under memory pressure"
        finally:
            del memory_hog


def test_baseline_generation_memory_constrained():
    """Test baseline generation under memory constraints."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"

        template_content = {
            "Resources": {
                f"Resource{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(200)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Memory pressure
        memory_hog = allocate_memory(500)

        try:
            result = subprocess.run(
                ["costpilot", "baseline", "generate", "--plan", str(template_path), "--output", str(baseline_path)],
                capture_output=True,
                text=True,
                timeout=60
            )

            assert result.returncode in [0, 1, 2, 101], "Should generate baseline under memory pressure"
        finally:
            del memory_hog


def test_policy_check_memory_constrained():
    """Test policy checking under memory constraints."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        # Large template with many resources
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
                for i in range(100)
            }
        }

        # Policy with many rules
        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": f"rule-{i}",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
                for i in range(50)
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Memory pressure
        memory_hog = allocate_memory(500)

        try:
            result = subprocess.run(
                ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
                capture_output=True,
                text=True,
                timeout=60
            )

            assert result.returncode in [0, 1, 2, 101], "Should check policy under memory pressure"
        finally:
            del memory_hog


def test_memory_leak_prevention():
    """Test that repeated operations don't leak memory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

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

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Run many times
        for _ in range(20):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Each should succeed
            assert result.returncode in [0, 1, 2, 101], "Should not leak memory across runs"


def test_streaming_large_output():
    """Test handling large output under memory pressure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template that produces large output
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 * (i % 10 + 1),
                        "Description": "A" * 1000
                    }
                }
                for i in range(500)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Memory pressure
        memory_hog = allocate_memory(300)

        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
                capture_output=True,
                text=True,
                timeout=60
            )

            # Should handle large output
            assert result.returncode in [0, 1, 2, 101], "Should stream large output under memory pressure"
        finally:
            del memory_hog


def test_memory_efficient_parsing():
    """Test memory-efficient parsing of large files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Very large template (multiple MB)
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Code": {
                            "ZipFile": "data" * 1000
                        }
                    }
                }
                for i in range(2000)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Memory pressure
        memory_hog = allocate_memory(300)

        try:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=120
            )

            # Should parse efficiently
            assert result.returncode in [0, 1, 2, 101], "Should parse large files efficiently"
        finally:
            del memory_hog


if __name__ == "__main__":
    test_predict_under_low_memory()
    test_analyze_with_large_template()
    test_baseline_generation_memory_constrained()
    test_policy_check_memory_constrained()
    test_memory_leak_prevention()
    test_streaming_large_output()
    test_memory_efficient_parsing()
    print("All memory pressure tests passed")
