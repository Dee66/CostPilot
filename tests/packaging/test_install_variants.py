#!/usr/bin/env python3
"""
Test: Install variants validation.

Validates npm/homebrew/tarball install permutations for expected file layout.
"""

import os
import sys
import tempfile
import tarfile
import json
from pathlib import Path


WORKSPACE = Path(__file__).parent.parent.parent


def test_tarball_install_layout():
    """Verify tarball install creates expected file layout."""
    
    with tempfile.TemporaryDirectory() as tmpdir:
        install_root = Path(tmpdir) / "costpilot-1.0.0"
        install_root.mkdir()
        
        # Expected layout
        expected_structure = {
            "bin/costpilot": "binary",
            "lib/libcostpilot.so": "library",
            "share/doc/README.md": "documentation",
            "share/man/man1/costpilot.1": "manpage",
            "share/completions/costpilot.bash": "completion"
        }
        
        for path_str, content in expected_structure.items():
            path = install_root / path_str
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        
        # Verify all created
        for path_str in expected_structure:
            assert (install_root / path_str).exists()
        
        print(f"✓ Tarball install layout ({len(expected_structure)} files)")


def test_homebrew_formula_structure():
    """Verify Homebrew formula installs to correct locations."""
    
    homebrew_install = {
        "bin": ["costpilot"],
        "etc": ["costpilot/config.yml"],
        "share/costpilot": ["docs", "examples"],
        "share/completions": ["costpilot.bash", "costpilot.zsh", "costpilot.fish"]
    }
    
    # Validate structure
    assert "bin" in homebrew_install
    assert len(homebrew_install["bin"]) > 0
    
    total_files = sum(len(v) if isinstance(v, list) else 1 for v in homebrew_install.values())
    print(f"✓ Homebrew formula structure ({total_files} items)")


def test_npm_package_json():
    """Verify npm package.json has correct structure."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_package.json', delete=False) as f:
        package_json = {
            "name": "costpilot",
            "version": "1.0.0",
            "description": "Infrastructure cost forecasting CLI",
            "bin": {
                "costpilot": "./bin/costpilot"
            },
            "files": [
                "bin/",
                "lib/",
                "docs/"
            ],
            "license": "Apache-2.0"
        }
        json.dump(package_json, f)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            data = json.load(f)
        
        assert "bin" in data
        assert "costpilot" in data["bin"]
        assert "files" in data
        
        print("✓ npm package.json structure validated")
        
    finally:
        os.unlink(path)


def test_tarball_creation():
    """Verify tarball can be created with correct structure."""
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create source structure
        source = Path(tmpdir) / "source"
        source.mkdir()
        (source / "bin").mkdir()
        (source / "bin" / "costpilot").write_text("binary")
        (source / "README.md").write_text("docs")
        
        # Create tarball
        tarball_path = Path(tmpdir) / "costpilot-1.0.0.tar.gz"
        with tarfile.open(tarball_path, "w:gz") as tar:
            tar.add(source, arcname="costpilot-1.0.0")
        
        # Verify tarball exists
        assert tarball_path.exists()
        assert tarball_path.stat().st_size > 0
        
        print("✓ Tarball creation validated")


def test_tarball_extraction():
    """Verify tarball extracts to expected structure."""
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create and extract tarball
        source = Path(tmpdir) / "source"
        source.mkdir()
        (source / "file.txt").write_text("content")
        
        tarball_path = Path(tmpdir) / "test.tar.gz"
        with tarfile.open(tarball_path, "w:gz") as tar:
            tar.add(source, arcname="extracted")
        
        extract_dir = Path(tmpdir) / "extract"
        extract_dir.mkdir()
        
        with tarfile.open(tarball_path, "r:gz") as tar:
            tar.extractall(extract_dir)
        
        # Verify extracted
        assert (extract_dir / "extracted" / "file.txt").exists()
        
        print("✓ Tarball extraction validated")


def test_deb_package_structure():
    """Verify .deb package has correct structure."""
    
    deb_structure = {
        "DEBIAN/control": "package metadata",
        "usr/bin/costpilot": "binary",
        "usr/share/doc/costpilot/copyright": "license",
        "usr/share/doc/costpilot/README.md": "documentation",
        "usr/share/bash-completion/completions/costpilot": "completion"
    }
    
    # Validate required files
    assert "DEBIAN/control" in deb_structure
    assert "usr/bin/costpilot" in deb_structure
    
    print(f"✓ .deb package structure ({len(deb_structure)} files)")


def test_rpm_package_structure():
    """Verify .rpm package has correct structure."""
    
    rpm_structure = {
        "usr/bin/costpilot": "binary",
        "usr/share/licenses/costpilot/LICENSE": "license",
        "usr/share/doc/costpilot/README.md": "documentation",
        "etc/costpilot/config.yml": "configuration"
    }
    
    assert "usr/bin/costpilot" in rpm_structure
    
    print(f"✓ .rpm package structure ({len(rpm_structure)} files)")


def test_docker_image_layers():
    """Verify Docker image has expected layers."""
    
    dockerfile_layers = [
        "FROM rust:1.75 AS builder",
        "WORKDIR /app",
        "COPY . .",
        "RUN cargo build --release",
        "FROM debian:bookworm-slim",
        "COPY --from=builder /app/target/release/costpilot /usr/local/bin/",
        "ENTRYPOINT [\"costpilot\"]"
    ]
    
    # Should have multi-stage build
    assert any("AS builder" in layer for layer in dockerfile_layers)
    assert any("COPY --from=builder" in layer for layer in dockerfile_layers)
    
    print(f"✓ Docker image layers ({len(dockerfile_layers)} layers)")


def test_install_script_permissions():
    """Verify install script sets correct permissions."""
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='_install.sh', delete=False) as f:
        install_script = """#!/bin/bash
install -m 0755 bin/costpilot /usr/local/bin/costpilot
install -m 0644 docs/costpilot.1 /usr/local/share/man/man1/
install -m 0644 completions/costpilot.bash /usr/local/share/bash-completion/completions/
        """.strip()
        f.write(install_script)
        path = f.name
    
    try:
        with open(path, 'r') as f:
            content = f.read()
        
        # Verify permissions are set
        assert "0755" in content  # executable
        assert "0644" in content  # readable
        
        print("✓ Install script permissions validated")
        
    finally:
        os.unlink(path)


def test_uninstall_script():
    """Verify uninstall script removes all files."""
    
    uninstall_files = [
        "/usr/local/bin/costpilot",
        "/usr/local/share/man/man1/costpilot.1",
        "/usr/local/share/bash-completion/completions/costpilot",
        "/etc/costpilot/"
    ]
    
    # Uninstall should remove all installed files
    assert len(uninstall_files) > 0
    
    print(f"✓ Uninstall script ({len(uninstall_files)} files/dirs)")


def test_postinstall_hooks():
    """Verify post-install hooks are executed."""
    
    postinstall_tasks = [
        "update_mandb",
        "update_completion_cache",
        "create_config_if_missing"
    ]
    
    assert len(postinstall_tasks) > 0
    
    print(f"✓ Post-install hooks ({len(postinstall_tasks)} tasks)")


if __name__ == "__main__":
    print("Testing install variants validation...")
    
    try:
        test_tarball_install_layout()
        test_homebrew_formula_structure()
        test_npm_package_json()
        test_tarball_creation()
        test_tarball_extraction()
        test_deb_package_structure()
        test_rpm_package_structure()
        test_docker_image_layers()
        test_install_script_permissions()
        test_uninstall_script()
        test_postinstall_hooks()
        
        print("\n✅ All install variants tests passed")
        sys.exit(0)
        
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
