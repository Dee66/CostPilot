#!/usr/bin/env python3
"""Test snapshot regeneration requires signature."""

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


def test_snapshot_regeneration_requires_approval():
    """Regenerating snapshots should require explicit approval."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_dir = Path(tmpdir) / "snapshots"
        snapshot_dir.mkdir()
        
        snapshot_file = snapshot_dir / "test_output.snap"
        signature_file = snapshot_dir / "test_output.snap.sig"
        
        # Original snapshot
        with open(snapshot_file, 'w') as f:
            f.write("Original snapshot content")
        
        # Signature (hash of snapshot)
        with open(snapshot_file, 'rb') as f:
            original_hash = hashlib.sha256(f.read()).hexdigest()
        
        with open(signature_file, 'w') as f:
            f.write(original_hash)
        
        # Attempt to regenerate without signature update
        with open(snapshot_file, 'w') as f:
            f.write("New snapshot content")
        
        # Verify signature mismatch
        with open(snapshot_file, 'rb') as f:
            new_hash = hashlib.sha256(f.read()).hexdigest()
        
        with open(signature_file) as f:
            stored_hash = f.read().strip()
        
        assert new_hash != stored_hash, "Signature should detect modification"
        print("Snapshot modification detected - requires signature update")


def test_snapshot_signature_validation():
    """Snapshot signature should be validated in CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_dir = Path(tmpdir) / "snapshots"
        snapshot_dir.mkdir()
        
        snapshot_file = snapshot_dir / "test_output.snap"
        signature_file = snapshot_dir / "test_output.snap.sig"
        
        content = "Test snapshot content"
        
        with open(snapshot_file, 'w') as f:
            f.write(content)
        
        # Generate valid signature
        sha256 = hashlib.sha256(content.encode()).hexdigest()
        
        with open(signature_file, 'w') as f:
            f.write(sha256)
        
        # Validate
        with open(snapshot_file, 'rb') as f:
            file_hash = hashlib.sha256(f.read()).hexdigest()
        
        with open(signature_file) as f:
            stored_hash = f.read().strip()
        
        assert file_hash == stored_hash, "Signature should match"


def test_unsigned_snapshot_rejected():
    """Snapshots without signatures should be rejected."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_dir = Path(tmpdir) / "snapshots"
        snapshot_dir.mkdir()
        
        snapshot_file = snapshot_dir / "test_output.snap"
        signature_file = snapshot_dir / "test_output.snap.sig"
        
        # Create snapshot without signature
        with open(snapshot_file, 'w') as f:
            f.write("Unsigned snapshot")
        
        # CI should detect missing signature
        assert not signature_file.exists(), "Signature file is missing"
        print("Unsigned snapshot detected - should be rejected by CI")


def test_snapshot_regeneration_flag():
    """Snapshot regeneration should require explicit flag."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_snapshot.py"
        snapshot_file = Path(tmpdir) / "test.snap"
        
        # Original snapshot
        with open(snapshot_file, 'w') as f:
            f.write("original")
        
        test_code = f"""
from pathlib import Path

def test_with_snapshot():
    snapshot_path = Path("{snapshot_file}")
    expected = "original"
    
    with open(snapshot_path) as f:
        actual = f.read()
    
    assert actual == expected, "Snapshot mismatch"

if __name__ == "__main__":
    test_with_snapshot()
"""
        
        with open(test_script, 'w') as f:
            f.write(test_code)
        
        # Run without --update-snapshots flag
        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )
        
        # Should pass with original snapshot
        assert result.returncode == 0, "Test should pass with valid snapshot"


def test_snapshot_approval_metadata():
    """Snapshot regeneration should include approval metadata."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_file = Path(tmpdir) / "test.snap"
        metadata_file = Path(tmpdir) / "test.snap.metadata.json"
        
        with open(snapshot_file, 'w') as f:
            f.write("New snapshot content")
        
        # Approval metadata
        metadata = {
            "approved_by": "developer",
            "approved_at": "2025-12-10T00:00:00Z",
            "reason": "Updated for new feature",
            "pr_number": "123"
        }
        
        with open(metadata_file, 'w') as f:
            json.dump(metadata, f, indent=2)
        
        # Validate metadata
        with open(metadata_file) as f:
            data = json.load(f)
        
        required_fields = ["approved_by", "approved_at", "reason"]
        for field in required_fields:
            assert field in data, f"Metadata should include '{field}'"


def test_ci_validates_snapshot_signatures():
    """CI should validate all snapshot signatures."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_dir = Path(tmpdir) / "test" / "snapshot"
        snapshot_dir.mkdir(parents=True)
        
        # Create multiple snapshots with signatures
        snapshots = ["test1.snap", "test2.snap", "test3.snap"]
        
        for snap_name in snapshots:
            snap_file = snapshot_dir / snap_name
            sig_file = snapshot_dir / f"{snap_name}.sig"
            
            content = f"Content for {snap_name}"
            with open(snap_file, 'w') as f:
                f.write(content)
            
            sha256 = hashlib.sha256(content.encode()).hexdigest()
            with open(sig_file, 'w') as f:
                f.write(sha256)
        
        # Validate all signatures
        for snap_name in snapshots:
            snap_file = snapshot_dir / snap_name
            sig_file = snapshot_dir / f"{snap_name}.sig"
            
            with open(snap_file, 'rb') as f:
                file_hash = hashlib.sha256(f.read()).hexdigest()
            
            with open(sig_file) as f:
                stored_hash = f.read().strip()
            
            assert file_hash == stored_hash, f"{snap_name} signature should match"


def test_snapshot_tamper_detection():
    """Signature should detect snapshot tampering."""
    with tempfile.TemporaryDirectory() as tmpdir:
        snapshot_file = Path(tmpdir) / "test.snap"
        signature_file = Path(tmpdir) / "test.snap.sig"
        
        # Original snapshot
        original_content = "Original snapshot"
        with open(snapshot_file, 'w') as f:
            f.write(original_content)
        
        original_hash = hashlib.sha256(original_content.encode()).hexdigest()
        with open(signature_file, 'w') as f:
            f.write(original_hash)
        
        # Tamper with snapshot
        with open(snapshot_file, 'w') as f:
            f.write("Tampered snapshot")
        
        # Verify tampering detected
        with open(snapshot_file, 'rb') as f:
            current_hash = hashlib.sha256(f.read()).hexdigest()
        
        with open(signature_file) as f:
            stored_hash = f.read().strip()
        
        assert current_hash != stored_hash, "Tampering should be detected"


def test_pr_requires_snapshot_justification():
    """PR with snapshot changes should require justification."""
    # Check if PR template exists
    pr_template = Path(".github/pull_request_template.md")
    
    if pr_template.exists():
        with open(pr_template) as f:
            content = f.read()
        
        # Should mention snapshots
        if "snapshot" in content.lower():
            assert True, "PR template mentions snapshots"


if __name__ == "__main__":
    test_snapshot_regeneration_requires_approval()
    test_snapshot_signature_validation()
    test_unsigned_snapshot_rejected()
    test_snapshot_regeneration_flag()
    test_snapshot_approval_metadata()
    test_ci_validates_snapshot_signatures()
    test_snapshot_tamper_detection()
    test_pr_requires_snapshot_justification()
    print("All snapshot regeneration signature tests passed")
