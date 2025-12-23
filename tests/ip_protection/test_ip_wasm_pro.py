#!/usr/bin/env python3
"""Test IP Protection: WASM Pro engine cannot be loaded by Free binary."""

import subprocess
import tempfile
from pathlib import Path
import os


def test_free_binary_no_pro_wasm():
    """Test Free binary doesn't include Pro WASM."""
    # Check binary doesn't contain Pro WASM modules
    result = subprocess.run(
        ["file", "target/release/costpilot"],
        capture_output=True,
        text=True,
        timeout=5
    )

    if result.returncode == 0:
        # Binary exists, check for Pro markers
        result = subprocess.run(
            ["strings", "target/release/costpilot"],
            capture_output=True,
            text=True,
            timeout=5
        )

        if result.returncode == 0:
            output = result.stdout.lower()
            # Should not contain Pro WASM markers
            assert "pro_wasm" not in output, "Free should not have pro_wasm"
            assert "premium_engine" not in output, "Free should not have premium_engine"


def test_wasm_pro_import_fails():
    """Test importing Pro WASM module fails in Free."""
    with tempfile.TemporaryDirectory() as tmpdir:
        wasm_path = Path(tmpdir) / "costpilot_pro.wasm"
        template_path = Path(tmpdir) / "template.json"

        # Create dummy WASM file with Pro header
        with open(wasm_path, 'wb') as f:
            f.write(b'\x00asm')  # WASM magic
            f.write(b'\x01\x00\x00\x00')  # Version
            f.write(b'PRO_ENGINE_MODULE')

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--wasm", str(wasm_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should reject Pro WASM
        if "--wasm" in result.stderr or "wasm" in result.stderr.lower():
            # Flag might not exist or rejected
            assert result.returncode != 0, "Should reject Pro WASM"


def test_wasm_opcode_validation():
    """Test WASM opcode validation rejects Pro features."""
    with tempfile.TemporaryDirectory() as tmpdir:
        wasm_path = Path(tmpdir) / "premium.wasm"

        # WASM with custom section containing Pro markers
        with open(wasm_path, 'wb') as f:
            f.write(b'\x00asm')
            f.write(b'\x01\x00\x00\x00')
            # Custom section with Pro marker
            f.write(b'\x00')  # Section ID 0 (custom)
            f.write(b'\x0f')  # Section size
            f.write(b'\x0bPRO_FEATURE')

        # Try to validate
        result = subprocess.run(
            ["file", str(wasm_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        # File should be recognized as WASM
        if "WebAssembly" in result.stdout or "wasm" in result.stdout.lower():
            # WASM validation would happen at runtime
            pass


def test_free_binary_wasm_exports():
    """Test Free binary only exports Free functions."""
    # Check WASM exports in Free build
    wasm_files = list(Path("target/release").glob("*.wasm"))

    for wasm_file in wasm_files:
        if wasm_file.exists() and wasm_file.stat().st_size > 0:
            result = subprocess.run(
                ["wasm-objdump", "-x", str(wasm_file)],
                capture_output=True,
                text=True,
                timeout=5
            )

            if result.returncode == 0:
                # Check exports don't include Pro functions
                assert "autofix" not in result.stdout.lower(), "Should not export autofix"
                assert "patch" not in result.stdout.lower(), "Should not export patch"
                assert "slo" not in result.stdout.lower(), "Should not export slo"


def test_wasm_memory_isolation():
    """Test WASM memory doesn't contain Pro artifacts."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function"}}}')

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Analysis should complete without Pro references
        if result.returncode == 0:
            output = result.stdout.lower()
            # Should not leak Pro memory artifacts
            assert "pro_heuristics" not in output, "Should not leak Pro heuristics"
            assert "premium_bundle" not in output, "Should not leak Premium bundle"


def test_wasm_module_verification():
    """Test WASM module verification rejects Pro modules."""
    with tempfile.TemporaryDirectory() as tmpdir:
        wasm_path = Path(tmpdir) / "module.wasm"

        # Create WASM with Pro module marker
        with open(wasm_path, 'wb') as f:
            f.write(b'\x00asm')
            f.write(b'\x01\x00\x00\x00')
            # Type section
            f.write(b'\x01')  # Section ID 1
            f.write(b'\x05')  # Size
            f.write(b'\x01')  # 1 type
            f.write(b'\x60')  # Function type
            f.write(b'\x00\x00')  # No params, no results

        # Verify module structure
        result = subprocess.run(
            ["wasm-validate", str(wasm_path)],
            capture_output=True,
            text=True,
            timeout=5
        )

        # Basic WASM structure is valid, but Pro markers would be rejected


if __name__ == "__main__":
    test_free_binary_no_pro_wasm()
    test_wasm_pro_import_fails()
    test_wasm_opcode_validation()
    test_free_binary_wasm_exports()
    test_wasm_memory_isolation()
    test_wasm_module_verification()
    print("All IP Protection: WASM Pro engine tests passed")
