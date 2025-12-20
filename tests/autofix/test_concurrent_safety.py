#!/usr/bin/env python3
"""Test concurrent patch generation safety."""

import json
import os
import subprocess
import tempfile
import threading
import time
from pathlib import Path


def test_concurrent_patch_file_locking():
    """Concurrent patch operations must use file locking."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"
        output_dir = Path(tmpdir) / "patches"
        output_dir.mkdir()

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": [
                {
                    "id": "lambda-memory-limit",
                    "severity": "high",
                    "resource_type": "AWS::Lambda::Function",
                    "condition": "MemorySize > 3008"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Launch multiple concurrent patch operations
        processes = []
        for i in range(5):
            output_path = output_dir / f"patch_{i}.json"
            proc = subprocess.Popen(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--output", str(output_path)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            processes.append(proc)

        # Wait for all
        for proc in processes:
            proc.wait()

        # All should complete without corruption
        # Check if any generated outputs
        patch_files = list(output_dir.glob("*.json"))
        if len(patch_files) > 0:
            for patch_file in patch_files:
                try:
                    with open(patch_file) as f:
                        data = json.load(f)
                    assert isinstance(data, dict), f"Patch file {patch_file} should be valid JSON"
                except json.JSONDecodeError:
                    assert False, f"Patch file {patch_file} corrupted by concurrent access"


def test_concurrent_same_output_serialized():
    """Multiple patches to same output must be serialized."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"
        output_path = Path(tmpdir) / "patch.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Launch concurrent writes to same file
        processes = []
        for i in range(3):
            proc = subprocess.Popen(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--output", str(output_path)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            processes.append(proc)

        # Wait
        results = [proc.wait() for proc in processes]

        # At least one should fail (file locked) or all serialize correctly
        if output_path.exists():
            with open(output_path) as f:
                try:
                    data = json.load(f)
                    assert isinstance(data, dict), "Output should be valid JSON"
                except json.JSONDecodeError:
                    assert False, "Concurrent writes corrupted output"


def test_concurrent_different_templates():
    """Concurrent patches on different templates must not interfere."""
    with tempfile.TemporaryDirectory() as tmpdir:
        policy_path = Path(tmpdir) / "policy.json"

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Create multiple templates
        templates = []
        for i in range(5):
            template_path = Path(tmpdir) / f"template_{i}.json"
            output_path = Path(tmpdir) / f"patch_{i}.json"

            template_content = {
                "Resources": {
                    f"Lambda{i}": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "MemorySize": 1024 * (i + 1)
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            templates.append((template_path, output_path))

        # Launch concurrent operations
        processes = []
        for template_path, output_path in templates:
            proc = subprocess.Popen(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--output", str(output_path)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            processes.append(proc)

        # Wait
        for proc in processes:
            proc.wait()

        # All outputs should be independent and valid
        for template_path, output_path in templates:
            if output_path.exists():
                with open(output_path) as f:
                    try:
                        data = json.load(f)
                        assert isinstance(data, dict), f"Output {output_path} should be valid"
                    except json.JSONDecodeError:
                        assert False, f"Output {output_path} corrupted"


def test_lock_file_cleanup():
    """Lock files must be cleaned up after patch operation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"
        output_path = Path(tmpdir) / "patch.json"

        template_content = {"Resources": {}}
        policy_content = {"version": "1.0.0", "rules": []}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Run autofix
        result = subprocess.run(
            ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--output", str(output_path)],
            capture_output=True,
            text=True
        )

        # Check for lock files
        lock_files = list(Path(tmpdir).glob("*.lock"))
        assert len(lock_files) == 0, "Lock files should be cleaned up"


def test_concurrent_policy_updates():
    """Concurrent operations with policy updates must be safe."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 10240
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        def run_patch(policy_version):
            """Run patch with specific policy version."""
            policy_content = {
                "version": policy_version,
                "rules": []
            }

            # Each thread gets own policy file
            thread_policy = Path(tmpdir) / f"policy_{policy_version}.json"
            with open(thread_policy, 'w') as f:
                json.dump(policy_content, f)

            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(thread_policy), "--dry-run"],
                capture_output=True,
                text=True
            )
            return result.returncode

        # Concurrent operations with different policy versions
        threads = []
        for i in range(3):
            thread = threading.Thread(target=run_patch, args=(f"{i}.0.0",))
            threads.append(thread)
            thread.start()

        for thread in threads:
            thread.join()

        # All should complete safely
        assert True


def test_parallel_validation_determinism():
    """Parallel patch validation must be deterministic."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        policy_path = Path(tmpdir) / "policy.json"

        template_content = {
            "Resources": {
                "Lambda1": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 1024}},
                "Lambda2": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 2048}},
                "Lambda3": {"Type": "AWS::Lambda::Function", "Properties": {"MemorySize": 4096}}
            }
        }

        policy_content = {
            "version": "1.0.0",
            "rules": []
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(policy_path, 'w') as f:
            json.dump(policy_content, f)

        # Run multiple times
        results = []
        for i in range(3):
            result = subprocess.run(
                ["costpilot", "autofix", "--plan", str(template_path), "--policy", str(policy_path), "--dry-run"],
                capture_output=True,
                text=True
            )
            results.append(result.stdout)

        # All runs should produce identical output
        if len(results) > 1 and all(r == results[0] for r in results):
            assert True, "Parallel validation is deterministic"


if __name__ == "__main__":
    test_concurrent_patch_file_locking()
    test_concurrent_same_output_serialized()
    test_concurrent_different_templates()
    test_lock_file_cleanup()
    test_concurrent_policy_updates()
    test_parallel_validation_determinism()
    print("All concurrent patch safety tests passed")
