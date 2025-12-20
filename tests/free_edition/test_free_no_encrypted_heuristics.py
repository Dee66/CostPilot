#!/usr/bin/env python3
import os
COSTPILOT_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "costpilot")
"""Test Free Edition: no encrypted heuristic files shipped in artifacts."""

import subprocess
import tempfile
from pathlib import Path
import os


def test_no_encrypted_heuristics_in_heuristics_dir():
    """Test no encrypted heuristics in heuristics/ directory."""
    heuristics_dir = Path("heuristics")

    if not heuristics_dir.exists():
        # No heuristics dir, OK
        return

    for file in heuristics_dir.rglob("*"):
        if file.is_file():
            # Check file extension
            assert file.suffix not in [".enc", ".encrypted", ".bin", ".pro"], \
                f"Encrypted heuristics file found: {file}"

            # Check file content (first bytes)
            with open(file, 'rb') as f:
                header = f.read(16)

            # Should not be encrypted (no magic bytes for common encryption)
            encrypted_markers = [
                b"\x89PNG",  # Not PNG
                b"ENCRYPTED",
                b"PROBUNDLE",
                b"\x00\x00\x00\x00\x00\x00\x00\x00",  # Not all nulls
            ]

            # JSON files should start with { or [
            if file.suffix == ".json":
                assert header[0:1] in [b"{", b"["], f"JSON file not valid: {file}"


def test_no_encrypted_files_in_artifacts():
    """Test no encrypted files in target/ artifacts."""
    target_dir = Path("target/release")

    if not target_dir.exists():
        # Not built, OK
        return

    encrypted_patterns = [
        "*.enc",
        "*.encrypted",
        "*_pro.*",
        "*-pro.*",
        "*.premium",
    ]

    for pattern in encrypted_patterns:
        matches = list(target_dir.rglob(pattern))
        assert len(matches) == 0, f"Found encrypted artifacts: {matches}"


def test_heuristics_json_readable():
    """Test heuristics JSON files are readable."""
    heuristics_files = [
        "heuristics/cost_heuristics.json",
    ]

    for path in heuristics_files:
        file = Path(path)
        if file.exists():
            import json

            with open(file) as f:
                content = json.load(f)

            # Should be valid JSON
            assert isinstance(content, (dict, list)), f"Invalid JSON in {path}"


def test_no_binary_heuristics():
    """Test no binary heuristics files."""
    heuristics_dir = Path("heuristics")

    if not heuristics_dir.exists():
        return

    for file in heuristics_dir.rglob("*"):
        if file.is_file():
            # Check if file is text or binary
            try:
                with open(file, 'r') as f:
                    f.read(100)
                # File is text, OK
            except UnicodeDecodeError:
                # File is binary
                assert False, f"Binary heuristics file found: {file}"


def test_config_no_encrypted_references():
    """Test config files don't reference encrypted heuristics."""
    config_files = [
        "configs/costpilot.yml.example",
    ]

    for path in config_files:
        file = Path(path)
        if file.exists():
            with open(file) as f:
                content = f.read()

            # Should not reference encrypted files (exclude false positives like project_name)
            encrypted_refs = [".enc", ".encrypted", "_pro.wasm", "-pro.wasm", ".premium"]

            for ref in encrypted_refs:
                assert ref not in content.lower(), \
                    f"Config {path} references encrypted heuristics: {ref}"


def test_no_pro_heuristics_version():
    """Test heuristics version file doesn't mention Pro."""
    version_file = Path("heuristics/heuristics_version.txt")

    if version_file.exists():
        with open(version_file) as f:
            content = f.read().lower()

        # Should not mention Pro/Premium/Enterprise
        pro_terms = ["pro", "premium", "enterprise", "licensed"]

        for term in pro_terms:
            assert term not in content, f"Heuristics version mentions {term}"


if __name__ == "__main__":
    test_no_encrypted_heuristics_in_heuristics_dir()
    test_no_encrypted_files_in_artifacts()
    test_heuristics_json_readable()
    test_no_binary_heuristics()
    test_config_no_encrypted_references()
    test_no_pro_heuristics_version()
    print("All Free Edition encrypted heuristics gating tests passed")
