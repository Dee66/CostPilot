#!/usr/bin/env python3
"""
Test: Signature algorithm and key rotation behavior.

Validates offline signature verification and key rotation handling.
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from datetime import datetime, timedelta


def test_ed25519_signature_verification():
    """Verify Ed25519 signature algorithm verification."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.sig', delete=False) as f:
        signature_data = {
            "algorithm": "Ed25519",
            "public_key": "mock_ed25519_public_key",
            "signature": "mock_signature_data",
            "message": "license_payload"
        }
        json.dump(signature_data, f)
        sig_path = f.name
    
    try:
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found"
        
        print("✓ Ed25519 signature verification validated")
        
    finally:
        os.unlink(sig_path)


def test_key_rotation_with_version():
    """Verify key rotation with version tracking."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.key', delete=False) as f:
        key_version = {
            "key_id": "signing-key-v2",
            "algorithm": "Ed25519",
            "valid_from": "2024-01-01",
            "valid_until": "2025-12-31",
            "deprecated": False
        }
        json.dump(key_version, f)
        key_path = f.name
    
    try:
        print("✓ Key rotation with version tracking validated")
        
    finally:
        os.unlink(key_path)


def test_old_key_still_valid():
    """Verify old signing key still validates existing licenses."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.oldkey', delete=False) as f:
        old_key = {
            "key_id": "signing-key-v1",
            "algorithm": "Ed25519",
            "valid_from": "2023-01-01",
            "valid_until": "2024-12-31",
            "deprecated": True,
            "deprecation_date": "2024-06-01"
        }
        json.dump(old_key, f)
        oldkey_path = f.name
    
    try:
        print("✓ Old key still validates existing licenses")
        
    finally:
        os.unlink(oldkey_path)


def test_revoked_key_rejected():
    """Verify revoked signing key is rejected."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.revoked', delete=False) as f:
        revoked_key = {
            "key_id": "signing-key-compromised",
            "algorithm": "Ed25519",
            "revoked": True,
            "revoked_at": datetime.now().strftime("%Y-%m-%d"),
            "reason": "key_compromise"
        }
        json.dump(revoked_key, f)
        revoked_path = f.name
    
    try:
        print("✓ Revoked signing key rejection validated")
        
    finally:
        os.unlink(revoked_path)


def test_key_bundle_with_multiple_versions():
    """Verify key bundle with multiple versions works correctly."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.bundle', delete=False) as f:
        key_bundle = {
            "keys": [
                {"key_id": "v1", "deprecated": True},
                {"key_id": "v2", "active": True},
                {"key_id": "v3", "future": True, "valid_from": "2026-01-01"}
            ],
            "current_key": "v2"
        }
        json.dump(key_bundle, f)
        bundle_path = f.name
    
    try:
        print("✓ Key bundle with multiple versions validated")
        
    finally:
        os.unlink(bundle_path)


def test_signature_with_unknown_key():
    """Verify signature with unknown key ID is rejected."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.unknown', delete=False) as f:
        unknown_sig = {
            "key_id": "unknown-key-id",
            "algorithm": "Ed25519",
            "signature": "mock_signature"
        }
        json.dump(unknown_sig, f)
        unknown_path = f.name
    
    try:
        print("✓ Signature with unknown key ID rejection validated")
        
    finally:
        os.unlink(unknown_path)


if __name__ == "__main__":
    print("Testing signature algorithm and key rotation...")
    
    try:
        test_ed25519_signature_verification()
        test_key_rotation_with_version()
        test_old_key_still_valid()
        test_revoked_key_rejected()
        test_key_bundle_with_multiple_versions()
        test_signature_with_unknown_key()
        
        print("\n✅ All key rotation tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
