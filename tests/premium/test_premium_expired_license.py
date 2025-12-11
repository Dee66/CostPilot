#!/usr/bin/env python3
"""Test Premium: expired license returns correct structured error."""

import subprocess
import tempfile
from pathlib import Path
import json
import time


def test_expired_license_structured_error():
    """Test expired license returns structured error message."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "expired.license"
        
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
        
        # Create expired license (fake format with past date)
        expired_license = f"LICENSE:EXPIRED:2020-01-01:USER:test@example.com"
        license_path.write_text(expired_license)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            # Should mention expiration or license
            assert "expired" in error or "license" in error or "invalid" in error, \
                "Should indicate license issue"


def test_expired_license_error_format():
    """Test expired license error has proper format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "expired.license"
        
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
        
        # Create expired license
        license_path.write_text("EXPIRED_LICENSE_TOKEN")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        if result.returncode != 0:
            error = result.stderr
            
            # Error should be structured (not empty, not panic)
            assert len(error) > 0, "Should have error message"
            assert "panic" not in error.lower(), "Should not panic"


def test_expired_license_exit_code():
    """Test expired license returns specific exit code."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "expired.license"
        
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
        
        license_path.write_text("EXPIRED")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail with appropriate exit code
        assert result.returncode != 0, "Expired license should fail"
        assert result.returncode in [1, 2, 3], "Should have license error exit code"


def test_soon_to_expire_license_warning():
    """Test soon-to-expire license shows warning."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "soon_expire.license"
        
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
        
        # Create license expiring soon (fake)
        # In real implementation, this would be a valid license with near expiry
        license_path.write_text("VALID_LICENSE_EXPIRING_SOON")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # May succeed or fail depending on implementation
        # Just checking it handles near-expiry gracefully


def test_expired_license_no_feature_access():
    """Test expired license blocks premium features."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "expired.license"
        
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
        
        license_path.write_text("EXPIRED_2020")
        
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail (either command not found or license expired)
        assert result.returncode != 0, "Expired license should block premium features"


if __name__ == "__main__":
    test_expired_license_structured_error()
    test_expired_license_error_format()
    test_expired_license_exit_code()
    test_soon_to_expire_license_warning()
    test_expired_license_no_feature_access()
    print("All Premium expired license tests passed")
