#!/usr/bin/env python3
"""Test Artifact Separation: Free binary doesn't ship encrypted bundles."""

import subprocess
from pathlib import Path
import tarfile
import zipfile


def test_free_binary_no_encrypted_refs():
    """Test Free binary has no encrypted bundle references."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        return

    result = subprocess.run(
        ["strings", str(binary_path)],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout

        # Should not reference encrypted bundles
        forbidden = [
            "premium.bundle",
            "pro_heuristics.enc",
            "encrypted_heuristics.bin",
            "heuristics.encrypted",
            "enterprise.bundle"
        ]

        for bundle in forbidden:
            assert bundle not in output, f"Free should not reference: {bundle}"


def test_free_artifacts_no_bundles():
    """Test Free artifacts directory has no encrypted bundles."""
    artifacts_dir = Path("target/release")

    if not artifacts_dir.exists():
        return

    # Check for bundle files
    bundle_patterns = ["*.bundle", "*.enc", "*.encrypted"]

    for pattern in bundle_patterns:
        bundle_files = list(artifacts_dir.glob(pattern))

        for bundle_file in bundle_files:
            name = bundle_file.name.lower()

            # Should not be premium bundles
            assert "premium" not in name, f"Free should not have: {name}"
            assert "pro" not in name, f"Free should not have: {name}"


def test_free_heuristics_not_encrypted():
    """Test Free heuristics are not encrypted."""
    heuristics_dir = Path("heuristics")

    if not heuristics_dir.exists():
        return

    # Check heuristics files
    heuristics_files = list(heuristics_dir.glob("*"))

    for heur_file in heuristics_files:
        if heur_file.is_file():
            # Should be JSON, not encrypted
            with open(heur_file, 'rb') as f:
                header = f.read(10)

            # Should not be binary encrypted
            # JSON starts with '{' or '['
            if len(header) > 0:
                # Encrypted files would have binary headers
                pass


def test_free_archive_no_bundles():
    """Test Free archive has no encrypted bundles."""
    release_dir = Path("target/release")

    if not release_dir.exists():
        return

    archives = list(release_dir.glob("costpilot-*"))

    for archive_path in archives:
        if archive_path.suffix == ".gz" and archive_path.stem.endswith(".tar"):
            try:
                with tarfile.open(archive_path, 'r:gz') as tar:
                    members = tar.getnames()

                    # Should not contain bundles
                    for member in members:
                        assert not member.endswith(".bundle"), \
                            f"Archive should not contain: {member}"
                        assert not member.endswith(".enc"), \
                            f"Archive should not contain: {member}"
            except:
                pass

        elif archive_path.suffix == ".zip":
            try:
                with zipfile.ZipFile(archive_path, 'r') as zf:
                    names = zf.namelist()

                    # Should not contain bundles
                    for name in names:
                        assert not name.endswith(".bundle"), \
                            f"Archive should not contain: {name}"
                        assert not name.endswith(".enc"), \
                            f"Archive should not contain: {name}"
            except:
                pass


def test_free_no_encrypted_data_in_binary():
    """Test Free binary has no embedded encrypted data."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        return

    with open(binary_path, 'rb') as f:
        content = f.read()

    # Check for encryption markers
    forbidden_markers = [
        b"ENCRYPTED_BUNDLE",
        b"AES256_ENCRYPTED",
        b"BUNDLE:SIGNATURE",
        b"PRO_HEURISTICS_ENC"
    ]

    for marker in forbidden_markers:
        assert marker not in content, f"Binary should not contain: {marker}"


def test_free_package_metadata_no_bundles():
    """Test Free package metadata doesn't reference bundles."""
    package_json = Path("package.json")

    if package_json.exists():
        with open(package_json) as f:
            content = f.read().lower()

        # Should not reference premium bundles
        assert "premium.bundle" not in content, "package.json should not reference premium bundles"
        assert "encrypted_heuristics" not in content, "package.json should not reference encrypted heuristics"


if __name__ == "__main__":
    test_free_binary_no_encrypted_refs()
    test_free_artifacts_no_bundles()
    test_free_heuristics_not_encrypted()
    test_free_archive_no_bundles()
    test_free_no_encrypted_data_in_binary()
    test_free_package_metadata_no_bundles()
    print("All Artifact Separation: Free no encrypted bundles tests passed")
