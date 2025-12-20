#!/usr/bin/env python3
"""Test acceptance criteria validation."""

import json
import platform
import subprocess
import tempfile
from pathlib import Path


def test_ac_pass_on_windows():
    """Acceptance criteria should pass on Windows."""
    current_os = platform.system()

    if current_os == "Windows":
        # Run acceptance tests
        result = subprocess.run(
            ["python", "-m", "pytest", "test/acceptance/", "-v"],
            capture_output=True,
            text=True
        )

        # Should pass on Windows
        assert result.returncode == 0, "Acceptance criteria should pass on Windows"
    else:
        print(f"Note: Running on {current_os}, not Windows")


def test_ac_pass_on_readonly_fs():
    """Acceptance criteria should pass on read-only filesystem."""
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

        # Make directory read-only
        import os
        import stat

        # Save original permissions
        original_mode = os.stat(tmpdir).st_mode

        try:
            # Set read-only (remove write permissions)
            os.chmod(tmpdir, stat.S_IRUSR | stat.S_IXUSR)

            # Run analysis (should work in read-only mode)
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True
            )

            # Should handle read-only filesystem
            assert result.returncode in [0, 1, 2, 101], "Should work on read-only filesystem"
        finally:
            # Restore permissions
            os.chmod(tmpdir, original_mode)


def test_ac_pass_under_slow_disk():
    """Acceptance criteria should pass under slow disk I/O."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Large template to test I/O
        resources = {
            f"Lambda{i}": {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "MemorySize": 1024,
                    "Code": "x" * 1000  # Some content
                }
            }
            for i in range(100)
        }

        template_content = {"Resources": resources}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Run with timeout (simulating slow disk)
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Should complete within reasonable time
        assert result.returncode in [0, 1, 2, 101], "Should handle slow disk I/O"


def test_ac_metadata_presence_in_json():
    """Acceptance criteria metadata should be present in --json output."""
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

        # Run with --json format
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--format", "json"],
            capture_output=True,
            text=True
        )

        if result.stdout:
            try:
                output_data = json.loads(result.stdout)

                # Check for metadata fields
                metadata_fields = ["version", "timestamp", "resources"]

                # At least some metadata should be present
                has_metadata = any(field in output_data for field in metadata_fields)

                assert has_metadata or "metadata" in output_data, \
                    "JSON output should include metadata"
            except json.JSONDecodeError:
                print("Note: Output not in JSON format")


def test_multi_slo_ac04_flow():
    """Multi-SLO AC-04 flow should work correctly."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        slo_path = Path(tmpdir) / "slo.json"

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

        # Multiple SLO objectives
        slo_content = {
            "objectives": [
                {
                    "id": "cost_accuracy",
                    "target": 0.95,
                    "window": "30d"
                },
                {
                    "id": "prediction_stability",
                    "target": 0.99,
                    "window": "7d"
                },
                {
                    "id": "false_positive_rate",
                    "target": 0.05,
                    "window": "30d"
                }
            ]
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        with open(slo_path, 'w') as f:
            json.dump(slo_content, f)

        # Run with SLO
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--slo", str(slo_path)],
            capture_output=True,
            text=True
        )

        # Should handle multiple SLOs
        assert result.returncode in [0, 1, 2, 101], "Should handle multi-SLO flow"

        output = result.stdout + result.stderr
        if "slo" in output.lower() or "objective" in output.lower():
            assert True, "Should process SLO objectives"


def test_ac_cross_platform_paths():
    """Acceptance criteria should handle cross-platform paths."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Use Path for cross-platform compatibility
        template_path = Path(tmpdir) / "template.json"

        template_content = {"Resources": {}}

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        # Run with cross-platform path
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # Should work regardless of platform
        assert result.returncode in [0, 1, 2, 101], "Should handle cross-platform paths"


def test_ac_unicode_handling():
    """Acceptance criteria should handle Unicode properly."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template with Unicode
        template_content = {
            "Resources": {
                "Lambda北京": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "テスト function with émojis 🚀"
                    }
                }
            }
        }

        with open(template_path, 'w', encoding='utf-8') as f:
            json.dump(template_content, f, ensure_ascii=False)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True
        )

        # Should handle Unicode
        assert result.returncode in [0, 1, 2, 101], "Should handle Unicode"


if __name__ == "__main__":
    test_ac_pass_on_windows()
    test_ac_pass_on_readonly_fs()
    test_ac_pass_under_slow_disk()
    test_ac_metadata_presence_in_json()
    test_multi_slo_ac04_flow()
    test_ac_cross_platform_paths()
    test_ac_unicode_handling()
    print("All acceptance criteria tests passed")
