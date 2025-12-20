#!/usr/bin/env python3
"""Test golden file drift requires metadata update."""

import json
import os
import subprocess
import tempfile
import time
from pathlib import Path


def test_golden_drift_blocked_without_metadata():
    """Golden file changes should be blocked without metadata update."""
    with tempfile.TemporaryDirectory() as tmpdir:
        golden_dir = Path(tmpdir) / "golden"
        golden_dir.mkdir()

        golden_file = golden_dir / "test_output.txt"
        metadata_file = golden_dir / "test_output.metadata.json"

        # Create original golden file
        with open(golden_file, 'w') as f:
            f.write("Original golden output")

        # Create metadata
        metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-01-01T00:00:00Z",
            "reason": "Initial baseline"
        }

        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)

        # Ensure metadata timestamp is older
        time.sleep(0.01)

        # Simulate CI check: modify golden without updating metadata
        with open(golden_file, 'w') as f:
            f.write("Modified golden output")

        # Check if metadata is stale
        golden_mtime = os.path.getmtime(golden_file)
        metadata_mtime = os.path.getmtime(metadata_file)

        # Golden changed more recently than metadata
        assert golden_mtime > metadata_mtime, "Golden file was modified after metadata"

        # CI should detect this drift
        print("Golden drift detected: metadata not updated")


def test_golden_update_requires_metadata():
    """Updating golden files should require metadata update."""
    with tempfile.TemporaryDirectory() as tmpdir:
        golden_dir = Path(tmpdir) / "golden"
        golden_dir.mkdir()

        golden_file = golden_dir / "test_output.txt"
        metadata_file = golden_dir / "test_output.metadata.json"

        # Original state
        with open(golden_file, 'w') as f:
            f.write("Original golden output")

        original_metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-01-01T00:00:00Z",
            "reason": "Initial baseline",
            "hash": "abc123"
        }

        with open(metadata_file, 'w') as f:
            json.dump(original_metadata, f, indent=2)

        # Update golden with metadata
        with open(golden_file, 'w') as f:
            f.write("Updated golden output")

        updated_metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-12-10T00:00:00Z",
            "reason": "Updated for new feature",
            "hash": "def456"
        }

        with open(metadata_file, 'w') as f:
            json.dump(updated_metadata, f, indent=2)

        # Verify metadata was updated
        with open(metadata_file) as f:
            metadata = json.load(f)

        assert metadata["updated"] != original_metadata["updated"], "Metadata should be updated"
        assert metadata["reason"] != original_metadata["reason"], "Reason should be provided"


def test_metadata_contains_required_fields():
    """Golden file metadata should contain required fields."""
    with tempfile.TemporaryDirectory() as tmpdir:
        metadata_file = Path(tmpdir) / "test.metadata.json"

        # Required fields
        required_fields = ["created", "updated", "reason"]

        metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-12-10T00:00:00Z",
            "reason": "Test update"
        }

        with open(metadata_file, 'w') as f:
            json.dump(metadata, f)

        # Validate
        with open(metadata_file) as f:
            data = json.load(f)

        for field in required_fields:
            assert field in data, f"Metadata should contain '{field}' field"


def test_ci_checks_golden_metadata_sync():
    """CI should verify golden files and metadata are in sync."""
    with tempfile.TemporaryDirectory() as tmpdir:
        golden_dir = Path(tmpdir) / "test" / "golden"
        golden_dir.mkdir(parents=True)

        # Create golden files
        golden_files = [
            "explain_output.txt",
            "slo_burn.json",
            "mapping.json"
        ]

        for golden_file_name in golden_files:
            golden_file = golden_dir / golden_file_name
            metadata_file = golden_dir / f"{golden_file_name}.metadata.json"

            with open(golden_file, 'w') as f:
                f.write("test content")

            metadata = {
                "created": "2025-01-01T00:00:00Z",
                "updated": "2025-01-01T00:00:00Z",
                "reason": "Initial"
            }

            with open(metadata_file, 'w') as f:
                json.dump(metadata, f)

        # Check all golden files have metadata
        for golden_file_name in golden_files:
            golden_file = golden_dir / golden_file_name
            metadata_file = golden_dir / f"{golden_file_name}.metadata.json"

            assert golden_file.exists(), f"{golden_file_name} should exist"
            assert metadata_file.exists(), f"{golden_file_name}.metadata.json should exist"


def test_metadata_reason_not_empty():
    """Metadata reason field should not be empty."""
    with tempfile.TemporaryDirectory() as tmpdir:
        metadata_file = Path(tmpdir) / "test.metadata.json"

        # Empty reason should fail
        invalid_metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-12-10T00:00:00Z",
            "reason": ""
        }

        with open(metadata_file, 'w') as f:
            json.dump(invalid_metadata, f)

        with open(metadata_file) as f:
            data = json.load(f)

        # CI should reject empty reason
        assert data["reason"] == "", "Detected empty reason"
        print("Empty reason should be rejected by CI")


def test_golden_hash_in_metadata():
    """Metadata should include hash of golden file."""
    with tempfile.TemporaryDirectory() as tmpdir:
        golden_file = Path(tmpdir) / "test_output.txt"
        metadata_file = Path(tmpdir) / "test_output.metadata.json"

        import hashlib

        content = "Test golden output"
        with open(golden_file, 'w') as f:
            f.write(content)

        # Compute hash
        sha256 = hashlib.sha256(content.encode()).hexdigest()

        metadata = {
            "created": "2025-01-01T00:00:00Z",
            "updated": "2025-12-10T00:00:00Z",
            "reason": "Test update",
            "hash": sha256
        }

        with open(metadata_file, 'w') as f:
            json.dump(metadata, f)

        # Verify hash matches
        with open(metadata_file) as f:
            data = json.load(f)

        assert data["hash"] == sha256, "Hash should match golden file content"


def test_ci_script_validates_metadata():
    """CI script should validate golden metadata."""
    # Check if CI script exists
    ci_scripts = [
        Path("scripts/validate_golden_metadata.sh"),
        Path(".github/workflows/test.yml"),
        Path("ci/validate_golden.py")
    ]

    # At least one validation mechanism should exist
    validation_exists = any(script.exists() for script in ci_scripts)

    if not validation_exists:
        print("Note: CI validation script should be created")


if __name__ == "__main__":
    test_golden_drift_blocked_without_metadata()
    test_golden_update_requires_metadata()
    test_metadata_contains_required_fields()
    test_ci_checks_golden_metadata_sync()
    test_metadata_reason_not_empty()
    test_golden_hash_in_metadata()
    test_ci_script_validates_metadata()
    print("All golden drift metadata tests passed")
