#!/usr/bin/env python3
"""Test marketplace terms file presence."""

from pathlib import Path


def test_marketplace_terms_file_exists():
    """Test that marketplace terms file exists."""
    terms_file = Path("docs/MARKETPLACE_TERMS.md")

    # Check if terms file exists
    if not terms_file.exists():
        # Check alternative locations
        alt_locations = [
            Path("MARKETPLACE_TERMS.md"),
            Path("docs/marketplace_terms.md"),
            Path("TERMS.md"),
            Path("docs/TERMS.md")
        ]

        exists = any(p.exists() for p in alt_locations)

        assert exists, "Marketplace terms file should exist"
    else:
        assert True, "Marketplace terms file exists"


def test_marketplace_terms_not_empty():
    """Test that marketplace terms file is not empty."""
    terms_locations = [
        Path("docs/MARKETPLACE_TERMS.md"),
        Path("MARKETPLACE_TERMS.md"),
        Path("docs/marketplace_terms.md"),
        Path("TERMS.md"),
        Path("docs/TERMS.md")
    ]

    for terms_file in terms_locations:
        if terms_file.exists():
            with open(terms_file, 'r') as f:
                content = f.read()

            assert len(content) > 100, "Marketplace terms should have content"
            return

    # If no terms file found, that's okay - might not be marketplace yet
    print("No marketplace terms file found - might not be on marketplace yet")


def test_marketplace_terms_has_license_info():
    """Test that marketplace terms mentions license."""
    terms_locations = [
        Path("docs/MARKETPLACE_TERMS.md"),
        Path("MARKETPLACE_TERMS.md"),
        Path("docs/marketplace_terms.md"),
        Path("TERMS.md"),
        Path("docs/TERMS.md")
    ]

    for terms_file in terms_locations:
        if terms_file.exists():
            with open(terms_file, 'r') as f:
                content = f.read().lower()

            # Should mention license
            assert "license" in content or "licensing" in content, \
                "Marketplace terms should mention license"
            return

    print("No marketplace terms file found")


def test_license_file_exists():
    """Test that LICENSE file exists."""
    license_file = Path("LICENSE")

    assert license_file.exists(), "LICENSE file should exist"


def test_license_not_empty():
    """Test that LICENSE file is not empty."""
    license_file = Path("LICENSE")

    if license_file.exists():
        with open(license_file, 'r') as f:
            content = f.read()

        assert len(content) > 100, "LICENSE should have content"


def test_readme_mentions_license():
    """Test that README mentions license."""
    readme_file = Path("README.md")

    if readme_file.exists():
        with open(readme_file, 'r') as f:
            content = f.read().lower()

        # Should mention license
        assert "license" in content or "licensing" in content, \
            "README should mention license"


def test_marketplace_metadata_in_cargo():
    """Test that Cargo.toml has marketplace metadata."""
    cargo_file = Path("Cargo.toml")

    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            content = f.read()

        # Should have metadata
        assert "license" in content.lower(), "Cargo.toml should have license"
        assert "repository" in content.lower() or "homepage" in content.lower(), \
            "Cargo.toml should have repository/homepage"


def test_marketplace_metadata_in_package_json():
    """Test that package.json has marketplace metadata."""
    package_file = Path("package.json")

    if package_file.exists():
        import json

        with open(package_file, 'r') as f:
            package_data = json.load(f)

        # Should have metadata
        assert "license" in package_data, "package.json should have license"
        assert "repository" in package_data or "homepage" in package_data, \
            "package.json should have repository/homepage"


if __name__ == "__main__":
    test_marketplace_terms_file_exists()
    test_marketplace_terms_not_empty()
    test_marketplace_terms_has_license_info()
    test_license_file_exists()
    test_license_not_empty()
    test_readme_mentions_license()
    test_marketplace_metadata_in_cargo()
    test_marketplace_metadata_in_package_json()
    print("All marketplace terms tests passed")
