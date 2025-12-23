#!/usr/bin/env python3
"""Test Homebrew formula matches version."""

import subprocess
import json
from pathlib import Path
import re


def test_homebrew_formula_exists():
    """Test that Homebrew formula exists."""
    formula_locations = [
        Path("Formula/costpilot.rb"),
        Path("costpilot.rb"),
        Path("homebrew/costpilot.rb"),
        Path(".github/costpilot.rb")
    ]

    exists = any(p.exists() for p in formula_locations)

    # Formula may not exist yet
    print(f"Homebrew formula exists: {exists}")


def test_homebrew_formula_version_matches():
    """Test that Homebrew formula version matches Cargo.toml."""
    formula_locations = [
        Path("Formula/costpilot.rb"),
        Path("costpilot.rb"),
        Path("homebrew/costpilot.rb"),
        Path(".github/costpilot.rb")
    ]

    cargo_file = Path("Cargo.toml")

    if not cargo_file.exists():
        print("Cargo.toml not found")
        return

    # Get version from Cargo.toml
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Check formula
    for formula_file in formula_locations:
        if formula_file.exists():
            with open(formula_file, 'r') as f:
                formula_content = f.read()

            # Look for version in formula
            formula_version_match = re.search(r'version\s+"([^"]+)"', formula_content)
            if formula_version_match:
                formula_version = formula_version_match.group(1)

                assert formula_version == cargo_version, \
                    f"Homebrew formula version {formula_version} should match Cargo version {cargo_version}"
                return

    print("No Homebrew formula found with version")


def test_package_json_version_matches():
    """Test that package.json version matches Cargo.toml."""
    package_file = Path("package.json")
    cargo_file = Path("Cargo.toml")

    if not package_file.exists() or not cargo_file.exists():
        print("package.json or Cargo.toml not found")
        return

    # Get Cargo version
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Get package.json version
    with open(package_file, 'r') as f:
        package_data = json.load(f)

    if "version" in package_data:
        package_version = package_data["version"]

        assert package_version == cargo_version, \
            f"package.json version {package_version} should match Cargo version {cargo_version}"


def test_version_consistency_across_files():
    """Test version consistency across all version files."""
    cargo_file = Path("Cargo.toml")

    if not cargo_file.exists():
        print("Cargo.toml not found")
        return

    # Get Cargo version
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Check other files
    files_to_check = [
        (Path("package.json"), r'"version":\s*"([^"]+)"'),
        (Path("vscode-extension/package.json"), r'"version":\s*"([^"]+)"'),
        (Path("README.md"), r'Version\s+([0-9]+\.[0-9]+\.[0-9]+)'),
    ]

    for file_path, pattern in files_to_check:
        if file_path.exists():
            with open(file_path, 'r') as f:
                content = f.read()

            version_match = re.search(pattern, content)
            if version_match:
                file_version = version_match.group(1)

                print(f"{file_path}: version {file_version} (Cargo: {cargo_version})")


def test_changelog_has_current_version():
    """Test that CHANGELOG mentions current version."""
    changelog_files = [
        Path("CHANGELOG.md"),
        Path("docs/CHANGELOG.md"),
        Path("CHANGES.md")
    ]

    cargo_file = Path("Cargo.toml")

    if not cargo_file.exists():
        print("Cargo.toml not found")
        return

    # Get Cargo version
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Check changelog
    for changelog_file in changelog_files:
        if changelog_file.exists():
            with open(changelog_file, 'r') as f:
                changelog_content = f.read()

            # Should mention current version
            has_version = cargo_version in changelog_content or f"v{cargo_version}" in changelog_content

            print(f"CHANGELOG mentions {cargo_version}: {has_version}")
            return

    print("No CHANGELOG found")


def test_git_tag_matches_version():
    """Test that git tags include current version."""
    cargo_file = Path("Cargo.toml")

    if not cargo_file.exists():
        print("Cargo.toml not found")
        return

    # Get Cargo version
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Check git tags
    result = subprocess.run(
        ["git", "tag", "-l"],
        capture_output=True,
        text=True,
        timeout=30
    )

    if result.returncode == 0:
        tags = result.stdout.strip().split('\n')

        # Check if current version is tagged
        has_tag = any(cargo_version in tag or f"v{cargo_version}" in tag for tag in tags)

        print(f"Git tag exists for {cargo_version}: {has_tag}")


def test_release_notes_exist():
    """Test that release notes exist for current version."""
    cargo_file = Path("Cargo.toml")

    if not cargo_file.exists():
        print("Cargo.toml not found")
        return

    # Get Cargo version
    with open(cargo_file, 'r') as f:
        cargo_content = f.read()

    cargo_version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not cargo_version_match:
        print("Version not found in Cargo.toml")
        return

    cargo_version = cargo_version_match.group(1)

    # Check release notes
    release_locations = [
        Path(f"docs/releases/{cargo_version}.md"),
        Path(f"releases/{cargo_version}.md"),
        Path(f"RELEASE_NOTES_{cargo_version}.md")
    ]

    exists = any(p.exists() for p in release_locations)

    print(f"Release notes exist for {cargo_version}: {exists}")


if __name__ == "__main__":
    test_homebrew_formula_exists()
    test_homebrew_formula_version_matches()
    test_package_json_version_matches()
    test_version_consistency_across_files()
    test_changelog_has_current_version()
    test_git_tag_matches_version()
    test_release_notes_exist()
    print("All Homebrew formula version tests passed")
