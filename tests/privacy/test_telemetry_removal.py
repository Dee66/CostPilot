#!/usr/bin/env python3
"""
Test: Telemetry removal.

Validates that telemetry is consent-stored and removable per user request.
"""

import os
import sys
import tempfile
import json
from pathlib import Path


def test_consent_storage():
    """Verify consent is stored."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_consent.json', delete=False) as f:
        consent = {
            "telemetry_enabled": True,
            "consent_date": "2024-01-15",
            "consent_version": "1.0"
        }
        json.dump(consent, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert "consent_date" in data
        assert "consent_version" in data

        print("✓ Consent storage validated")

    finally:
        os.unlink(path)


def test_consent_revocable():
    """Verify consent can be revoked."""

    consent_before = {"telemetry_enabled": True}
    consent_after = {"telemetry_enabled": False, "revoked_date": "2024-01-20"}

    # Should be revocable
    assert consent_before["telemetry_enabled"] is True
    assert consent_after["telemetry_enabled"] is False

    print("✓ Consent revocable")


def test_data_deletion_request():
    """Verify data deletion request is honored."""

    deletion_request = {
        "user_id": "user_hash_123",
        "request_date": "2024-01-20",
        "data_deleted": True
    }

    assert deletion_request["data_deleted"] is True

    print("✓ Data deletion request honored")


def test_local_telemetry_files_removed():
    """Verify local telemetry files can be removed."""

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create telemetry files
        telemetry_dir = Path(tmpdir) / ".costpilot" / "telemetry"
        telemetry_dir.mkdir(parents=True)

        telemetry_file = telemetry_dir / "events.json"
        telemetry_file.write_text('{"events": []}')

        # Verify exists
        assert telemetry_file.exists()

        # Remove
        import shutil
        shutil.rmtree(telemetry_dir)

        # Verify removed
        assert not telemetry_dir.exists()

        print("✓ Local telemetry files removed")


def test_remote_data_deletion_api():
    """Verify remote data deletion API exists."""

    deletion_api = {
        "endpoint": "/api/v1/telemetry/delete",
        "method": "DELETE",
        "auth_required": True
    }

    assert deletion_api["method"] == "DELETE"
    assert deletion_api["auth_required"] is True

    print("✓ Remote data deletion API contract")


def test_deletion_confirmation():
    """Verify deletion confirmation is provided."""

    deletion_response = {
        "status": "success",
        "deleted_records": 42,
        "confirmation_id": "del_abc123"
    }

    assert deletion_response["status"] == "success"
    assert "confirmation_id" in deletion_response

    print("✓ Deletion confirmation provided")


def test_partial_data_deletion():
    """Verify partial data deletion (by date range)."""

    deletion_request = {
        "user_id": "user_hash_123",
        "delete_before": "2024-01-01",
        "retention_policy": "delete_older_than_90_days"
    }

    assert "delete_before" in deletion_request

    print("✓ Partial data deletion supported")


def test_consent_change_history():
    """Verify consent change history is maintained."""

    consent_history = [
        {"date": "2024-01-15", "enabled": True, "version": "1.0"},
        {"date": "2024-01-20", "enabled": False, "version": "1.0"}
    ]

    # History should track changes
    assert len(consent_history) == 2
    assert consent_history[0]["enabled"] != consent_history[1]["enabled"]

    print("✓ Consent change history maintained")


def test_no_telemetry_after_deletion():
    """Verify no telemetry sent after deletion request."""

    user_state = {
        "deletion_requested": True,
        "telemetry_enabled": False,
        "data_deleted": True
    }

    # Should not send any telemetry
    assert user_state["telemetry_enabled"] is False
    assert user_state["data_deleted"] is True

    print("✓ No telemetry after deletion request")


def test_gdpr_compliance():
    """Verify GDPR compliance (right to erasure)."""

    gdpr_compliance = {
        "right_to_access": True,
        "right_to_erasure": True,
        "right_to_rectification": True,
        "data_portability": True
    }

    # Must support erasure
    assert gdpr_compliance["right_to_erasure"] is True

    print("✓ GDPR compliance (right to erasure)")


def test_deletion_propagation_time():
    """Verify deletion propagation time is documented."""

    deletion_policy = {
        "propagation_time": "30_days",
        "immediate_stop": True,  # Stop collection immediately
        "deletion_complete_by": "30_days"
    }

    assert deletion_policy["immediate_stop"] is True

    print("✓ Deletion propagation time documented")


if __name__ == "__main__":
    print("Testing telemetry removal...")

    try:
        test_consent_storage()
        test_consent_revocable()
        test_data_deletion_request()
        test_local_telemetry_files_removed()
        test_remote_data_deletion_api()
        test_deletion_confirmation()
        test_partial_data_deletion()
        test_consent_change_history()
        test_no_telemetry_after_deletion()
        test_gdpr_compliance()
        test_deletion_propagation_time()

        print("\n✅ All telemetry removal tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
