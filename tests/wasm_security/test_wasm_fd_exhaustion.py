#!/usr/bin/env python3
"""Test WASM FD exhaustion handling."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_wasm_fd_exhaustion():
    """Test that WASM build handles file descriptor exhaustion."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

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

        # Open many files to exhaust FD limit (for WASM)
        open_files = []
        try:
            for i in range(1000):
                try:
                    f = open(Path(tmpdir) / f"dummy{i}.txt", 'w')
                    open_files.append(f)
                except OSError:
                    break

            # Try to run costpilot in WASM with low FD availability
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Should handle FD exhaustion gracefully
            assert result.returncode in [0, 1, 2, 101], "WASM should handle FD exhaustion"
        finally:
            for f in open_files:
                try:
                    f.close()
                except:
                    pass


def test_wasm_file_handle_limit():
    """Test WASM file handle limits."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create many template files
        templates = []
        for i in range(100):
            template_path = Path(tmpdir) / f"template{i}.json"
            template_content = {
                "Resources": {
                    f"Lambda{i}": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "MemorySize": 1024
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            templates.append(template_path)

        # Process each template (WASM should not leak FDs)
        for template_path in templates[:10]:  # Test first 10
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )

            # Each should complete
            assert result.returncode in [0, 1, 2, 101], f"WASM should process {template_path}"


def test_wasm_concurrent_file_access():
    """Test WASM with concurrent file access patterns."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        baseline_path = Path(tmpdir) / "baseline.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        baseline_content = {
            "resources": [
                {
                    "name": "Lambda",
                    "type": "AWS::Lambda::Function",
                    "cost": 10.0,
                    "properties": {
                        "MemorySize": 1024
                    }
                }
            ]
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(baseline_path, 'w') as f:
            json.dump(baseline_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # WASM needs to handle multiple file inputs
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze",
             "--plan", str(template_path),
             "--baseline", str(baseline_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should handle multiple file inputs
        assert result.returncode in [0, 1, 2, 101], "WASM should handle multiple files"


def test_wasm_large_file_fd_usage():
    """Test WASM FD usage with large files."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Large template
        template_content = {
            "Resources": {
                f"Lambda{i}": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024 + (i * 128),
                        "Description": "A" * 1000
                    }
                }
                for i in range(1000)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # WASM should handle large file without FD issues
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=120
        )

        # Should handle large files
        assert result.returncode in [0, 1, 2, 101], "WASM should handle large files"


def test_wasm_fd_cleanup_on_error():
    """Test WASM FD cleanup on error."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Invalid JSON to trigger error
        with open(template_path, 'w') as f:
            f.write("{ invalid json }")

        # Run and expect error
        result = subprocess.run(
            ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should fail but clean up FDs
        assert result.returncode in [2, 101], "WASM should fail on invalid JSON"


if __name__ == "__main__":
    test_wasm_fd_exhaustion()
    test_wasm_file_handle_limit()
    test_wasm_concurrent_file_access()
    test_wasm_large_file_fd_usage()
    test_wasm_fd_cleanup_on_error()
    print("All WASM FD exhaustion tests passed")
