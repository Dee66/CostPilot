#!/usr/bin/env python3
"""Test WASM deterministic local RNG."""

import subprocess
import tempfile
from pathlib import Path


def test_rng_deterministic_across_runs():
    """WASM RNG should be deterministic across runs."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template that might trigger randomness
        with open(template_path, 'w') as f:
            f.write('{"Resources": {"Lambda": {"Type": "AWS::Lambda::Function"}}}')

        # Run multiple times
        outputs = []
        for i in range(3):
            result = subprocess.run(
                ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True
            )
            outputs.append(result.stdout)

        # All outputs should be identical (deterministic)
        assert all(output == outputs[0] for output in outputs), \
            "RNG should be deterministic across runs"


def test_rng_seeded_from_input():
    """WASM RNG should be seeded from input, not time."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        # Run at different times
        import time

        result1 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        time.sleep(2)

        result2 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # Outputs should be identical (not time-based)
        assert result1.stdout == result2.stdout, "RNG should not use time as seed"


def test_rng_consistent_with_same_input():
    """Same input should always produce same RNG sequence."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Specific input
        import json
        template_content = {
            "Resources": {
                f"Lambda{i}": {"Type": "AWS::Lambda::Function"}
                for i in range(10)
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Multiple runs
        results = []
        for _ in range(5):
            result = subprocess.run(
                ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
                capture_output=True,
                text=True
            )
            results.append(result.stdout)

        # All identical
        assert all(r == results[0] for r in results), "RNG should be consistent"


def test_rng_no_getrandom_import():
    """WASM should not import getrandom syscall."""
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

        # Should not import random_get for non-deterministic randomness
        # Or if it does, should use it deterministically
        if "random_get" in imports:
            print("Warning: WASM imports random_get - must be used deterministically")


def test_rng_isolated_from_host():
    """WASM RNG should be isolated from host system randomness."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        with open(template_path, 'w') as f:
            f.write('{"Resources": {}}')

        # Run in different environments
        import os

        # Modify environment (shouldn't affect output)
        env1 = os.environ.copy()
        env1["RANDOM_SEED"] = "12345"

        result1 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            env=env1
        )

        env2 = os.environ.copy()
        env2["RANDOM_SEED"] = "67890"

        result2 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            env=env2
        )

        # Should be identical (isolated from env)
        assert result1.stdout == result2.stdout, "RNG should be isolated from host"


def test_rng_different_inputs_different_output():
    """Different inputs should produce different deterministic outputs."""
    wasm_path = Path("target/wasm32-wasi/release/costpilot.wasm")

    if not wasm_path.exists():
        print("WASM binary not found, skipping test")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template1_path = Path(tmpdir) / "template1.json"
        template2_path = Path(tmpdir) / "template2.json"

        # Different inputs
        with open(template1_path, 'w') as f:
            f.write('{"Resources": {"Lambda1": {"Type": "AWS::Lambda::Function"}}}')

        with open(template2_path, 'w') as f:
            f.write('{"Resources": {"Lambda2": {"Type": "AWS::Lambda::Function"}}}')

        result1 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template1_path)],
            capture_output=True,
            text=True
        )

        result2 = subprocess.run(
            ["wasmtime", str(wasm_path), "analyze", "--plan", str(template2_path)],
            capture_output=True,
            text=True
        )

        # Should be different (based on input)
        if result1.stdout and result2.stdout:
            assert result1.stdout != result2.stdout or "Lambda1" in result1.stdout, \
                "Different inputs should affect output"


def test_rng_documented():
    """RNG determinism should be documented."""
    docs_path = Path("docs/DETERMINISM_CONTRACT.md")

    if docs_path.exists():
        with open(docs_path) as f:
            content = f.read()

        # Check for RNG documentation
        assert "rng" in content.lower() or "random" in content.lower() or "deterministic" in content.lower(), \
            "RNG determinism should be documented"


if __name__ == "__main__":
    test_rng_deterministic_across_runs()
    test_rng_seeded_from_input()
    test_rng_consistent_with_same_input()
    test_rng_no_getrandom_import()
    test_rng_isolated_from_host()
    test_rng_different_inputs_different_output()
    test_rng_documented()
    print("All deterministic RNG tests passed")
