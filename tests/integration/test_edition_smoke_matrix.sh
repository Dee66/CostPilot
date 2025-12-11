#!/bin/bash
# Edition smoke matrix - integration test for Free vs Premium CLI behavior

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🧪 Edition Smoke Matrix Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Build free-mode binary
echo "📦 Building free-mode binary..."
cargo build --release

BINARY="target/release/costpilot"

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}❌ Binary not found at $BINARY${NC}"
    exit 1
fi

# Create sample Terraform plan
SAMPLE_PLAN="test/fixtures/sample.tfplan.json"

if [ ! -f "$SAMPLE_PLAN" ]; then
    echo "⚠️  Sample plan not found, creating minimal plan..."
    mkdir -p test/fixtures
    cat > "$SAMPLE_PLAN" << 'EOF'
{
  "format_version": "1.0",
  "terraform_version": "1.5.0",
  "resource_changes": [
    {
      "address": "aws_instance.example",
      "type": "aws_instance",
      "change": {
        "actions": ["create"],
        "before": null,
        "after": {
          "instance_type": "t3.micro"
        }
      }
    }
  ]
}
EOF
fi

echo ""
echo "🔍 Test 1: Scan (should succeed in free mode)"
if $BINARY scan --plan "$SAMPLE_PLAN" > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓ Scan succeeded${NC}"
else
    echo -e "  ${RED}✗ Scan failed unexpectedly${NC}"
    exit 1
fi

echo ""
echo "🔧 Test 2: Autofix (should fail with upgrade message)"
if $BINARY autofix --plan "$SAMPLE_PLAN" 2>&1 | grep -q "Premium"; then
    echo -e "  ${GREEN}✓ Autofix blocked with upgrade message${NC}"
else
    echo -e "  ${YELLOW}⚠ Autofix behavior unexpected${NC}"
fi

echo ""
echo "📈 Test 3: Trend (should fail with upgrade message)"
if $BINARY trend --plan "$SAMPLE_PLAN" 2>&1 | grep -q "Premium"; then
    echo -e "  ${GREEN}✓ Trend blocked with upgrade message${NC}"
else
    echo -e "  ${YELLOW}⚠ Trend behavior unexpected${NC}"
fi

echo ""
echo "🗺️  Test 4: Map depth=1 (should succeed)"
if $BINARY map --plan "$SAMPLE_PLAN" > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓ Map depth=1 succeeded${NC}"
else
    echo -e "  ${YELLOW}⚠ Map depth=1 failed${NC}"
fi

echo ""
echo "🗺️  Test 5: Map depth=3 (should fail with upgrade message)"
if $BINARY map --plan "$SAMPLE_PLAN" --depth 3 2>&1 | grep -q "Premium"; then
    echo -e "  ${GREEN}✓ Deep map blocked with upgrade message${NC}"
else
    echo -e "  ${YELLOW}⚠ Deep map behavior unexpected${NC}"
fi

echo ""
echo -e "${GREEN}✅ Edition smoke matrix complete${NC}"
echo ""
echo "Note: Premium mode tests require license + WASM harness"
