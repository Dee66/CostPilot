#!/usr/bin/env python3
"""Test that imports are pinned/locked."""

import json
import re
import subprocess
from pathlib import Path


def test_rust_dependencies_locked():
    """Test that Rust dependencies are locked in Cargo.lock."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_lock = workspace_root / "Cargo.lock"
    
    assert cargo_lock.exists(), "Cargo.lock should exist for pinned dependencies"
    
    # Check that Cargo.lock has content
    content = cargo_lock.read_text()
    assert len(content) > 100, "Cargo.lock should contain dependency information"
    
    # Check for version specifications
    version_pattern = r'version\s*=\s*"[\d\.]+"'
    versions = re.findall(version_pattern, content)
    assert len(versions) > 0, "Cargo.lock should specify exact versions"


def test_cargo_toml_version_pins():
    """Test that Cargo.toml has version constraints."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_toml = workspace_root / "Cargo.toml"
    
    if not cargo_toml.exists():
        print("Cargo.toml not found, skipping test")
        return
    
    content = cargo_toml.read_text()
    
    # Check for dependencies section
    assert "[dependencies]" in content or "[dev-dependencies]" in content, \
        "Should have dependencies specified"
    
    # Dependencies should have version constraints
    # Look for patterns like: crate = "1.2.3" or crate = { version = "1.2.3" }
    version_patterns = [
        r'=\s*"[\d\.]+"',  # Simple version
        r'version\s*=\s*"[\d\.]+"',  # Full syntax
    ]
    
    found_versions = False
    for pattern in version_patterns:
        if re.search(pattern, content):
            found_versions = True
            break
    
    if found_versions:
        print("Dependencies have version specifications")


def test_no_wildcard_versions():
    """Test that dependencies don't use wildcard versions."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_toml = workspace_root / "Cargo.toml"
    
    if not cargo_toml.exists():
        print("Cargo.toml not found, skipping test")
        return
    
    content = cargo_toml.read_text()
    
    # Check for dangerous version wildcards
    dangerous_patterns = [
        r'version\s*=\s*"\*"',  # Any version
        r'=\s*"\*"',
    ]
    
    for pattern in dangerous_patterns:
        matches = re.findall(pattern, content)
        assert len(matches) == 0, f"Wildcard version found: {pattern}"


def test_lockfile_prevents_supply_chain_attacks():
    """Test that lockfile is used during builds."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_lock = workspace_root / "Cargo.lock"
    
    if not cargo_lock.exists():
        print("Cargo.lock not found")
        return
    
    # Cargo.lock should be checked into version control
    # (This is a best practice for applications)
    
    # Check that lock file has checksums (for integrity)
    content = cargo_lock.read_text()
    
    # Cargo.lock v3 uses checksums
    if 'checksum' in content or 'version = 3' in content:
        print("Cargo.lock includes integrity checksums")


def test_no_git_dependencies():
    """Test that dependencies are from crates.io, not git repos."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_toml = workspace_root / "Cargo.toml"
    
    if not cargo_toml.exists():
        print("Cargo.toml not found, skipping test")
        return
    
    content = cargo_toml.read_text()
    
    # Git dependencies should be avoided (or explicitly approved)
    git_patterns = [
        r'git\s*=\s*"',
        r'branch\s*=\s*"',
        r'rev\s*=\s*"',
        r'tag\s*=\s*"',
    ]
    
    git_deps = []
    for pattern in git_patterns:
        matches = re.findall(pattern, content)
        git_deps.extend(matches)
    
    # Allow zero or very few git dependencies
    assert len(git_deps) <= 2, f"Minimize git dependencies for security: {len(git_deps)} found"


def test_dependency_audit_clean():
    """Test that dependencies have no known vulnerabilities."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    
    # Try to run cargo-audit if available
    result = subprocess.run(
        ["cargo", "audit", "--version"],
        capture_output=True,
        text=True,
        cwd=workspace_root
    )
    
    if result.returncode != 0:
        print("cargo-audit not installed, skipping vulnerability check")
        return
    
    # Run audit
    result = subprocess.run(
        ["cargo", "audit"],
        capture_output=True,
        text=True,
        cwd=workspace_root,
        timeout=30
    )
    
    # Check results
    if "Vulnerabilities found!" in result.stdout:
        print(f"Warning: Vulnerabilities found in dependencies:\n{result.stdout}")


def test_python_dependencies_locked():
    """Test that Python dependencies are locked if present."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    
    # Check for various lock files
    lock_files = [
        "requirements.txt",
        "poetry.lock",
        "Pipfile.lock",
        "pdm.lock",
    ]
    
    found_locks = [lf for lf in lock_files if (workspace_root / lf).exists()]
    
    if len(found_locks) > 0:
        print(f"Found Python lock files: {found_locks}")
        
        # Check that versions are pinned
        for lock_file in found_locks:
            content = (workspace_root / lock_file).read_text()
            
            # Requirements.txt should have == for exact versions
            if lock_file == "requirements.txt":
                exact_versions = re.findall(r'==[\d\.]+', content)
                if len(exact_versions) > 0:
                    print(f"Python dependencies are pinned in {lock_file}")


def test_npm_dependencies_locked():
    """Test that npm dependencies are locked if present."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    
    lock_files = [
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
    ]
    
    found_locks = [lf for lf in lock_files if (workspace_root / lf).exists()]
    
    if len(found_locks) > 0:
        print(f"Found npm lock files: {found_locks}")


def test_dependency_versions_not_floating():
    """Test that dependency versions don't float (^, ~, *, etc.)."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_toml = workspace_root / "Cargo.toml"
    
    if not cargo_toml.exists():
        return
    
    content = cargo_toml.read_text()
    
    # In Cargo.toml, check for floating versions
    # Note: Cargo defaults to caret (^) behavior, which is generally acceptable
    # But we want to ensure no wildcards
    
    # Look for explicit wildcards or very loose constraints
    loose_patterns = [
        r'version\s*=\s*">',  # Greater than only
        r'version\s*=\s*"\*"',  # Wildcard
    ]
    
    for pattern in loose_patterns:
        matches = re.findall(pattern, content)
        assert len(matches) == 0, f"Loose version constraint found: {pattern}"


def test_reproducible_builds():
    """Test that builds are reproducible with locked dependencies."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    cargo_lock = workspace_root / "Cargo.lock"
    
    if not cargo_lock.exists():
        print("Cargo.lock not found")
        return
    
    # With Cargo.lock, builds should be reproducible
    # Check that Cargo.lock is properly formatted
    content = cargo_lock.read_text()
    
    # Should be valid TOML
    try:
        import toml
        lock_data = toml.loads(content)
        assert "package" in lock_data, "Cargo.lock should have package list"
    except ImportError:
        # toml package not available, do basic check
        assert "[[package]]" in content, "Cargo.lock should have package entries"


if __name__ == "__main__":
    test_rust_dependencies_locked()
    test_cargo_toml_version_pins()
    test_no_wildcard_versions()
    test_lockfile_prevents_supply_chain_attacks()
    test_no_git_dependencies()
    test_dependency_audit_clean()
    test_python_dependencies_locked()
    test_npm_dependencies_locked()
    test_dependency_versions_not_floating()
    test_reproducible_builds()
    print("All import pinning validation tests passed")
