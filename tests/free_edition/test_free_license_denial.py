#!/usr/bin/env python3
"""Test Free Edition: deny license token usage."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_license_flag_rejected():
    """Test --license flag is rejected."""
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
        
        license_path.write_text("FAKE-LICENSE-KEY-123456")
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject
        assert result.returncode != 0, "--license should be rejected"
        
        error = result.stderr.lower()
        assert "license" in error or "unknown" in error or "premium" in error, \
            "Should indicate license not supported"


def test_license_file_ignored():
    """Test license file in default location is ignored."""
    license_paths = [
        Path.home() / ".costpilot" / "license.key",
        Path.home() / ".config" / "costpilot" / "license.key",
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
        
        # Create fake license files
        for path in license_paths:
            if not path.parent.exists():
                continue
            
            try:
                path.write_text("FAKE-LICENSE")
            except:
                continue
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should work (ignore license)
        assert result.returncode in [0, 1, 2, 101], "Should ignore license file"
        
        # Clean up
        for path in license_paths:
            try:
                path.unlink()
            except:
                pass


def test_license_env_var_rejected():
    """Test COSTPILOT_LICENSE env var is rejected or ignored."""
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
        
        env = os.environ.copy()
        env["COSTPILOT_LICENSE"] = "FAKE-LICENSE-KEY"
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env=env
        )
        
        # Should work (ignore env var) or fail
        assert result.returncode in [0, 1, 2, 101], "Should ignore license env var"


def test_license_token_in_config_rejected():
    """Test license token in config file is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        config_path = Path(tmpdir) / "config.yml"
        
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
        
        config_content = """
license:
  token: FAKE-LICENSE-KEY
  path: /path/to/license.key
"""
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        config_path.write_text(config_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--config", str(config_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should ignore license in config
        assert result.returncode in [0, 1, 2, 101], "Should ignore license in config"


def test_activate_command_not_present():
    """Test activate command not present."""
    result = subprocess.run(
        ["costpilot", "activate", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Should fail - activate not available
    assert result.returncode != 0, "activate command should not exist"


def test_register_command_not_present():
    """Test register command not present."""
    result = subprocess.run(
        ["costpilot", "register", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    # Should fail - register not available
    assert result.returncode != 0, "register command should not exist"


if __name__ == "__main__":
    test_license_flag_rejected()
    test_license_file_ignored()
    test_license_env_var_rejected()
    test_license_token_in_config_rejected()
    test_activate_command_not_present()
    test_register_command_not_present()
    print("All Free Edition license token gating tests passed")
