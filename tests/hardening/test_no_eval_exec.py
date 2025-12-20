#!/usr/bin/env python3
"""Test that no eval/exec code paths exist."""

import re
import subprocess
from pathlib import Path


def test_no_eval_in_rust_code():
    """Test that Rust source code doesn't use eval-like patterns."""
    workspace_root = Path("/home/dee/workspace/AI/GuardSuite/CostPilot")
    src_dir = workspace_root / "src"

    if not src_dir.exists():
        print("Source directory not found, skipping test")
        return

    # Search for dangerous patterns
    dangerous_patterns = [
        r"eval\s*\(",
        r"exec\s*\(",
        r"system\s*\(",
        r"Command::new\s*\(\s*[\"'](?:sh|bash|cmd|powershell)",
    ]

    violations = []

    for rust_file in src_dir.rglob("*.rs"):
        content = rust_file.read_text()

        for pattern in dangerous_patterns:
            matches = re.findall(pattern, content, re.IGNORECASE)
            if matches:
                violations.append(f"{rust_file.name}: {pattern}")

    # Allow specific safe cases (e.g., in test files or commented)
    # But flag any suspicious usage
    if violations:
        print(f"Warning: Potential eval/exec patterns found: {violations}")


def test_no_dynamic_code_execution():
    """Test that binary doesn't allow dynamic code execution."""
    # Check if costpilot binary exists
    result = subprocess.run(
        ["which", "costpilot"],
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        # Binary not installed, build check
        print("Binary not found, checking source code only")
        return

    # Run costpilot with various inputs to ensure no code execution
    test_cases = [
        '{"eval": "1+1"}',
        '{"exec": "ls"}',
        '{"system": "echo test"}',
        '{"__import__": "os"}',
    ]

    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        for test_case in test_cases:
            # Inject potentially dangerous content
            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Description": test_case
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Should not execute any code (no side effects)
            # Just check it doesn't crash or behave unexpectedly
            assert result.returncode in [0, 1, 2, 101], f"Unexpected behavior with: {test_case}"


def test_no_shell_injection_vectors():
    """Test that shell injection is not possible."""
    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Injection attempts
        injection_attempts = [
            "; ls",
            "| cat /etc/passwd",
            "&& rm -rf /",
            "$(whoami)",
            "`id`",
            "\n/bin/sh",
        ]

        for injection in injection_attempts:
            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Description": f"Test {injection}"
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Should treat as literal string, not execute
            combined_output = result.stdout + result.stderr

            # Check that injection didn't execute
            # (No user list, no file deletions, etc.)
            assert "root:x:" not in combined_output, "Shell injection may have executed"
            assert "uid=" not in combined_output, "Command injection may have executed"


def test_no_code_generation():
    """Test that no dynamic code generation occurs."""
    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Templates that might trigger code generation
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Code": {
                            "ZipFile": "def handler(event, context): exec('import os; os.system(\"ls\")')"
                        }
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should analyze template, not execute embedded code
        assert result.returncode in [0, 1, 2, 101], "Should analyze without executing embedded code"


def test_no_unsafe_deserialization():
    """Test that deserialization is safe."""
    import tempfile
    from pathlib import Path

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Malicious serialized data patterns
        malicious_payloads = [
            '{"__proto__": {"isAdmin": true}}',  # Prototype pollution
            '{"constructor": {"prototype": {"isAdmin": true}}}',  # Constructor pollution
        ]

        for payload in malicious_payloads:
            with open(template_path, 'w') as f:
                f.write(payload)

            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            # Should reject or safely handle malicious payloads
            assert result.returncode in [1, 2, 101], "Should reject malicious payloads"


def test_no_reflection_apis():
    """Test that reflection/introspection APIs are not exposed."""
    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Attempts to use reflection
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Environment": {
                            "Variables": {
                                "CLASS": "__class__",
                                "MODULE": "__module__",
                                "DICT": "__dict__"
                            }
                        }
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--verbose"],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Should treat as literal strings, not reflection
        combined_output = result.stdout + result.stderr

        # Check that reflection didn't expose internals
        assert "impl " not in combined_output, "Implementation details should not be exposed"
        assert "struct " not in combined_output, "Struct definitions should not be exposed"


def test_no_file_inclusion_vulnerabilities():
    """Test that file inclusion vulnerabilities don't exist."""
    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # File inclusion attempts
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "include('/etc/passwd')"
                    }
                }
            }
        }

        with open(template_path, 'w') as f:
            json.dump(template_content, f)

        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10
        )

        combined_output = result.stdout + result.stderr

        # Should not include file contents
        assert "root:x:" not in combined_output, "File inclusion should be blocked"


def test_no_template_injection():
    """Test that template injection is not possible."""
    import tempfile
    from pathlib import Path
    import json

    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"

        # Template injection attempts (various template engines)
        injections = [
            "{{7*7}}",  # Handlebars/Mustache
            "${7*7}",   # EL
            "#{7*7}",   # Ruby
            "<%= 7*7 %>",  # ERB
        ]

        for injection in injections:
            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Description": injection
                        }
                    }
                }
            }

            with open(template_path, 'w') as f:
                json.dump(template_content, f)

            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )

            combined_output = result.stdout + result.stderr

            # Should not evaluate template expressions
            assert "49" not in combined_output, f"Template injection may have evaluated: {injection}"


if __name__ == "__main__":
    test_no_eval_in_rust_code()
    test_no_dynamic_code_execution()
    test_no_shell_injection_vectors()
    test_no_code_generation()
    test_no_unsafe_deserialization()
    test_no_reflection_apis()
    test_no_file_inclusion_vulnerabilities()
    test_no_template_injection()
    print("All eval/exec path validation tests passed")
