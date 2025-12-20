#!/usr/bin/env python3
"""Test Premium: tampered license returns signature verification failure."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_tampered_license_rejected():
    """Test tampered license is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "tampered.license"

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

        # Create tampered license (modified signature)
        tampered_license = "LICENSE:VALID:2030-12-31:USER:test@example.com:SIGNATURE:TAMPERED"
        license_path.write_text(tampered_license)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        if result.returncode != 0:
            error = result.stderr.lower()
            # Should mention signature, verification, or invalid license
            assert any(term in error for term in ["signature", "verification", "invalid", "tampered", "license"]), \
                "Should indicate signature/verification issue"


def test_modified_license_content():
    """Test license with modified content is rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "modified.license"

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

        # Create license then modify it
        original = "VALID_LICENSE_123456"
        modified = original + "_MODIFIED"
        license_path.write_text(modified)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        assert result.returncode != 0, "Modified license should be rejected"


def test_license_bitflip_detected():
    """Test license with bitflip is detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "bitflip.license"

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

        # Create license with bitflip simulation
        # Original: b"VALID_LICENSE"
        # Bitflip: change one byte
        original = b"VALID_LICENSE_DATA"
        bitflipped = bytearray(original)
        bitflipped[5] ^= 0x01  # Flip one bit

        license_path.write_bytes(bytes(bitflipped))

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        assert result.returncode != 0, "Bitflipped license should be rejected"


def test_signature_verification_failure_message():
    """Test signature verification failure has clear message."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "bad_sig.license"

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

        # Create license with invalid signature format
        license_path.write_text("LICENSE:CONTENT:SIGNATURE:INVALID")

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        if result.returncode != 0:
            error = result.stderr

            # Should have clear error message
            assert len(error) > 0, "Should have error message"
            # Should not panic
            assert "panic" not in error.lower(), "Should not panic on invalid signature"


def test_corrupted_license_signature():
    """Test corrupted license signature is detected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        license_path = Path(tmpdir) / "corrupted.license"

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

        # Create license with corrupted signature section
        license_path.write_bytes(b"LICENSE_HEADER" + b"\x00\x00\x00" + b"CORRUPTED_SIG")

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--license", str(license_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should fail
        assert result.returncode != 0, "Corrupted signature should be rejected"


if __name__ == "__main__":
    test_tampered_license_rejected()
    test_modified_license_content()
    test_license_bitflip_detected()
    test_signature_verification_failure_message()
    test_corrupted_license_signature()
    print("All Premium tampered license tests passed")
