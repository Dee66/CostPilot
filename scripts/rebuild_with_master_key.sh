#!/bin/bash
# Rebuild CostPilot with embedded master public key

set -e

MASTER_KEY="costpilot_master.pub.pem"

if [ ! -f "$MASTER_KEY" ]; then
    echo "❌ Error: Master public key not found: $MASTER_KEY"
    echo "Generate it first with: ./target/release/license-issuer generate-key costpilot_master"
    exit 1
fi

echo "📝 Extracting public key hex from $MASTER_KEY..."
PUBKEY_HEX=$(python3 scripts/extract_pubkey_hex.py "$MASTER_KEY")

echo "🔑 Public key hex: $PUBKEY_HEX"
echo ""
echo "🔨 Rebuilding CostPilot with embedded master key..."

COSTPILOT_LICENSE_PUBKEY="$PUBKEY_HEX" cargo build --release --bin costpilot

echo ""
echo "✅ Build complete!"
echo "The binary now uses the master key for license verification."
echo ""
echo "Test with:"
echo "  ./target/release/costpilot scan test_comprehensive.json"
