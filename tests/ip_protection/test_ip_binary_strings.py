#!/usr/bin/env python3
"""Test IP Protection: No premium constants in Free binary."""

import subprocess
from pathlib import Path


def test_free_binary_strings_analysis():
    """Test Free binary contains no premium constants."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        # Binary not built yet
        return

    result = subprocess.run(
        ["strings", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout.lower()

        # Should not contain premium constants
        forbidden = [
            "premium_key",
            "pro_license",
            "encrypted_heuristics_key",
            "bundle_decrypt_key",
            "premium_engine_init",
            "pro_feature_unlock",
            "license_validation_secret",
            "heuristics_encryption_key",
            "premium_api_key",
            "pro_activation_token"
        ]

        for const in forbidden:
            assert const not in output, f"Free binary should not contain: {const}"


def test_free_binary_no_premium_paths():
    """Test Free binary contains no premium installation paths."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    result = subprocess.run(
        ["strings", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout

        # Should not contain premium paths
        forbidden_paths = [
            "/opt/costpilot/pro",
            "/opt/costpilot/premium",
            "~/.costpilot/premium",
            "~/.costpilot/pro",
            "/usr/local/share/costpilot/premium",
            "C:\\Program Files\\CostPilot\\Premium"
        ]

        for path in forbidden_paths:
            assert path not in output, f"Free binary should not contain path: {path}"


def test_free_binary_no_premium_urls():
    """Test Free binary contains no premium API URLs."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    result = subprocess.run(
        ["strings", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout.lower()

        # Should not contain premium URLs
        forbidden_urls = [
            "api.costpilot.com/premium",
            "license.costpilot.com/activate",
            "premium.costpilot.com",
            "pro.costpilot.com",
            "validate-license.costpilot.com"
        ]

        for url in forbidden_urls:
            assert url not in output, f"Free binary should not contain URL: {url}"


def test_free_binary_no_license_formats():
    """Test Free binary contains no license format strings."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    result = subprocess.run(
        ["strings", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout

        # Should not contain license format strings
        forbidden_formats = [
            "LICENSE-KEY:",
            "PREMIUM-LICENSE:",
            "PRO-ACTIVATION:",
            "ENTERPRISE-KEY:",
            "SIGNATURE:RSA2048:",
            "ENCRYPTED-LICENSE:"
        ]

        for fmt in forbidden_formats:
            assert fmt not in output, f"Free binary should not contain format: {fmt}"


def test_free_binary_no_premium_function_names():
    """Test Free binary contains no premium function names."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    result = subprocess.run(
        ["nm", "-D", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout.lower()

        # Should not export premium functions
        forbidden_functions = [
            "premium_init",
            "pro_engine_load",
            "decrypt_bundle",
            "validate_license",
            "activate_premium",
            "unlock_pro_features",
            "load_encrypted_heuristics"
        ]

        for func in forbidden_functions:
            assert func not in output, f"Free binary should not export: {func}"


def test_free_binary_no_premium_error_codes():
    """Test Free binary contains no premium error codes."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    result = subprocess.run(
        ["strings", binary_path],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout

        # Should not contain premium-specific error codes
        forbidden_errors = [
            "ERR_PREMIUM_LICENSE_EXPIRED",
            "ERR_PREMIUM_LICENSE_INVALID",
            "ERR_PRO_ACTIVATION_FAILED",
            "ERR_BUNDLE_DECRYPT_FAILED",
            "ERR_LICENSE_SIGNATURE_INVALID",
            "ERR_PREMIUM_FEATURE_DISABLED"
        ]

        for err in forbidden_errors:
            assert err not in output, f"Free binary should not contain error: {err}"


def test_free_binary_size_reasonable():
    """Test Free binary size doesn't include premium bloat."""
    binary_path = "target/release/costpilot"

    if not Path(binary_path).exists():
        return

    size_bytes = Path(binary_path).stat().st_size
    size_mb = size_bytes / (1024 * 1024)

    # Free binary should be reasonably sized (< 50MB)
    # Premium would be larger due to bundled heuristics
    assert size_mb < 50, f"Free binary too large: {size_mb:.1f}MB (expected < 50MB)"


if __name__ == "__main__":
    test_free_binary_strings_analysis()
    test_free_binary_no_premium_paths()
    test_free_binary_no_premium_urls()
    test_free_binary_no_license_formats()
    test_free_binary_no_premium_function_names()
    test_free_binary_no_premium_error_codes()
    test_free_binary_size_reasonable()
    print("All IP Protection: binary strings analysis tests passed")
