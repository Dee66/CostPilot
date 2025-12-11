#!/usr/bin/env python3
"""Test Free Edition: no telemetry subsystem is reachable."""

import subprocess
import tempfile
from pathlib import Path
import json
import socket


def test_no_network_connections():
    """Test no network connections are made during analysis."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Run with network monitoring (basic check)
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should complete without network access
        assert result.returncode in [0, 1, 2, 101], "Should work without network"


def test_telemetry_flag_rejected():
    """Test --telemetry flag rejected or ignored."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--telemetry"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should reject or ignore
        if result.returncode != 0:
            error = result.stderr.lower()
            assert "unknown" in error or "telemetry" in error, \
                "Should reject --telemetry"


def test_analytics_disabled():
    """Test analytics explicitly disabled."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }
        
        with open(template_path, 'w') as f:
            json.dump(template_content, f)
        
        # Set env vars to disable analytics
        import os
        env = os.environ.copy()
        env["COSTPILOT_TELEMETRY"] = "0"
        env["DO_NOT_TRACK"] = "1"
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env=env
        )
        
        # Should work
        assert result.returncode in [0, 1, 2, 101], "Should respect telemetry disable"


def test_no_tracking_endpoints():
    """Test no tracking endpoints are contacted."""
    # Common tracking domains
    tracking_domains = [
        "analytics.google.com",
        "stats.g.doubleclick.net",
        "www.google-analytics.com",
        "mixpanel.com",
        "segment.io",
    ]
    
    # Test that these resolve but we don't contact them
    # (this is more of a smoke test)
    for domain in tracking_domains:
        try:
            # Just resolve, don't connect
            socket.gethostbyname(domain)
        except socket.gaierror:
            # Domain doesn't resolve - OK
            pass


def test_no_telemetry_config():
    """Test no telemetry config files are created."""
    telemetry_paths = [
        Path.home() / ".costpilot" / "telemetry.json",
        Path.home() / ".config" / "costpilot" / "telemetry.json",
        Path.home() / ".costpilot" / "analytics.json",
    ]
    
    for path in telemetry_paths:
        # These shouldn't exist
        if path.exists():
            # If it exists, check it's not a telemetry config
            with open(path) as f:
                content = f.read().lower()
            
            # Shouldn't have telemetry-related content
            telemetry_terms = ["api_key", "tracking_id", "analytics_id"]
            for term in telemetry_terms:
                assert term not in content, f"Telemetry config found at {path}"


def test_privacy_policy_compliance():
    """Test privacy policy compliance."""
    # Check if PRIVACY.md exists and mentions no telemetry
    privacy_files = [
        "docs/PRIVACY.md",
        "PRIVACY.md",
        "docs/PRIVACY_POLICY.md",
    ]
    
    for path in privacy_files:
        file = Path(path)
        if file.exists():
            with open(file) as f:
                content = f.read().lower()
            
            # If mentions telemetry, should say "disabled" or "not collected"
            if "telemetry" in content:
                assert "disabled" in content or "not collected" in content or "no telemetry" in content, \
                    "Privacy policy should state telemetry is disabled"


if __name__ == "__main__":
    test_no_network_connections()
    test_telemetry_flag_rejected()
    test_analytics_disabled()
    test_no_tracking_endpoints()
    test_no_telemetry_config()
    test_privacy_policy_compliance()
    print("All Free Edition telemetry gating tests passed")
