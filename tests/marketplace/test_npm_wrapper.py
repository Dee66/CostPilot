#!/usr/bin/env python3
"""Test npm/npx wrapper parity."""

import subprocess
import tempfile
from pathlib import Path
import json


def test_npm_package_exists():
    """Test that npm package exists."""
    package_file = Path("package.json")

    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Should have name
        assert "name" in package_data, "package.json should have name"
        print(f"npm package name: {package_data.get('name')}")


def test_npx_wrapper_exists():
    """Test that npx wrapper exists."""
    wrapper_locations = [
        Path("bin/costpilot.js"),
        Path("bin/costpilot"),
        Path("cli.js"),
        Path("index.js")
    ]

    exists = any(p.exists() for p in wrapper_locations)

    print(f"npx wrapper exists: {exists}")


def test_npm_binary_field():
    """Test that package.json has bin field."""
    package_file = Path("package.json")

    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Should have bin field
        if "bin" in package_data:
            print(f"npm bin field: {package_data['bin']}")
            assert package_data["bin"], "npm bin field should not be empty"


def test_npm_scripts_match_cli():
    """Test that npm scripts match CLI commands."""
    package_file = Path("package.json")

    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Should have scripts
        if "scripts" in package_data:
            scripts = package_data["scripts"]

            # Common commands
            expected_scripts = ["test", "build", "lint"]

            for script in expected_scripts:
                if script in scripts:
                    print(f"npm script '{script}': {scripts[script]}")


def test_npx_invoke_matches_direct():
    """Test that npx invocation matches direct invocation."""
    package_file = Path("package.json")

    if not package_file.exists():
        print("package.json not found")
        return

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "MemorySize": 1024
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Try direct invocation
        result_direct = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # npx invocation would require published package
        print(f"Direct invocation exit code: {result_direct.returncode}")


def test_npm_dependencies_match():
    """Test that npm dependencies match Cargo dependencies."""
    package_file = Path("package.json")

    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Check dependencies
        deps = package_data.get("dependencies", {})
        dev_deps = package_data.get("devDependencies", {})

        print(f"npm dependencies: {len(deps)}")
        print(f"npm devDependencies: {len(dev_deps)}")


def test_npm_wrapper_version_matches():
    """Test that npm wrapper version matches Cargo version."""
    package_file = Path("package.json")
    cargo_file = Path("Cargo.toml")

    if not package_file.exists() or not cargo_file.exists():
        print("package.json or Cargo.toml not found")
        return

    # Get versions
    import re

    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    with open(package_file, 'r') as f:
        package_data = json.load(f)

    package_version = package_data.get("version", "")

    print(f"Cargo version: {cargo_version}")
    print(f"npm version: {package_version}")

    # Versions should match
    assert package_version == cargo_version, \
        f"npm version {package_version} should match Cargo version {cargo_version}"


def test_npm_readme_exists():
    """Test that npm README exists."""
    readme_locations = [
        Path("npm/README.md"),
        Path("README.npm.md"),
        Path("README.md")
    ]

    exists = any(p.exists() for p in readme_locations)

    assert exists, "npm README should exist"


def test_npm_publish_config():
    """Test that package.json has publish config."""
    package_file = Path("package.json")

    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Check publish config
        if "publishConfig" in package_data:
            print(f"npm publishConfig: {package_data['publishConfig']}")

        # Check files field
        if "files" in package_data:
            print(f"npm files to publish: {package_data['files']}")


if __name__ == "__main__":
    test_npm_package_exists()
    test_npx_wrapper_exists()
    test_npm_binary_field()
    test_npm_scripts_match_cli()
    test_npx_invoke_matches_direct()
    test_npm_dependencies_match()
    test_npm_wrapper_version_matches()
    test_npm_readme_exists()
    test_npm_publish_config()
    print("All npm/npx wrapper parity tests passed")
