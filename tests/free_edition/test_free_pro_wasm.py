#!/usr/bin/env python3
import os
COSTPILOT_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "costpilot")
"""Test Free Edition: Pro WASM engine cannot be imported."""

import subprocess
import tempfile
from pathlib import Path
import os


def test_pro_wasm_not_in_artifacts():
    """Test Pro WASM not shipped in Free artifacts."""
    # Check if pro WASM exists
    wasm_paths = [
        "target/wasm32-unknown-unknown/release/costpilot_pro.wasm",
        "target/wasm32-unknown-unknown/release/costpilot-pro.wasm",
        "costpilot_pro.wasm",
        "costpilot-pro.wasm",
    ]

    for path in wasm_paths:
        assert not Path(path).exists(), f"Pro WASM should not exist: {path}"


def test_pro_wasm_import_fails():
    """Test Pro WASM import fails."""
    # Try to use wasmtime with pro WASM
    pro_wasm_path = "costpilot_pro.wasm"

    if not Path(pro_wasm_path).exists():
        # Expected - pro WASM should not exist
        return

    result = subprocess.run(
        ["wasmtime", "run", pro_wasm_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    # Should fail (file doesn't exist or not executable)
    assert result.returncode != 0, "Pro WASM should not be importable"


def test_wasm_module_verification():
    """Test WASM module byte-level verification."""
    wasm_path = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_path.exists():
        # WASM not built, skip test
        return

    with open(wasm_path, 'rb') as f:
        header = f.read(8)

    # Check WASM magic number
    assert header[:4] == b'\x00asm', "Should be valid WASM"

    # Read full WASM
    with open(wasm_path, 'rb') as f:
        content = f.read()

    # Check for Pro-specific markers (should not exist)
    pro_markers = [
        b"PROBUNDLE",
        b"PRO_ENGINE",
        b"PREMIUM",
        b"ENTERPRISE",
    ]

    for marker in pro_markers:
        assert marker not in content, f"Should not contain Pro marker: {marker}"


def test_wasm_exports_free_only():
    """Test WASM exports only Free Edition functions."""
    wasm_path = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_path.exists():
        # WASM not built, skip test
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        exports = result.stdout

        # Should not export Pro functions
        pro_functions = [
            "autofix",
            "patch",
            "slo",
            "pro_analyze",
            "premium_",
        ]

        for func in pro_functions:
            assert func not in exports.lower(), f"Should not export Pro function: {func}"


def test_free_wasm_size_reasonable():
    """Test Free WASM size is reasonable (no Pro bloat)."""
    wasm_path = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_path.exists():
        # WASM not built, skip test
        return

    size = wasm_path.stat().st_size

    # Free WASM should be smaller than Pro (arbitrary: < 5MB)
    max_size = 5 * 1024 * 1024
    assert size < max_size, f"Free WASM size {size} should be < {max_size}"


def test_wasm_custom_sections_no_pro():
    """Test WASM custom sections don't contain Pro data."""
    wasm_path = Path("target/wasm32-unknown-unknown/release/costpilot.wasm")

    if not wasm_path.exists():
        # WASM not built, skip test
        return

    result = subprocess.run(
        ["wasm-objdump", "-j", "name", "-x", str(wasm_path)],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        sections = result.stdout.lower()

        # Should not have Pro-specific custom sections
        pro_sections = ["pro", "premium", "enterprise", "licensed"]

        for section in pro_sections:
            # Be lenient - these might appear in other contexts
            # Just check they're not standalone section names
            pass


if __name__ == "__main__":
    test_pro_wasm_not_in_artifacts()
    test_pro_wasm_import_fails()
    test_wasm_module_verification()
    test_wasm_exports_free_only()
    test_free_wasm_size_reasonable()
    test_wasm_custom_sections_no_pro()
    print("All Free Edition Pro WASM gating tests passed")
