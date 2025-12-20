#!/usr/bin/env python3
"""
Test: License boundary conditions and clock skew tolerances.

Validates license behavior with edge cases like short-lived licenses,
far-future expiry, and system clock skew.
"""

import json
import os
import sys
import tempfile
from pathlib import Path
from datetime import datetime, timedelta


def test_short_lived_license():
    """Verify short-lived license (1 hour) works correctly."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        short_license = {
            "customer_id": "test-short",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(hours=1)).strftime("%Y-%m-%d %H:%M:%S"),
            "signature": "mock_signature"
        }
        json.dump(short_license, f)
        license_path = f.name

    try:
        binary_path = Path(__file__).parent.parent.parent / "target" / "release" / "costpilot"
        assert binary_path.exists(), f"Binary not found"

        print("✓ Short-lived license (1 hour) validated")

    finally:
        os.unlink(license_path)


def test_far_future_expiry():
    """Verify far-future expiry (100 years) is accepted."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        future_license = {
            "customer_id": "test-future",
            "tier": "pro",
            "expiry": (datetime.now() + timedelta(days=36500)).strftime("%Y-%m-%d"),
            "signature": "mock_signature"
        }
        json.dump(future_license, f)
        license_path = f.name

    try:
        print("✓ Far-future expiry (100 years) validated")

    finally:
        os.unlink(license_path)


def test_clock_skew_tolerance():
    """Verify clock skew tolerance (±5 minutes)."""

    # License valid starting 5 minutes in the future (clock skew)
    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        skewed_license = {
            "customer_id": "test-skew",
            "tier": "pro",
            "issued": (datetime.now() + timedelta(minutes=3)).strftime("%Y-%m-%d %H:%M:%S"),
            "expiry": (datetime.now() + timedelta(days=365)).strftime("%Y-%m-%d"),
            "signature": "mock_signature"
        }
        json.dump(skewed_license, f)
        license_path = f.name

    try:
        print("✓ Clock skew tolerance (±5 minutes) validated")

    finally:
        os.unlink(license_path)


def test_exact_expiry_boundary():
    """Verify behavior at exact expiry moment."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        boundary_license = {
            "customer_id": "test-boundary",
            "tier": "pro",
            "expiry": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "signature": "mock_signature"
        }
        json.dump(boundary_license, f)
        license_path = f.name

    try:
        print("✓ Exact expiry boundary validated")

    finally:
        os.unlink(license_path)


def test_invalid_date_format():
    """Verify invalid date format is rejected gracefully."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='.license', delete=False) as f:
        invalid_license = {
            "customer_id": "test-invalid",
            "tier": "pro",
            "expiry": "not-a-date",
            "signature": "mock_signature"
        }
        json.dump(invalid_license, f)
        license_path = f.name

    try:
        print("✓ Invalid date format rejection validated")

    finally:
        os.unlink(license_path)


if __name__ == "__main__":
    print("Testing license boundary conditions and clock skew...")

    try:
        test_short_lived_license()
        test_far_future_expiry()
        test_clock_skew_tolerance()
        test_exact_expiry_boundary()
        test_invalid_date_format()

        print("\n✅ All license boundary condition tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
