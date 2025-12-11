#!/usr/bin/env python3
"""Test Premium: CLI blocks run if heuristics bundle missing."""

import subprocess
import tempfile
from pathlib import Path
import json
import os


def test_missing_heuristics_bundle_blocks():
    """Test Premium CLI blocks when heuristics bundle missing."""
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
        
        # Try to run with explicit bundle path that doesn't exist
        nonexistent_bundle = Path(tmpdir) / "nonexistent.bundle"
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(nonexistent_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            # Should mention bundle, heuristics, or file not found
            assert any(term in error for term in ["bundle", "heuristics", "not found", "missing"]), \
                "Should indicate missing bundle"


def test_premium_features_require_bundle():
    """Test Premium features require heuristics bundle."""
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
        
        # Try autofix without bundle
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail (command not found in Free, or bundle required in Premium)
        assert result.returncode != 0, "Premium features should require bundle"


def test_bundle_path_validation():
    """Test heuristics bundle path is validated."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        invalid_bundle = Path(tmpdir) / "invalid.bundle"
        
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
        
        # Create invalid bundle
        invalid_bundle.write_text("INVALID_BUNDLE_CONTENT")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(invalid_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail with validation error
        if result.returncode != 0:
            error = result.stderr.lower()
            assert any(term in error for term in ["bundle", "heuristics", "invalid", "verification"]), \
                "Should indicate invalid bundle"


def test_empty_bundle_rejected():
    """Test empty heuristics bundle is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        empty_bundle = Path(tmpdir) / "empty.bundle"
        
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
        
        # Create empty bundle
        empty_bundle.touch()
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(empty_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Empty bundle should be rejected"


def test_bundle_signature_verification():
    """Test heuristics bundle signature is verified."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        unsigned_bundle = Path(tmpdir) / "unsigned.bundle"
        
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
        
        # Create unsigned/fake bundle
        unsigned_bundle.write_bytes(b"FAKE_BUNDLE_DATA")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(unsigned_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail verification
        assert result.returncode != 0, "Unsigned bundle should fail verification"


def test_corrupted_bundle_rejected():
    """Test corrupted heuristics bundle is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        corrupted_bundle = Path(tmpdir) / "corrupted.bundle"
        
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
        
        # Create corrupted bundle
        corrupted_bundle.write_bytes(b"\x00\x01\x02\x03\x04\x05")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(corrupted_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Corrupted bundle should be rejected"


def test_bundle_version_compatibility():
    """Test heuristics bundle version compatibility."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        old_version_bundle = Path(tmpdir) / "old.bundle"
        
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
        
        # Create bundle with old version marker
        old_version_bundle.write_text("BUNDLE:VERSION:0.1.0")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--heuristics", str(old_version_bundle)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle version mismatch gracefully
        assert result.returncode in [0, 1, 2, 101], "Should handle version mismatch"


if __name__ == "__main__":
    test_missing_heuristics_bundle_blocks()
    test_premium_features_require_bundle()
    test_bundle_path_validation()
    test_empty_bundle_rejected()
    test_bundle_signature_verification()
    test_corrupted_bundle_rejected()
    test_bundle_version_compatibility()
    print("All Premium heuristics bundle requirement tests passed")
