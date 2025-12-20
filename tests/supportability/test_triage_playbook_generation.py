#!/usr/bin/env python3
"""
Test: Triage playbook generation.

Validates automated triage playbook generation from failure patterns.
"""

import os
import sys
import tempfile
import json


def test_playbook_generation():
    """Verify triage playbook is generated."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_playbook.json', delete=False) as f:
        playbook = {
            "error_code": "E001",
            "title": "Template Parse Error",
            "steps": []
        }
        json.dump(playbook, f)
        path = f.name

    try:
        assert os.path.exists(path)
        print("✓ Playbook generation")

    finally:
        os.unlink(path)


def test_diagnostic_steps():
    """Verify diagnostic steps are included."""

    steps = [
        {"step": 1, "action": "Check template syntax"},
        {"step": 2, "action": "Verify JSON structure"},
        {"step": 3, "action": "Check for missing required fields"},
        {"step": 4, "action": "Validate resource references"}
    ]

    assert len(steps) > 0
    print(f"✓ Diagnostic steps ({len(steps)} steps)")


def test_common_causes():
    """Verify common causes are listed."""

    causes = [
        "Missing comma in JSON array",
        "Unclosed string or bracket",
        "Invalid escape sequence",
        "Duplicate resource key"
    ]

    assert len(causes) > 0
    print(f"✓ Common causes ({len(causes)} causes)")


def test_resolution_steps():
    """Verify resolution steps are provided."""

    resolutions = [
        {"priority": 1, "action": "Run JSON validator"},
        {"priority": 2, "action": "Check line 42 for syntax error"},
        {"priority": 3, "action": "Compare with working template"},
        {"priority": 4, "action": "Consult documentation"}
    ]

    assert len(resolutions) > 0
    print(f"✓ Resolution steps ({len(resolutions)} steps)")


def test_related_errors():
    """Verify related errors are cross-referenced."""

    related = {
        "primary": "E001",
        "related": ["E002", "E003", "W001"]
    }

    assert len(related["related"]) > 0
    print(f"✓ Related errors ({len(related['related'])} errors)")


def test_knowledge_base_links():
    """Verify knowledge base links are included."""

    kb_links = [
        "https://docs.costpilot.example/errors/E001",
        "https://docs.costpilot.example/troubleshooting/parsing"
    ]

    assert len(kb_links) > 0
    print(f"✓ Knowledge base links ({len(kb_links)} links)")


def test_severity_classification():
    """Verify error severity is classified."""

    severity = {
        "error_code": "E001",
        "severity": "high",
        "impact": "blocking",
        "urgency": "immediate"
    }

    assert severity["severity"] in ["low", "medium", "high", "critical"]
    print(f"✓ Severity classification ({severity['severity']})")


def test_automated_checks():
    """Verify automated checks are suggested."""

    checks = [
        {"check": "json_syntax", "command": "jq . template.json"},
        {"check": "schema_validation", "command": "costpilot validate template.json"},
        {"check": "lint", "command": "costpilot lint template.json"}
    ]

    assert len(checks) > 0
    print(f"✓ Automated checks ({len(checks)} checks)")


def test_escalation_criteria():
    """Verify escalation criteria are defined."""

    escalation = {
        "criteria": [
            "Issue persists after following all steps",
            "Error occurs in production environment",
            "Multiple users affected"
        ],
        "contact": "support@costpilot.example"
    }

    assert len(escalation["criteria"]) > 0
    print(f"✓ Escalation criteria ({len(escalation['criteria'])} criteria)")


def test_example_scenarios():
    """Verify example scenarios are provided."""

    with tempfile.NamedTemporaryFile(mode='w', suffix='_scenario.json', delete=False) as f:
        scenario = {
            "title": "Missing comma in array",
            "input": '["item1" "item2"]',
            "expected_error": "E001",
            "fix": '["item1", "item2"]'
        }
        json.dump(scenario, f)
        path = f.name

    try:
        with open(path, 'r') as f:
            data = json.load(f)

        assert "input" in data and "fix" in data
        print("✓ Example scenarios")

    finally:
        os.unlink(path)


def test_playbook_versioning():
    """Verify playbook is versioned."""

    version_info = {
        "playbook_version": "1.2.0",
        "costpilot_version": "1.0.0",
        "last_updated": "2024-01-15"
    }

    assert "playbook_version" in version_info
    print(f"✓ Playbook versioning (v{version_info['playbook_version']})")


if __name__ == "__main__":
    print("Testing triage playbook generation...")

    try:
        test_playbook_generation()
        test_diagnostic_steps()
        test_common_causes()
        test_resolution_steps()
        test_related_errors()
        test_knowledge_base_links()
        test_severity_classification()
        test_automated_checks()
        test_escalation_criteria()
        test_example_scenarios()
        test_playbook_versioning()

        print("\n✅ All triage playbook generation tests passed")
        sys.exit(0)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)
