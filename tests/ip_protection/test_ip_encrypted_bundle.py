#!/usr/bin/env python3
"""Test IP Protection: Encrypted heuristics bundle cannot be opened by Free edition."""

import subprocess
import tempfile
from pathlib import Path
import os


def test_free_cannot_load_encrypted_bundle():
    """Test Free edition cannot load encrypted bundle."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "premium.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Create dummy encrypted bundle
        with open(bundle_path, 'wb') as f:
            f.write(b"ENCRYPTED_BUNDLE:V1.0.0\n")
            f.write(b"\x00\x01\x02\x03\x04\x05")  # Binary encrypted data
        
        # Simple template
        with open(template_path, 'w') as f:
            f.write('{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function"}}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Free edition should reject encrypted bundle
        assert result.returncode != 0, "Free should reject encrypted bundle"
        output = (result.stdout + result.stderr).lower()
        assert "bundle" in output or "encrypted" in output or "premium" in output, \
            "Error should mention bundle/encrypted/premium"


def test_free_rejects_bundle_flag():
    """Test Free edition rejects --bundle flag."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "premium.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        with open(bundle_path, 'wb') as f:
            f.write(b"BUNDLE_DATA")
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail
        assert result.returncode != 0, "Free should not accept --bundle"


def test_encrypted_bundle_signature_check():
    """Test encrypted bundle signature validation in Free."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "signed.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Bundle with signature
        with open(bundle_path, 'wb') as f:
            f.write(b"BUNDLE:SIGNATURE:SHA256\n")
            f.write(b"ENCRYPTED_DATA\x00\x01\x02")
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Free should not load signed bundles
        assert result.returncode != 0, "Free should reject signed bundles"


def test_free_no_bundle_decrypt():
    """Test Free edition has no decryption capability."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "encrypted.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # AES-like encrypted data
        with open(bundle_path, 'wb') as f:
            f.write(b"AES256_ENCRYPTED\n")
            f.write(os.urandom(256))  # Random encrypted data
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should fail to decrypt
        assert result.returncode != 0, "Free should have no decryption"


def test_bundle_loading_error_deterministic():
    """Test bundle loading error is deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "premium.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        with open(bundle_path, 'wb') as f:
            f.write(b"ENCRYPTED_BUNDLE")
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        exit_codes = []
        for _ in range(3):
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            exit_codes.append(result.returncode)
        
        # All runs should have same exit code
        assert len(set(exit_codes)) == 1, "Bundle error should be deterministic"
        assert exit_codes[0] != 0, "Should fail"


if __name__ == "__main__":
    test_free_cannot_load_encrypted_bundle()
    test_free_rejects_bundle_flag()
    test_encrypted_bundle_signature_check()
    test_free_no_bundle_decrypt()
    test_bundle_loading_error_deterministic()
    print("All IP Protection: encrypted bundle tests passed")
