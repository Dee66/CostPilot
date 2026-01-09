#!/bin/bash
# Test the complete license issuance and verification flow

set -e

echo "========================================================================"
echo "CostPilot License Flow Test"
echo "========================================================================"

# Step 1: Extract and rebuild with master key
echo ""
echo "Step 1: Rebuilding with master public key..."
bash scripts/rebuild_with_master_key.sh

# Step 2: Install the license
echo ""
echo "Step 2: Installing test license..."
mkdir -p ~/.costpilot
cp license_customer.json ~/.costpilot/license.json
echo "✅ License installed to ~/.costpilot/license.json"

# Step 3: Test scan with premium license
echo ""
echo "Step 3: Testing scan with premium license..."
echo "========================================================================"
./target/release/costpilot scan test_comprehensive.json 2>&1 | head -20

echo ""
echo "========================================================================"
echo "✅ License flow test complete!"
echo ""
echo "Check output above for:"
echo "  - No 'License signature verification failed' error"
echo "  - Premium mode should be active"
