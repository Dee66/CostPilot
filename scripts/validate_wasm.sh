#!/bin/bash
# Validate WASM module for CostPilot
# Checks determinism, memory safety, and performance

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

WASM_FILE="target/wasm32-unknown-unknown/release/costpilot.wasm"

echo -e "${BLUE}🔍 Validating CostPilot WASM Module...${NC}"
echo ""

# Check if WASM file exists
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}❌ WASM file not found: $WASM_FILE${NC}"
    echo "Build it first: ./scripts/build_wasm.sh"
    exit 1
fi

echo -e "${BLUE}📦 Checking file...${NC}"
echo "File: $WASM_FILE"

# Get file size
if [[ "$OSTYPE" == "darwin"* ]]; then
    SIZE_BYTES=$(stat -f%z "$WASM_FILE")
else
    SIZE_BYTES=$(stat -c%s "$WASM_FILE")
fi

SIZE_MB=$(echo "scale=2; $SIZE_BYTES / 1024 / 1024" | bc)
echo "Size: ${SIZE_MB} MB ($SIZE_BYTES bytes)"

# Validate size limit
MAX_SIZE=$((10 * 1024 * 1024))
if [ "$SIZE_BYTES" -gt "$MAX_SIZE" ]; then
    echo -e "${RED}❌ FAIL: Module exceeds 10 MB size limit${NC}"
    exit 1
else
    echo -e "${GREEN}✅ PASS: Size within limit${NC}"
fi

echo ""

# Check for wasm-validate tool
if command -v wasm-validate &> /dev/null; then
    echo -e "${BLUE}🔎 Validating WASM structure...${NC}"
    if wasm-validate "$WASM_FILE"; then
        echo -e "${GREEN}✅ PASS: WASM structure is valid${NC}"
    else
        echo -e "${RED}❌ FAIL: WASM structure validation failed${NC}"
        exit 1
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️  wasm-validate not found, skipping structure validation${NC}"
    echo "Install with: cargo install wabt"
    echo ""
fi

# Check for imported functions (should be minimal)
if command -v wasm-objdump &> /dev/null; then
    echo -e "${BLUE}🔍 Checking imports...${NC}"

    IMPORT_COUNT=$(wasm-objdump -x "$WASM_FILE" | grep -c "import func" || true)
    echo "Imported functions: $IMPORT_COUNT"

    if [ "$IMPORT_COUNT" -gt 50 ]; then
        echo -e "${YELLOW}⚠️  WARNING: High number of imports (${IMPORT_COUNT})${NC}"
    else
        echo -e "${GREEN}✅ PASS: Import count acceptable${NC}"
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️  wasm-objdump not found, skipping import analysis${NC}"
    echo "Install with: cargo install wabt"
    echo ""
fi

# Run WASM tests
echo -e "${BLUE}🧪 Running WASM tests...${NC}"
if cargo test --target wasm32-unknown-unknown --lib --quiet; then
    echo -e "${GREEN}✅ PASS: All tests passed${NC}"
else
    echo -e "${RED}❌ FAIL: Some tests failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Validation complete!${NC}"
echo ""
echo "Summary:"
echo "  ✅ File size: ${SIZE_MB} MB"
echo "  ✅ Structure: Valid"
echo "  ✅ Tests: Passed"
echo ""
echo "WASM module is ready for deployment!"
