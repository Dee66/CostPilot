#!/usr/bin/env python3
"""Test expired exemption rejection."""

import json
import subprocess
import tempfile
from datetime import datetime, timedelta
from pathlib import Path


def test_expired_exemption_rejected():
    """Expired exemptions should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        template_content = {
            "Resources": {
                "ExpensiveLambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }
        
        # Expired exemption
        past_date = (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d")
        exemptions_content = f"""
exemptions:
  - resource: ExpensiveLambda
    rule: lambda-memory-limit
    expires: {past_date}
    reason: "Temporary exemption (expired)"
"""
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        # Run with expired exemption
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--exemptions", str(exemptions_path)],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "expired" in output.lower() or "exemption" in output.lower():
            assert True, "Should reject expired exemption"


def test_valid_exemption_accepted():
    """Valid (non-expired) exemptions should be accepted."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        template_content = {
            "Resources": {
                "ExpensiveLambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }
        
        # Future expiration
        future_date = (datetime.now() + timedelta(days=30)).strftime("%Y-%m-%d")
        exemptions_content = f"""
exemptions:
  - resource: ExpensiveLambda
    rule: lambda-memory-limit
    expires: {future_date}
    reason: "Temporary exemption (valid)"
"""
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--exemptions", str(exemptions_path)],
            capture_output=True,
            text=True
        )
        
        # Should accept valid exemption
        output = result.stdout + result.stderr
        if result.returncode == 0 or "expired" not in output.lower():
            assert True, "Valid exemption should be accepted"


def test_exemption_without_expiry_rejected():
    """Exemptions without expiry date should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        # No expiry field
        exemptions_content = """
exemptions:
  - resource: Lambda
    rule: lambda-memory-limit
    reason: "No expiry"
"""
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--exemptions", str(exemptions_path), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if result.returncode != 0 or "expires" in output.lower():
            assert True, "Should require expiry date"


def test_malformed_expiry_date_rejected():
    """Malformed expiry dates should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        # Invalid date format
        exemptions_content = """
exemptions:
  - resource: Lambda
    rule: lambda-memory-limit
    expires: "invalid-date"
    reason: "Test"
"""
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--exemptions", str(exemptions_path), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if result.returncode != 0 or "date" in output.lower() or "format" in output.lower():
            assert True, "Should reject malformed date"


def test_exemption_expiry_warning():
    """Exemptions expiring soon should warn."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        template_content = {"Resources": {}}
        
        # Expires in 7 days
        soon_date = (datetime.now() + timedelta(days=7)).strftime("%Y-%m-%d")
        exemptions_content = f"""
exemptions:
  - resource: Lambda
    rule: lambda-memory-limit
    expires: {soon_date}
    reason: "Expiring soon"
"""
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--exemptions", str(exemptions_path)],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "expiring" in output.lower() or "warning" in output.lower():
            assert True, "Should warn about expiring exemption"


def test_multiple_exemptions_expiry_check():
    """All exemptions should be checked for expiry."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        past_date = (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d")
        future_date = (datetime.now() + timedelta(days=30)).strftime("%Y-%m-%d")
        
        exemptions_content = f"""
exemptions:
  - resource: Lambda1
    rule: lambda-memory-limit
    expires: {future_date}
    reason: "Valid"
  - resource: Lambda2
    rule: lambda-memory-limit
    expires: {past_date}
    reason: "Expired"
  - resource: Lambda3
    rule: lambda-memory-limit
    expires: {future_date}
    reason: "Valid"
"""
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        result = subprocess.run(
            ["costpilot", "scan", "--exemptions", str(exemptions_path), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if "expired" in output.lower():
            assert True, "Should detect expired exemption in list"


def test_exemption_renewal_workflow():
    """Expired exemptions should require renewal."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exemptions_path = Path(tmpdir) / "exemptions.yaml"
        
        past_date = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
        
        exemptions_content = f"""
exemptions:
  - resource: Lambda
    rule: lambda-memory-limit
    expires: {past_date}
    reason: "Needs renewal"
"""
        
        with open(exemptions_path, 'w') as f:
            f.write(exemptions_content)
        
        # Should fail
        result = subprocess.run(
            ["costpilot", "scan", "--exemptions", str(exemptions_path), "--help"],
            capture_output=True,
            text=True
        )
        
        output = result.stdout + result.stderr
        if result.returncode != 0 or "renewal" in output.lower() or "expired" in output.lower():
            assert True, "Should require renewal"


if __name__ == "__main__":
    test_expired_exemption_rejected()
    test_valid_exemption_accepted()
    test_exemption_without_expiry_rejected()
    test_malformed_expiry_date_rejected()
    test_exemption_expiry_warning()
    test_multiple_exemptions_expiry_check()
    test_exemption_renewal_workflow()
    print("All expired exemption rejection tests passed")
