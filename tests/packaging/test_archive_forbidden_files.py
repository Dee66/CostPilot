#!/usr/bin/env python3
"""
Test: Validate archives contain no forbidden files.

Validates that release archives don't contain forbidden or sensitive files.
"""

import os
import sys
import tarfile
import zipfile


# Forbidden patterns
FORBIDDEN_PATTERNS = [
    ".git",
    ".env",
    "*.key",
    "*.pem",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "*.secret",
    "*.password",
    ".aws",
    ".ssh",
    "credentials",
    "secrets.yml",
    "secrets.yaml",
    ".npmrc",
    ".pypirc",
    "__pycache__",
    "*.pyc",
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
    "*.swp",
    "*.swo",
    "*~",
    ".vscode",
    ".idea",
    "node_modules",
    "target/debug",
]


def matches_forbidden(filename):
    """Check if filename matches forbidden patterns."""

    for pattern in FORBIDDEN_PATTERNS:
        if pattern.startswith("*"):
            # Suffix match
            if filename.endswith(pattern[1:]):
                return True, pattern
        elif pattern.endswith("*"):
            # Prefix match
            if filename.startswith(pattern[:-1]):
                return True, pattern
        else:
            # Exact match or contains
            if pattern in filename:
                return True, pattern

    return False, None


def check_tar_archive(path):
    """Check tar/tar.gz archive for forbidden files."""

    print(f"Checking {path}...")

    if not os.path.exists(path):
        print(f"⚠️  Archive not found: {path}")
        return True, []

    forbidden_files = []

    try:
        with tarfile.open(path, 'r:*') as tar:
            members = tar.getmembers()
            print(f"  Total files: {len(members)}")

            for member in members:
                forbidden, pattern = matches_forbidden(member.name)
                if forbidden:
                    forbidden_files.append((member.name, pattern))

            if forbidden_files:
                print(f"❌ Found {len(forbidden_files)} forbidden files:")
                for fname, pattern in forbidden_files[:10]:  # Show first 10
                    print(f"    - {fname} (matches {pattern})")
                if len(forbidden_files) > 10:
                    print(f"    ... and {len(forbidden_files) - 10} more")
                return False, forbidden_files
            else:
                print("✓ No forbidden files found")
                return True, []

    except Exception as e:
        print(f"❌ Error checking archive: {e}")
        return False, []


def check_zip_archive(path):
    """Check zip archive for forbidden files."""

    print(f"Checking {path}...")

    if not os.path.exists(path):
        print(f"⚠️  Archive not found: {path}")
        return True, []

    forbidden_files = []

    try:
        with zipfile.ZipFile(path, 'r') as zf:
            members = zf.namelist()
            print(f"  Total files: {len(members)}")

            for member in members:
                forbidden, pattern = matches_forbidden(member)
                if forbidden:
                    forbidden_files.append((member, pattern))

            if forbidden_files:
                print(f"❌ Found {len(forbidden_files)} forbidden files:")
                for fname, pattern in forbidden_files[:10]:  # Show first 10
                    print(f"    - {fname} (matches {pattern})")
                if len(forbidden_files) > 10:
                    print(f"    ... and {len(forbidden_files) - 10} more")
                return False, forbidden_files
            else:
                print("✓ No forbidden files found")
                return True, []

    except Exception as e:
        print(f"❌ Error checking archive: {e}")
        return False, []


def test_common_archives():
    """Test common archive patterns."""

    archives = [
        ("target/package/costpilot.tar.gz", check_tar_archive),
        ("target/package/costpilot.zip", check_zip_archive),
        ("costpilot-linux-x64.tar.gz", check_tar_archive),
        ("costpilot-macos-x64.tar.gz", check_tar_archive),
        ("costpilot-windows-x64.zip", check_zip_archive),
    ]

    passed = 0
    failed = 0
    total_forbidden = []

    for archive_path, check_func in archives:
        result, forbidden = check_func(archive_path)
        if result:
            passed += 1
        else:
            failed += 1
            total_forbidden.extend(forbidden)
        print()

    return passed, failed, total_forbidden


def test_forbidden_patterns():
    """Test that forbidden pattern matching works."""

    print("Testing forbidden pattern matching...")

    test_cases = [
        (".git/config", True),
        (".env", True),
        ("config.key", True),
        ("cert.pem", True),
        ("src/main.rs", False),
        ("README.md", False),
        ("Cargo.toml", False),
        ("target/debug/costpilot", True),
        ("target/release/costpilot", False),
        (".DS_Store", True),
        ("__pycache__/test.pyc", True),
        ("node_modules/package.json", True),
    ]

    all_passed = True

    for filename, expected_forbidden in test_cases:
        forbidden, pattern = matches_forbidden(filename)
        if forbidden == expected_forbidden:
            status = "✓"
        else:
            status = "❌"
            all_passed = False

        print(f"  {status} {filename}: forbidden={forbidden} (expected {expected_forbidden})")

    if all_passed:
        print("✓ Pattern matching works correctly")
        return True
    else:
        print("❌ Pattern matching has errors")
        return False


if __name__ == "__main__":
    print("Testing archive forbidden files...\n")

    # Test pattern matching first
    if not test_forbidden_patterns():
        print("\n❌ Pattern matching test failed")
        sys.exit(1)
    print()

    # Test archives
    passed, failed, total_forbidden = test_common_archives()

    print(f"\nResults: {passed} archives passed, {failed} failed")

    if total_forbidden:
        print(f"\nTotal forbidden files found: {len(total_forbidden)}")
        print("\nUnique patterns matched:")
        patterns = set(pattern for _, pattern in total_forbidden)
        for pattern in sorted(patterns):
            count = sum(1 for _, p in total_forbidden if p == pattern)
            print(f"  - {pattern}: {count} files")

    if failed == 0:
        print("\n✅ All archive tests passed")
        sys.exit(0)
    else:
        print(f"\n❌ {failed} archive(s) contain forbidden files")
        sys.exit(1)
