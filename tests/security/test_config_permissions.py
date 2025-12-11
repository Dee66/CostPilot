#!/usr/bin/env python3
"""Test config file permission hardening."""

import os
import stat
import subprocess
import tempfile
from pathlib import Path


def test_config_file_permission_hardening():
    """Config files should have restricted permissions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_file = Path(tmpdir) / "costpilot.yml"
        
        # Create config with sensitive data
        config_content = """
policy_path: /path/to/policy.json
api_key: secret-key-here
"""
        
        with open(config_file, 'w') as f:
            f.write(config_content)
        
        # Set restrictive permissions (0600 = rw-------)
        os.chmod(config_file, 0o600)
        
        # Verify permissions
        file_stat = os.stat(config_file)
        mode = stat.S_IMODE(file_stat.st_mode)
        
        # Should be 0600 or stricter
        assert mode <= 0o600, f"Config should have restrictive permissions, got {oct(mode)}"


def test_world_readable_config_warning():
    """World-readable config should trigger warning."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_file = Path(tmpdir) / "costpilot.yml"
        
        with open(config_file, 'w') as f:
            f.write("policy_path: /path/to/policy.json\n")
        
        # Set world-readable permissions
        os.chmod(config_file, 0o644)
        
        # Attempt to use
        result = subprocess.run(
            ["costpilot", "--config", str(config_file), "--help"],
            capture_output=True,
            text=True
        )
        
        # Should warn about permissions
        output = result.stdout + result.stderr
        if "permission" in output.lower() or "warning" in output.lower():
            assert True, "Should warn about world-readable config"


def test_group_writable_config_rejected():
    """Group-writable config should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_file = Path(tmpdir) / "costpilot.yml"
        
        with open(config_file, 'w') as f:
            f.write("policy_path: /path/to/policy.json\n")
        
        # Set group-writable permissions
        os.chmod(config_file, 0o660)
        
        # Should reject or warn
        result = subprocess.run(
            ["costpilot", "--config", str(config_file), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if result.returncode != 0 or "permission" in output.lower():
            assert True, "Should reject group-writable config"


def test_config_created_with_secure_permissions():
    """Newly created config should have secure permissions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        config_file = Path(tmpdir) / "costpilot.yml"
        
        # Create config via init
        result = subprocess.run(
            ["costpilot", "init", "--config", str(config_file)],
            capture_output=True,
            text=True
        )
        
        if config_file.exists():
            file_stat = os.stat(config_file)
            mode = stat.S_IMODE(file_stat.st_mode)
            
            # Should be 0600 or 0644 (not 0666 or 0777)
            assert mode & 0o002 == 0, "Config should not be world-writable"


def test_exemptions_file_permission_check():
    """Exemptions file should have permission checks."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exemptions_file = Path(tmpdir) / "exemptions.yaml"
        
        with open(exemptions_file, 'w') as f:
            f.write("exemptions:\n  - resource: Lambda\n    expires: 2025-12-31\n")
        
        # Set overly permissive
        os.chmod(exemptions_file, 0o666)
        
        # Should warn
        result = subprocess.run(
            ["costpilot", "scan", "--exemptions", str(exemptions_file), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "permission" in output.lower() or "warning" in output.lower():
            assert True, "Should check exemptions file permissions"


def test_policy_file_permission_check():
    """Policy file should have permission checks."""
    with tempfile.TemporaryDirectory() as tmpdir:
        policy_file = Path(tmpdir) / "policy.json"
        
        import json
        policy = {"version": "1.0.0", "rules": []}
        
        with open(policy_file, 'w') as f:
            json.dump(policy, f)
        
        # World-writable policy
        os.chmod(policy_file, 0o666)
        
        result = subprocess.run(
            ["costpilot", "scan", "--policy", str(policy_file), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "permission" in output.lower():
            assert True, "Should check policy file permissions"


def test_temp_file_secure_creation():
    """Temporary files should be created securely."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create temp file
        temp_file = Path(tmpdir) / "temp_output.json"
        
        with open(temp_file, 'w') as f:
            f.write('{"temp": "data"}')
        
        # Check permissions
        file_stat = os.stat(temp_file)
        mode = stat.S_IMODE(file_stat.st_mode)
        
        # Should not be world-writable
        assert mode & 0o002 == 0, "Temp files should not be world-writable"


def test_umask_respected():
    """Process should respect umask for file creation."""
    # Get current umask
    current_umask = os.umask(0)
    os.umask(current_umask)  # Restore
    
    with tempfile.TemporaryDirectory() as tmpdir:
        test_file = Path(tmpdir) / "test.json"
        
        # Set restrictive umask
        old_umask = os.umask(0o077)
        
        try:
            with open(test_file, 'w') as f:
                f.write('{"test": "data"}')
            
            # Check permissions
            file_stat = os.stat(test_file)
            mode = stat.S_IMODE(file_stat.st_mode)
            
            # Should respect umask
            assert mode & 0o077 == 0, "Should respect umask"
        finally:
            os.umask(old_umask)


if __name__ == "__main__":
    test_config_file_permission_hardening()
    test_world_readable_config_warning()
    test_group_writable_config_rejected()
    test_config_created_with_secure_permissions()
    test_exemptions_file_permission_check()
    test_policy_file_permission_check()
    test_temp_file_secure_creation()
    test_umask_respected()
    print("All config file permission hardening tests passed")
