# License Issuance Operations Guide

## Overview

CostPilot uses Ed25519 cryptographic signatures for license verification. This guide covers the complete operational workflow for issuing customer licenses.

## Prerequisites

1. Master keypair generated: `costpilot_master.pem` and `costpilot_master.pub.pem`
2. Binary built with embedded master public key
3. Python 3 installed for issuer script

## Initial Setup (One-Time)

### 1. Generate Master Keypair

If `costpilot_master.pem` doesn't exist:

```bash
./target/release/license-issuer generate-key costpilot_master
```

**⚠️ CRITICAL**: Store `costpilot_master.pem` securely. This private key is required to issue all licenses. Loss = inability to issue new licenses. Compromise = anyone can issue fraudulent licenses.

Recommended: Store in password manager + encrypted backup.

### 2. Build Binary with Master Key

Every time you rebuild CostPilot for distribution:

```bash
bash scripts/rebuild_with_master_key.sh
```

This embeds the master public key into the binary for license verification.

## License Issuance (Per Customer)

### Interactive Method

```bash
python3 scripts/issue_license.py
```

Follow prompts:
1. Customer email
2. License key (auto-generated or custom)
3. Expiration (30 days, 365 days, or custom)
4. Output filename

### Example: Issue Annual License

```bash
python3 scripts/issue_license.py
```

Input:
```
Customer email: customer@company.com
License key: [press Enter for auto-generated]
Expiration: 2  [365 days]
Output filename: [press Enter for default]
Generate this license? yes
```

Output: `license_customer.json`

## License Delivery

Send `license_customer.json` to customer with instructions:

```
Installation:
1. Create directory: mkdir -p ~/.costpilot
2. Save license: mv license_customer.json ~/.costpilot/license.json
3. Verify: costpilot scan <your-plan.json>

You should see premium features enabled with no license errors.
```

## Verification

Test a generated license before sending:

```bash
# Install license
mkdir -p ~/.costpilot
cp license_customer.json ~/.costpilot/license.json

# Test scan
./target/release/costpilot scan test_comprehensive.json
```

Expected output: No "License signature verification failed" error.

## License Format

```json
{
  "email": "customer@example.com",
  "license_key": "ABC123XYZ789",
  "expires": "2026-12-31T23:59:59Z",
  "issued_at": "2026-01-07T12:00:00Z",
  "signature": "hexstring...",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

**Critical Fields:**
- `issuer`: Must be "costpilot-v1" (production) or "test-costpilot" (testing)
- `signature`: Ed25519 signature of canonical message: `{email}|{license_key}|{expires}|{issuer}`

## Troubleshooting

### "License signature verification failed"

**Cause**: Binary's embedded public key doesn't match master key used for signing.

**Fix**: Rebuild binary with `bash scripts/rebuild_with_master_key.sh`

### "Invalid license file: Missing required field"

**Cause**: Malformed JSON or missing field.

**Fix**: Regenerate license with `python3 scripts/issue_license.py`

### Master key lost

**No recovery possible.** All existing licenses become invalid. Must:
1. Generate new master keypair
2. Rebuild all binaries
3. Reissue all customer licenses

## Security Notes

1. **Private key security**: `costpilot_master.pem` is the root of trust
2. **Key rotation**: To rotate keys, add new issuer in `src/pro_engine/crypto.rs` and rebuild
3. **License expiration**: Enforced at runtime, no grace period
4. **Offline validation**: All verification happens locally, no phone-home

## Testing

Test suite includes end-to-end license issuance and validation:

```bash
cargo test license --release
cargo test --test license_issuer_integration_tests
```

All tests use issuer="test-costpilot" with hardcoded test keys for reproducibility.
