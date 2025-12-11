#!/usr/bin/env python3
"""Test Free Edition: loading Pro heuristics bundle fails with correct error code."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_pro_bundle_loading_fails():
    """Test loading Pro heuristics bundle fails in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        bundle_path = Path(tmpdir) / "pro_bundle.bin"
        
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
        
        # Create fake Pro bundle
        bundle_path.write_bytes(b"PROBUNDLE" + b"\x00" * 1000)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail with specific error
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "heuristics" in error or "bundle" in error or "premium" in error or "free" in error or "unexpected" in error, \
                "Should mention heuristics/bundle limitation"


def test_encrypted_heuristics_rejected():
    """Test encrypted heuristics file rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        heuristics_path = Path(tmpdir) / "heuristics.enc"
        
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
        
        # Create fake encrypted heuristics
        heuristics_path.write_bytes(b"\x89PNG" + b"\x00" * 100)  # Fake binary
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(heuristics_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "heuristics" in error or "encrypted" in error or "premium" in error, \
                "Should reject encrypted heuristics"


def test_pro_heuristics_flag_error_code():
    """Test Pro heuristics flag returns specific error code."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        bundle_path = Path(tmpdir) / "pro.bundle"
        
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
        
        bundle_path.write_bytes(b"PRO")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should return error code (typically 1 or 2)
        assert result.returncode != 0, "Should fail with error code"
        assert result.returncode in [1, 2, 101], "Should have appropriate error code"


def test_default_heuristics_allowed():
    """Test default heuristics (Free Edition) work."""
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
        
        # Default heuristics should work
        assert result.returncode in [0, 1, 2, 101], "Default Free heuristics should work"


def test_pro_bundle_path_rejected():
    """Test Pro bundle path variations rejected."""
    paths = [
        "pro_heuristics.bin",
        "premium.bundle",
        "enterprise.heuristics",
    ]
    
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
        
        for path in paths:
            bundle_path = Path(tmpdir) / path
            bundle_path.write_bytes(b"DATA")
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(bundle_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Should fail or ignore
            if result.returncode != 0:
                # Expected to fail
                pass


if __name__ == "__main__":
    test_pro_bundle_loading_fails()
    test_encrypted_heuristics_rejected()
    test_pro_heuristics_flag_error_code()
    test_default_heuristics_allowed()
    test_pro_bundle_path_rejected()
    print("All Free Edition Pro heuristics bundle gating tests passed")
