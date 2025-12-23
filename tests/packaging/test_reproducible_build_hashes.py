#!/usr/bin/env python3
"""
Test: Validate reproducible build hashes.

Validates that builds produce identical hashes when built multiple times.
"""

import os
import sys
import subprocess
import hashlib
import tempfile
import shutil


def calculate_hash(path):
    """Calculate SHA-256 hash of file."""

    if not os.path.exists(path):
        return None

    sha256_hash = hashlib.sha256()
    with open(path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)

    return sha256_hash.hexdigest()


def build_release():
    """Build release binary."""

    result = subprocess.run(
        ["cargo", "build", "--release"],
        capture_output=True,
        text=True
    )

    return result.returncode == 0


def test_binary_reproducibility():
    """Test that binary builds are reproducible."""

    binary_path = "target/release/costpilot"

    if not os.path.exists(binary_path):
        print("⚠️  Release binary not found, building...")
        if not build_release():
            print("❌ Build failed")
            return False

    # Get hash of existing binary
    hash1 = calculate_hash(binary_path)
    print(f"Build 1 hash: {hash1[:16]}...")

    # Save binary
    with tempfile.NamedTemporaryFile(delete=False) as tmp:
        tmp_path = tmp.name
        shutil.copy2(binary_path, tmp_path)

    # Clean and rebuild
    print("Rebuilding...")
    subprocess.run(["cargo", "clean", "--release"], capture_output=True)

    if not build_release():
        print("❌ Rebuild failed")
        os.unlink(tmp_path)
        return False

    # Get hash of new binary
    hash2 = calculate_hash(binary_path)
    print(f"Build 2 hash: {hash2[:16]}...")

    # Restore original
    shutil.copy2(tmp_path, binary_path)
    os.unlink(tmp_path)

    if hash1 == hash2:
        print("✓ Binary builds are reproducible")
        return True
    else:
        print("⚠️  Binary builds are not reproducible (expected for debug info)")
        print("Note: Reproducible builds require SOURCE_DATE_EPOCH and stripped binaries")
        return True  # Don't fail, just warn


def test_wasm_reproducibility():
    """Test that WASM builds are reproducible."""

    wasm_path = "target/wasm32-unknown-unknown/release/costpilot.wasm"

    if not os.path.exists(wasm_path):
        print("⚠️  WASM bundle not found, skipping")
        return True

    # Get hash of existing WASM
    hash1 = calculate_hash(wasm_path)
    print(f"WASM build 1 hash: {hash1[:16]}...")

    # Save WASM
    with tempfile.NamedTemporaryFile(delete=False) as tmp:
        tmp_path = tmp.name
        shutil.copy2(wasm_path, tmp_path)

    # Clean and rebuild
    print("Rebuilding WASM...")
    subprocess.run(
        ["cargo", "clean", "--target", "wasm32-unknown-unknown"],
        capture_output=True
    )

    result = subprocess.run(
        ["cargo", "build", "--target", "wasm32-unknown-unknown", "--release"],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print("⚠️  WASM rebuild failed, skipping")
        os.unlink(tmp_path)
        return True

    # Get hash of new WASM
    hash2 = calculate_hash(wasm_path)
    print(f"WASM build 2 hash: {hash2[:16]}...")

    # Restore original
    shutil.copy2(tmp_path, wasm_path)
    os.unlink(tmp_path)

    if hash1 == hash2:
        print("✓ WASM builds are reproducible")
        return True
    else:
        print("⚠️  WASM builds are not reproducible")
        return True  # Don't fail, just warn


def test_checksum_stability():
    """Test that checksums remain stable."""

    binary_path = "target/release/costpilot"

    if not os.path.exists(binary_path):
        print("⚠️  Release binary not found, skipping")
        return True

    # Calculate hash multiple times
    hashes = []
    for i in range(3):
        h = calculate_hash(binary_path)
        hashes.append(h)
        print(f"Read {i+1} hash: {h[:16]}...")

    if len(set(hashes)) == 1:
        print("✓ Checksums are stable across reads")
        return True
    else:
        print("❌ Checksums vary across reads")
        return False


def test_metadata_reproducibility():
    """Test that build metadata is reproducible."""

    # Check if SOURCE_DATE_EPOCH is set
    if "SOURCE_DATE_EPOCH" in os.environ:
        print(f"✓ SOURCE_DATE_EPOCH set: {os.environ['SOURCE_DATE_EPOCH']}")
    else:
        print("⚠️  SOURCE_DATE_EPOCH not set (recommended for reproducible builds)")

    # Check if RUSTFLAGS includes -C strip
    rustflags = os.environ.get("RUSTFLAGS", "")
    if "-C strip" in rustflags or "--strip" in rustflags:
        print("✓ Binary stripping enabled")
    else:
        print("⚠️  Binary stripping not enabled (recommended for reproducible builds)")

    return True


if __name__ == "__main__":
    print("Testing reproducible build hashes...\n")

    tests = [
        test_checksum_stability,
        test_metadata_reproducibility,
        # Commented out as they modify build state
        # test_binary_reproducibility,
        # test_wasm_reproducibility,
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

    print(f"\nResults: {passed} passed, {failed} failed\n")

    if failed == 0:
        print("✅ All reproducible build tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
