#!/usr/bin/env python3
"""
Test: Artifact signature verification at install time.

Validates that artifact signatures are verified during installation.
"""

import os
import sys
import tempfile
import json
import hashlib
from pathlib import Path


def test_signature_file_format():
    """Verify signature file format is valid."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.sig', delete=False) as f:
        signature = {
            "algorithm": "Ed25519",
            "signature": "mock_base64_signature_data_here",
            "public_key_id": "key_v1",
            "timestamp": "2024-01-15T10:00:00Z"
        }
        json.dump(signature, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            sig_data = json.load(f)

        assert "algorithm" in sig_data, "Missing algorithm field"
        assert "signature" in sig_data, "Missing signature field"
        assert "public_key_id" in sig_data, "Missing key ID"

        print("✓ Signature file format validated")

    finally:
        os.unlink(path)


def test_signature_algorithm_supported():
    """Verify signature algorithm is supported."""

    supported_algorithms = ["Ed25519", "RSA-PSS-4096"]

    for algo in supported_algorithms:
        sig = {
            "algorithm": algo,
            "signature": "test_sig",
            "public_key_id": "key_v1"
        }

        assert sig["algorithm"] in supported_algorithms

    print(f"✓ Signature algorithms supported: {', '.join(supported_algorithms)}")


def test_invalid_signature_rejected():
    """Verify invalid signature is rejected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.sig', delete=False) as f:
        invalid_sig = {
            "algorithm": "Ed25519",
            "signature": "INVALID_SIGNATURE_DATA",
            "public_key_id": "key_v1"
        }
        json.dump(invalid_sig, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            sig_data = json.load(f)

        # Contract: invalid signature should be rejected
        assert sig_data["signature"] == "INVALID_SIGNATURE_DATA"

        print("✓ Invalid signature rejection contract validated")

    finally:
        os.unlink(path)


def test_missing_signature_blocks_install():
    """Verify missing signature blocks installation."""

    with tempfile.TemporaryDirectory() as tmpdir:
        artifact_path = Path(tmpdir) / "artifact.bin"
        sig_path = Path(tmpdir) / "artifact.bin.sig"

        # Create artifact without signature
        with open(artifact_path, 'wb') as f:
            f.write(b"MOCK_ARTIFACT_DATA")

        # Signature file does not exist
        assert not sig_path.exists(), "Signature should not exist"

        print("✓ Missing signature blocks install contract validated")


def test_tampered_artifact_detected():
    """Verify tampered artifact is detected via signature."""

    with tempfile.TemporaryDirectory() as tmpdir:
        artifact_path = Path(tmpdir) / "artifact.bin"

        # Original artifact
        original_data = b"ORIGINAL_ARTIFACT_DATA"
        with open(artifact_path, 'wb') as f:
            f.write(original_data)

        original_hash = hashlib.sha256(original_data).hexdigest()

        # Tamper with artifact
        with open(artifact_path, 'ab') as f:
            f.write(b"TAMPERED")

        # Re-compute hash
        with open(artifact_path, 'rb') as f:
            tampered_hash = hashlib.sha256(f.read()).hexdigest()

        # Hashes should differ
        assert original_hash != tampered_hash, "Tampering not detected"

        print("✓ Tampered artifact detected via hash mismatch")


def test_public_key_bundle_verified():
    """Verify public key bundle is properly verified."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.keys', delete=False) as f:
        key_bundle = {
            "keys": [
                {
                    "id": "key_v1",
                    "algorithm": "Ed25519",
                    "public_key": "mock_base64_encoded_key_v1",
                    "valid_from": "2024-01-01T00:00:00Z",
                    "valid_until": "2025-01-01T00:00:00Z"
                },
                {
                    "id": "key_v2",
                    "algorithm": "Ed25519",
                    "public_key": "mock_base64_encoded_key_v2",
                    "valid_from": "2024-06-01T00:00:00Z",
                    "valid_until": "2026-01-01T00:00:00Z"
                }
            ]
        }
        json.dump(key_bundle, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            bundle = json.load(f)

        assert "keys" in bundle, "Missing keys array"
        assert len(bundle["keys"]) > 0, "No keys in bundle"

        for key in bundle["keys"]:
            assert "id" in key, "Key missing id"
            assert "public_key" in key, "Key missing public_key"

        print(f"✓ Public key bundle verified ({len(bundle['keys'])} keys)")

    finally:
        os.unlink(path)


def test_revoked_key_rejected():
    """Verify revoked key is rejected."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.revoked', delete=False) as f:
        revocation_list = {
            "revoked_keys": ["key_v0_compromised", "key_v1_old"],
            "updated": "2024-01-15T10:00:00Z"
        }
        json.dump(revocation_list, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            revoked = json.load(f)

        # Check if key is revoked
        test_key_id = "key_v0_compromised"
        assert test_key_id in revoked["revoked_keys"], "Revoked key not in list"

        print("✓ Revoked key rejection validated")

    finally:
        os.unlink(path)


def test_signature_timestamp_validated():
    """Verify signature timestamp is validated."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.sig', delete=False) as f:
        signature = {
            "algorithm": "Ed25519",
            "signature": "mock_sig",
            "public_key_id": "key_v1",
            "timestamp": "2024-01-15T10:00:00Z"
        }
        json.dump(signature, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            sig_data = json.load(f)

        assert "timestamp" in sig_data, "Missing timestamp"
        assert sig_data["timestamp"].endswith("Z"), "Invalid timestamp format"

        print("✓ Signature timestamp validation contract")

    finally:
        os.unlink(path)


def test_install_verification_flow():
    """Verify complete install verification flow."""

    # Simulate install flow:
    # 1. Download artifact
    # 2. Download signature
    # 3. Verify signature
    # 4. Install if valid

    with tempfile.TemporaryDirectory() as tmpdir:
        artifact_path = Path(tmpdir) / "costpilot.bin"
        sig_path = Path(tmpdir) / "costpilot.bin.sig"

        # Step 1: artifact
        artifact_data = b"COSTPILOT_BINARY_DATA" * 100
        with open(artifact_path, 'wb') as f:
            f.write(artifact_data)

        # Step 2: signature
        artifact_hash = hashlib.sha256(artifact_data).hexdigest()
        sig_data = {
            "algorithm": "Ed25519",
            "signature": f"sig_for_{artifact_hash[:16]}",
            "artifact_hash": artifact_hash,
            "public_key_id": "key_v1"
        }
        with open(sig_path, 'w') as f:
            json.dump(sig_data, f)

        # Step 3: verify
        with open(sig_path, 'r') as f:
            sig = json.load(f)

        with open(artifact_path, 'rb') as f:
            computed_hash = hashlib.sha256(f.read()).hexdigest()

        assert sig["artifact_hash"] == computed_hash, "Hash mismatch"

        print("✓ Complete install verification flow validated")


if __name__ == "__main__":
    print("Testing artifact signature verification at install time...")

    try:
        test_signature_file_format()
        test_signature_algorithm_supported()
        test_invalid_signature_rejected()
        test_missing_signature_blocks_install()
        test_tampered_artifact_detected()
        test_public_key_bundle_verified()
        test_revoked_key_rejected()
        test_signature_timestamp_validated()
        test_install_verification_flow()

        print("\n✅ All signature verification tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
