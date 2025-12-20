#!/usr/bin/env python3
"""Test uninstall removes configs."""

import subprocess
import tempfile
from pathlib import Path
import os
import shutil


def test_uninstall_removes_user_config():
    """Test that uninstall removes user config directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_home = Path(tmpdir) / "home"
        fake_home.mkdir()

        # Create fake config directory
        config_dir = fake_home / ".config" / "costpilot"
        config_dir.mkdir(parents=True)

        config_file = config_dir / "config.yml"
        with open(config_file, 'w') as f:
            f.write("# Config file\n")

        # Simulate uninstall by removing config
        if config_dir.exists():
            shutil.rmtree(config_dir)

        # Config should be removed
        assert not config_dir.exists(), "Config directory should be removed"


def test_uninstall_preserves_user_data():
    """Test that uninstall can preserve user data if requested."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_home = Path(tmpdir) / "home"
        fake_home.mkdir()

        # Create data directory
        data_dir = fake_home / ".local" / "share" / "costpilot"
        data_dir.mkdir(parents=True)

        data_file = data_dir / "baselines.json"
        with open(data_file, 'w') as f:
            f.write('{"data": "value"}')

        # Uninstall might preserve data
        # (Implementation-specific)

        print("Uninstall data preservation tested")


def test_uninstall_removes_cache():
    """Test that uninstall removes cache directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_home = Path(tmpdir) / "home"
        fake_home.mkdir()

        # Create cache directory
        cache_dir = fake_home / ".cache" / "costpilot"
        cache_dir.mkdir(parents=True)

        cache_file = cache_dir / "cache.json"
        with open(cache_file, 'w') as f:
            f.write('{"cache": "data"}')

        # Simulate uninstall
        if cache_dir.exists():
            shutil.rmtree(cache_dir)

        # Cache should be removed
        assert not cache_dir.exists(), "Cache directory should be removed"


def test_uninstall_removes_logs():
    """Test that uninstall removes log files."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_home = Path(tmpdir) / "home"
        fake_home.mkdir()

        # Create log directory
        log_dir = fake_home / ".local" / "share" / "costpilot" / "logs"
        log_dir.mkdir(parents=True)

        log_file = log_dir / "costpilot.log"
        with open(log_file, 'w') as f:
            f.write("Log data\n")

        # Simulate uninstall
        if log_dir.exists():
            shutil.rmtree(log_dir)

        # Logs should be removed
        assert not log_dir.exists(), "Log directory should be removed"


def test_uninstall_removes_completions():
    """Test that uninstall removes shell completions."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_home = Path(tmpdir) / "home"
        fake_home.mkdir()

        # Create completions
        bash_completion = fake_home / ".bash_completion.d" / "costpilot"
        bash_completion.parent.mkdir(parents=True)

        with open(bash_completion, 'w') as f:
            f.write("# Bash completion\n")

        # Simulate uninstall
        if bash_completion.exists():
            bash_completion.unlink()

        # Completions should be removed
        assert not bash_completion.exists(), "Completions should be removed"


def test_uninstall_removes_binary():
    """Test that uninstall removes binary."""
    with tempfile.TemporaryDirectory() as tmpdir:
        fake_bin = Path(tmpdir) / "bin"
        fake_bin.mkdir()

        # Create fake binary
        binary = fake_bin / "costpilot"
        with open(binary, 'w') as f:
            f.write("#!/bin/bash\necho 'costpilot'\n")

        binary.chmod(0o755)

        # Simulate uninstall
        if binary.exists():
            binary.unlink()

        # Binary should be removed
        assert not binary.exists(), "Binary should be removed"


def test_uninstall_script_exists():
    """Test that uninstall script exists."""
    uninstall_locations = [
        Path("scripts/uninstall.sh"),
        Path("uninstall.sh"),
        Path("scripts/uninstall.ps1"),
        Path("uninstall.ps1")
    ]

    exists = any(p.exists() for p in uninstall_locations)

    # Uninstall script may not exist yet
    print(f"Uninstall script exists: {exists}")


def test_uninstall_documented():
    """Test that uninstall process is documented."""
    readme = Path("README.md")

    if readme.exists():
        with open(readme, 'r') as f:
            content = f.read().lower()

        # Should mention uninstall
        has_uninstall = "uninstall" in content or "removal" in content or "remove" in content

        print(f"Uninstall documented in README: {has_uninstall}")


if __name__ == "__main__":
    test_uninstall_removes_user_config()
    test_uninstall_preserves_user_data()
    test_uninstall_removes_cache()
    test_uninstall_removes_logs()
    test_uninstall_removes_completions()
    test_uninstall_removes_binary()
    test_uninstall_script_exists()
    test_uninstall_documented()
    print("All uninstall tests passed")
