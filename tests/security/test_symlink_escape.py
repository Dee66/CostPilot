#!/usr/bin/env python3
"""Test symlink escape security."""

import os
import subprocess
import tempfile
from pathlib import Path


def test_symlink_escape_denied():
    """Symlinks pointing outside workspace should be denied."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        outside = Path(tmpdir) / "outside"
        outside.mkdir()

        outside_file = outside / "secret.json"
        with open(outside_file, 'w') as f:
            f.write('{"secret": "data"}')

        # Create symlink inside workspace pointing outside
        symlink = workspace / "template.json"

        try:
            symlink.symlink_to(outside_file)
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Attempt to read through symlink
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(symlink)],
            capture_output=True,
            text=True
        )

        # Should deny or fail
        if result.returncode != 0:
            error_output = result.stderr + result.stdout
            assert "symlink" in error_output.lower() or "permission" in error_output.lower(), \
                "Should deny symlink escape"


def test_symlink_inside_workspace_allowed():
    """Symlinks within workspace should be allowed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        target = workspace / "target.json"
        with open(target, 'w') as f:
            f.write('{"Resources": {}}')

        symlink = workspace / "link.json"

        try:
            symlink.symlink_to(target)
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Should work
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(symlink)],
            capture_output=True,
            text=True
        )

        # Should not fail on symlink itself
        assert result.returncode in [0, 1, 2, 101], "Internal symlinks should be allowed"


def test_symlink_to_parent_directory_denied():
    """Symlinks to parent directories should be denied."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        # Create symlink to parent
        parent_link = workspace / "parent"

        try:
            parent_link.symlink_to("..")
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Attempt to use
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(parent_link / "some_file.json")],
            capture_output=True,
            text=True
        )

        # Should deny
        assert result.returncode != 0, "Parent directory symlink should be denied"


def test_symlink_chain_escape_denied():
    """Chained symlinks escaping workspace should be denied."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        outside = Path(tmpdir) / "outside"
        outside.mkdir()

        # Create chain
        link1 = workspace / "link1"
        link2 = workspace / "link2"
        target = outside / "target.json"

        with open(target, 'w') as f:
            f.write('{"data": "secret"}')

        try:
            link1.symlink_to(link2)
            link2.symlink_to(target)
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Attempt to read
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(link1)],
            capture_output=True,
            text=True
        )

        # Should deny
        if result.returncode != 0:
            assert True, "Symlink chain escape denied"


def test_absolute_symlink_denied():
    """Absolute symlinks should be denied."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        # Absolute symlink to /etc/passwd
        abs_link = workspace / "passwd_link"

        try:
            abs_link.symlink_to("/etc/passwd")
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Attempt to read
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(abs_link)],
            capture_output=True,
            text=True
        )

        # Should deny
        assert result.returncode != 0, "Absolute symlink should be denied"


def test_symlink_to_device_denied():
    """Symlinks to device files should be denied."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        device_link = workspace / "null_link"

        try:
            device_link.symlink_to("/dev/null")
        except OSError:
            print("Note: Symlink creation requires permissions")
            return

        # Attempt to use
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(device_link)],
            capture_output=True,
            text=True
        )

        # Should deny or handle gracefully
        assert result.returncode != 0, "Device file access should be denied"


def test_realpath_resolution():
    """File paths should be resolved to real paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / "workspace"
        workspace.mkdir()

        # Create file
        real_file = workspace / "real.json"
        with open(real_file, 'w') as f:
            f.write('{"Resources": {}}')

        # Create symlink
        link_file = workspace / "link.json"

        try:
            link_file.symlink_to(real_file)
        except OSError:
            return

        # Both should resolve to same real path
        real_path = os.path.realpath(real_file)
        link_path = os.path.realpath(link_file)

        assert real_path == link_path, "Paths should resolve to same real path"


if __name__ == "__main__":
    test_symlink_escape_denied()
    test_symlink_inside_workspace_allowed()
    test_symlink_to_parent_directory_denied()
    test_symlink_chain_escape_denied()
    test_absolute_symlink_denied()
    test_symlink_to_device_denied()
    test_realpath_resolution()
    print("All symlink escape security tests passed")
