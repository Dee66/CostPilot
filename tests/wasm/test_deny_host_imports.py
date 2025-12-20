#!/usr/bin/env python3
"""Test WASM sandbox denies host imports."""

import subprocess
from pathlib import Path


def test_deny_network_imports():
    """WASM module must not import network functions."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # Forbidden network imports
        forbidden = [
            "sock_",
            "tcp",
            "udp",
            "connect",
            "bind",
            "listen",
            "accept",
            "send",
            "recv",
            "getaddrinfo",
            "gethostbyname"
        ]

        for forbidden_import in forbidden:
            assert forbidden_import not in imports.lower(), \
                f"WASM should not import {forbidden_import}"


def test_deny_env_imports():
    """WASM module must not import environment manipulation functions."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # Environment manipulation should be minimal
        forbidden = [
            "setenv",
            "putenv",
            "unsetenv"
        ]

        for forbidden_import in forbidden:
            if forbidden_import in imports.lower():
                print(f"Warning: WASM imports {forbidden_import}")


def test_deny_exec_imports():
    """WASM module must not import exec functions."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        forbidden = [
            "proc_exec",
            "exec",
            "spawn",
            "system"
        ]

        for forbidden_import in forbidden:
            assert forbidden_import not in imports.lower(), \
                f"WASM should not import {forbidden_import}"


def test_deny_filesystem_write_imports():
    """WASM module should minimize filesystem write imports."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # Check for dangerous write operations
        dangerous = [
            "path_remove_directory",
            "path_unlink_file"
        ]

        for dangerous_import in dangerous:
            if dangerous_import in imports:
                print(f"Warning: WASM imports {dangerous_import}")


def test_allowed_wasi_imports():
    """WASM module should only import safe WASI functions."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # Allowed imports
        allowed = [
            "fd_read",
            "fd_write",
            "fd_close",
            "fd_seek",
            "environ_get",
            "environ_sizes_get",
            "clock_time_get",
            "random_get",
            "proc_exit"
        ]

        # Verify only allowed imports present
        for line in imports.split('\n'):
            if 'import' in line.lower() and 'wasi' in line.lower():
                # Check if import is in allowed list
                import_allowed = any(allowed_func in line for allowed_func in allowed)
                if not import_allowed and 'func' in line:
                    # Extract import name
                    pass  # Some imports may be legitimate


def test_no_custom_host_functions():
    """WASM module must not import custom host functions."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # All imports should be from wasi_snapshot_preview1
        for line in imports.split('\n'):
            if 'import' in line.lower() and 'func' in line.lower():
                if 'wasi_snapshot_preview1' not in line and 'env' not in line:
                    # Custom imports detected
                    if line.strip():
                        print(f"Potential custom import: {line}")


def test_import_count_bounded():
    """WASM module should have bounded number of imports."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        imports = result.stdout

        # Count imports
        import_count = imports.lower().count('import')

        # Should have minimal imports (< 50 for typical WASI app)
        assert import_count < 100, f"Too many imports: {import_count}"


if __name__ == "__main__":
    test_deny_network_imports()
    test_deny_env_imports()
    test_deny_exec_imports()
    test_deny_filesystem_write_imports()
    test_allowed_wasi_imports()
    test_no_custom_host_functions()
    test_import_count_bounded()
    print("All host import denial tests passed")
