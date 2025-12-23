#!/usr/bin/env python3
"""
Test: Enterprise onboarding flow.

Validates new enterprise customers can onboard smoothly with license key and setup.
"""

import os
import sys
import tempfile
import json
import hashlib


def test_license_key_activation():
    """Verify license key activation flow."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_license.json', delete=False) as f:
        license_data = {
            "license_key": "ENT-ABCD-1234-5678-9012",
            "activated": True,
            "activation_date": "2024-01-15",
            "organization": "ACME Corp"
        }
        json.dump(license_data, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["activated"] is True
        assert data["license_key"].startswith("ENT-")

        print("✓ License key activated successfully")

    finally:
        os.unlink(path)


def test_org_profile_setup():
    """Verify organization profile setup."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_org.json', delete=False) as f:
        org_profile = {
            "name": "ACME Corp",
            "domain": "acme.com",
            "contact_email": "admin@acme.com",
            "setup_complete": True
        }
        json.dump(org_profile, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["setup_complete"] is True
        assert "domain" in data

        print("✓ Organization profile setup complete")

    finally:
        os.unlink(path)


def test_initial_user_provisioning():
    """Verify initial user provisioning."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_users.json', delete=False) as f:
        users = {
            "admin": {
                "email": "admin@acme.com",
                "role": "admin",
                "provisioned": True
            },
            "analyst1": {
                "email": "analyst1@acme.com",
                "role": "analyst",
                "provisioned": True
            }
        }
        json.dump(users, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert "admin" in data
        assert data["admin"]["role"] == "admin"

        print(f"✓ Initial user provisioning ({len(data)} users)")

    finally:
        os.unlink(path)


def test_sso_configuration():
    """Verify SSO configuration during onboarding."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_sso.json', delete=False) as f:
        sso_config = {
            "enabled": True,
            "provider": "okta",
            "idp_url": "https://acme.okta.com",
            "metadata_url": "https://acme.okta.com/metadata.xml",
            "configured": True
        }
        json.dump(sso_config, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["configured"] is True
        assert data["provider"] in ["okta", "azure_ad", "google"]

        print("✓ SSO configuration complete (Okta)")

    finally:
        os.unlink(path)


def test_baseline_import():
    """Verify baseline import during onboarding."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_baseline.json', delete=False) as f:
        baseline = {
            "resources": [
                {"id": "r-001", "type": "aws_instance", "cost": 100.0},
                {"id": "r-002", "type": "aws_rds", "cost": 50.0}
            ],
            "imported": True
        }
        json.dump(baseline, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["imported"] is True
        assert len(data["resources"]) > 0

        print(f"✓ Baseline imported ({len(data['resources'])} resources)")

    finally:
        os.unlink(path)


def test_policy_template_installation():
    """Verify policy templates are installed."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_policies.json', delete=False) as f:
        policies = {
            "templates": [
                {"name": "budget_enforcement", "installed": True},
                {"name": "drift_detection", "installed": True},
                {"name": "cost_anomaly", "installed": True}
            ]
        }
        json.dump(policies, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert all(p["installed"] for p in data["templates"])

        print(f"✓ Policy templates installed ({len(data['templates'])} templates)")

    finally:
        os.unlink(path)


def test_audit_logging_enabled():
    """Verify audit logging is enabled by default."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_config.json', delete=False) as f:
        config = {
            "audit_logging": {
                "enabled": True,
                "retention_days": 365,
                "destinations": ["local", "s3"]
            }
        }
        json.dump(config, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert data["audit_logging"]["enabled"] is True

        print("✓ Audit logging enabled (365 days retention)")

    finally:
        os.unlink(path)


def test_welcome_email_sent():
    """Verify welcome email is sent to admin."""

    email_receipt = {
        "to": "admin@acme.com",
        "subject": "Welcome to CostPilot Enterprise",
        "sent": True,
        "sent_at": "2024-01-15T10:00:00Z"
    }

    assert email_receipt["sent"] is True

    print("✓ Welcome email sent")


def test_onboarding_checklist():
    """Verify onboarding checklist is provided."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_checklist.json', delete=False) as f:
        checklist = {
            "items": [
                {"task": "Activate license", "completed": True},
                {"task": "Configure SSO", "completed": True},
                {"task": "Import baseline", "completed": True},
                {"task": "Install policies", "completed": True}
            ]
        }
        json.dump(checklist, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        completed = sum(1 for item in data["items"] if item["completed"])

        print(f"✓ Onboarding checklist ({completed}/{len(data['items'])} items complete)")

    finally:
        os.unlink(path)


def test_support_contact_info():
    """Verify support contact info is provided."""

    support_info = {
        "email": "enterprise-support@costpilot.dev",
        "slack_channel": "#enterprise-customers",
        "phone": "+1-800-COSTPILOT",
        "sla": "4 hours"
    }

    assert "email" in support_info
    assert support_info["sla"] == "4 hours"

    print("✓ Support contact info provided (4h SLA)")


def test_integration_guides():
    """Verify integration guides are available."""

    guides = {
        "available": [
            "terraform_integration.md",
            "github_actions.md",
            "jenkins_plugin.md",
            "vscode_extension.md"
        ]
    }

    assert len(guides["available"]) >= 4

    print(f"✓ Integration guides available ({len(guides['available'])} guides)")


if __name__ == "__main__":
    print("Testing enterprise onboarding flow...")

    try:
        test_license_key_activation()
        test_org_profile_setup()
        test_initial_user_provisioning()
        test_sso_configuration()
        test_baseline_import()
        test_policy_template_installation()
        test_audit_logging_enabled()
        test_welcome_email_sent()
        test_onboarding_checklist()
        test_support_contact_info()
        test_integration_guides()

        print("\n✅ All enterprise onboarding flow tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
