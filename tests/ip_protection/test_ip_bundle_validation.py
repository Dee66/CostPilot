#!/usr/bin/env python3
"""Test IP Protection: Premium bundle fails validation if modified."""

import subprocess
import tempfile
from pathlib import Path
import os


def test_bundle_bitflip_detection():
    """Test bundle validation detects bitflip."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "premium.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Create bundle with checksum
        bundle_content = b"BUNDLE:VERSION:1.0.0\n"
        bundle_content += b"CHECKSUM:SHA256:abcdef123456\n"
        bundle_content += b"DATA:" + b"X" * 100
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        # Flip a bit
        with open(bundle_path, 'r+b') as f:
            f.seek(50)
            byte = f.read(1)
            flipped = bytes([byte[0] ^ 0x01])
            f.seek(50)
            f.write(flipped)
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject modified bundle
        assert result.returncode != 0, "Should detect bitflip"
        output = (result.stdout + result.stderr).lower()
        assert any(word in output for word in ["tamper", "corrupt", "invalid", "checksum", "signature"]), \
            "Error should indicate tampering"


def test_bundle_signature_verification():
    """Test bundle signature verification fails on modification."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "signed.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Bundle with signature
        bundle_content = b"SIGNED_BUNDLE:V1\n"
        bundle_content += b"SIGNATURE:RSA2048:" + b"A" * 32 + b"\n"
        bundle_content += b"PAYLOAD:" + b"DATA" * 50
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        # Modify payload
        with open(bundle_path, 'r+b') as f:
            f.seek(-10, 2)  # Near end
            f.write(b"MODIFIED!!")
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject due to signature mismatch
        assert result.returncode != 0, "Should fail signature verification"


def test_bundle_hash_mismatch():
    """Test bundle hash mismatch detection."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "hashed.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Bundle with embedded hash
        original_data = b"HEURISTICS_DATA" * 20
        bundle_content = b"BUNDLE:HASH:SHA256\n"
        bundle_content += b"HASH:1234567890abcdef\n"
        bundle_content += original_data
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        # Corrupt data
        with open(bundle_path, 'r+b') as f:
            f.seek(100)
            f.write(b"CORRUPT")
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should detect hash mismatch
        assert result.returncode != 0, "Should detect hash mismatch"


def test_bundle_truncation_detection():
    """Test bundle truncation is detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "truncated.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Bundle with size header
        bundle_content = b"BUNDLE:SIZE:1000\n"
        bundle_content += b"X" * 500  # Only half the declared size
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should detect incomplete bundle
        assert result.returncode != 0, "Should detect truncation"


def test_bundle_header_modification():
    """Test bundle header modification is detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "header_mod.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Valid bundle
        bundle_content = b"BUNDLE:VERSION:1.0.0\n"
        bundle_content += b"SIGNATURE:" + b"S" * 64 + b"\n"
        bundle_content += b"DATA:" + b"X" * 100
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        # Modify version in header
        with open(bundle_path, 'r+b') as f:
            content = f.read()
            modified = content.replace(b"VERSION:1.0.0", b"VERSION:2.0.0")
            f.seek(0)
            f.write(modified)
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should detect header modification
        assert result.returncode != 0, "Should detect header modification"


def test_bundle_injection_attack():
    """Test bundle injection attack is prevented."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bundle_path = Path(tmpdir) / "injected.bundle"
        template_path = Path(tmpdir) / "template.json"
        
        # Bundle with injected malicious data
        bundle_content = b"BUNDLE:VERSION:1.0.0\n"
        bundle_content += b"DATA:LEGITIMATE\n"
        bundle_content += b"INJECTED:MALICIOUS_CODE\n"  # Injection attempt
        bundle_content += b"END"
        
        with open(bundle_path, 'wb') as f:
            f.write(bundle_content)
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--bundle", str(bundle_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject bundle with unexpected fields
        assert result.returncode != 0, "Should reject injected data"


if __name__ == "__main__":
    test_bundle_bitflip_detection()
    test_bundle_signature_verification()
    test_bundle_hash_mismatch()
    test_bundle_truncation_detection()
    test_bundle_header_modification()
    test_bundle_injection_attack()
    print("All IP Protection: bundle validation tests passed")
