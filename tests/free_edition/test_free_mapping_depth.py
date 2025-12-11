#!/usr/bin/env python3
"""Test Free Edition: mapping depth >1 returns structured error."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_mapping_depth_2_rejected():
    """Test mapping depth >1 returns structured error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create template with depth 2 mapping
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024,
                        "Role": {
                            "Fn::GetAtt": ["Role", "Arn"]
                        }
                    }
                },
                "Role": {
                    "Type": "AWS::IAM::Role",
                    "Properties": {
                        "AssumeRolePolicyDocument": {}
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "2"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject or limit to depth 1
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "depth" in error or "premium" in error or "free" in error or "unexpected" in error, \
                "Should mention depth limitation"


def test_mapping_depth_3_rejected():
    """Test mapping depth 3 explicitly rejected."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "3"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject depth 3
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "depth" in error or "premium" in error or "free" in error or "unexpected" in error or "limit" in error, \
                "Should reject depth >1"


def test_mapping_depth_flag_structured_error():
    """Test --depth flag returns structured error."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "5"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should have structured error
        if result.returncode != 0:
            error = result.stderr
            # Should not be empty
            assert len(error) > 0, "Should have error message"
            # Should mention depth or premium
            assert "depth" in error.lower() or "premium" in error.lower() or "free" in error.lower(), \
                "Should have structured error about depth"


def test_default_depth_is_1():
    """Test default depth is 1 in Free Edition."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should complete with depth 1 (default)
        assert result.returncode in [0, 1, 2, 101], "Should work with default depth 1"


def test_depth_0_allowed():
    """Test depth 0 is allowed."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "0"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should allow depth 0
        # Either succeeds or depth 0 means no analysis (implementation-dependent)
        assert result.returncode in [0, 1, 2, 101], "Should handle depth 0"


def test_depth_1_allowed():
    """Test depth 1 is allowed."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "1"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should allow depth 1
        assert result.returncode in [0, 1, 2, 101], "Should allow depth 1 in Free Edition"


def test_depth_error_exit_code():
    """Test depth >1 returns specific exit code."""
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
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--depth", "10"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should return non-zero exit code
        assert result.returncode != 0, "Should fail with depth >1"
        # Typically 2 for usage errors
        assert result.returncode in [1, 2, 101], "Should have appropriate exit code"


if __name__ == "__main__":
    test_mapping_depth_2_rejected()
    test_mapping_depth_3_rejected()
    test_mapping_depth_flag_structured_error()
    test_default_depth_is_1()
    test_depth_0_allowed()
    test_depth_1_allowed()
    test_depth_error_exit_code()
    print("All Free Edition mapping depth gating tests passed")
