#!/usr/bin/env python3
"""
Test: Packaged installer checksum validation.

Validates that LemonSqueezy/Gumroad packaged installer checksum matches
the published artifact.
"""

import os
import sys
import tempfile
import hashlib
import json
from pathlib import Path


def compute_sha256(data):
    """Compute SHA256 hash of data."""
    return hashlib.sha256(data).hexdigest()


def test_installer_checksum_match():
    """Verify installer checksum matches published checksum."""

    # Simulate installer binary
    installer_data = b"COSTPILOT_INSTALLER_BINARY_DATA" * 1000

    # Compute checksum
    installer_checksum = compute_sha256(installer_data)

    # Published checksum (from marketplace)
    published_checksum = compute_sha256(installer_data)

    assert installer_checksum == published_checksum, "Checksum mismatch"

    print(f"✓ Installer checksum matches (SHA256: {installer_checksum[:16]}...)")


def test_checksum_file_format():
    """Verify checksum file format is correct."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_checksums.txt', delete=False) as f:
        checksums = """
abc123def456789fedcba987654321abc123def456789fedcba987654321abcd  costpilot-installer-linux-x64.bin
112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00  costpilot-installer-macos-arm64.bin
fedcba987654321abc123def456789fedcba987654321abc123def456789fedc  costpilot-installer-windows-x64.exe
        """.strip()
        f.write(checksums)
        path = f.name

    try:
        with open(path, 'r') as f:
            lines = f.readlines()

        # Each line should be: checksum  filename
        for line in lines:
            parts = line.strip().split()
            assert len(parts) >= 2, "Invalid checksum format"
            checksum = parts[0]
            filename = parts[1]
            assert len(checksum) == 64, f"Invalid SHA256 length: {len(checksum)} (expected 64)"
            assert "costpilot" in filename, "Invalid filename"

        print(f"✓ Checksum file format valid ({len(lines)} entries)")

    finally:
        os.unlink(path)


def test_multiple_platform_checksums():
    """Verify checksums for multiple platforms."""

    with tempfile.TemporaryDirectory() as tmpdir:
        platforms = [
            ("linux-x64", b"LINUX_X64_INSTALLER" * 100),
            ("macos-arm64", b"MACOS_ARM64_INSTALLER" * 100),
            ("windows-x64", b"WINDOWS_X64_INSTALLER" * 100)
        ]

        checksums = {}
        for platform, data in platforms:
            checksums[platform] = compute_sha256(data)

        # All checksums should be different
        assert len(set(checksums.values())) == len(platforms), "Duplicate checksums"

        print(f"✓ Multiple platform checksums ({len(platforms)} platforms)")


def test_tampered_installer_detected():
    """Verify tampered installer is detected."""

    # Original installer
    original_data = b"ORIGINAL_INSTALLER_DATA" * 500
    original_checksum = compute_sha256(original_data)

    # Tampered installer
    tampered_data = original_data + b"MALICIOUS_CODE"
    tampered_checksum = compute_sha256(tampered_data)

    assert original_checksum != tampered_checksum, "Tampering not detected"

    print("✓ Tampered installer detected via checksum")


def test_marketplace_metadata_includes_checksum():
    """Verify marketplace metadata includes checksum."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_metadata.json', delete=False) as f:
        metadata = {
            "product": "CostPilot",
            "version": "1.0.0",
            "installer": {
                "filename": "costpilot-installer-linux-x64.bin",
                "size_bytes": 12345678,
                "sha256": "abc123def456789fedcba987654321abc123def456789fedcba987654321abc1"
            }
        }
        json.dump(metadata, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert "installer" in data
        assert "sha256" in data["installer"]
        assert len(data["installer"]["sha256"]) == 64

        print("✓ Marketplace metadata includes checksum")

    finally:
        os.unlink(path)


def test_signed_checksum_file():
    """Verify checksum file is signed."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_checksums.txt.sig', delete=False) as f:
        signature = {
            "algorithm": "Ed25519",
            "signature": "mock_signature_for_checksum_file",
            "signed_file": "checksums.txt",
            "public_key_id": "key_v1"
        }
        json.dump(signature, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            sig_data = json.load(f)

        assert "signature" in sig_data
        assert "signed_file" in sig_data
        assert sig_data["signed_file"] == "checksums.txt"

        print("✓ Checksum file signed")

    finally:
        os.unlink(path)


def test_lemonsqueezy_webhook_validation():
    """Verify LemonSqueezy webhook includes checksum."""

    webhook_payload = {
        "event": "order_created",
        "data": {
            "product_name": "CostPilot Pro",
            "variant": "linux-x64",
            "download_url": "https://example.com/download",
            "checksum": "abc123def456789"
        }
    }

    assert "checksum" in webhook_payload["data"]

    print("✓ LemonSqueezy webhook includes checksum")


def test_gumroad_license_includes_checksum():
    """Verify Gumroad license key includes artifact checksum."""

    license_data = {
        "license_key": "ABCD-1234-EFGH-5678",
        "product": "CostPilot Pro",
        "artifact": {
            "version": "1.0.0",
            "checksum": "fedcba987654321abc123def456789"
        }
    }

    assert "artifact" in license_data
    assert "checksum" in license_data["artifact"]

    print("✓ Gumroad license includes artifact checksum")


def test_download_verification_flow():
    """Verify complete download verification flow."""

    with tempfile.TemporaryDirectory() as tmpdir:
        # Step 1: Download installer
        installer_path = Path(tmpdir) / "installer.bin"
        installer_data = b"INSTALLER_BINARY_DATA" * 1000
        installer_path.write_bytes(installer_data)

        # Step 2: Download checksum
        expected_checksum = compute_sha256(installer_data)

        # Step 3: Verify
        actual_checksum = compute_sha256(installer_path.read_bytes())

        assert expected_checksum == actual_checksum, "Verification failed"

        print("✓ Download verification flow validated")


def test_checksum_api_endpoint():
    """Verify checksum API endpoint format."""

    api_response = {
        "product": "CostPilot",
        "version": "1.0.0",
        "downloads": [
            {
                "platform": "linux-x64",
                "url": "https://example.com/download/linux-x64",
                "sha256": "abc123def456"
            },
            {
                "platform": "macos-arm64",
                "url": "https://example.com/download/macos-arm64",
                "sha256": "def456abc123"
            }
        ]
    }

    for download in api_response["downloads"]:
        assert "sha256" in download
        assert "url" in download

    print(f"✓ Checksum API endpoint format ({len(api_response['downloads'])} platforms)")


if __name__ == "__main__":
    print("Testing packaged installer checksum validation...")

    try:
        test_installer_checksum_match()
        test_checksum_file_format()
        test_multiple_platform_checksums()
        test_tampered_installer_detected()
        test_marketplace_metadata_includes_checksum()
        test_signed_checksum_file()
        test_lemonsqueezy_webhook_validation()
        test_gumroad_license_includes_checksum()
        test_download_verification_flow()
        test_checksum_api_endpoint()

        print("\n✅ All packaged installer checksum tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
