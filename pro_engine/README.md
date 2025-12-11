# ProEngine WASM Bundle

This directory contains the WebAssembly interface contract for CostPilot's Premium edition.

## Distribution Policy

**The actual `pro_engine.wasm` binary is NOT included in this repository.**

The compiled WASM module is:
- Distributed only to licensed Premium customers
- Encrypted with customer-specific keys
- Signed with Ed25519 signatures
- Validated on each load

## Interface Contract

The `pro_engine.wit` file defines the stable interface between CostPilot and the Premium engine.

All functions use JSON-serialized string input/output for cross-language compatibility.

## For Premium Customers

After purchase, you will receive:
1. `pro_engine.wasm.enc` - Encrypted WASM binary
2. `license.json` - License file with signature
3. Installation instructions

Place these files in your `~/.costpilot/` directory as directed.
