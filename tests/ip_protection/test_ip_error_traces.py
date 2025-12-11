#!/usr/bin/env python3
"""Test IP Protection: Error traces scrub premium references."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_error_trace_no_premium_functions():
    """Test error traces don't mention premium functions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "malformed.json"
        
        # Malformed JSON to trigger error
        with open(template_path, 'w') as f:
            f.write('{"Resources": {INVALID}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode != 0, "Should fail on malformed JSON"
        
        output = result.stdout + result.stderr
        
        # Should not mention premium functions in trace
        forbidden = [
            "premium_engine::init",
            "pro_features::load",
            "bundle::decrypt",
            "license::validate",
            "heuristics::decrypt",
            "premium::activate"
        ]
        
        for func in forbidden:
            assert func not in output, f"Error trace should not mention: {func}"


def test_stack_trace_scrubbed():
    """Test stack traces are scrubbed of premium references."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "invalid.json"
        
        # Create scenario that might produce stack trace
        with open(template_path, 'w') as f:
            f.write('null')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        output = result.stdout + result.stderr
        
        # Stack trace should not include premium modules
        forbidden_modules = [
            "costpilot::premium",
            "costpilot::pro",
            "costpilot::license",
            "costpilot::bundle",
            "costpilot::encrypted"
        ]
        
        for module in forbidden_modules:
            assert module not in output, f"Stack trace should not include: {module}"


def test_error_messages_sanitized():
    """Test error messages are sanitized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "empty.json"
        
        with open(template_path, 'w') as f:
            f.write('')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode != 0, "Should fail on empty file"
        
        output = (result.stdout + result.stderr).lower()
        
        # Error messages should not leak internals
        forbidden = [
            "premium feature",
            "pro license",
            "encrypted bundle",
            "heuristics key",
            "license token"
        ]
        
        for phrase in forbidden:
            assert phrase not in output, f"Error should not mention: {phrase}"


def test_panic_output_scrubbed():
    """Test panic output is scrubbed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "huge.json"
        
        # Create very large file to potentially trigger panic
        with open(template_path, 'w') as f:
            f.write('{"Resources": {')
            for i in range(100000):
                f.write(f'"R{i}": {{"Type": "AWS::Lambda::Function"}},')
            f.write('"Final": {}}}')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        # Might timeout or fail
        output = result.stdout + result.stderr
        
        # Panic should not leak premium info
        if "panic" in output.lower() or "thread" in output.lower():
            forbidden = [
                "premium_engine",
                "pro_features",
                "license_validator",
                "bundle_decryptor"
            ]
            
            for item in forbidden:
                assert item not in output, f"Panic should not mention: {item}"


def test_error_codes_consistent():
    """Test error codes don't leak premium info."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "missing.json"
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        assert result.returncode != 0, "Should fail on missing file"
        
        # Exit code should be generic (1, 2, etc.)
        # Not premium-specific codes (100-199 might be reserved for premium)
        assert result.returncode in [1, 2, 3, 4, 5], \
            f"Error code should be generic, got: {result.returncode}"


def test_rust_backtrace_sanitized():
    """Test RUST_BACKTRACE output is sanitized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "corrupt.json"
        
        with open(template_path, 'wb') as f:
            f.write(b'\x00\x01\x02\x03')
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env={"RUST_BACKTRACE": "1"}
        )
        
        output = result.stdout + result.stderr
        
        # Backtrace should not include premium paths
        forbidden = [
            "/premium/",
            "/pro/",
            "/enterprise/",
            "::premium::",
            "::pro::",
            "::license::"
        ]
        
        for item in forbidden:
            assert item not in output, f"Backtrace should not include: {item}"


if __name__ == "__main__":
    test_error_trace_no_premium_functions()
    test_stack_trace_scrubbed()
    test_error_messages_sanitized()
    test_panic_output_scrubbed()
    test_error_codes_consistent()
    test_rust_backtrace_sanitized()
    print("All IP Protection: error trace scrubbing tests passed")
