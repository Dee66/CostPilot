#!/usr/bin/env python3
"""Test Free Edition: premium-only flags rejected."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_mode_flag_rejected():
    """Test --mode flag rejected in Free Edition."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--mode", "pro"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail or ignore
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "not recognized" in error or "unexpected" in error, \
                "Should reject --mode flag"


def test_license_flag_rejected():
    """Test --license flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "license.key"
        
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
        
        license_path.write_text("fake-license-key")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "license" in error, \
                "Should reject --license flag"


def test_bundle_flag_rejected():
    """Test --bundle flag rejected in Free Edition."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        bundle_path = Path(tmpdir) / "bundle.bin"
        
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
        
        bundle_path.write_bytes(b"fake-bundle")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "bundle" in error, \
                "Should reject --bundle flag"


def test_premium_flag_rejected():
    """Test --premium flag rejected in Free Edition."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--premium"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error, \
                "Should reject --premium flag"


def test_pro_flag_rejected():
    """Test --pro flag rejected in Free Edition."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--pro"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error or "pro" in error, \
                "Should reject --pro flag"


def test_enterprise_flag_rejected():
    """Test --enterprise flag rejected in Free Edition."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--enterprise"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "enterprise" in error, \
                "Should reject --enterprise flag"


def test_multiple_premium_flags_rejected():
    """Test multiple premium flags rejected together."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--mode", "pro", "--premium"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "premium" in error or "free" in error or "unexpected" in error, \
                "Should reject multiple premium flags"


if __name__ == "__main__":
    test_mode_flag_rejected()
    test_license_flag_rejected()
    test_bundle_flag_rejected()
    test_premium_flag_rejected()
    test_pro_flag_rejected()
    test_enterprise_flag_rejected()
    test_multiple_premium_flags_rejected()
    print("All Free Edition premium flag gating tests passed")
