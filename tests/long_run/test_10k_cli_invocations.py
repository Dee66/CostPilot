#!/usr/bin/env python3
"""Test 10k CLI invocations stability."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_10k_cli_invocations():
    """Test 10,000 CLI invocations for stability."""
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
        
        errors = 0
        
        # Run 10,000 invocations
        for i in range(10000):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode not in [0, 1]:
                errors += 1
            
            # Progress reporting
            if (i + 1) % 1000 == 0:
                print(f"Completed {i + 1}/10000 invocations, errors: {errors}")
        
        error_rate = errors / 10000
        
        print(f"10k CLI invocations: {errors} errors ({error_rate:.4%})")
        
        # Accept up to 0.5% error rate
        assert error_rate < 0.005, f"Error rate {error_rate:.4%} too high"


def test_10k_cli_simulation():
    """Simulate 10k CLI invocations with 1000 iterations."""
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
        
        errors = 0
        
        # Simulate with 1000 invocations
        for i in range(1000):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode not in [0, 1]:
                errors += 1
            
            # Progress reporting
            if (i + 1) % 100 == 0:
                print(f"Completed {i + 1}/1000 invocations")
        
        error_rate = errors / 1000
        
        print(f"10k simulation: {errors} errors ({error_rate:.2%})")
        
        # Should have low error rate
        assert error_rate < 0.01, f"Error rate {error_rate:.2%} too high"


def test_mixed_commands_stability():
    """Test stability with mixed CLI commands."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"
        
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
        
        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory",
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
        
        commands = [
            ["costpilot", "scan", "--plan", str(template_path)],
            ["costpilot", "predict", "--plan", str(template_path)],
            ["costpilot", "check", "--plan", str(template_path), "--policy", str(policy_path)],
        ]
        
        errors = 0
        
        # Run 1000 mixed commands
        for i in range(1000):
            cmd = commands[i % len(commands)]
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode not in [0, 1]:
                errors += 1
        
        error_rate = errors / 1000
        
        print(f"Mixed commands: {errors} errors ({error_rate:.2%})")
        
        assert error_rate < 0.01, f"Error rate {error_rate:.2%} too high"


def test_concurrent_invocations():
    """Test concurrent CLI invocations."""
    import multiprocessing
    
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
        
        def run_invocation(_):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            return result.returncode in [0, 1]
        
        # Run 100 invocations concurrently (10 at a time)
        with multiprocessing.Pool(10) as pool:
            results = pool.map(run_invocation, range(100))
        
        successes = sum(results)
        errors = len(results) - successes
        
        print(f"Concurrent invocations: {successes} successes, {errors} errors")
        
        # Most should succeed
        assert successes >= 95, "Concurrent invocations should be stable"


def test_rapid_fire_invocations():
    """Test rapid-fire CLI invocations."""
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
        
        errors = 0
        
        # Rapid-fire 500 invocations (no delay)
        for _ in range(500):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode not in [0, 1]:
                errors += 1
        
        error_rate = errors / 500
        
        print(f"Rapid-fire: {errors} errors ({error_rate:.2%})")
        
        assert error_rate < 0.02, f"Rapid-fire error rate {error_rate:.2%} too high"


if __name__ == "__main__":
    import sys
    
    if "--full" in sys.argv:
        print("Running full 10k CLI invocations test...")
        test_10k_cli_invocations()
    else:
        print("Running 10k simulation (1000 invocations)...")
        test_10k_cli_simulation()
    
    test_mixed_commands_stability()
    test_concurrent_invocations()
    test_rapid_fire_invocations()
    print("All 10k CLI invocations tests passed")
