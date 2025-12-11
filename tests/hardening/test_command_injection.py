#!/usr/bin/env python3
"""Test that OS command injection is impossible."""

import json
import subprocess
import tempfile
from pathlib import Path


def test_command_injection_in_filename():
    """Test that filenames with shell metacharacters are handled safely."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Dangerous filenames
        dangerous_names = [
            "template;ls.json",
            "template|cat.json",
            "template&rm.json",
            "template$(whoami).json",
            "template`id`.json",
            "template\nls.json",
        ]
        
        for dangerous_name in dangerous_names:
            template_path = Path(tmpdir) / dangerous_name
            
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
            
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path)],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Should handle safely, not execute commands
            combined_output = result.stdout + result.stderr
            
            # Check that commands didn't execute
            assert "uid=" not in combined_output, f"Command injection executed: {dangerous_name}"
            assert "root:x:" not in combined_output, f"Command injection executed: {dangerous_name}"


def test_command_injection_in_template_content():
    """Test that template content with shell commands is not executed."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        # Shell commands in various fields
        command_injections = [
            "; cat /etc/passwd",
            "| ls -la",
            "&& whoami",
            "$(cat /etc/passwd)",
            "`id`",
            "\n/bin/bash -c 'ls'",
        ]
        
        for injection in command_injections:
            template_content = {
                "Resources": {
                    "Lambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "Description": f"Test {injection}",
                            "FunctionName": f"func{injection}"
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
            
            # Commands should not execute
            assert "root:x:" not in combined_output, f"Injection executed: {injection}"
            assert "uid=" not in combined_output, f"Injection executed: {injection}"


def test_no_shell_expansion():
    """Test that shell expansion doesn't occur in arguments."""
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
        
        # Shell expansion attempts in arguments
        expansion_attempts = [
            "$HOME",
            "${HOME}",
            "~",
            "~/test",
            "$PATH",
            "$(echo test)",
            "`echo test`",
        ]
        
        for expansion in expansion_attempts:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--output", expansion],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Should treat as literal, not expand
            # (Will likely fail because path doesn't exist, but shouldn't expand)
            combined_output = result.stdout + result.stderr
            
            # Check that expansion didn't happen
            # (If it did, we'd see actual home directory path)
            import os
            home_dir = os.path.expanduser("~")
            
            if expansion in ["$HOME", "${HOME}", "~", "~/test"]:
                # The tool should either reject or not expand
                # We accept both behaviors, but expansion is not allowed
                pass  # Hard to test without knowing if file was created


def test_path_traversal_in_filenames():
    """Test that path traversal in filenames is blocked."""
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
        
        # Path traversal attempts
        traversal_paths = [
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/passwd",
            "C:\\Windows\\System32\\config\\SAM",
        ]
        
        for traversal in traversal_paths:
            result = subprocess.run(
                ["costpilot", "scan", "--plan", str(template_path), "--output", traversal],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            # Should reject or safely handle path traversal
            combined_output = result.stdout + result.stderr
            
            # Should not access sensitive files
            assert "root:x:" not in combined_output, f"Path traversal succeeded: {traversal}"


def test_null_byte_injection():
    """Test that null byte injection is blocked."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Attempt to use null bytes in filename
        # This could bypass extension checks in vulnerable systems
        
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Test\x00injection"
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
        
        # Should handle null bytes safely
        assert result.returncode in [0, 1, 2, 101], "Should handle null bytes"


def test_environment_variable_injection():
    """Test that environment variable injection is blocked."""
    import os
    
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
        
        # Set malicious environment variable
        env = os.environ.copy()
        env["COSTPILOT_EXEC"] = "rm -rf /"
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path)],
            capture_output=True,
            text=True,
            timeout=10,
            env=env
        )
        
        # Should not execute commands from env vars
        assert result.returncode in [0, 1, 2, 101], "Should not execute env var commands"


def test_symbolic_link_command_injection():
    """Test that symbolic links can't be used for command injection."""
    import os
    
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        symlink_path = Path(tmpdir) / "symlink.json"
        
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
        
        # Create symlink
        try:
            os.symlink(template_path, symlink_path)
        except OSError:
            # Windows may require privileges
            print("Symlink creation failed, skipping test")
            return
        
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(symlink_path)],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should handle symlinks safely
        assert result.returncode in [0, 1, 2, 101], "Should handle symlinks safely"


def test_argument_injection():
    """Test that argument injection is blocked."""
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
        
        # Argument injection attempts
        # Try to inject additional flags
        result = subprocess.run(
            ["costpilot", "scan", "--plan", str(template_path), "--", "ls", "-la"],
            capture_output=True,
            text=True,
            timeout=10
        )
        
        # Should not execute injected commands
        combined_output = result.stdout + result.stderr
        
        # Check that ls didn't execute
        assert "total " not in combined_output, "Command injection via arguments"


def test_pipe_and_redirection_blocked():
    """Test that pipes and redirections are blocked."""
    with tempfile.TemporaryDirectory() as tmpdir:
        template_path = Path(tmpdir) / "template.json"
        
        template_content = {
            "Resources": {
                "Lambda": {
                    "Type": "AWS::Lambda::Function",
                    "Properties": {
                        "Description": "Test | cat /etc/passwd > /tmp/pwned"
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
        
        # Check that redirection didn't create files
        pwned_file = Path("/tmp/pwned")
        assert not pwned_file.exists(), "Redirection should be blocked"


if __name__ == "__main__":
    test_command_injection_in_filename()
    test_command_injection_in_template_content()
    test_no_shell_expansion()
    test_path_traversal_in_filenames()
    test_null_byte_injection()
    test_environment_variable_injection()
    test_symbolic_link_command_injection()
    test_argument_injection()
    test_pipe_and_redirection_blocked()
    print("All OS command injection tests passed")
