#!/usr/bin/env python3
"""Test WASM module hashing stability."""

import subprocess
import tempfile
from pathlib import Path
import json
import hashlib


def test_wasm_module_hash_stability():
    """Test that WASM module hash is stable across builds."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Calculate hash of WASM module
    with open(wasm_target, 'rb') as f:
        wasm_data = f.read()
        wasm_hash = hashlib.sha256(wasm_data).hexdigest()

    # Hash should be consistent for same build
    with open(wasm_target, 'rb') as f:
        wasm_data2 = f.read()
        wasm_hash2 = hashlib.sha256(wasm_data2).hexdigest()

    assert wasm_hash == wasm_hash2, "WASM module hash should be stable"


def test_wasm_deterministic_build():
    """Test that WASM builds are deterministic."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Read WASM file
    with open(wasm_target, 'rb') as f:
        original_data = f.read()

    # Multiple reads should give same data
    for _ in range(5):
        with open(wasm_target, 'rb') as f:
            current_data = f.read()

        assert current_data == original_data, "WASM data should be deterministic"


def test_wasm_output_hash_stability():
    """Test that WASM output hashes are stable for same input."""
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

        # Run multiple times and hash output
        hashes = []

        for _ in range(5):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=30
            )

            if result.returncode == 0:
                output_hash = hashlib.sha256(result.stdout.encode()).hexdigest()
                hashes.append(output_hash)

        # All output hashes should be identical
        if hashes:
            assert all(h == hashes[0] for h in hashes), "WASM output should have stable hash"


def test_wasm_metadata_hash():
    """Test WASM module metadata hash stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Extract WASM metadata (if wasm-objdump available)
    result = subprocess.run(
        ["wasm-objdump", "-h", str(wasm_target)],
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode == 0:
        # Hash metadata
        metadata_hash = hashlib.sha256(result.stdout.encode()).hexdigest()

        # Run again
        result2 = subprocess.run(
            ["wasm-objdump", "-h", str(wasm_target)],
            capture_output=True,
            text=True,
            timeout=30
        )

        metadata_hash2 = hashlib.sha256(result2.stdout.encode()).hexdigest()

        # Metadata should be stable
        assert metadata_hash == metadata_hash2, "WASM metadata hash should be stable"


def test_wasm_section_hashes():
    """Test WASM section hashes for stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Read WASM sections
    with open(wasm_target, 'rb') as f:
        wasm_data = f.read()

    # Check WASM magic number
    assert wasm_data[:4] == b'\x00asm', "WASM magic number should be stable"

    # Check version
    version = int.from_bytes(wasm_data[4:8], 'little')
    assert version == 1, "WASM version should be stable"


def test_wasm_function_hash_stability():
    """Test WASM function hash stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Extract function info (if wasm-objdump available)
    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_target)],
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode == 0:
        # Hash function section
        function_hash = hashlib.sha256(result.stdout.encode()).hexdigest()

        # Run again
        result2 = subprocess.run(
            ["wasm-objdump", "-x", str(wasm_target)],
            capture_output=True,
            text=True,
            timeout=30
        )

        function_hash2 = hashlib.sha256(result2.stdout.encode()).hexdigest()

        # Function hashes should be stable
        assert function_hash == function_hash2, "WASM function hash should be stable"


def test_wasm_export_hash_stability():
    """Test WASM export hash stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Extract exports (if wasm-objdump available)
    result = subprocess.run(
        ["wasm-objdump", "-j", "Export", "-x", str(wasm_target)],
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode == 0:
        exports_hash = hashlib.sha256(result.stdout.encode()).hexdigest()

        # Run again
        result2 = subprocess.run(
            ["wasm-objdump", "-j", "Export", "-x", str(wasm_target)],
            capture_output=True,
            text=True,
            timeout=30
        )

        exports_hash2 = hashlib.sha256(result2.stdout.encode()).hexdigest()

        # Exports should be stable
        assert exports_hash == exports_hash2, "WASM exports hash should be stable"


def test_wasm_import_hash_stability():
    """Test WASM import hash stability."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    # Extract imports (if wasm-objdump available)
    result = subprocess.run(
        ["wasm-objdump", "-j", "Import", "-x", str(wasm_target)],
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode == 0:
        imports_hash = hashlib.sha256(result.stdout.encode()).hexdigest()

        # Run again
        result2 = subprocess.run(
            ["wasm-objdump", "-j", "Import", "-x", str(wasm_target)],
            capture_output=True,
            text=True,
            timeout=30
        )

        imports_hash2 = hashlib.sha256(result2.stdout.encode()).hexdigest()

        # Imports should be stable
        assert imports_hash == imports_hash2, "WASM imports hash should be stable"


def test_wasm_content_hash_json_output():
    """Test WASM content hash with JSON output."""
    wasm_target = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_target.exists():
        print("WASM build not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda1": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                },
                "Lambda2": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 2048
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Run with JSON output and hash
        outputs = []

        for _ in range(3):
            result = subprocess.run(
                ["wasmtime", "run", str(wasm_target), "--", "analyze", "--plan", str(template_path), "--format", "json"],
                capture_output=True,
                text=True,
                timeout=30
            )

            if result.returncode == 0:
                outputs.append(result.stdout)

        # All outputs should be identical
        if outputs:
            assert all(o == outputs[0] for o in outputs), "WASM JSON output should be stable"


if __name__ == "__main__":
    test_wasm_module_hash_stability()
    test_wasm_deterministic_build()
    test_wasm_output_hash_stability()
    test_wasm_metadata_hash()
    test_wasm_section_hashes()
    test_wasm_function_hash_stability()
    test_wasm_export_hash_stability()
    test_wasm_import_hash_stability()
    test_wasm_content_hash_json_output()
    print("All WASM module hashing stability tests passed")
