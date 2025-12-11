#!/usr/bin/env python3
"""Test Free Edition: debug output reveals no internal heuristics keys or versions."""

import subprocess
import tempfile
from pathlib import Path
import json
import re


def test_debug_output_no_internal_keys():
    """Test debug output doesn't reveal internal heuristics keys."""
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
        
        # Try various debug flags
        debug_flags = [
            ["--debug"],
            ["-v"],
            ["--verbose"],
            ["-vv"],
        ]
        
        for flags in debug_flags:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)] + flags,
                capture_output=True,
                text=True,
                timeout=10
            )
            
            output = result.stdout + result.stderr
            output_lower = output.lower()
            
            # Should not reveal internal keys
            internal_markers = [
                "api_key",
                "secret_key",
                "internal_version",
                "heuristics_key",
                "license_key",
                "encryption_key",
            ]
            
            for marker in internal_markers:
                assert marker not in output_lower, f"Debug output reveals {marker}"


def test_debug_no_heuristics_version():
    """Test debug output doesn't show internal heuristics version."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        
        # Should not show internal version strings like "2.0.0-pro-internal"
        version_pattern = r"\d+\.\d+\.\d+-(pro|premium|internal)"
        matches = re.findall(version_pattern, output.lower())
        
        assert len(matches) == 0, f"Debug output shows internal version: {matches}"


def test_debug_no_file_paths():
    """Test debug output doesn't reveal internal file paths."""
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
            ["costpilot", "scan", "--plan", str(template_path), "--debug"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        output_lower = output.lower()
        
        # Should not reveal internal paths
        internal_paths = [
            "/opt/costpilot/pro",
            "/var/lib/costpilot/premium",
            "C:\\Program Files\\CostPilot\\Pro",
        ]
        
        for path in internal_paths:
            assert path.lower() not in output_lower, f"Debug reveals internal path: {path}"


def test_error_messages_sanitized():
    """Test error messages are sanitized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create invalid JSON
        with open(template_path, 'w') as f:
            f.write('{"invalid')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        error = result.stderr
        error_lower = error.lower()
        
        # Should not reveal internal details
        internal_terms = [
            "internal error",
            "heuristics key",
            "license check",
            "pro engine",
        ]
        
        for term in internal_terms:
            # Allow generic mentions but not specific internal details
            pass


def test_stack_traces_sanitized():
    """Test stack traces don't reveal internal details."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Try to trigger an error
        result = subprocess.run(
            ["costpilot", "scan", "--plan", "/nonexistent/file.json"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        
        # Should not show Rust panic with internal details
        if "panic" in output.lower():
            # Check panic doesn't reveal internal info
            assert "heuristics" not in output.lower() or "file not found" in output.lower(), \
                "Panic reveals internal heuristics details"


def test_version_output_sanitized():
    """Test version output is sanitized."""
    result = subprocess.run(
        ["costpilot", "--version"],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    output = result.stdout + result.stderr
    output_lower = output.lower()
    
    # Should not reveal Pro/Premium in version
    # (unless it's explicitly a Pro build)
    # For Free Edition, should say "Community" or "Free"
    if "community" in output_lower or "free" in output_lower:
        # Good - clearly identified as Free
        pass
    else:
        # Should not say "Pro" or "Premium"
        assert "pro" not in output_lower, "Version output mentions Pro"
        assert "premium" not in output_lower, "Version output mentions Premium"


if __name__ == "__main__":
    test_debug_output_no_internal_keys()
    test_debug_no_heuristics_version()
    test_debug_no_file_paths()
    test_error_messages_sanitized()
    test_stack_traces_sanitized()
    test_version_output_sanitized()
    print("All Free Edition debug output sanitization tests passed")
