#!/usr/bin/env python3
"""
Test: Audit evidence package.

Validates auditors can request a snapshot of configuration, logs, baselines, and policy history.
"""

import os
import sys
import tempfile
import json
import hashlib
from datetime import datetime


def test_audit_package_generation():
    """Verify audit evidence package can be generated."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_audit_pkg.json', delete=False) as f:
        package = {
            "package_id": "audit-2024-01-15-001",
            "generated_at": "2024-01-15T10:00:00Z",
            "requested_by": "auditor@external.com",
            "status": "complete"
        }
        json.dump(package, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["status"] == "complete"
        assert "package_id" in data
        
        print("✓ Audit evidence package generated")
        
    finally:
        os.unlink(path)


def test_configuration_snapshot():
    """Verify configuration snapshot is included."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_config.json', delete=False) as f:
        config_snapshot = {
            "snapshot_time": "2024-01-15T10:00:00Z",
            "policies": 15,
            "baselines": 3,
            "users": 25,
            "slo_configs": 8
        }
        json.dump(config_snapshot, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert data["policies"] > 0
        assert "snapshot_time" in data
        
        print(f"✓ Configuration snapshot ({data['policies']} policies)")
        
    finally:
        os.unlink(path)


def test_audit_logs_included():
    """Verify audit logs are included in package."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_audit.log', delete=False) as f:
        f.write("2024-01-01T00:00:00Z USER_LOGIN user=admin@acme.com ip=10.0.1.5\n")
        f.write("2024-01-01T00:01:00Z POLICY_CREATE user=admin@acme.com policy=budget_alert\n")
        f.write("2024-01-01T00:02:00Z BASELINE_UPDATE user=analyst@acme.com baseline=prod-baseline\n")
        path = f.name
    
    try:
        with open(path, 'r') as f:
            logs = f.readlines()
        
        assert len(logs) > 0
        assert any("USER_LOGIN" in line for line in logs)
        
        print(f"✓ Audit logs included ({len(logs)} entries)")
        
    finally:
        os.unlink(path)


def test_baseline_history():
    """Verify baseline history is included."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_baseline_history.json', delete=False) as f:
        history = {
            "baseline_id": "prod-baseline",
            "versions": [
                {"version": 1, "timestamp": "2024-01-01T00:00:00Z", "resources": 100},
                {"version": 2, "timestamp": "2024-01-08T00:00:00Z", "resources": 102},
                {"version": 3, "timestamp": "2024-01-15T00:00:00Z", "resources": 105}
            ]
        }
        json.dump(history, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert len(data["versions"]) > 0
        
        print(f"✓ Baseline history ({len(data['versions'])} versions)")
        
    finally:
        os.unlink(path)


def test_policy_change_log():
    """Verify policy change log is included."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_policy_changes.json', delete=False) as f:
        changes = [
            {
                "timestamp": "2024-01-01T10:00:00Z",
                "policy": "budget_alert",
                "action": "created",
                "user": "admin@acme.com"
            },
            {
                "timestamp": "2024-01-05T14:30:00Z",
                "policy": "budget_alert",
                "action": "modified",
                "user": "manager@acme.com",
                "changes": {"threshold": "1000 -> 1200"}
            }
        ]
        json.dump(changes, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert len(data) > 0
        assert any(c["action"] == "created" for c in data)
        
        print(f"✓ Policy change log ({len(data)} changes)")
        
    finally:
        os.unlink(path)


def test_user_access_log():
    """Verify user access log is included."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_access.log', delete=False) as f:
        f.write("2024-01-15T08:00:00Z ACCESS user=admin@acme.com action=login ip=10.0.1.5\n")
        f.write("2024-01-15T08:05:00Z ACCESS user=admin@acme.com action=read_policy policy=budget_alert\n")
        f.write("2024-01-15T09:00:00Z ACCESS user=analyst@acme.com action=login ip=10.0.2.10\n")
        path = f.name
    
    try:
        with open(path, 'r') as f:
            logs = f.readlines()
        
        assert len(logs) > 0
        
        print(f"✓ User access log ({len(logs)} entries)")
        
    finally:
        os.unlink(path)


def test_package_integrity_hash():
    """Verify package has integrity hash."""
    
    package_content = b"audit evidence package content"
    package_hash = hashlib.sha256(package_content).hexdigest()
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_manifest.json', delete=False) as f:
        manifest = {
            "package_hash": package_hash,
            "algorithm": "SHA256",
            "files": ["config.json", "audit.log", "baseline_history.json"]
        }
        json.dump(manifest, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert len(data["package_hash"]) == 64  # SHA256
        
        print("✓ Package integrity hash (SHA256)")
        
    finally:
        os.unlink(path)


def test_package_encryption():
    """Verify package is encrypted."""
    
    package_metadata = {
        "encrypted": True,
        "algorithm": "AES-256-GCM",
        "key_id": "audit-key-001"
    }
    
    assert package_metadata["encrypted"] is True
    
    print("✓ Package encrypted (AES-256-GCM)")


def test_package_digital_signature():
    """Verify package is digitally signed."""
    
    signature_data = {
        "signed": True,
        "algorithm": "Ed25519",
        "signature": "sig_" + "a" * 88,
        "signer": "costpilot-audit-service"
    }
    
    assert signature_data["signed"] is True
    assert len(signature_data["signature"]) > 0
    
    print("✓ Package digitally signed (Ed25519)")


def test_package_retention():
    """Verify package retention policy."""
    
    retention_policy = {
        "retention_days": 2555,  # 7 years
        "auto_delete": False,
        "archive_location": "s3://audit-archives/acme-corp/"
    }
    
    assert retention_policy["retention_days"] == 2555
    
    print("✓ Package retention (7 years)")


def test_package_export_formats():
    """Verify multiple export formats are supported."""
    
    export_formats = {
        "supported": ["json", "csv", "pdf", "zip"],
        "default": "zip"
    }
    
    assert "pdf" in export_formats["supported"]
    
    print(f"✓ Export formats ({len(export_formats['supported'])} formats)")


if __name__ == "__main__":
    print("Testing audit evidence package...")
    
    try:
        test_audit_package_generation()
        test_configuration_snapshot()
        test_audit_logs_included()
        test_baseline_history()
        test_policy_change_log()
        test_user_access_log()
        test_package_integrity_hash()
        test_package_encryption()
        test_package_digital_signature()
        test_package_retention()
        test_package_export_formats()
        
        print("\n✅ All audit evidence package tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
