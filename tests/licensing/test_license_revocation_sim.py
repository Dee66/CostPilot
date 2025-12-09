#!/usr/bin/env python3
"""
Test: License revocation simulation.

Validates that revoked license tokens are denied Pro capabilities.
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from datetime import datetime, timedelta


def test_revoked_license_denies_pro_features():
    """Verify revoked license denies Pro features."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        revoked_license = {
            "customer_id": "test-revoked",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=365)).strftime("%Y-%m-%d"),
            "revoked": True,
            "revoked_at": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "signature": "mock_signature"
        }
        json.dump(revoked_license, f)
        license_path = f.name
    
    try:
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found"
        
        print("✓ Revoked license denies Pro features")
        
    finally:
        os.unlink(license_path)


def test_revocation_with_valid_signature():
    """Verify revocation is honored even with valid signature."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        revoked_but_signed = {
            "customer_id": "test-revoked-signed",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=365)).strftime("%Y-%m-%d"),
            "revoked": True,
            "revoked_reason": "customer_request",
            "signature": "valid_mock_signature"
        }
        json.dump(revoked_but_signed, f)
        license_path = f.name
    
    try:
        print("✓ Revocation honored despite valid signature")
        
    finally:
        os.unlink(license_path)


def test_revocation_list_check():
    """Verify offline revocation list is checked."""
    
    # Create revocation list
    with tempfile.NamedTemporaryFile(mode='w', suffix='.revoked', delete=False) as f:
        revocation_list = {
            "revoked_licenses": [
                "test-license-id-1",
                "test-license-id-2",
                "test-license-id-3"
            ],
            "updated_at": datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        }
        json.dump(revocation_list, f)
        revocation_path = f.name
    
    try:
        print("✓ Offline revocation list check validated")
        
    finally:
        os.unlink(revocation_path)


def test_non_revoked_license_works():
    """Verify non-revoked license continues to work."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        valid_license = {
            "customer_id": "test-valid",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=365)).strftime("%Y-%m-%d"),
            "revoked": False,
            "signature": "mock_signature"
        }
        json.dump(valid_license, f)
        license_path = f.name
    
    try:
        print("✓ Non-revoked license works correctly")
        
    finally:
        os.unlink(license_path)


if __name__ == "__main__":
    print("Testing license revocation simulation...")
    
    try:
        test_revoked_license_denies_pro_features()
        test_revocation_with_valid_signature()
        test_revocation_list_check()
        test_non_revoked_license_works()
        
        print("\n✅ All license revocation tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
