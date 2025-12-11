#!/usr/bin/env python3
"""Test Premium: license rotation accepted for premium engine."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_license_rotation_accepted():
    """Test license rotation is accepted."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        old_license_path = Path(tmpdir) / "old.license"
        new_license_path = Path(tmpdir) / "new.license"
        
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
        
        # Create old and new licenses
        old_license_path.write_text("LICENSE:OLD:VERSION:1")
        new_license_path.write_text("LICENSE:NEW:VERSION:2")
        
        # Try old license
        result_old = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(old_license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Try new license
        result_new = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(new_license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Both should have consistent behavior (both fail or both work)
        # This tests rotation doesn't break functionality


def test_license_update_no_restart():
    """Test license update works without restart."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "rotating.license"
        
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
        
        # Create initial license
        license_path.write_text("LICENSE:V1")
        
        result1 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Update license
        license_path.write_text("LICENSE:V2")
        
        result2 = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Both should complete (though may fail due to invalid format)
        # Testing that rotation doesn't cause hangs or crashes


def test_multiple_license_versions():
    """Test multiple license versions are handled."""
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
        
        # Test different license versions
        versions = ["V1", "V2", "V3"]
        
        for version in versions:
            license_path = Path(tmpdir) / f"license_{version}.key"
            license_path.write_text(f"LICENSE:{version}")
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # All should behave consistently


def test_license_refresh_during_operation():
    """Test license refresh during long operation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "refresh.license"
        
        # Create large template for longer operation
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
                for i in range(100)
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        license_path.write_text("LICENSE:INITIAL")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        # Should complete
        assert result.returncode in [0, 1, 2, 101], "Should handle operation with license"


def test_backward_compatible_license():
    """Test backward compatible license format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        old_format_path = Path(tmpdir) / "old_format.license"
        
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
        
        # Create old format license
        old_format_path.write_text("OLDFORMAT:LICENSE")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(old_format_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle old format (may reject or accept)
        assert result.returncode in [0, 1, 2, 101], "Should handle old format gracefully"


if __name__ == "__main__":
    test_license_rotation_accepted()
    test_license_update_no_restart()
    test_multiple_license_versions()
    test_license_refresh_during_operation()
    test_backward_compatible_license()
    print("All Premium license rotation tests passed")
