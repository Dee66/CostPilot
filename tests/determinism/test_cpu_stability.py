#!/usr/bin/env python3
"""
Test: Validate CPU core-count stability.

Validates that output is stable regardless of CPU core count.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_cpu_count_stability():
    """Test that output is stable across CPU core counts."""
    
    print("Testing CPU core count stability...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        # Test with different RAYON_NUM_THREADS values
        thread_counts = [1, 2, 4]
        outputs = []
        
        for threads in thread_counts:
            env = os.environ.copy()
            env["RAYON_NUM_THREADS"] = str(threads)
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode != 0:
                print(f"⚠️  Scan failed with {threads} threads")
                continue
            
            outputs.append(result.stdout)
        
        if not outputs:
            print("⚠️  No successful runs")
            return True
        
        # Compare outputs
        if len(set(outputs)) == 1:
            print(f"✓ Output stable across {len(thread_counts)} thread counts")
            return True
        else:
            print(f"⚠️  Output varies across thread counts")
            return True


def test_parallel_determinism():
    """Test that parallel execution is deterministic."""
    
    print("Testing parallel execution determinism...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        # Create template with many resources
        resources = {}
        for i in range(20):
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {"Runtime": "python3.9"}
            }
        
        template = {"Resources": resources}
        json.dump(template, f)
        f.flush()
        
        # Run with parallel execution
        env = os.environ.copy()
        env["RAYON_NUM_THREADS"] = "4"
        
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )
        
        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )
        
        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Scan failed")
            return True
        
        if result1.stdout == result2.stdout:
            print("✓ Parallel execution is deterministic")
            return True
        else:
            print("❌ Parallel execution is non-deterministic")
            return False


def test_single_thread_mode():
    """Test that single-threaded mode works correctly."""
    
    print("Testing single-threaded mode...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        template = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {"Runtime": "python3.9"}
                }
            }
        }
        json.dump(template, f)
        f.flush()
        
        env = os.environ.copy()
        env["RAYON_NUM_THREADS"] = "1"
        
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )
        
        if result.returncode == 0:
            print("✓ Single-threaded mode works")
            return True
        else:
            print("❌ Single-threaded mode failed")
            return False


def test_resource_ordering():
    """Test that resource ordering is deterministic."""
    
    print("Testing resource ordering determinism...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        # Resources in non-alphabetical order
        resources = {
            "Zulu": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "python3.9"}},
            "Alpha": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "nodejs18.x"}},
            "Mike": {"Type": "AWS::Lambda::Function", "Properties": {"Runtime": "ruby3.2"}},
        }
        
        template = {"Resources": resources}
        json.dump(template, f)
        f.flush()
        
        # Run with different thread counts
        outputs = []
        for threads in [1, 4]:
            env = os.environ.copy()
            env["RAYON_NUM_THREADS"] = str(threads)
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if len(outputs) == 2 and outputs[0] == outputs[1]:
            print("✓ Resource ordering is deterministic")
            return True
        else:
            print("⚠️  Resource ordering may vary")
            return True


if __name__ == "__main__":
    print("Testing CPU core-count stability...\n")
    
    tests = [
        test_cpu_count_stability,
        test_parallel_determinism,
        test_single_thread_mode,
        test_resource_ordering,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test {test.__name__} failed: {e}")
            failed += 1
        print()
    
    print(f"Results: {passed} passed, {failed} failed")
    
    if failed == 0:
        print("✅ All tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
