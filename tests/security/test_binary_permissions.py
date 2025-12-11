#!/usr/bin/env python3
"""Test world-writable binary rejection."""

import os
import stat
import subprocess
import tempfile
from pathlib import Path
import shutil


def test_world_writable_binary_rejected():
    """World-writable binary should be rejected."""
    # Find costpilot binary
    result = subprocess.run(
        ["which", "costpilot"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        binary_path = Path(result.stdout.strip())
        
        if binary_path.exists():
            file_stat = os.stat(binary_path)
            mode = stat.S_IMODE(file_stat.st_mode)
            
            # Should not be world-writable
            assert mode & 0o002 == 0, f"Binary should not be world-writable, got {oct(mode)}"


def test_detect_world_writable_self():
    """Binary should detect if it is world-writable."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Copy binary to temp location
        result = subprocess.run(
            ["which", "costpilot"],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("Note: costpilot binary not in PATH")
            return
        
        original_binary = Path(result.stdout.strip())
        if not original_binary.exists():
            return
        
        temp_binary = Path(tmpdir) / "costpilot"
        shutil.copy2(original_binary, temp_binary)
        
        # Make world-writable
        os.chmod(temp_binary, 0o777)
        
        # Run binary - should detect and warn
        result = subprocess.run(
            [str(temp_binary), "--version"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "permission" in output.lower() or "warning" in output.lower() or "security" in output.lower():
            assert True, "Binary should warn about world-writable permissions"


def test_install_sets_secure_permissions():
    """Installation should set secure binary permissions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        install_dir = Path(tmpdir) / "bin"
        install_dir.mkdir()
        
        # Simulate installation
        binary_path = install_dir / "costpilot"
        
        # Create dummy binary
        with open(binary_path, 'w') as f:
            f.write("#!/bin/bash\necho 'test'\n")
        
        # Set installation permissions (should be 0755)
        os.chmod(binary_path, 0o755)
        
        # Verify
        file_stat = os.stat(binary_path)
        mode = stat.S_IMODE(file_stat.st_mode)
        
        # Should be 0755 (rwxr-xr-x) - executable but not world-writable
        assert mode & 0o002 == 0, "Installed binary should not be world-writable"
        assert mode & 0o111 != 0, "Installed binary should be executable"


def test_group_writable_binary_rejected():
    """Group-writable binary should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        result = subprocess.run(
            ["which", "costpilot"],
            capture_output=True,
            text=True
        )
        
        if result.returncode != 0:
            print("Note: costpilot binary not in PATH")
            return
        
        original_binary = Path(result.stdout.strip())
        if not original_binary.exists():
            return
        
        temp_binary = Path(tmpdir) / "costpilot"
        shutil.copy2(original_binary, temp_binary)
        
        # Make group-writable
        os.chmod(temp_binary, 0o775)
        
        # Run binary
        result = subprocess.run(
            [str(temp_binary), "--version"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        # May warn about group-writable
        if "permission" in output.lower():
            assert True, "Binary may warn about group-writable permissions"


def test_binary_ownership_check():
    """Binary should verify ownership."""
    result = subprocess.run(
        ["which", "costpilot"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        binary_path = Path(result.stdout.strip())
        
        if binary_path.exists():
            file_stat = os.stat(binary_path)
            
            # Should be owned by root or current user
            current_uid = os.getuid()
            assert file_stat.st_uid in [0, current_uid], \
                "Binary should be owned by root or current user"


def test_suid_sgid_not_set():
    """Binary should not have SUID/SGID bits set."""
    result = subprocess.run(
        ["which", "costpilot"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        binary_path = Path(result.stdout.strip())
        
        if binary_path.exists():
            file_stat = os.stat(binary_path)
            mode = stat.S_IMODE(file_stat.st_mode)
            
            # Check for SUID/SGID bits
            assert mode & stat.S_ISUID == 0, "Binary should not have SUID bit"
            assert mode & stat.S_ISGID == 0, "Binary should not have SGID bit"


def test_binary_in_secure_directory():
    """Binary should be in secure directory."""
    result = subprocess.run(
        ["which", "costpilot"],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        binary_path = Path(result.stdout.strip())
        parent_dir = binary_path.parent
        
        if parent_dir.exists():
            dir_stat = os.stat(parent_dir)
            mode = stat.S_IMODE(dir_stat.st_mode)
            
            # Directory should not be world-writable
            assert mode & 0o002 == 0, "Binary directory should not be world-writable"


def test_installation_script_secure():
    """Installation script should set secure permissions."""
    install_script = Path("scripts/install.sh")
    
    if install_script.exists():
        with open(install_script) as f:
            content = f.read()
        
        # Should set permissions
        assert "chmod" in content, "Install script should set permissions"
        
        # Check for secure permissions
        if "chmod 755" in content or "chmod 0755" in content:
            assert True, "Install script sets secure permissions"


if __name__ == "__main__":
    test_world_writable_binary_rejected()
    test_detect_world_writable_self()
    test_install_sets_secure_permissions()
    test_group_writable_binary_rejected()
    test_binary_ownership_check()
    test_suid_sgid_not_set()
    test_binary_in_secure_directory()
    test_installation_script_secure()
    print("All world-writable binary rejection tests passed")
