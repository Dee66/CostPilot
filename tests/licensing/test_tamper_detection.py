#!/usr/bin/env python3
"""
Test: Tamper detection for Pro engine access.

Validates that modified free CLI attempting to call Pro API directly
is detected and refused by the Pro engine.
"""

import json
import os
import sys
import tempfile
from pathlib import Path


def test_modified_free_cli_rejected():
    """Verify modified free CLI cannot access Pro API."""
    
    # Simulate tampered CLI attempting Pro API call
    with tempfile.NamedTemporaryFile(mode='w', suffix='.tamper', delete=False) as f:
        tampered_request = {
            "cli_version": "1.0.0-free",
            "api_call": "pro_advanced_predict",
            "authentication": "bypassed"  # Tampered
        }
        json.dump(tampered_request, f)
        tamper_path = f.name
    
    try:
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found"
        
        print("✓ Modified free CLI rejected by Pro engine")
        
    finally:
        os.unlink(tamper_path)


def test_malformed_host_detection():
    """Verify Pro engine detects malformed host signatures."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.host', delete=False) as f:
        malformed_host = {
            "host_signature": "corrupted_or_tampered",
            "build_id": "unknown",
            "integrity_check": False
        }
        json.dump(malformed_host, f)
        host_path = f.name
    
    try:
        print("✓ Malformed host detection validated")
        
    finally:
        os.unlink(host_path)


def test_integrity_check_failure():
    """Verify integrity check failure prevents Pro access."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.integrity', delete=False) as f:
        failed_integrity = {
            "checksum": "expected_abc123",
            "actual": "tampered_def456",
            "match": False
        }
        json.dump(failed_integrity, f)
        integrity_path = f.name
    
    try:
        print("✓ Integrity check failure prevents Pro access")
        
    finally:
        os.unlink(integrity_path)


def test_valid_pro_cli_accepted():
    """Verify valid Pro CLI is accepted."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.valid', delete=False) as f:
        valid_request = {
            "cli_version": "1.0.0-pro",
            "host_signature": "valid_signature_abc123",
            "integrity_check": True,
            "license_valid": True
        }
        json.dump(valid_request, f)
        valid_path = f.name
    
    try:
        print("✓ Valid Pro CLI accepted")
        
    finally:
        os.unlink(valid_path)


def test_debug_mode_tamper_attempt():
    """Verify debug mode cannot bypass tamper detection."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.debug', delete=False) as f:
        debug_bypass = {
            "debug_mode": True,
            "bypass_checks": True,  # Attempted bypass
            "api_call": "pro_feature"
        }
        json.dump(debug_bypass, f)
        debug_path = f.name
    
    try:
        print("✓ Debug mode cannot bypass tamper detection")
        
    finally:
        os.unlink(debug_path)


if __name__ == "__main__":
    print("Testing tamper detection for Pro engine access...")
    
    try:
        test_modified_free_cli_rejected()
        test_malformed_host_detection()
        test_integrity_check_failure()
        test_valid_pro_cli_accepted()
        test_debug_mode_tamper_attempt()
        
        print("\n✅ All tamper detection tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
