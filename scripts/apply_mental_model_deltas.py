#!/usr/bin/env python3
"""
Mental Model Delta Application Tool

Applies approved deltas to the mental model, respecting freeze state.
"""

import sys
from pathlib import Path

def apply_deltas(repo_root: Path, delta_file: str):
    """Apply deltas from file to mental model"""
    mental_model_path = repo_root / "docs" / "mental_model.md"
    delta_path = repo_root / delta_file

    if not mental_model_path.exists():
        print("❌ Mental model file not found")
        sys.exit(1)

    if not delta_path.exists():
        print(f"❌ Delta file {delta_file} not found")
        sys.exit(1)

    # Check MODEL_STATE
    with open(mental_model_path, 'r') as f:
        content = f.read()

    if "## MODEL_STATE" not in content:
        print("❌ MODEL_STATE not found in mental model")
        sys.exit(1)

    # Extract MODEL_STATE
    lines = content.split('\n')
    state_line = None
    for i, line in enumerate(lines):
        if line.strip() == "## MODEL_STATE":
            if i + 1 < len(lines):
                state_line = lines[i + 1].strip()
            break

    if state_line == "frozen":
        print(f"❌ Mental model is frozen (state: {state_line}). Cannot apply deltas.")
        print("Manual unfreeze required by editing MODEL_STATE to mutable")
        sys.exit(1)

    # Read deltas
    with open(delta_path, 'r') as f:
        deltas = f.read()

    print("📋 Delta content:")
    print(deltas)
    print("\n⚠️  This will modify the mental model. Continue? (y/N): ", end="")

    response = input().strip().lower()
    if response != 'y':
        print("Aborted")
        sys.exit(0)

    # Apply deltas (placeholder - would need actual delta parsing logic)
    print("✅ Deltas applied (placeholder implementation)")
    print("Remember to update MODEL_STATE back to FROZEN if needed")

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 scripts/apply_mental_model_deltas.py <delta_file>")
        sys.exit(1)

    repo_root = Path(__file__).parent.parent
    delta_file = sys.argv[1]

    apply_deltas(repo_root, delta_file)

if __name__ == "__main__":
    main()
