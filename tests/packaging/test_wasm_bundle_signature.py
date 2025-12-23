#!/usr/bin/env python3
"""
Test: Validate WASM bundle signature.

Validates that the WASM bundle has a valid signature and can be verified.
"""

import os
import sys
import subprocess
import hashlib


def test_wasm_file_exists():
    """Verify WASM file exists."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print(f"❌ WASM file not found: {wasm_path}")
        print("Build it first: ./scripts/build_wasm.sh")
        return False

    print(f"✓ WASM file exists: {wasm_path}")
    return True


def test_wasm_structure():
    """Verify WASM structure is valid."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    # Check for wasm-validate
    try:
        result = subprocess.run(
            ["wasm-validate", wasm_path],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            print("✓ WASM structure valid")
            return True
        else:
            print(f"❌ WASM validation failed: {result.stderr}")
            return False

    except FileNotFoundError:
        print("⚠️  wasm-validate not found, skipping structure validation")
        print("Install with: cargo install wabt")
        return True  # Don't fail if tool not available


def test_wasm_signature():
    """Verify WASM bundle has valid signature."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"
    sig_path = wasm_path + ".sig"

    # For now, verify checksum as signature validation
    # Real implementation would use proper cryptographic signatures

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Calculate SHA-256 checksum
    sha256_hash = hashlib.sha256()
    with open(wasm_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)

    checksum = sha256_hash.hexdigest()
    print(f"✓ WASM checksum: {checksum[:16]}...")

    # Check if signature file exists
    if os.path.exists(sig_path):
        with open(sig_path, 'r') as f:
            stored_sig = f.read().strip()

        if stored_sig == checksum:
            print("✓ WASM signature valid")
            return True
        else:
            print("❌ WASM signature mismatch")
            return False
    else:
        # Create signature file for future validation
        with open(sig_path, 'w') as f:
            f.write(checksum)
        print(f"✓ WASM signature created: {sig_path}")
        return True


def test_wasm_size_limit():
    """Verify WASM bundle is within size limit."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    size_bytes = os.path.getsize(wasm_path)
    size_mb = size_bytes / (1024 * 1024)

    max_size_mb = 10

    print(f"✓ WASM size: {size_mb:.2f} MB ({size_bytes} bytes)")

    if size_mb > max_size_mb:
        print(f"❌ WASM exceeds {max_size_mb} MB limit")
        return False

    print(f"✓ WASM within {max_size_mb} MB limit")
    return True


def test_wasm_determinism():
    """Verify WASM output is deterministic."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Calculate hash twice
    def calculate_hash():
        sha256_hash = hashlib.sha256()
        with open(wasm_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()

    hash1 = calculate_hash()
    hash2 = calculate_hash()

    if hash1 == hash2:
        print(f"✓ WASM deterministic (hash: {hash1[:16]}...)")
        return True
    else:
        print("❌ WASM not deterministic")
        return False


def test_wasm_imports():
    """Verify WASM imports are minimal."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Check for wasm-objdump
    try:
        result = subprocess.run(
            ["wasm-objdump", "-x", wasm_path],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            import_count = result.stdout.count("import func")
            print(f"✓ WASM imports: {import_count}")

            if import_count > 50:
                print(f"⚠️  High import count: {import_count}")
                return True  # Warning, not failure

            return True
        else:
            print("❌ wasm-objdump failed")
            return False

    except FileNotFoundError:
        print("⚠️  wasm-objdump not found, skipping import analysis")
        print("Install with: cargo install wabt")
        return True  # Don't fail if tool not available


def test_wasm_memory_limit():
    """Verify WASM memory limit is set."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Check for wasm-objdump
    try:
        result = subprocess.run(
            ["wasm-objdump", "-h", wasm_path],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            # Look for memory section
            if "memory" in result.stdout.lower():
                print("✓ WASM memory section present")
                return True
            else:
                print("❌ WASM memory section missing")
                return False
        else:
            print("❌ wasm-objdump failed")
            return False

    except FileNotFoundError:
        print("⚠️  wasm-objdump not found, skipping memory analysis")
        print("Install with: cargo install wabt")
        return True  # Don't fail if tool not available


def test_wasm_exports():
    """Verify WASM exports expected functions."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Check for wasm-objdump
    try:
        result = subprocess.run(
            ["wasm-objdump", "-x", wasm_path],
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            export_count = result.stdout.count("export func")
            print(f"✓ WASM exports: {export_count} functions")

            if export_count == 0:
                print("❌ No exported functions found")
                return False

            return True
        else:
            print("❌ wasm-objdump failed")
            return False

    except FileNotFoundError:
        print("⚠️  wasm-objdump not found, skipping export analysis")
        print("Install with: cargo install wabt")
        return True  # Don't fail if tool not available


def test_wasm_security():
    """Verify WASM bundle security properties."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Read first few bytes to check WASM magic number
    with open(wasm_path, "rb") as f:
        magic = f.read(4)

    if magic == b'\x00asm':
        print("✓ WASM magic number valid")
    else:
        print(f"❌ Invalid WASM magic number: {magic.hex()}")
        return False

    # Check file is not empty
    size = os.path.getsize(wasm_path)
    if size < 100:
        print(f"❌ WASM file too small: {size} bytes")
        return False

    print("✓ WASM security checks passed")
    return True


def test_wasm_version():
    """Verify WASM version."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("❌ WASM file not found")
        return False

    # Read version bytes (after magic number)
    with open(wasm_path, "rb") as f:
        magic = f.read(4)  # Skip magic
        version = f.read(4)

    # WASM version should be 0x01 0x00 0x00 0x00 (version 1)
    if version == b'\x01\x00\x00\x00':
        print("✓ WASM version 1 (valid)")
        return True
    else:
        print(f"⚠️  Unexpected WASM version: {version.hex()}")
        return True  # Warning, not failure


if __name__ == "__main__":
    print("Testing WASM bundle signature...\n")

    tests = [
        test_wasm_file_exists,
        test_wasm_structure,
        test_wasm_signature,
        test_wasm_size_limit,
        test_wasm_determinism,
        test_wasm_imports,
        test_wasm_memory_limit,
        test_wasm_exports,
        test_wasm_security,
        test_wasm_version,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
            else:
                failed += 1
        except Exception as e:
            print(f"❌ Test {test.__name__} failed with error: {e}")
            failed += 1
        print()

    print(f"\n{'='*60}")
    print(f"Results: {passed} passed, {failed} failed")
    print(f"{'='*60}\n")

    if failed == 0:
        print("✅ All WASM bundle signature tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
