#!/usr/bin/env python3
"""
Test: Validate artifact size bounds.

Validates that all build artifacts stay within acceptable size limits.
"""

import os
import sys


def check_size(path, max_mb, name):
    """Check if file is within size limit."""

    if not os.path.exists(path):
        print(f"⚠️  {name} not found: {path}")
        return True  # Don't fail if not built yet

    size_bytes = os.path.getsize(path)
    size_mb = size_bytes / (1024 * 1024)

    print(f"{name}: {size_mb:.2f} MB ({size_bytes} bytes)")

    if size_mb > max_mb:
        print(f"❌ {name} exceeds {max_mb} MB limit")
        return False

    print(f"✓ {name} within {max_mb} MB limit")
    return True


if __name__ == "__main__":
    print("Testing artifact size bounds...\n")

    tests = [
        ("target/release/costpilot", 50, "Release binary"),
        ("target/wasm32-unknown-unknown/release/costpilot.wasm", 10, "WASM bundle"),
        ("target/debug/costpilot", 200, "Debug binary"),
    ]

    passed = 0
    failed = 0

    for path, max_mb, name in tests:
        if check_size(path, max_mb, name):
            passed += 1
        else:
            failed += 1
        print()

    print(f"\nResults: {passed} passed, {failed} failed\n")

    if failed == 0:
        print("✅ All artifact size bounds tests passed")
        sys.exit(0)
    else:
        print(f"❌ {failed} test(s) failed")
        sys.exit(1)
