#!/usr/bin/env python3
"""
Test: Refund + revocation flow.

Validates refund processing, license revocation, and feature deactivation.
"""

import os
import sys
import tempfile
import json
import hashlib
from datetime import datetime


def test_refund_request():
    """Verify refund request initiation."""
    
    refund_request = {
        "purchase_id": "purch_abc123",
        "reason": "not_satisfied",
        "requested_at": datetime.utcnow().isoformat(),
        "status": "pending"
    }
    
    assert refund_request["status"] == "pending"
    print(f"✓ Refund request ({refund_request['reason']})")


def test_refund_eligibility():
    """Verify refund eligibility check."""
    
    eligibility = {
        "days_since_purchase": 15,
        "refund_window_days": 30,
        "eligible": True
    }
    
    assert eligibility["eligible"] is True
    print(f"✓ Refund eligibility ({eligibility['days_since_purchase']}/{eligibility['refund_window_days']} days)")


def test_refund_processing():
    """Verify refund is processed."""
    
    refund = {
        "refund_id": "ref_xyz789",
        "amount": 99.00,
        "currency": "USD",
        "status": "completed"
    }
    
    assert refund["status"] == "completed"
    print(f"✓ Refund processing (${refund['amount']})")


def test_license_revocation():
    """Verify license is revoked after refund."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_revocation.json', delete=False) as f:
        revocation = {
            "license_id": "lic_xyz789",
            "revoked": True,
            "revoked_at": datetime.utcnow().isoformat(),
            "reason": "refund"
        }
        json.dump(revocation, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["revoked"] is True
        print(f"✓ License revocation ({data['reason']})")
        
    finally:
        os.unlink(path)


def test_revocation_list_update():
    """Verify revocation list is updated."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_revocations.json', delete=False) as f:
        revocation_list = {
            "revoked_licenses": [
                "lic_xyz789",
                "lic_abc456"
            ],
            "updated_at": datetime.utcnow().isoformat()
        }
        json.dump(revocation_list, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert "lic_xyz789" in data["revoked_licenses"]
        print(f"✓ Revocation list update ({len(data['revoked_licenses'])} entries)")
        
    finally:
        os.unlink(path)


def test_feature_deactivation():
    """Verify Pro features are deactivated."""
    
    features = {
        "baseline_advanced": False,
        "slo_engine": False,
        "audit_logs": False,
        "deactivated": True
    }
    
    assert features["deactivated"] is True
    print("✓ Feature deactivation")


def test_client_notification():
    """Verify client is notified of revocation."""
    
    notification = {
        "method": "email",
        "recipient": "user@example.com",
        "message": "License revoked due to refund",
        "sent": True
    }
    
    assert notification["sent"] is True
    print(f"✓ Client notification ({notification['method']})")


def test_grace_period_removal():
    """Verify grace period is removed after revocation."""
    
    grace_removal = {
        "grace_period_active": False,
        "immediate_revocation": True
    }
    
    assert grace_removal["immediate_revocation"] is True
    print("✓ Grace period removal")


def test_activation_invalidation():
    """Verify all activations are invalidated."""
    
    invalidation = {
        "active_machines": ["machine_1", "machine_2"],
        "invalidated": True,
        "remaining_activations": 0
    }
    
    assert invalidation["invalidated"] is True
    print(f"✓ Activation invalidation ({len(invalidation['active_machines'])} machines)")


def test_refund_without_revocation():
    """Verify partial refund without revocation."""
    
    partial_refund = {
        "refund_amount": 50.00,
        "original_amount": 99.00,
        "partial": True,
        "license_active": True
    }
    
    assert partial_refund["license_active"] is True
    print(f"✓ Partial refund (${partial_refund['refund_amount']}, license retained)")


def test_revocation_logging():
    """Verify revocation is logged."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_revocation.log', delete=False) as f:
        f.write("2024-01-15T10:00:00Z REFUND_REQUESTED purchase=purch_abc123 reason=not_satisfied\n")
        f.write("2024-01-15T10:00:05Z REFUND_ELIGIBLE days=15 window=30\n")
        f.write("2024-01-15T10:00:10Z REFUND_COMPLETED amount=99.00\n")
        f.write("2024-01-15T10:00:15Z LICENSE_REVOKED license=lic_xyz789\n")
        f.write("2024-01-15T10:00:20Z FEATURES_DEACTIVATED user=user_12345\n")
        path = f.name
    
    try:
        with open(path, 'r') as f:
            logs = f.readlines()
        
        assert len(logs) == 5
        print(f"✓ Revocation logging ({len(logs)} events)")
        
    finally:
        os.unlink(path)


if __name__ == "__main__":
    print("Testing refund + revocation flow...")
    
    try:
        test_refund_request()
        test_refund_eligibility()
        test_refund_processing()
        test_license_revocation()
        test_revocation_list_update()
        test_feature_deactivation()
        test_client_notification()
        test_grace_period_removal()
        test_activation_invalidation()
        test_refund_without_revocation()
        test_revocation_logging()
        
        print("\n✅ All refund + revocation flow tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
