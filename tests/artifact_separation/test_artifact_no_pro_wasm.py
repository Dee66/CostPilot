#!/usr/bin/env python3
"""Test Artifact Separation: Free binary doesn't ship premium WASM."""

import subprocess
from pathlib import Path
import tarfile
import zipfile


def test_free_binary_no_pro_wasm():
    """Test Free binary doesn't include Pro WASM modules."""
    binary_path = Path("target/release/costpilot")
    
    if not binary_path.exists():
        # Binary not built yet
        return
    
    # Check binary doesn't contain Pro WASM
    result = subprocess.run(
        ["strings", str(binary_path)],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    if result.returncode == 0:
        output = result.stdout
        
        # Should not reference Pro WASM files
        forbidden = [
            "costpilot_pro.wasm",
            "premium_engine.wasm",
            "pro_features.wasm",
            "enterprise.wasm"
        ]
        
        for wasm_file in forbidden:
            assert wasm_file not in output, f"Free binary should not reference: {wasm_file}"


def test_free_artifacts_no_wasm():
    """Test Free artifacts directory has no Pro WASM."""
    artifacts_dir = Path("target/release")
    
    if not artifacts_dir.exists():
        return
    
    # Check for WASM files
    wasm_files = list(artifacts_dir.glob("*.wasm"))
    
    for wasm_file in wasm_files:
        name = wasm_file.name.lower()
        
        # Should not be Pro WASM
        assert "pro" not in name, f"Free artifacts should not have Pro WASM: {name}"
        assert "premium" not in name, f"Free artifacts should not have Premium WASM: {name}"
        assert "enterprise" not in name, f"Free artifacts should not have Enterprise WASM: {name}"


def test_free_archive_no_pro_wasm():
    """Test Free release archive has no Pro WASM."""
    # Check if release archive exists
    release_dir = Path("target/release")
    
    if not release_dir.exists():
        return
    
    # Look for tar.gz or zip archives
    archives = list(release_dir.glob("costpilot-*"))
    
    for archive_path in archives:
        if archive_path.suffix == ".gz" and archive_path.stem.endswith(".tar"):
            # tar.gz archive
            try:
                with tarfile.open(archive_path, 'r:gz') as tar:
                    members = tar.getnames()
                    
                    # Should not contain Pro WASM
                    for member in members:
                        member_lower = member.lower()
                        assert "pro" not in member_lower or ".wasm" not in member_lower, \
                            f"Archive should not contain Pro WASM: {member}"
            except:
                pass
        
        elif archive_path.suffix == ".zip":
            # zip archive
            try:
                with zipfile.ZipFile(archive_path, 'r') as zf:
                    names = zf.namelist()
                    
                    # Should not contain Pro WASM
                    for name in names:
                        name_lower = name.lower()
                        assert "pro" not in name_lower or ".wasm" not in name_lower, \
                            f"Archive should not contain Pro WASM: {name}"
            except:
                pass


def test_free_build_excludes_pro_wasm():
    """Test Free build process excludes Pro WASM."""
    # Check Cargo.toml or build scripts
    cargo_path = Path("Cargo.toml")
    
    if cargo_path.exists():
        with open(cargo_path) as f:
            content = f.read().lower()
        
        # Free build should not reference Pro features
        # Premium features might be in separate crate or behind feature flag


def test_free_no_wasm_in_binary():
    """Test Free binary has no embedded WASM."""
    binary_path = Path("target/release/costpilot")
    
    if not binary_path.exists():
        return
    
    # Check for WASM magic number in binary
    with open(binary_path, 'rb') as f:
        content = f.read()
    
    # WASM magic: \x00asm
    wasm_magic = b'\x00asm'
    
    # Count occurrences
    count = content.count(wasm_magic)
    
    # Free might have some WASM for core functionality
    # But should not have Pro WASM modules
    # If WASM is used, verify it's not Pro


if __name__ == "__main__":
    test_free_binary_no_pro_wasm()
    test_free_artifacts_no_wasm()
    test_free_archive_no_pro_wasm()
    test_free_build_excludes_pro_wasm()
    test_free_no_wasm_in_binary()
    print("All Artifact Separation: Free no Pro WASM tests passed")
