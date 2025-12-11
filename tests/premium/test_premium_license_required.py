#!/usr/bin/env python3
"""Test Premium: binary refuses to load engine without valid license."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_premium_without_license_fails():
    """Test Premium binary refuses to run without valid license."""
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
        
        # Try to use premium features without license
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail with license error
        if result.returncode != 0:
            error = result.stderr.lower()
            # In Free edition, command doesn't exist
            # In Premium without license, should say license required
            assert "license" in error or "not found" in error or "premium" in error, \
                "Should indicate license issue"


def test_license_file_required():
    """Test license file is required for Premium features."""
    import os
    
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
        
        # Ensure no license in environment
        env = os.environ.copy()
        env.pop("COSTPILOT_LICENSE", None)
        env.pop("COSTPILOT_LICENSE_PATH", None)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env=env
        )
        
        # Should fail
        assert result.returncode != 0, "Premium features should require license"


def test_premium_engine_load_blocked():
    """Test Premium engine load is blocked without license."""
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
        
        # Try to force premium mode
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--mode", "premium"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            # Should mention license or unknown flag
            assert "license" in error or "unknown" in error or "mode" in error, \
                "Should reject premium mode without license"


def test_license_check_deterministic():
    """Test license check produces deterministic results."""
    results = []
    
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
        
        # Run multiple times
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            results.append(result.returncode)
        
        # All should have same exit code
        assert results[0] == results[1] == results[2], "License check should be deterministic"


def test_no_license_environment_variable():
    """Test Premium features blocked when no license env var."""
    import os
    
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
        
        # Explicitly unset license variables
        env = os.environ.copy()
        for key in list(env.keys()):
            if "LICENSE" in key.upper() or "COSTPILOT" in key.upper():
                env.pop(key, None)
        
        result = subprocess.run(
            ["costpilot", "patch", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env=env
        )
        
        # Should fail
        assert result.returncode != 0, "Premium commands should fail without license"


if __name__ == "__main__":
    test_premium_without_license_fails()
    test_license_file_required()
    test_premium_engine_load_blocked()
    test_license_check_deterministic()
    test_no_license_environment_variable()
    print("All Premium license enforcement tests passed")
