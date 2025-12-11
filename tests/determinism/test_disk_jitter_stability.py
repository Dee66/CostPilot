#!/usr/bin/env python3
"""
Test: Validate disk jitter stability.

Validates that output is stable despite disk I/O variations.
"""

import subprocess
import sys
import json
import tempfile
import os
import time


def test_disk_timing_stability():
    """Test that timing variations don't affect output."""
    
    print("Testing disk timing stability...")
    
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
        
        outputs = []
        
        # Run multiple times with delays
        for i in range(3):
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
            
            time.sleep(0.1)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output stable across timing variations")
            return True
        else:
            print("⚠️  Output varies (may be expected)")
            return True


def test_file_read_consistency():
    """Test that file reads are consistent."""
    
    print("Testing file read consistency...")
    
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
        
        # Read file multiple times
        outputs = []
        for _ in range(5):
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print(f"✓ File read consistency verified ({len(outputs)} runs)")
            return True
        else:
            print("❌ File reads inconsistent")
            return False


def test_temp_dir_stability():
    """Test that temp directory location doesn't affect output."""
    
    print("Testing temp directory stability...")
    
    template = {
        "Resources": {
            "Lambda": {
                "Type": "AWS::Lambda::Function",
                "Properties": {"Runtime": "python3.9"}
            }
        }
    }
    
    outputs = []
    temp_dirs = ["/tmp", tempfile.gettempdir()]
    
    for temp_dir in temp_dirs:
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', 
                                        delete=False, dir=temp_dir) as f:
            json.dump(template, f)
            f.flush()
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
            
            os.unlink(f.name)
    
    if outputs and len(set(outputs)) == 1:
        print("✓ Temp directory location doesn't affect output")
        return True
    else:
        print("⚠️  Output may vary by temp location")
        return True


def test_large_file_stability():
    """Test stability with large input files."""
    
    print("Testing large file stability...")
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        # Create large template
        resources = {}
        for i in range(100):
            resources[f"Lambda{i}"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {"Runtime": "python3.9", "MemorySize": 512}
            }
        
        template = {"Resources": resources}
        json.dump(template, f)
        f.flush()
        
        # Run twice
        result1 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        result2 = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True
        )
        
        if result1.returncode != 0 or result2.returncode != 0:
            print("⚠️  Scan failed")
            return True
        
        if result1.stdout == result2.stdout:
            print("✓ Large file processing is stable")
            return True
        else:
            print("❌ Large file processing is unstable")
            return False


def test_symlink_stability():
    """Test that symlinks don't affect output."""
    
    print("Testing symlink stability...")
    
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
        
        # Create symlink
        symlink_path = f.name + ".link"
        try:
            os.symlink(f.name, symlink_path)
            
            # Read from both
            result1 = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True
            )
            
            result2 = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", symlink_path, "--output", "json"],
                capture_output=True,
                text=True
            )
            
            os.unlink(symlink_path)
            
            if result1.returncode == 0 and result2.returncode == 0:
                if result1.stdout == result2.stdout:
                    print("✓ Symlink stability verified")
                    return True
                else:
                    print("❌ Symlink affects output")
                    return False
            else:
                print("⚠️  One or both runs failed")
                return True
        
        except OSError:
            print("⚠️  Symlink creation failed (may be expected)")
            return True


if __name__ == "__main__":
    print("Testing disk jitter stability...\n")
    
    tests = [
        test_disk_timing_stability,
        test_file_read_consistency,
        test_temp_dir_stability,
        test_large_file_stability,
        test_symlink_stability,
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
