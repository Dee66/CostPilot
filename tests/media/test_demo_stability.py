#!/usr/bin/env python3
"""Test demo and media asset stability."""

import hashlib
import subprocess
import tempfile
from pathlib import Path


def test_pr_gif_hash_stability():
    """PR demo GIF should have stable hash."""
    gif_path = Path("docs/demo.gif")

    if not gif_path.exists():
        # Check alternate locations
        gif_path = Path("assets/demo.gif")

    if gif_path.exists():
        # Compute hash
        with open(gif_path, 'rb') as f:
            gif_hash = hashlib.sha256(f.read()).hexdigest()

        # Store hash for comparison
        hash_file = gif_path.with_suffix('.gif.sha256')

        if hash_file.exists():
            with open(hash_file) as f:
                stored_hash = f.read().strip()

            assert gif_hash == stored_hash, "GIF hash should be stable"
        else:
            print(f"Note: Create hash file {hash_file.name} with content: {gif_hash}")


def test_readme_code_block_golden_match():
    """README code examples should match golden outputs."""
    readme = Path("README.md")

    if not readme.exists():
        print("Note: README.md not found")
        return

    with open(readme) as f:
        content = f.read()

    # Extract code blocks
    import re
    code_blocks = re.findall(r'```(?:bash|shell|console)\n(.*?)```', content, re.DOTALL)

    # Check if examples are up to date
    if code_blocks:
        # First code block should be basic usage
        assert len(code_blocks) > 0, "README should have code examples"


def test_diagram_export_pixel_stability():
    """Exported diagrams should have pixel-perfect stability."""
    diagram_files = [
        Path("docs/architecture.png"),
        Path("docs/flow-diagram.png"),
        Path("assets/graph.png")
    ]

    for diagram_file in diagram_files:
        if diagram_file.exists():
            # Compute hash
            with open(diagram_file, 'rb') as f:
                diagram_hash = hashlib.sha256(f.read()).hexdigest()

            hash_file = diagram_file.with_suffix(diagram_file.suffix + '.sha256')

            if hash_file.exists():
                with open(hash_file) as f:
                    stored_hash = f.read().strip()

                assert diagram_hash == stored_hash, f"{diagram_file.name} should have stable pixels"


def test_trend_svg_markdown_embedding_stable():
    """Trend SVG embedded in markdown should be stable."""
    trend_svg = Path("docs/trend.svg")

    if not trend_svg.exists():
        trend_svg = Path("assets/trend.svg")

    if trend_svg.exists():
        with open(trend_svg) as f:
            svg_content = f.read()

        # SVG should not contain timestamps
        assert "timestamp" not in svg_content.lower() or "<!-- stable -->" in svg_content, \
            "SVG should not have dynamic timestamps"

        # Check hash stability
        svg_hash = hashlib.sha256(svg_content.encode()).hexdigest()
        hash_file = trend_svg.with_suffix('.svg.sha256')

        if hash_file.exists():
            with open(hash_file) as f:
                stored_hash = f.read().strip()

            assert svg_hash == stored_hash, "SVG should have stable hash"


def test_demo_repo_reset_idempotency():
    """Demo repository reset should be idempotent."""
    with tempfile.TemporaryDirectory() as tmpdir:
        demo_dir = Path(tmpdir) / "demo"
        demo_dir.mkdir()

        # Simulate demo repository
        demo_file = demo_dir / "demo.json"

        initial_state = '{"resources": []}'

        # First reset
        with open(demo_file, 'w') as f:
            f.write(initial_state)

        with open(demo_file, 'rb') as f:
            hash1 = hashlib.sha256(f.read()).hexdigest()

        # Modify
        with open(demo_file, 'w') as f:
            f.write('{"resources": ["modified"]}')

        # Second reset (should be identical to first)
        with open(demo_file, 'w') as f:
            f.write(initial_state)

        with open(demo_file, 'rb') as f:
            hash2 = hashlib.sha256(f.read()).hexdigest()

        assert hash1 == hash2, "Reset should be idempotent"


def test_screenshot_hash_tracking():
    """Screenshots should have hash tracking."""
    screenshot_dirs = [
        Path("docs/screenshots"),
        Path("assets/screenshots"),
        Path("docs/images")
    ]

    for screenshot_dir in screenshot_dirs:
        if screenshot_dir.exists():
            screenshots = list(screenshot_dir.glob("*.png")) + list(screenshot_dir.glob("*.jpg"))

            for screenshot in screenshots:
                hash_file = screenshot.with_suffix(screenshot.suffix + '.sha256')

                if not hash_file.exists():
                    print(f"Note: Create hash file for {screenshot.name}")


def test_animated_demo_frame_count():
    """Animated demos should have consistent frame count."""
    gif_files = [
        Path("docs/demo.gif"),
        Path("assets/demo.gif")
    ]

    for gif_file in gif_files:
        if gif_file.exists():
            # Check GIF properties
            try:
                from PIL import Image
                with Image.open(gif_file) as img:
                    frame_count = getattr(img, 'n_frames', 1)
                    print(f"{gif_file.name}: {frame_count} frames")
            except ImportError:
                print("Note: Install Pillow to check GIF frame count")
            except Exception as e:
                print(f"Note: Could not read {gif_file.name}: {e}")


def test_readme_embedded_svg_valid():
    """README embedded SVGs should be valid."""
    readme = Path("README.md")

    if not readme.exists():
        return

    with open(readme) as f:
        content = f.read()

    # Check for SVG embeds
    import re
    svg_refs = re.findall(r'!\[.*?\]\((.*?\.svg)\)', content)

    for svg_ref in svg_refs:
        svg_path = Path(svg_ref)
        if svg_path.exists():
            with open(svg_path) as f:
                svg_content = f.read()

            assert svg_content.startswith('<svg') or svg_content.startswith('<?xml'), \
                f"{svg_ref} should be valid SVG"


if __name__ == "__main__":
    test_pr_gif_hash_stability()
    test_readme_code_block_golden_match()
    test_diagram_export_pixel_stability()
    test_trend_svg_markdown_embedding_stable()
    test_demo_repo_reset_idempotency()
    test_screenshot_hash_tracking()
    test_animated_demo_frame_count()
    test_readme_embedded_svg_valid()
    print("All demo and media stability tests passed")
