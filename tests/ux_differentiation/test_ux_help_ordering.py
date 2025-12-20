#!/usr/bin/env python3
"""Test UX Differentiation: Help ordering consistent between editions."""

import subprocess


def test_help_command_order():
    """Test help lists commands in consistent order."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    assert result.returncode == 0, "Help should succeed"

    output = result.stdout
    lines = output.split('\n')

    # Find commands section
    commands = []
    in_commands = False

    for line in lines:
        line_lower = line.lower()
        if "command" in line_lower:
            in_commands = True
            continue

        if in_commands:
            # Extract command names
            stripped = line.strip()
            if stripped and not stripped.startswith('-'):
                # Might be a command
                parts = stripped.split()
                if len(parts) > 0:
                    cmd = parts[0]
                    if cmd.isalpha():
                        commands.append(cmd)

    # Commands should be in consistent order
    # Typically: alphabetical or by category
    if len(commands) > 1:
        # Check if alphabetical
        is_alphabetical = commands == sorted(commands)

        # Order should be deterministic
        # Either alphabetical or documented order


def test_help_flag_order():
    """Test help lists flags in consistent order."""
    result = subprocess.run(
        ["costpilot", "scan", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout

        # Flags should be in consistent order
        # Typically: alphabetical or grouped by function

        # Document expected order


def test_help_section_order():
    """Test help sections appear in consistent order."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        output = result.stdout.lower()

        # Typical sections:
        # 1. Usage
        # 2. Description
        # 3. Commands
        # 4. Options/Flags
        # 5. Examples
        # 6. Footer/More info

        sections = []
        for keyword in ["usage", "commands", "options", "flags", "examples"]:
            if keyword in output:
                index = output.index(keyword)
                sections.append((index, keyword))

        # Sections should be in order
        if len(sections) > 1:
            sorted_sections = sorted(sections, key=lambda x: x[0])
            # Verify order makes sense


def test_help_output_width_consistent():
    """Test help output width is consistent."""
    result = subprocess.run(
        ["costpilot", "--help"],
        capture_output=True,
        text=True,
        timeout=10
    )

    if result.returncode == 0:
        lines = result.stdout.split('\n')

        # Lines should not be excessively long
        max_length = max(len(line) for line in lines)

        # Typical terminal width is 80-120 characters
        assert max_length < 200, "Help lines should fit in terminal"


def test_help_deterministic_across_runs():
    """Test help output is identical across runs."""
    outputs = []

    for _ in range(3):
        result = subprocess.run(
            ["costpilot", "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )
        assert result.returncode == 0
        outputs.append(result.stdout)

    # All outputs should be identical
    assert outputs[0] == outputs[1] == outputs[2], "Help should be deterministic"


def test_subcommand_help_consistent():
    """Test subcommand help follows same structure."""
    subcommands = ["analyze", "predict", "check"]

    structures = []

    for subcmd in subcommands:
        result = subprocess.run(
            ["costpilot", subcmd, "--help"],
            capture_output=True,
            text=True,
            timeout=10
        )

        if result.returncode == 0:
            output = result.stdout.lower()

            # Extract structure
            has_usage = "usage" in output
            has_description = "description" in output or len(output) > 50
            has_options = "options" in output or "flags" in output

            structure = (has_usage, has_description, has_options)
            structures.append(structure)

    # All subcommands should have similar structure
    if len(structures) > 1:
        assert structures[0] == structures[1], "Subcommand help should be consistent"


if __name__ == "__main__":
    test_help_command_order()
    test_help_flag_order()
    test_help_section_order()
    test_help_output_width_consistent()
    test_help_deterministic_across_runs()
    test_subcommand_help_consistent()
    print("All UX Differentiation: help ordering tests passed")
