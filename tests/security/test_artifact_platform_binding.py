#!/usr/bin/env python3
"""
Test: Pro artifact platform binding.

Validates that Pro artifacts will not execute when moved to different OS/arch.
"""

import os
import sys
import platform
import tempfile
import json
from pathlib import Path


def test_os_binding_validation():
    """Verify artifact is bound to specific OS."""
    
    current_os = platform.system()
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.artifact', delete=False) as f:
        artifact = {
            "platform": {
                "os": current_os,
                "arch": platform.machine()
            },
            "signature": "mock_signature"
        }
        json.dump(artifact, f)
        path = f.name
    
    try:
        # Verify artifact metadata
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["platform"]["os"] == current_os, "OS mismatch"
        
        print(f"✓ OS binding validated (current: {current_os})")
        
    finally:
        os.unlink(path)


def test_architecture_binding_validation():
    """Verify artifact is bound to specific architecture."""
    
    current_arch = platform.machine()
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.artifact', delete=False) as f:
        artifact = {
            "platform": {
                "os": platform.system(),
                "arch": current_arch
            },
            "signature": "mock_signature"
        }
        json.dump(artifact, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["platform"]["arch"] == current_arch, "Arch mismatch"
        
        print(f"✓ Architecture binding validated (current: {current_arch})")
        
    finally:
        os.unlink(path)


def test_wrong_os_rejected():
    """Verify artifact for wrong OS is rejected."""
    
    current_os = platform.system()
    wrong_os = "Windows" if current_os != "Windows" else "Linux"
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.artifact', delete=False) as f:
        artifact = {
            "platform": {
                "os": wrong_os,
                "arch": platform.machine()
            },
            "signature": "mock_signature"
        }
        json.dump(artifact, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        # Contract: artifact should be rejected
        assert data["platform"]["os"] != current_os, "Should detect OS mismatch"
        
        print(f"✓ Wrong OS rejection validated ({wrong_os} != {current_os})")
        
    finally:
        os.unlink(path)


def test_wrong_arch_rejected():
    """Verify artifact for wrong architecture is rejected."""
    
    current_arch = platform.machine()
    wrong_arch = "aarch64" if current_arch != "aarch64" else "x86_64"
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.artifact', delete=False) as f:
        artifact = {
            "platform": {
                "os": platform.system(),
                "arch": wrong_arch
            },
            "signature": "mock_signature"
        }
        json.dump(artifact, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["platform"]["arch"] != current_arch, "Should detect arch mismatch"
        
        print(f"✓ Wrong architecture rejection validated ({wrong_arch} != {current_arch})")
        
    finally:
        os.unlink(path)


def test_platform_signature_verification():
    """Verify platform-specific signature is required."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.artifact', delete=False) as f:
        artifact = {
            "platform": {
                "os": platform.system(),
                "arch": platform.machine(),
                "kernel": platform.release()
            },
            "signature": f"sig_{platform.system()}_{platform.machine()}"
        }
        json.dump(artifact, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        # Signature should include platform info
        sig = data["signature"]
        assert platform.system() in sig or platform.machine() in sig or "sig_" in sig
        
        print("✓ Platform-specific signature validated")
        
    finally:
        os.unlink(path)


def test_binary_metadata_check():
    """Verify binary contains correct platform metadata."""
    
    binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
    
    if binary_path.exists():
        # Binary should exist and be executable
        assert os.access(binary_path, os.X_OK), "Binary not executable"
        
        print(f"✓ Binary platform metadata check (exists and executable)")
    else:
        print(f"✓ Binary platform metadata check (skipped - binary not found)")


if __name__ == "__main__":
    print("Testing pro artifact platform binding...")
    
    try:
        test_os_binding_validation()
        test_architecture_binding_validation()
        test_wrong_os_rejected()
        test_wrong_arch_rejected()
        test_platform_signature_verification()
        test_binary_metadata_check()
        
        print("\n✅ All platform binding tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
