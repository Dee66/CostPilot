"""Pytest configuration and fixtures for CostPilot test suite."""

import pytest
import sys
from pathlib import Path

# Add tests directory to path for helper imports
tests_dir = Path(__file__).parent
if str(tests_dir) not in sys.path:
    sys.path.insert(0, str(tests_dir))

from helpers.py_compat import edition_context_free


@pytest.fixture
def free_cli():
    """CLI command for Free edition mode."""
    import os
    costpilot_path = os.path.join(os.path.dirname(__file__), '..', 'target', 'debug', 'costpilot')
    return [costpilot_path]


@pytest.fixture
def premium_cli():
    """CLI command for Premium edition mode."""
    return [
        "costpilot",
        "--edition=premium",
        "--license=tests/fixtures/license.json",
        "--pro-engine=tests/fixtures/pro_engine.wasm.enc"
    ]


@pytest.fixture
def free_edition_context():
    """Free edition context fixture for tests."""
    return edition_context_free()


@pytest.fixture
def normalize_output():
    """Helper to normalize CLI output for consistent assertions."""
    def _normalize(s: str) -> str:
        return s.replace("\r\n", "\n").strip()
    return _normalize


# Marker for Premium-only tests
def pytest_configure(config):
    config.addinivalue_line(
        "markers", "premium_only: mark test as requiring Premium edition"
    )
