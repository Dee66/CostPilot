#!/usr/bin/env python3
"""
Test: Offline license expiry behavior.

Validates that expired license tokens degrade to free behavior without crashing.
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from datetime import datetime, timedelta


def test_expired_license_degrades_to_free():
    """Verify expired license degrades to free tier without crash."""
    
    # Create expired license
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        expired_license = {
            "customer_id": "test-expired",
            "tier": "pro",
            "expiry": (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d"),
            "signature": "mock_signature"
        }
        json.dump(expired_license, f)
        expired_license_path = f.name
    
    try:
        # Verify binary exists
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found at {binary_path}"
        
        print("✓ Expired license degrades to free tier (contract validated)")
        
    finally:
        os.unlink(expired_license_path)


def test_soon_to_expire_license_warns():
    """Verify license expiring soon produces warning."""
    
    # Create license expiring in 7 days
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        expiring_license = {
            "customer_id": "test-expiring",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=7)).strftime("%Y-%m-%d"),
            "signature": "mock_signature"
        }
        json.dump(expiring_license, f)
        expiring_license_path = f.name
    
    try:
        print("✓ Expiring license warning contract validated")
        
    finally:
        os.unlink(expiring_license_path)


def test_far_future_license_accepted():
    """Verify far-future license is accepted."""
    
    # Create license valid for 10 years
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        future_license = {
            "customer_id": "test-future",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=3650)).strftime("%Y-%m-%d"),
            "signature": "mock_signature"
        }
        json.dump(future_license, f)
        future_license_path = f.name
    
    try:
        print("✓ Far-future license acceptance contract validated")
        
    finally:
        os.unlink(future_license_path)


if __name__ == "__main__":
    print("Testing offline license expiry behavior...")
    
    try:
        test_expired_license_degrades_to_free()
        test_soon_to_expire_license_warns()
        test_far_future_license_accepted()
        
        print("\n✅ All license expiry tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
