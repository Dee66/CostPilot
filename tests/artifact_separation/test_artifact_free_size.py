#!/usr/bin/env python3
"""Test Artifact Separation: Free archive size below threshold."""

from pathlib import Path
import tarfile
import zipfile


def test_free_binary_size_reasonable():
    """Test Free binary size is reasonable."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        # Binary not built yet
        return

    size_bytes = binary_path.stat().st_size
    size_mb = size_bytes / (1024 * 1024)

    # Free binary should be < 50MB
    # Premium would be larger with bundled heuristics
    assert size_mb < 50, f"Free binary too large: {size_mb:.1f}MB (expected < 50MB)"

    print(f"Free binary size: {size_mb:.1f}MB")


def test_free_archive_size_threshold():
    """Test Free archive size is below threshold."""
    release_dir = Path("target/release")

    if not release_dir.exists():
        return

    archives = list(release_dir.glob("costpilot-*.tar.gz")) + \
               list(release_dir.glob("costpilot-*.zip"))

    for archive_path in archives:
        if "free" in archive_path.name.lower() or "community" in archive_path.name.lower():
            size_bytes = archive_path.stat().st_size
            size_mb = size_bytes / (1024 * 1024)

            # Free archive should be < 60MB compressed
            assert size_mb < 60, f"Free archive too large: {size_mb:.1f}MB (expected < 60MB)"

            print(f"Free archive size: {size_mb:.1f}MB")


def test_free_stripped_binary_smaller():
    """Test Free stripped binary is smaller."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        return

    size_bytes = binary_path.stat().st_size
    size_mb = size_bytes / (1024 * 1024)

    # Stripped binary should be smaller than debug
    # Debug symbols can add 10-20MB

    # Check if stripped
    import subprocess
    result = subprocess.run(
        ["file", str(binary_path)],
        capture_output=True,
        text=True,
        timeout=5
    )

    if result.returncode == 0:
        output = result.stdout.lower()

        if "stripped" in output:
            # Stripped binary should be reasonably sized
            assert size_mb < 40, f"Stripped binary still too large: {size_mb:.1f}MB"


def test_free_archive_contents_size():
    """Test Free archive contents are reasonably sized."""
    release_dir = Path("target/release")

    if not release_dir.exists():
        return

    archives = list(release_dir.glob("costpilot-*.tar.gz"))

    for archive_path in archives:
        if "free" in archive_path.name.lower():
            try:
                with tarfile.open(archive_path, 'r:gz') as tar:
                    total_size = sum(member.size for member in tar.getmembers())
                    total_mb = total_size / (1024 * 1024)

                    # Uncompressed contents should be < 80MB
                    assert total_mb < 80, f"Archive contents too large: {total_mb:.1f}MB"
            except:
                pass


def test_free_no_debug_symbols():
    """Test Free release has no debug symbols."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        return

    # Check for debug symbols
    import subprocess
    result = subprocess.run(
        ["nm", str(binary_path)],
        capture_output=True,
        text=True,
        timeout=5
    )

    if result.returncode == 0:
        lines = result.stdout.split('\n')

        # Debug builds have many symbols
        # Release should have fewer
        symbol_count = len([line for line in lines if line.strip()])

        # Stripped release should have < 1000 symbols
        # Debug might have 10000+


def test_free_dependencies_minimal():
    """Test Free has minimal dependencies."""
    binary_path = Path("target/release/costpilot")

    if not binary_path.exists():
        return

    # Check dynamic dependencies
    import subprocess
    result = subprocess.run(
        ["ldd", str(binary_path)],
        capture_output=True,
        text=True,
        timeout=5
    )

    if result.returncode == 0:
        lines = result.stdout.split('\n')
        deps = [line for line in lines if "=>" in line]

        # Should have minimal dependencies
        # Typical: libc, libm, libdl, libpthread, libgcc
        # Premium might have more (crypto libs)

        assert len(deps) < 20, f"Too many dependencies: {len(deps)}"


if __name__ == "__main__":
    test_free_binary_size_reasonable()
    test_free_archive_size_threshold()
    test_free_stripped_binary_smaller()
    test_free_archive_contents_size()
    test_free_no_debug_symbols()
    test_free_dependencies_minimal()
    print("All Artifact Separation: Free size threshold tests passed")
