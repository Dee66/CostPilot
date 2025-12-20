#!/usr/bin/env python3
"""
Test: Validate zip/tar artifacts are reproducible (byte-identical).

Builds release bundles twice and verifies identical hashes.
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


def create_bundle(version, platform, output_dir):
    """Create release bundle using the packaging script."""
    script_path = "scripts/packaging_tools/make_release_bundle.sh"
    if not os.path.exists(script_path):
        # Try alternative path
        script_path = "packaging/make_release_bundle.sh"
        if not os.path.exists(script_path):
            raise FileNotFoundError(f"Bundle script not found at {script_path}")

    result = subprocess.run(
        ["bash", script_path, output_dir, version, platform],
        capture_output=True,
        text=True,
        env={**os.environ, "SOURCE_DATE_EPOCH": "0"}  # For determinism
    )
    if result.returncode != 0:
        print(f"Script failed: {result.stderr}")
    return result.returncode == 0


def test_archive_reproducibility():
    """Test that zip/tar archives are byte-identical across builds."""

    # Ensure release binary exists
    binary_path = "target/release/costpilot"
    if not os.path.exists(binary_path):
        print("Building release binary...")
        if not build_release():
            raise RuntimeError("Failed to build release binary")

    version = "1.0.0-test"
    platform = "linux-x64"

    # Create two temporary directories for bundles
    with tempfile.TemporaryDirectory() as temp_dir1, \
         tempfile.TemporaryDirectory() as temp_dir2:

        # First build
        print("Creating first bundle...")
        if not create_bundle(version, platform, temp_dir1):
            raise RuntimeError("Failed to create first bundle")

        # Second build
        print("Creating second bundle...")
        if not create_bundle(version, platform, temp_dir2):
            raise RuntimeError("Failed to create second bundle")

        # Check tar.gz
        tar1 = os.path.join(temp_dir1, f"costpilot-{version}-{platform}.tar.gz")
        tar2 = os.path.join(temp_dir2, f"costpilot-{version}-{platform}.tar.gz")

        if not os.path.exists(tar1) or not os.path.exists(tar2):
            raise FileNotFoundError("TAR.GZ bundle not created")

        hash1_tar = calculate_hash(tar1)
        hash2_tar = calculate_hash(tar2)

        print(f"TAR1 hash: {hash1_tar}")
        print(f"TAR2 hash: {hash2_tar}")

        if hash1_tar != hash2_tar:
            print("TAR.GZ bundles differ - investigating...")
            # Extract and compare contents
            import tarfile
            with tarfile.open(tar1, 'r:gz') as t1, tarfile.open(tar2, 'r:gz') as t2:
                members1 = sorted(t1.getmembers(), key=lambda m: m.name)
                members2 = sorted(t2.getmembers(), key=lambda m: m.name)
                if len(members1) != len(members2):
                    print(f"Different number of files: {len(members1)} vs {len(members2)}")
                else:
                    for m1, m2 in zip(members1, members2):
                        if m1.name != m2.name:
                            print(f"Different file: {m1.name} vs {m2.name}")
                        elif m1.size != m2.size:
                            print(f"Different size for {m1.name}: {m1.size} vs {m2.size}")
                        elif m1.mtime != m2.mtime:
                            print(f"Different mtime for {m1.name}: {m1.mtime} vs {m2.mtime}")
            raise AssertionError(f"TAR.GZ bundles differ: {hash1_tar} != {hash2_tar}")

        # Check zip
        zip1 = os.path.join(temp_dir1, f"costpilot-{version}-{platform}.zip")
        zip2 = os.path.join(temp_dir2, f"costpilot-{version}-{platform}.zip")

        if not os.path.exists(zip1) or not os.path.exists(zip2):
            raise FileNotFoundError("ZIP bundle not created")

        hash1_zip = calculate_hash(zip1)
        hash2_zip = calculate_hash(zip2)

        if hash1_zip != hash2_zip:
            raise AssertionError(f"ZIP bundles differ: {hash1_zip} != {hash2_zip}")

        print("✅ Archives are reproducible")


if __name__ == "__main__":
    test_archive_reproducibility()
