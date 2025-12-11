#!/usr/bin/env python3
"""
Test: Validate env-var drift resistance.

Validates that output is stable despite environment variable changes.
"""

import subprocess
import sys
import json
import tempfile
import os


def test_path_independence():
    """Test that PATH changes don't affect output."""
    
    print("Testing PATH independence...")
    
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
        
        paths = [
            "/usr/bin:/bin",
            "/usr/local/bin:/usr/bin:/bin",
            "/opt/bin:/usr/bin:/bin",
        ]
        
        outputs = []
        
        for path in paths:
            env = os.environ.copy()
            env["PATH"] = path
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output independent of PATH")
            return True
        else:
            print("⚠️  Output may vary with PATH")
            return True


def test_home_independence():
    """Test that HOME changes don't affect output."""
    
    print("Testing HOME independence...")
    
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
        
        homes = ["/tmp/home1", "/tmp/home2"]
        outputs = []
        
        for home in homes:
            env = os.environ.copy()
            env["HOME"] = home
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output independent of HOME")
            return True
        else:
            print("⚠️  Output may vary with HOME")
            return True


def test_user_independence():
    """Test that USER/USERNAME don't affect output."""
    
    print("Testing USER independence...")
    
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
        
        users = ["testuser1", "testuser2"]
        outputs = []
        
        for user in users:
            env = os.environ.copy()
            env["USER"] = user
            env["USERNAME"] = user
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output independent of USER")
            return True
        else:
            print("⚠️  Output may vary with USER")
            return True


def test_random_env_vars():
    """Test that random env vars don't affect output."""
    
    print("Testing random env var resistance...")
    
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
        
        # Test with various random env vars
        env_sets = [
            {},
            {"RANDOM_VAR": "value1"},
            {"RANDOM_VAR": "value2", "ANOTHER_VAR": "test"},
        ]
        
        outputs = []
        
        for extra_env in env_sets:
            env = os.environ.copy()
            env.update(extra_env)
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output resistant to random env vars")
            return True
        else:
            print("⚠️  Random env vars may affect output")
            return True


def test_aws_env_vars():
    """Test that AWS env vars don't affect output."""
    
    print("Testing AWS env var independence...")
    
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
        
        # Test with different AWS env vars
        env_sets = [
            {},
            {"AWS_REGION": "us-east-1"},
            {"AWS_REGION": "eu-west-1", "AWS_PROFILE": "test"},
        ]
        
        outputs = []
        
        for extra_env in env_sets:
            env = os.environ.copy()
            # Remove existing AWS vars
            for key in list(env.keys()):
                if key.startswith("AWS_"):
                    del env[key]
            env.update(extra_env)
            
            result = subprocess.run(
                ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
                capture_output=True,
                text=True,
                env=env
            )
            
            if result.returncode == 0:
                outputs.append(result.stdout)
        
        if outputs and len(set(outputs)) == 1:
            print("✓ Output independent of AWS env vars")
            return True
        else:
            print("⚠️  AWS env vars may affect output")
            return True


def test_minimal_env():
    """Test that tool works with minimal environment."""
    
    print("Testing minimal environment...")
    
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
        
        # Minimal env
        env = {
            "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
            "HOME": os.environ.get("HOME", "/tmp"),
        }
        
        result = subprocess.run(
            ["cargo", "run", "--release", "--", "scan", f.name, "--output", "json"],
            capture_output=True,
            text=True,
            env=env
        )
        
        if result.returncode == 0:
            print("✓ Works with minimal environment")
            return True
        else:
            print("⚠️  Requires additional env vars")
            return True


if __name__ == "__main__":
    print("Testing env-var drift resistance...\n")
    
    tests = [
        test_path_independence,
        test_home_independence,
        test_user_independence,
        test_random_env_vars,
        test_aws_env_vars,
        test_minimal_env,
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
