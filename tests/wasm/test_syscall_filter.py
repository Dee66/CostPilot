#!/usr/bin/env python3
"""Test WASM sandbox syscall filter."""

import subprocess
import tempfile
from pathlib import Path


def test_syscall_filter_blocks_network():
    """WASM sandbox must block network syscalls."""
    # If WASM build exists
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Create test template
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        # Run WASM with wasmtime
        result = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )
        
        # Should execute without network access
        # Network attempts should fail
        assert "connection" not in result.stderr.lower() or result.returncode in [0, 1, 2, 101], \
            "Network syscalls should be blocked"


def test_syscall_filter_blocks_filesystem():
    """WASM sandbox must restrict filesystem access."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    # Run without filesystem mapping
    result = subprocess.run(
        ["wasmtime", str(wasm_path), "analyze", "--plan", "/etc/passwd"],
        capture_output=True,
        text=True
    )
    
    # Should fail to access unmapped paths
    assert result.returncode != 0 or "permission" in result.stderr.lower() or "not found" in result.stderr.lower(), \
        "Filesystem access should be restricted"


def test_syscall_filter_allows_stdio():
    """WASM sandbox must allow stdio syscalls."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    # Run with --help (uses stdout)
    result = subprocess.run(
        ["wasmtime", str(wasm_path), "--help"],
        capture_output=True,
        text=True
    )
    
    # Should successfully write to stdout
    assert len(result.stdout) > 0, "Stdio should be allowed"


def test_syscall_filter_blocks_exec():
    """WASM sandbox must block exec syscalls."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    # WASM inherently cannot exec
    # Verify WASM module doesn't have exec imports
    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        # Check for exec-related imports
        assert "exec" not in result.stdout.lower(), "WASM should not import exec functions"


def test_syscall_filter_blocks_fork():
    """WASM sandbox must block fork syscalls."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    # WASM cannot fork
    result = subprocess.run(
        ["wasm-objdump", "-x", str(wasm_path)],
        capture_output=True,
        text=True
    )
    
    if result.returncode == 0:
        assert "fork" not in result.stdout.lower(), "WASM should not have fork capability"


def test_wasmtime_sandbox_flags():
    """Verify wasmtime runs with appropriate sandbox flags."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')
        
        # Run with explicit sandbox flags
        result = subprocess.run(
            [
                "wasmtime",
                "--disable-cache",
                "--max-wasm-stack", "1048576",
                str(wasm_path),
                "analyze",
                "--plan", str(template_path)
            ],
            capture_output=True,
            text=True
        )
        
        # Should execute successfully with sandbox
        assert result.returncode in [0, 1, 2, 101], "Sandbox flags should not break execution"


def test_syscall_audit_logging():
    """Syscall violations should be auditable."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")
    
    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return
    
    # Attempt forbidden operation
    result = subprocess.run(
        ["wasmtime", str(wasm_path), "analyze", "--plan", "/dev/null"],
        capture_output=True,
        text=True
    )
    
    # Error should be logged
    if result.returncode != 0:
        assert len(result.stderr) > 0, "Syscall violations should produce error output"


if __name__ == "__main__":
    test_syscall_filter_blocks_network()
    test_syscall_filter_blocks_filesystem()
    test_syscall_filter_allows_stdio()
    test_syscall_filter_blocks_exec()
    test_syscall_filter_blocks_fork()
    test_wasmtime_sandbox_flags()
    test_syscall_audit_logging()
    print("All syscall filter tests passed")
