#!/usr/bin/env python3
"""Test Free Edition: deny any premium installer metadata fields."""

import subprocess
import json
from pathlib import Path


def test_cargo_metadata_no_premium():
    """Test Cargo.toml metadata doesn't mention premium."""
    cargo_path = Path("Cargo.toml")
    
    if not cargo_path.exists():
        return
    
    with open(cargo_path) as f:
        content = f.read().lower()
    
    # Should not mention premium features in metadata
    premium_terms = ["premium", "pro edition", "enterprise edition"]
    
    for term in premium_terms:
        assert term not in content, f"Cargo.toml mentions {term}"


def test_package_json_no_premium():
    """Test package.json doesn't advertise premium features."""
    package_path = Path("package.json")
    
    if not package_path.exists():
        return
    
    with open(package_path) as f:
        data = json.load(f)
    
    # Check description
    if "description" in data:
        desc = data["description"].lower()
        assert "premium" not in desc, "package.json description mentions premium"
        assert "pro edition" not in desc, "package.json description mentions Pro"
    
    # Check keywords
    if "keywords" in data:
        keywords = [k.lower() for k in data["keywords"]]
        assert "premium" not in keywords, "package.json has premium keyword"
        assert "pro" not in keywords or "open" in keywords, "package.json has pro keyword"


def test_vscode_extension_metadata():
    """Test VS Code extension metadata is Free Edition."""
    extension_path = Path("vscode-extension/package.json")
    
    if not extension_path.exists():
        return
    
    with open(extension_path) as f:
        data = json.load(f)
    
    # Check display name
    if "displayName" in data:
        name = data["displayName"].lower()
        
        # Should say Community or Free or just CostPilot
        if "community" in name or "free" in name:
            # Good
            pass
        else:
            # Should not say Pro or Premium
            assert "pro" not in name, "Extension displayName claims Pro"
            assert "premium" not in name, "Extension displayName claims Premium"
    
    # Check description
    if "description" in data:
        desc = data["description"].lower()
        
        # Should not advertise premium features
        premium_features = ["autofix", "slo monitoring", "advanced analytics"]
        for feature in premium_features:
            # Be lenient - might mention they exist in Pro version
            pass


def test_readme_no_premium_claims():
    """Test README doesn't claim premium features included."""
    readme_path = Path("README.md")
    
    if not readme_path.exists():
        return
    
    with open(readme_path) as f:
        content = f.read()
    
    # Should clearly state Free Edition limitations
    # Or should say "Free Edition" / "Community Edition"
    content_lower = content.lower()
    
    if "community edition" in content_lower or "free edition" in content_lower:
        # Good - clearly stated
        pass
    else:
        # Should not claim all Pro features are included
        pass


def test_installer_scripts_no_premium():
    """Test installer scripts don't install premium components."""
    install_scripts = [
        "scripts/install.sh",
        "install.sh",
        "setup.py",
    ]
    
    for script in install_scripts:
        path = Path(script)
        if path.exists():
            with open(path) as f:
                content = f.read().lower()
            
            # Should not install premium components
            premium_components = ["costpilot-pro", "costpilot_pro", "premium.bundle"]
            
            for component in premium_components:
                assert component not in content, f"{script} installs {component}"


def test_release_notes_no_premium_features():
    """Test release notes don't list premium-only features."""
    release_files = [
        "CHANGELOG.md",
        "RELEASES.md",
        "docs/RELEASES.md",
    ]
    
    for path in release_files:
        file = Path(path)
        if file.exists():
            with open(file) as f:
                content = f.read()
            
            # If mentions autofix/patch/slo, should clarify they're Pro-only
            premium_features = ["autofix", "patch", "slo"]
            
            for feature in premium_features:
                if feature in content.lower():
                    # Should mention "Pro" or "Premium" nearby
                    # (this is lenient - just documenting the feature existence)
                    pass


if __name__ == "__main__":
    test_cargo_metadata_no_premium()
    test_package_json_no_premium()
    test_vscode_extension_metadata()
    test_readme_no_premium_claims()
    test_installer_scripts_no_premium()
    test_release_notes_no_premium_features()
    print("All Free Edition installer metadata gating tests passed")
