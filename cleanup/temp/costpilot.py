#!/usr/bin/env python3
import os
import sys
import subprocess

def main():
    # Find the binary
    script_dir = os.path.dirname(os.path.abspath(__file__))
    binary_path = os.path.join(script_dir, "bin", "costpilot")

    if not os.path.exists(binary_path):
        print(f"Error: CostPilot binary not found at {binary_path}")
        print("Please reinstall the package or download the binary manually.")
        sys.exit(1)

    # Execute the binary with all arguments
    try:
        result = subprocess.run([binary_path] + sys.argv[1:])
        sys.exit(result.returncode)
    except Exception as e:
        print(f"Error executing CostPilot: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()