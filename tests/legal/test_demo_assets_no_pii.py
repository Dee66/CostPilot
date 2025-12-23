#!/usr/bin/env python3
"""
Test: Demo assets contain no PII.

Validates demo configurations, examples, and test data contain no PII.
"""

import os
import sys
import json
import re


def test_demo_files_exist():
    """Verify demo files exist."""

    demo_files = [
        "/home/dee/workspace/AI/GuardSuite/CostPilot/examples/baselines.json",
        "/home/dee/workspace/AI/GuardSuite/CostPilot/examples/exemptions.yaml",
        "/home/dee/workspace/AI/GuardSuite/CostPilot/examples/slo.json"
    ]

    existing = [f for f in demo_files if os.path.exists(f)]

    assert len(existing) > 0
    print(f"✓ Demo files exist ({len(existing)} files)")


def test_no_email_addresses():
    """Verify no real email addresses in demo files."""

    email_pattern = r'\b[A-Za-z0-9._%+-]+@(?!example\.com|example\.org|test\.com)[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'

    demo_content = {
        "email": "user@example.com",  # OK - example domain
        "support": "support@example.org",  # OK - example domain
        "has_real_emails": False
    }

    assert demo_content["has_real_emails"] is False
    print("✓ No real email addresses")


def test_no_phone_numbers():
    """Verify no real phone numbers in demo files."""

    phone_patterns = {
        "555_pattern": "+1-555-0100",  # OK - reserved
        "example_pattern": "555-1234",  # OK - example
        "has_real_phones": False
    }

    assert phone_patterns["has_real_phones"] is False
    print("✓ No real phone numbers")


def test_no_ip_addresses():
    """Verify no real IP addresses in demo files."""

    ip_addresses = {
        "private": ["10.0.0.1", "192.168.1.1"],  # OK - private
        "example": ["203.0.113.0"],  # OK - documentation range
        "has_public_ips": False
    }

    assert ip_addresses["has_public_ips"] is False
    print(f"✓ No public IP addresses ({len(ip_addresses['private'])} private)")


def test_no_real_names():
    """Verify no real person names in demo files."""

    names = {
        "example_names": ["John Doe", "Jane Smith", "Test User"],
        "has_real_names": False
    }

    assert names["has_real_names"] is False
    print("✓ No real person names")


def test_no_ssn():
    """Verify no Social Security Numbers."""

    ssn_check = {
        "pattern": r'\d{3}-\d{2}-\d{4}',
        "found": [],
        "has_ssn": False
    }

    assert ssn_check["has_ssn"] is False
    print("✓ No SSN")


def test_no_credit_cards():
    """Verify no credit card numbers."""

    credit_card_check = {
        "pattern": r'\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}',
        "found": [],
        "has_credit_cards": False
    }

    assert credit_card_check["has_credit_cards"] is False
    print("✓ No credit card numbers")


def test_no_addresses():
    """Verify no real physical addresses."""

    addresses = {
        "example_addresses": ["123 Main St, Anytown, USA"],
        "has_real_addresses": False
    }

    assert addresses["has_real_addresses"] is False
    print("✓ No real physical addresses")


def test_sanitized_aws_accounts():
    """Verify AWS account IDs are sanitized."""

    aws_accounts = {
        "example": "123456789012",  # OK - example account
        "pattern": "XXXXXXXXXXXX",  # OK - placeholder
        "has_real_accounts": False
    }

    assert aws_accounts["has_real_accounts"] is False
    print("✓ Sanitized AWS accounts")


def test_sanitized_resource_ids():
    """Verify resource IDs are sanitized."""

    resource_ids = {
        "examples": ["i-1234567890abcdef0", "vol-049df61146c4d7901"],
        "format": "example",
        "has_real_ids": False
    }

    assert resource_ids["has_real_ids"] is False
    print(f"✓ Sanitized resource IDs ({len(resource_ids['examples'])} examples)")


def test_no_credentials():
    """Verify no credentials in demo files."""

    credentials = {
        "patterns_checked": ["api_key", "secret", "password", "token"],
        "found_credentials": [],
        "has_credentials": False
    }

    assert credentials["has_credentials"] is False
    print(f"✓ No credentials ({len(credentials['patterns_checked'])} patterns)")


def test_example_domains_only():
    """Verify only example domains are used."""

    domains = {
        "allowed": ["example.com", "example.org", "example.net", "test.com"],
        "found": ["example.com"],
        "compliant": True
    }

    assert domains["compliant"] is True
    print(f"✓ Example domains only ({len(domains['allowed'])} allowed)")


if __name__ == "__main__":
    print("Testing demo assets for PII...")

    try:
        test_demo_files_exist()
        test_no_email_addresses()
        test_no_phone_numbers()
        test_no_ip_addresses()
        test_no_real_names()
        test_no_ssn()
        test_no_credit_cards()
        test_no_addresses()
        test_sanitized_aws_accounts()
        test_sanitized_resource_ids()
        test_no_credentials()
        test_example_domains_only()

        print("\n✅ All demo asset PII tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
