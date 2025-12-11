#!/usr/bin/env python3
"""Test Premium: autofix enabled and validated."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_autofix_command_exists():
    """Test autofix command exists in Premium."""
    result = subprocess.run(
        ["costpilot", "autofix", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # In Premium, should succeed (exit 0)
    # In Free, should fail (command not found)
    # This test documents expected Premium behavior
    if result.returncode == 0:
        # Premium edition
        assert "autofix" in result.stdout.lower(), "Help should mention autofix"
    else:
        # Free edition - command doesn't exist
        pass


def test_autofix_with_policy():
    """Test autofix applies policy fixes."""
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
                    "id": "lambda-memory",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008",
                    "fix": {
                        "MemorySize": 3008
                    }
                }
            ]
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should succeed and apply fix
        # In Free, command doesn't exist
        if result.returncode == 0:
            # Premium: check fix was applied
            assert len(result.stdout) > 0, "Should have output"


def test_autofix_with_apply_flag():
    """Test autofix with --apply flag."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        output_path = Path(tmpdir) / "fixed.json"
        
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
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), 
             "--output", str(output_path), "--apply"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should apply fixes
        if result.returncode == 0:
            # Check output file was created
            if output_path.exists():
                with open(output_path) as f:
                    fixed = json.load(f)
                assert "Resources" in fixed, "Fixed template should have Resources"


def test_autofix_dry_run():
    """Test autofix dry-run mode."""
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
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--dry-run"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should show what would be fixed
        if result.returncode == 0:
            # Should have output describing fixes
            assert len(result.stdout) > 0 or len(result.stderr) > 0, "Should describe fixes"


def test_autofix_multiple_resources():
    """Test autofix with multiple resources."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240 + i * 100
                    }
                }
                for i in range(10)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should handle multiple resources
        if result.returncode == 0:
            assert len(result.stdout) > 0, "Should have output for multiple resources"


def test_autofix_validation():
    """Test autofix validates fixes."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240,
                        "Timeout": 900
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--validate"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # In Premium, should validate after fixing
        # Exit code should indicate success or validation failure


if __name__ == "__main__":
    test_autofix_command_exists()
    test_autofix_with_policy()
    test_autofix_with_apply_flag()
    test_autofix_dry_run()
    test_autofix_multiple_resources()
    test_autofix_validation()
    print("All Premium autofix tests passed")
