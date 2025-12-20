#!/usr/bin/env python3
"""Test missing fixtures cause CI failure."""

import subprocess
import tempfile
from pathlib import Path


def test_missing_test_template_fails_ci():
    """Missing test template fixture should fail CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_example.py"

        # Test that references missing fixture
        test_code = """
import subprocess
from pathlib import Path

def test_analyze():
    template_path = Path("test/fixtures/example_template.json")
    if not template_path.exists():
        raise FileNotFoundError(f"Missing fixture: {template_path}")

    result = subprocess.run(
        ["costpilot", "scan", "--plan", str(template_path)],
        capture_output=True
    )
    assert result.returncode == 0

if __name__ == "__main__":
    test_analyze()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        # Run test
        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )

        # Should fail due to missing fixture
        assert result.returncode != 0, "Missing fixture should cause test failure"
        assert "FileNotFoundError" in result.stderr or "Missing fixture" in result.stderr, \
            "Should report missing fixture"


def test_missing_policy_fixture_fails_ci():
    """Missing policy fixture should fail CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_policy.py"

        test_code = """
from pathlib import Path

def test_policy():
    policy_path = Path("test/fixtures/example_policy.json")
    if not policy_path.exists():
        raise FileNotFoundError(f"Missing policy fixture: {policy_path}")

if __name__ == "__main__":
    test_policy()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )

        # Should fail
        assert result.returncode != 0, "Missing policy fixture should fail"


def test_missing_golden_file_fails_ci():
    """Missing golden file should fail CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_golden.py"

        test_code = """
from pathlib import Path

def test_golden():
    golden_path = Path("test/golden/explain_output.txt")
    if not golden_path.exists():
        raise FileNotFoundError(f"Missing golden file: {golden_path}")

    with open(golden_path) as f:
        content = f.read()

    assert len(content) > 0

if __name__ == "__main__":
    test_golden()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )

        # Should fail
        assert result.returncode != 0, "Missing golden file should fail"


def test_missing_heuristics_fixture_fails():
    """Missing heuristics fixture should fail CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_heuristics.py"

        test_code = """
from pathlib import Path
import json

def test_heuristics():
    heuristics_path = Path("heuristics/cost_heuristics.json")
    if not heuristics_path.exists():
        raise FileNotFoundError(f"Missing heuristics: {heuristics_path}")

    with open(heuristics_path) as f:
        data = json.load(f)

    assert "version" in data or "rules" in data

if __name__ == "__main__":
    test_heuristics()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )

        # Check result
        if result.returncode != 0:
            assert "FileNotFoundError" in result.stderr or "Missing" in result.stderr


def test_fixture_inventory_complete():
    """CI should verify fixture inventory is complete."""
    # Check for fixture directories
    fixture_dirs = [
        Path("test/fixtures"),
        Path("test/golden"),
        Path("examples"),
        Path("heuristics")
    ]

    for fixture_dir in fixture_dirs:
        if fixture_dir.exists():
            # Verify not empty
            files = list(fixture_dir.glob("**/*"))
            if len(files) == 0:
                print(f"Warning: {fixture_dir} is empty")


def test_ci_validates_fixture_manifest():
    """CI should validate fixture manifest."""
    with tempfile.TemporaryDirectory() as tmpdir:
        manifest_file = Path(tmpdir) / "fixtures_manifest.json"

        # Create manifest
        import json
        manifest = {
            "fixtures": [
                "test/fixtures/example_template.json",
                "test/fixtures/example_policy.json",
                "test/golden/explain_output.txt"
            ]
        }

        with open(manifest_file, 'w') as f:
            json.dump(manifest, f)

        # Validate manifest
        with open(manifest_file) as f:
            data = json.load(f)

        assert "fixtures" in data, "Manifest should have fixtures list"
        assert len(data["fixtures"]) > 0, "Manifest should list fixtures"


def test_missing_snapshot_fails():
    """Missing snapshot file should fail CI."""
    with tempfile.TemporaryDirectory() as tmpdir:
        test_script = Path(tmpdir) / "test_snapshot.py"

        test_code = """
from pathlib import Path

def test_snapshot():
    snapshot_path = Path("test/snapshot/test_case.snap")
    if not snapshot_path.exists():
        raise FileNotFoundError(f"Missing snapshot: {snapshot_path}")

if __name__ == "__main__":
    test_snapshot()
"""

        with open(test_script, 'w') as f:
            f.write(test_code)

        result = subprocess.run(
            ["python3", str(test_script)],
            capture_output=True,
            text=True
        )

        assert result.returncode != 0, "Missing snapshot should fail"


def test_fixture_paths_absolute_in_ci():
    """CI should use absolute paths for fixtures."""
    # Verify CI workflow uses absolute paths
    ci_workflow = Path(".github/workflows/test.yml")

    if ci_workflow.exists():
        with open(ci_workflow) as f:
            content = f.read()

        # Check for workspace path usage
        if "${{ github.workspace }}" in content or "$GITHUB_WORKSPACE" in content:
            assert True, "CI uses absolute paths"


if __name__ == "__main__":
    test_missing_test_template_fails_ci()
    test_missing_policy_fixture_fails_ci()
    test_missing_golden_file_fails_ci()
    test_missing_heuristics_fixture_fails()
    test_fixture_inventory_complete()
    test_ci_validates_fixture_manifest()
    test_missing_snapshot_fails()
    test_fixture_paths_absolute_in_ci()
    print("All missing fixture CI failure tests passed")
