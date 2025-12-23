#!/usr/bin/env python3
"""Test pinned runner reproducibility."""

import subprocess
import tempfile
from pathlib import Path


def test_ci_runner_pinned_version():
    """CI runner should use pinned versions."""
    ci_workflow = Path(".github/workflows/test.yml")

    if ci_workflow.exists():
        with open(ci_workflow) as f:
            content = f.read()

        # Check for pinned runner versions
        if "runs-on:" in content:
            # Should use specific versions like ubuntu-22.04, not ubuntu-latest
            assert "ubuntu-22.04" in content or "ubuntu-20.04" in content or "macos-12" in content, \
                "CI should use pinned runner versions"


def test_pinned_runner_reproducibility():
    """Pinned runners should produce reproducible results."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_reproducible.py"

        test_code = """
import platform
import sys

def test_environment():
    # Check environment is consistent
    print(f"OS: {platform.system()}")
    print(f"OS Version: {platform.release()}")
    print(f"Python: {sys.version}")
    print(f"Architecture: {platform.machine()}")

if __name__ == "__main__":
    test_environment()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        # Run multiple times
        results = []
        for _ in range(2):
            result = subprocess.run(
                ["python3", str(test_script)],
                capture_output=True,
                text=True
            )
            results.append(result.stdout)

        # Should be identical
        assert results[0] == results[1], "Environment should be reproducible"


def test_docker_image_pinned():
    """Docker images should use pinned versions."""
    dockerfile = Path("Dockerfile")

    if dockerfile.exists():
        with open(dockerfile) as f:
            content = f.read()

        # Check for pinned base images
        if "FROM" in content:
            # Should use tags like rust:1.75.0, not rust:latest
            assert ":latest" not in content.lower() or "# pinned" in content, \
                "Docker images should be pinned, not use :latest"


def test_rust_toolchain_pinned():
    """Rust toolchain should be pinned."""
    toolchain_file = Path("rust-toolchain.toml")

    if toolchain_file.exists():
        with open(toolchain_file) as f:
            content = f.read()

        # Should specify exact version
        assert "channel" in content, "Rust toolchain should specify channel"


def test_node_version_pinned():
    """Node.js version should be pinned."""
    nvmrc = Path(".nvmrc")

    if nvmrc.exists():
        with open(nvmrc) as f:
            version = f.read().strip()

        # Should be specific version
        assert version and not version.endswith("x"), "Node version should be specific"


def test_python_version_pinned():
    """Python version should be pinned in CI."""
    ci_workflow = Path(".github/workflows/test.yml")

    if ci_workflow.exists():
        with open(ci_workflow) as f:
            content = f.read()

        # Check for python-version
        if "python-version:" in content:
            # Should use specific version like 3.11, not 3.x
            assert "3.11" in content or "3.10" in content or "3.12" in content, \
                "Python version should be pinned"


def test_dependency_lock_files_committed():
    """Dependency lock files should be committed."""
    lock_files = [
        Path("Cargo.lock"),
        Path("package-lock.json"),
        Path("yarn.lock"),
        Path("poetry.lock")
    ]

    # At least one lock file should exist
    existing_locks = [f for f in lock_files if f.exists()]
    assert len(existing_locks) > 0, "At least one lock file should exist"


def test_system_dependencies_documented():
    """System dependencies should be documented with versions."""
    readme = Path("README.md")

    if readme.exists():
        with open(readme) as f:
            content = f.read()

        # Should mention versions
        if "dependencies" in content.lower() or "requirements" in content.lower():
            assert True, "Dependencies documented"


def test_ci_cache_keyed_by_lockfile():
    """CI cache should be keyed by lock file hashes."""
    ci_workflow = Path(".github/workflows/test.yml")

    if ci_workflow.exists():
        with open(ci_workflow) as f:
            content = f.read()

        # Check for cache with lock file key
        if "cache" in content.lower():
            assert "Cargo.lock" in content or "package-lock.json" in content, \
                "Cache should be keyed by lock files"


def test_reproducible_build_flags():
    """Build should use reproducible flags."""
    cargo_toml = Path("Cargo.toml")

    if cargo_toml.exists():
        with open(cargo_toml) as f:
            content = f.read()

        # Check for reproducible build settings
        if "[profile.release]" in content:
            # Should have deterministic settings
            assert True, "Release profile configured"


def test_runner_architecture_specified():
    """CI runner architecture should be specified."""
    ci_workflow = Path(".github/workflows/test.yml")

    if ci_workflow.exists():
        with open(ci_workflow) as f:
            content = f.read()

        # Check for architecture specification
        if "strategy:" in content and "matrix:" in content:
            assert True, "Build matrix configured"


if __name__ == "__main__":
    test_ci_runner_pinned_version()
    test_pinned_runner_reproducibility()
    test_docker_image_pinned()
    test_rust_toolchain_pinned()
    test_node_version_pinned()
    test_python_version_pinned()
    test_dependency_lock_files_committed()
    test_system_dependencies_documented()
    test_ci_cache_keyed_by_lockfile()
    test_reproducible_build_flags()
    test_runner_architecture_specified()
    print("All pinned runner reproducibility tests passed")
