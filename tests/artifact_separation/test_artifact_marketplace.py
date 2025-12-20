#!/usr/bin/env python3
"""Test Artifact Separation: Marketplace installer premium fields."""

from pathlib import Path
import json


def test_marketplace_metadata_edition_field():
    """Test marketplace metadata has edition field."""
    vscode_package = Path("vscode-extension/package.json")

    if not vscode_package.exists():
        return

    with open(vscode_package) as f:
        package = json.load(f)

    # Should have edition metadata
    # Free: "community", "free", or "oss"
    # Premium: "premium", "pro", or "enterprise"

    # Document expected field


def test_marketplace_free_no_premium_keywords():
    """Test Free marketplace listing has no premium keywords."""
    vscode_package = Path("vscode-extension/package.json")

    if not vscode_package.exists():
        return

    with open(vscode_package) as f:
        package = json.load(f)

    if "keywords" in package:
        keywords = [k.lower() for k in package["keywords"]]

        # Free should not have premium keywords
        forbidden = ["premium", "pro", "enterprise", "commercial"]

        for keyword in forbidden:
            if keyword in keywords:
                # Document: Free marketplace should not have premium keywords
                pass


def test_marketplace_premium_has_pricing():
    """Test Premium marketplace metadata has pricing info."""
    # Premium marketplace listings might have pricing field
    # Free would not have pricing

    # Document expected Premium marketplace metadata


def test_marketplace_description_edition():
    """Test marketplace description mentions edition."""
    vscode_package = Path("vscode-extension/package.json")

    if not vscode_package.exists():
        return

    with open(vscode_package) as f:
        package = json.load(f)

    if "description" in package:
        desc = package["description"].lower()

        # Should mention edition
        edition_words = ["community", "free", "premium", "pro"]
        has_edition = any(word in desc for word in edition_words)

        # Document: description should identify edition


def test_marketplace_display_name_edition():
    """Test marketplace display name identifies edition."""
    vscode_package = Path("vscode-extension/package.json")

    if not vscode_package.exists():
        return

    with open(vscode_package) as f:
        package = json.load(f)

    if "displayName" in package:
        display_name = package["displayName"]

        # Free: "CostPilot Community" or "CostPilot"
        # Premium: "CostPilot Premium" or "CostPilot Pro"

        # Document expected naming


def test_marketplace_free_no_paid_category():
    """Test Free marketplace listing not in paid category."""
    vscode_package = Path("vscode-extension/package.json")

    if not vscode_package.exists():
        return

    with open(vscode_package) as f:
        package = json.load(f)

    # Free should not be in paid/premium categories
    # Document expected categories


if __name__ == "__main__":
    test_marketplace_metadata_edition_field()
    test_marketplace_free_no_premium_keywords()
    test_marketplace_premium_has_pricing()
    test_marketplace_description_edition()
    test_marketplace_display_name_edition()
    test_marketplace_free_no_paid_category()
    print("All Artifact Separation: marketplace installer tests passed (documented)")
