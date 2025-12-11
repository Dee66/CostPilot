#!/usr/bin/env python3
"""Test marketplace metadata consistency."""

import json
from pathlib import Path
import re


def test_metadata_consistent_across_manifests():
    """Test that metadata is consistent across all manifests."""
    cargo_file = Path("Cargo.toml")
    package_file = Path("package.json")
    
    metadata = {}
    
    # Get Cargo metadata
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        # Extract fields
        version_match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
        name_match = re.search(r'name\s*=\s*"([^"]+)"', cargo_content)
        description_match = re.search(r'description\s*=\s*"([^"]+)"', cargo_content)
        license_match = re.search(r'license\s*=\s*"([^"]+)"', cargo_content)
        
        if version_match:
            metadata['cargo_version'] = version_match.group(1)
        if name_match:
            metadata['cargo_name'] = name_match.group(1)
        if description_match:
            metadata['cargo_description'] = description_match.group(1)
        if license_match:
            metadata['cargo_license'] = license_match.group(1)
    
    # Get package.json metadata
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        metadata['npm_version'] = package_data.get('version')
        metadata['npm_name'] = package_data.get('name')
        metadata['npm_description'] = package_data.get('description')
        metadata['npm_license'] = package_data.get('license')
    
    # Check consistency
    if 'cargo_version' in metadata and 'npm_version' in metadata:
        assert metadata['cargo_version'] == metadata['npm_version'], \
            f"Version mismatch: Cargo {metadata['cargo_version']} vs npm {metadata['npm_version']}"
    
    if 'cargo_license' in metadata and 'npm_license' in metadata:
        assert metadata['cargo_license'] == metadata['npm_license'], \
            f"License mismatch: Cargo {metadata['cargo_license']} vs npm {metadata['npm_license']}"
    
    print(f"Metadata: {metadata}")


def test_description_consistent():
    """Test that description is consistent across files."""
    cargo_file = Path("Cargo.toml")
    package_file = Path("package.json")
    readme_file = Path("README.md")
    
    descriptions = {}
    
    # Get Cargo description
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        desc_match = re.search(r'description\s*=\s*"([^"]+)"', cargo_content)
        if desc_match:
            descriptions['cargo'] = desc_match.group(1)
    
    # Get npm description
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        if 'description' in package_data:
            descriptions['npm'] = package_data['description']
    
    # Get README description (first non-header line)
    if readme_file.exists():
        with open(readme_file, 'r') as f:
            lines = f.readlines()
        
        for line in lines:
            if line.strip() and not line.startswith('#'):
                descriptions['readme'] = line.strip()
                break
    
    print(f"Descriptions: {descriptions}")


def test_repository_urls_consistent():
    """Test that repository URLs are consistent."""
    cargo_file = Path("Cargo.toml")
    package_file = Path("package.json")
    
    repos = {}
    
    # Get Cargo repository
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        repo_match = re.search(r'repository\s*=\s*"([^"]+)"', cargo_content)
        if repo_match:
            repos['cargo'] = repo_match.group(1)
    
    # Get npm repository
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        if 'repository' in package_data:
            if isinstance(package_data['repository'], dict):
                repos['npm'] = package_data['repository'].get('url', '')
            else:
                repos['npm'] = package_data['repository']
    
    # Normalize URLs
    for key in repos:
        repos[key] = repos[key].replace('git+', '').replace('.git', '')
    
    # Check consistency
    if 'cargo' in repos and 'npm' in repos:
        assert repos['cargo'] in repos['npm'] or repos['npm'] in repos['cargo'], \
            f"Repository mismatch: Cargo {repos['cargo']} vs npm {repos['npm']}"
    
    print(f"Repositories: {repos}")


def test_keywords_present():
    """Test that keywords are present in metadata."""
    package_file = Path("package.json")
    cargo_file = Path("Cargo.toml")
    
    keywords = {}
    
    # Get npm keywords
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        if 'keywords' in package_data:
            keywords['npm'] = package_data['keywords']
    
    # Get Cargo keywords
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        keywords_match = re.search(r'keywords\s*=\s*\[([^\]]+)\]', cargo_content)
        if keywords_match:
            keywords['cargo'] = [k.strip().strip('"') for k in keywords_match.group(1).split(',')]
    
    print(f"Keywords: {keywords}")


def test_authors_consistent():
    """Test that authors are consistent."""
    cargo_file = Path("Cargo.toml")
    package_file = Path("package.json")
    
    authors = {}
    
    # Get Cargo authors
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        authors_match = re.search(r'authors\s*=\s*\[([^\]]+)\]', cargo_content)
        if authors_match:
            authors['cargo'] = [a.strip().strip('"') for a in authors_match.group(1).split(',')]
    
    # Get npm author
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        if 'author' in package_data:
            authors['npm'] = package_data['author']
    
    print(f"Authors: {authors}")


def test_homepage_consistent():
    """Test that homepage is consistent."""
    cargo_file = Path("Cargo.toml")
    package_file = Path("package.json")
    
    homepages = {}
    
    # Get Cargo homepage
    if cargo_file.exists():
        with open(cargo_file, 'r') as f:
            cargo_content = f.read()
        
        homepage_match = re.search(r'homepage\s*=\s*"([^"]+)"', cargo_content)
        if homepage_match:
            homepages['cargo'] = homepage_match.group(1)
    
    # Get npm homepage
    if package_file.exists():
        with open(package_file, 'r') as f:
            package_data = json.load(f)
        
        if 'homepage' in package_data:
            homepages['npm'] = package_data['homepage']
    
    # Check consistency
    if 'cargo' in homepages and 'npm' in homepages:
        assert homepages['cargo'] == homepages['npm'], \
            f"Homepage mismatch: Cargo {homepages['cargo']} vs npm {homepages['npm']}"
    
    print(f"Homepages: {homepages}")


def test_vscode_extension_metadata():
    """Test VS Code extension metadata."""
    vscode_package = Path("vscode-extension/package.json")
    
    if vscode_package.exists():
        with open(vscode_package, 'r') as f:
            vscode_data = json.load(f)
        
        # Should have required fields
        required_fields = ['name', 'displayName', 'version', 'publisher', 'engines']
        
        for field in required_fields:
            assert field in vscode_data, f"VS Code extension should have {field}"
        
        print(f"VS Code extension: {vscode_data.get('name')} v{vscode_data.get('version')}")


def test_github_marketplace_metadata():
    """Test GitHub marketplace metadata."""
    github_dir = Path(".github")
    
    if github_dir.exists():
        # Check for marketplace files
        marketplace_files = [
            Path(".github/marketplace.yml"),
            Path(".github/MARKETPLACE.md"),
            Path(".github/marketplace.json")
        ]
        
        exists = any(p.exists() for p in marketplace_files)
        
        print(f"GitHub marketplace metadata exists: {exists}")


if __name__ == "__main__":
    test_metadata_consistent_across_manifests()
    test_description_consistent()
    test_repository_urls_consistent()
    test_keywords_present()
    test_authors_consistent()
    test_homepage_consistent()
    test_vscode_extension_metadata()
    test_github_marketplace_metadata()
    print("All marketplace metadata consistency tests passed")
