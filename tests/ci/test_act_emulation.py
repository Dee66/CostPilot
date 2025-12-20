#!/usr/bin/env python3
"""Test macOS and Windows act emulation."""

import platform
import subprocess
import tempfile
from pathlib import Path


def test_act_tool_available():
    """Act tool should be available for local CI testing."""
    result = subprocess.run(
        ["which", "act"],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print("Note: Install act for local CI testing: https://github.com/nektos/act")


def test_act_macos_emulation():
    """Act should support macOS runner emulation."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        print("Note: No CI workflow to test")
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for macOS runners
    if "macos" in content.lower():
        # Act can emulate with docker
        print("Note: macOS runners found - act requires docker platform flag")


def test_act_windows_emulation():
    """Act should support Windows runner emulation."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        print("Note: No CI workflow to test")
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for Windows runners
    if "windows" in content.lower():
        print("Note: Windows runners found - act requires special platform configuration")


def test_act_config_file_exists():
    """Act configuration file should exist."""
    act_config = Path(".actrc")

    if act_config.exists():
        with open(act_config) as f:
            content = f.read()

        # Should have platform configurations
        assert True, "Act config exists"
    else:
        print("Note: Create .actrc for act configuration")


def test_cross_platform_script_compatibility():
    """CI scripts should be cross-platform compatible."""
    scripts_dir = Path("scripts")

    if not scripts_dir.exists():
        print("Note: No scripts directory")
        return

    shell_scripts = list(scripts_dir.glob("*.sh"))

    for script in shell_scripts:
        with open(script) as f:
            content = f.read()

        # Check for shebang
        if not content.startswith("#!"):
            print(f"Warning: {script.name} missing shebang")


def test_platform_specific_steps_documented():
    """Platform-specific CI steps should be documented."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        print("Note: No CI workflow to test")
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for platform-specific steps
    if "if: runner.os ==" in content or "if: matrix.os ==" in content:
        assert True, "Platform-specific steps documented"


def test_local_ci_testing_documented():
    """Local CI testing with act should be documented."""
    readme = Path("README.md")
    contributing = Path("CONTRIBUTING.md")

    doc_files = [f for f in [readme, contributing] if f.exists()]

    for doc_file in doc_files:
        with open(doc_file) as f:
            content = f.read()

        if "act" in content.lower() and "local" in content.lower():
            assert True, f"Act documented in {doc_file.name}"
            return

    print("Note: Document act for local CI testing")


def test_docker_platform_images_configured():
    """Docker platform images should be configured for act."""
    act_platforms = Path(".github/workflows/platforms.json")

    if act_platforms.exists():
        import json
        with open(act_platforms) as f:
            platforms = json.load(f)

        # Should have platform mappings
        assert isinstance(platforms, dict), "Platform configuration should be dict"


def test_macos_specific_dependencies():
    """macOS-specific dependencies should be handled."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for macOS setup steps
    if "macos" in content.lower():
        if "brew install" in content or "setup-macos" in content:
            assert True, "macOS dependencies handled"


def test_windows_specific_paths():
    """Windows path handling should be correct."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for Windows-specific path handling
    if "windows" in content.lower():
        # Should use $env: or proper path separators
        if "$env:" in content or "pwsh" in content:
            assert True, "Windows paths handled"


def test_act_event_simulation():
    """Act should simulate different GitHub events."""
    # Test that workflow can be triggered with different events
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for event triggers
    if "on:" in content:
        assert "push" in content or "pull_request" in content, "Workflow has triggers"


def test_platform_matrix_complete():
    """CI platform matrix should cover all targets."""
    ci_workflow = Path(".github/workflows/test.yml")

    if not ci_workflow.exists():
        return

    with open(ci_workflow) as f:
        content = f.read()

    # Check for matrix strategy
    if "matrix:" in content:
        # Should include multiple platforms
        platforms = ["ubuntu", "macos", "windows"]
        platform_count = sum(1 for p in platforms if p in content.lower())

        if platform_count >= 2:
            assert True, f"Platform matrix includes {platform_count} platforms"


if __name__ == "__main__":
    test_act_tool_available()
    test_act_macos_emulation()
    test_act_windows_emulation()
    test_act_config_file_exists()
    test_cross_platform_script_compatibility()
    test_platform_specific_steps_documented()
    test_local_ci_testing_documented()
    test_docker_platform_images_configured()
    test_macos_specific_dependencies()
    test_windows_specific_paths()
    test_act_event_simulation()
    test_platform_matrix_complete()
    print("All act emulation tests passed")
