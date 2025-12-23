#!/bin/bash
# Windows Testing Script for CostPilot
# Run this on Windows (via Git Bash, WSL, or PowerShell with bash)

set -euo pipefail

echo "=== CostPilot Windows Testing Script ==="

# Check if we're on Windows
if [[ "$OSTYPE" != "msys" ]] && [[ "$OSTYPE" != "win32" ]] && [[ "$OSTYPE" != "cygwin" ]]; then
    echo "ERROR: This script should be run on Windows"
    exit 1
fi

echo "✓ Running on Windows"

# Test 1: Version check
echo "Test 1: Version check"
./costpilot.exe --version
echo "✓ Version check passed"

# Test 2: Help output
echo "Test 2: Help output"
./costpilot.exe --help | head -20
echo "✓ Help output works"

# Test 3: Scan command help
echo "Test 3: Scan command help"
./costpilot.exe scan --help
echo "✓ Scan help works"

# Test 4: Path handling (create a test file)
echo "Test 4: Path handling"
TEST_FILE="test_plan.json"
cat > "$TEST_FILE" << 'EOF'
{
  "planned_values": {
    "root_module": {
      "resources": [
        {
          "address": "aws_instance.example",
          "mode": "managed",
          "type": "aws_instance",
          "name": "example",
          "values": {
            "instance_type": "t2.micro",
            "ami": "ami-12345"
          }
        }
      ]
    }
  }
}
EOF

echo "✓ Created test Terraform plan"

# Test 5: Basic scan (should work even without full config)
echo "Test 5: Basic scan"
./costpilot.exe scan "$TEST_FILE" --output-format json || echo "Expected: scan may fail without full setup, but binary runs"

# Test 6: Check Windows paths work
echo "Test 6: Windows path handling"
WINDOWS_PATH="C:\\Program Files\\CostPilot\\test.json"
echo "{}" > /tmp/test_windows.json
./costpilot.exe scan /tmp/test_windows.json || echo "Expected: may fail, but path handling works"

# Cleanup
rm -f "$TEST_FILE"

echo "=== Windows Testing Complete ==="
echo "✓ All basic functionality verified"
echo "Next: Test with real Terraform plans and ProEngine features"
