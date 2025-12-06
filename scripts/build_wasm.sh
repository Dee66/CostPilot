#!/bin/bash
# Build CostPilot for WebAssembly
# Usage: ./scripts/build_wasm.sh [--optimize]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

OPTIMIZE=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --optimize)
            OPTIMIZE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--optimize]"
            echo ""
            echo "Options:"
            echo "  --optimize    Run wasm-opt to reduce binary size"
            echo "  --help        Show this help message"
            exit 0
            ;;
    esac
done

echo -e "${BLUE}🔧 Building CostPilot for WebAssembly...${NC}"
echo ""

# Check prerequisites
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${YELLOW}📦 Installing wasm32-unknown-unknown target...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# Clean previous builds
echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
cargo clean --target wasm32-unknown-unknown

# Build WASM module
echo -e "${BLUE}🔨 Building WASM module...${NC}"
cargo build --target wasm32-unknown-unknown --release --lib --profile wasm-release

WASM_FILE="target/wasm32-unknown-unknown/release/costpilot.wasm"

# Check if build succeeded
if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}❌ Build failed - WASM file not found${NC}"
    exit 1
fi

# Get file size
if [[ "$OSTYPE" == "darwin"* ]]; then
    SIZE_BYTES=$(stat -f%z "$WASM_FILE")
else
    SIZE_BYTES=$(stat -c%s "$WASM_FILE")
fi

SIZE_MB=$(echo "scale=2; $SIZE_BYTES / 1024 / 1024" | bc)

echo -e "${GREEN}✅ WASM build successful!${NC}"
echo -e "${BLUE}📊 Module size: ${SIZE_MB} MB ($SIZE_BYTES bytes)${NC}"

# Validate size limit (10 MB unoptimized)
MAX_SIZE=$((10 * 1024 * 1024))
if [ "$SIZE_BYTES" -gt "$MAX_SIZE" ]; then
    echo -e "${RED}❌ ERROR: WASM module exceeds 10 MB size limit${NC}"
    echo -e "${YELLOW}💡 Try running with --optimize flag${NC}"
    exit 1
fi

# Optimize if requested
if [ "$OPTIMIZE" = true ]; then
    if ! command -v wasm-opt &> /dev/null; then
        echo -e "${YELLOW}⚠️  wasm-opt not found, skipping optimization${NC}"
        echo "Install with: cargo install wasm-opt"
    else
        echo -e "${BLUE}⚡ Optimizing WASM module...${NC}"
        
        OPT_FILE="target/wasm32-unknown-unknown/release/costpilot_opt.wasm"
        wasm-opt -Oz -o "$OPT_FILE" "$WASM_FILE"
        
        if [[ "$OSTYPE" == "darwin"* ]]; then
            OPT_SIZE_BYTES=$(stat -f%z "$OPT_FILE")
        else
            OPT_SIZE_BYTES=$(stat -c%s "$OPT_FILE")
        fi
        
        OPT_SIZE_MB=$(echo "scale=2; $OPT_SIZE_BYTES / 1024 / 1024" | bc)
        REDUCTION=$(echo "scale=1; 100 * ($SIZE_BYTES - $OPT_SIZE_BYTES) / $SIZE_BYTES" | bc)
        
        echo -e "${GREEN}✅ Optimization complete!${NC}"
        echo -e "${BLUE}📊 Optimized size: ${OPT_SIZE_MB} MB ($OPT_SIZE_BYTES bytes)${NC}"
        echo -e "${GREEN}📉 Size reduction: ${REDUCTION}%${NC}"
        
        # Replace original with optimized
        mv "$OPT_FILE" "$WASM_FILE"
    fi
fi

# Generate JS bindings if wasm-bindgen is available
if command -v wasm-bindgen &> /dev/null; then
    echo -e "${BLUE}🔗 Generating JavaScript bindings...${NC}"
    
    mkdir -p pkg
    wasm-bindgen "$WASM_FILE" \
        --out-dir pkg \
        --target web \
        --no-typescript
    
    echo -e "${GREEN}✅ JavaScript bindings generated in pkg/${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Build complete!${NC}"
echo ""
echo "WASM module: $WASM_FILE"
echo "Size: ${SIZE_MB} MB"
echo ""
echo "Next steps:"
echo "  • Run tests: cargo test --target wasm32-unknown-unknown"
echo "  • Validate: ./scripts/validate_wasm.sh"
echo "  • Deploy: Copy $WASM_FILE to your runtime"
