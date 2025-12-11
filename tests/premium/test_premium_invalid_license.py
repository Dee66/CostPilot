#!/usr/bin/env python3
"""Test Premium: invalid license returns deterministic exit code."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_invalid_license_exit_code():
    """Test invalid license returns specific exit code."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "invalid.license"
        
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
        
        # Create invalid license
        license_path.write_text("INVALID-LICENSE-KEY-123456")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail with specific exit code
        assert result.returncode != 0, "Invalid license should fail"
        # Typically 2 for license/config errors
        assert result.returncode in [1, 2, 3], "Should have appropriate exit code"


def test_invalid_license_deterministic():
    """Test invalid license produces deterministic exit code."""
    exit_codes = []
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "invalid.license"
        
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
        
        license_path.write_text("INVALID-LICENSE")
        
        # Run 5 times
        for _ in range(5):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            exit_codes.append(result.returncode)
        
        # All should be identical
        assert len(set(exit_codes)) == 1, f"Exit codes should be deterministic: {exit_codes}"


def test_malformed_license_exit_code():
    """Test malformed license file returns consistent error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "malformed.license"
        
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
        
        # Create malformed license (binary garbage)
        license_path.write_bytes(b'\x00\x01\x02\x03')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Malformed license should fail"


def test_empty_license_exit_code():
    """Test empty license file returns error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "empty.license"
        
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
        
        # Create empty license
        license_path.touch()
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Empty license should fail"


def test_wrong_format_license():
    """Test wrong format license returns error."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "wrong.license"
        
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
        
        # Create license in wrong format (JSON instead of expected format)
        license_path.write_text('{"license": "invalid"}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Wrong format license should fail"


if __name__ == "__main__":
    test_invalid_license_exit_code()
    test_invalid_license_deterministic()
    test_malformed_license_exit_code()
    test_empty_license_exit_code()
    test_wrong_format_license()
    print("All Premium invalid license exit code tests passed")
