# CostPilot License Contract Specification

**Date:** 2026-01-10
**Version:** 1.0.0
**Purpose:** Define the integration contract between the external license issuer (AWS Lambda) and CostPilot consumer

---

## Integration Architecture

```
┌─────────────────────┐         ┌──────────────────────┐
│  License Issuer     │         │   CostPilot          │
│  (AWS Lambda)       │ ───────▶│   (Consumer)         │
│  Separate Repo      │  JSON   │   This Repo          │
└─────────────────────┘         └──────────────────────┘
        │                                  │
        │ Produces                         │ Consumes
        ▼                                  ▼
   license.json                      ~/.costpilot/license.json
```

**Critical Fact:** The license payload (JSON structure) is the ONLY contract between the two systems.

---

## License Payload Contract

### JSON Structure

**File Location (Consumer):** `~/.costpilot/license.json`

**Required Fields:**

```json
{
  "email": "customer@example.com",
  "license_key": "CP-PREMIUM-XXXX-XXXX-XXXX",
  "expires": "2026-12-31T23:59:59Z",
  "signature": "<base64-encoded-ed25519-signature>",
  "issuer": "costpilot-license-v1"
}
```

### Field Specifications

#### `email` (String, Required)
- **Type:** String
- **Constraints:** Non-empty, valid email format expected but not enforced
- **Purpose:** Customer identification
- **Example:** `"user@company.com"`

#### `license_key` (String, Required)
- **Type:** String
- **Constraints:** Non-empty
- **Format:** Arbitrary string (commonly `CP-PREMIUM-*`)
- **Purpose:** Human-readable license identifier
- **Example:** `"CP-PREMIUM-2024-ABC123-XYZ"`

#### `expires` (String, Required)
- **Type:** RFC 3339 / ISO 8601 datetime string
- **Constraints:** Must parse as valid datetime
- **Timezone:** Must include timezone (typically UTC with `Z` suffix)
- **Validation:** Compared against current system time
- **Example:** `"2026-12-31T23:59:59Z"`
- **Expired Behavior:** License rejected if `expires < now()`

#### `signature` (String, Required)
- **Type:** Base64-encoded Ed25519 signature
- **Algorithm:** Ed25519 (Curve25519 + SHA-512)
- **Signing Input:** Canonical JSON representation of license fields (excluding signature itself)
- **Public Key:** Embedded in CostPilot binary (fingerprint: `db52fc95...`)
- **Verification:** Signature must verify against embedded public key
- **Failure Mode:** License rejected if signature invalid

#### `issuer` (String, Required)
- **Type:** String
- **Constraints:** Non-empty
- **Expected Value:** `"costpilot-license-v1"` (or similar versioned identifier)
- **Purpose:** Version tracking and issuer identification
- **Validation:** Currently informational, not enforced

---

## Signature Algorithm

### Signing Process (Issuer Side)

**Input:** License fields (excluding `signature`)

**Canonical Form:**
```json
{"email":"user@example.com","expires":"2026-12-31T23:59:59Z","issuer":"costpilot-license-v1","license_key":"CP-PREMIUM-2024-ABC"}
```

**Steps:**
1. Sort JSON keys alphabetically
2. Serialize without whitespace
3. Sign with Ed25519 private key
4. Base64-encode signature
5. Add signature to license payload

**Key Pair:**
- **Private Key:** Held by issuer (AWS Lambda Secrets Manager)
- **Public Key:** Embedded in CostPilot binary at compile time

### Verification Process (Consumer Side)

**Location:** `src/pro_engine/license.rs:218-245`

```rust
pub fn validate(&self) -> Result<(), String> {
    // 1. Rate limiting check
    // 2. Field presence validation
    // 3. Expiry check
    // 4. Signature verification with embedded public key
}
```

**Public Key Location:** `src/keys.rs` (compiled into binary)

**Fingerprint:** `db52fc95` (first 8 hex chars)

---

## Validation Workflow

### Consumer Side (CostPilot)

```
1. License Discovery
   └─▶ Check: ~/.costpilot/license.json exists?
       ├─ No  → Free Edition (no error)
       └─ Yes → Proceed to validation

2. Load & Parse
   └─▶ Parse JSON
       ├─ Success → Proceed
       └─ Failure → Reject (invalid JSON)

3. Field Validation
   └─▶ Check: All required fields present and non-empty?
       ├─ Yes → Proceed
       └─ No  → Reject (missing field)

4. Expiry Check
   └─▶ Parse expires as RFC3339
       ├─ Valid & Future → Proceed
       └─ Invalid/Past  → Reject (expired)

5. Rate Limiting
   └─▶ Check: Not rate-limited?
       ├─ Yes → Proceed
       └─ No  → Reject (rate limit exceeded)

6. Signature Verification
   └─▶ Verify Ed25519 signature
       ├─ Valid   → Premium Edition Activated ✅
       └─ Invalid → Reject (signature mismatch)
```

**Failure Mode:** Any rejection → CostPilot runs in Free Edition

---

## Rate Limiting

**File:** `~/.costpilot/rate_limit.json`

**Purpose:** Prevent brute-force signature validation attacks

**Limits:**
- **Max Attempts:** 5 per minute
- **Block Duration:** 5 minutes after exceeding limit
- **Integrity:** HMAC-protected to prevent tampering

**Impact on Integration:**
- Legitimate licenses: No impact (valid on first attempt)
- Invalid licenses: Blocked after 5 failed attempts

---

## Error Handling

### Issuer Responsibilities

1. **Generate Valid Signatures:** Use correct Ed25519 private key
2. **Set Future Expiry:** Ensure `expires` is RFC3339 and in the future
3. **Include All Fields:** `email`, `license_key`, `expires`, `signature`, `issuer`
4. **Use Correct Public Key:** Match the key embedded in CostPilot binary

### Consumer Behavior

**Silent Failures:**
- No license file → Free Edition (no warning)
- Invalid license (debug mode off) → Free Edition (no warning)

**Warnings (Debug Mode):**
- Set `COSTPILOT_DEBUG=1` for verbose license validation errors

**Hard Failures:**
- Rate limit exceeded → Temporary rejection (5-minute cooldown)

---

## Testing Contract Compliance

### Issuer Testing (External Repository)

**Required Tests:**
1. Generate license with valid signature
2. Verify signature with matching public key
3. Test expiry date parsing (RFC3339 compliance)
4. Ensure all required fields present

### Consumer Testing (This Repository)

**Script:** `scripts/e2e_license_validation.sh`

**Tests:**
1. **Free Edition Detection:** No license → Free mode
2. **Premium Detection:** Valid license → Premium mode
3. **Expired License:** Past `expires` → Free mode
4. **Invalid Signature:** Tampered signature → Free mode
5. **Missing Fields:** Incomplete JSON → Free mode

**Test Data:** Real licenses generated by issuer (NOT mocks)

---

## Integration Points

### Issuer → Consumer

**Delivery Method:** Email (manual)
1. Customer purchases via website
2. Issuer generates license.json
3. License emailed to customer
4. Customer places license in `~/.costpilot/license.json`

**No API Integration:** CostPilot does NOT call issuer at runtime

### Public Key Rotation

**Current Key:** `db52fc95...` (embedded at compile time)

**Rotation Process:**
1. Generate new Ed25519 keypair (issuer side)
2. Update `src/keys.rs` with new public key
3. Rebuild CostPilot binary
4. Deploy new issuer with new private key
5. Customers using old licenses must upgrade binary

**Frequency:** As needed for security (not routine)

---

## Contract Versioning

**Current Version:** `1.0.0`

**Version Identifier:** `issuer` field (e.g., `"costpilot-license-v1"`)

**Backward Compatibility:**
- Add new optional fields freely
- Never remove required fields
- Signature algorithm changes require new version

---

## Failure Modes

### Issuer-Side Failures

| Failure | Impact | Detection |
|---------|--------|-----------|
| Wrong private key | Signature invalid | Consumer rejects license |
| Expired date | License unusable | Consumer rejects as expired |
| Missing field | Parsing error | Consumer rejects as invalid |
| Invalid JSON | Parse failure | Consumer rejects immediately |

### Consumer-Side Failures

| Failure | Impact | Detection |
|---------|--------|-----------|
| Missing public key | All licenses invalid | Binary compile error |
| Wrong public key | All licenses invalid | All licenses rejected |
| Rate limit reached | Temporary block | License validation fails temporarily |

---

## Security Properties

### Guarantees

✅ **License Authenticity:** Ed25519 signature ensures license issued by holder of private key

✅ **Tamper Resistance:** Any modification invalidates signature

✅ **Expiry Enforcement:** Time-based license validity

✅ **Rate Limiting:** Brute-force protection via HMAC-protected state

### Non-Guarantees

❌ **License Sharing Prevention:** License files are copyable (by design for offline use)

❌ **Revocation:** No real-time revocation (licenses valid until expiry)

❌ **Network Validation:** No phone-home or online activation

---

## Example License Payloads

### Valid License

```json
{
  "email": "customer@acme.com",
  "license_key": "CP-PREMIUM-2024-ACME-001",
  "expires": "2026-12-31T23:59:59Z",
  "signature": "vQpL8j3KxN2mR7tYwZ4cF6hJ9sP1qX8vU5aT3nM7lK0=",
  "issuer": "costpilot-license-v1"
}
```

### Expired License

```json
{
  "email": "old-customer@example.com",
  "license_key": "CP-PREMIUM-2023-OLD",
  "expires": "2024-01-01T00:00:00Z",
  "signature": "aB3dE5fG7hI9jK1lM2nO4pQ6rS8tU0vW2xY4zA6bC8=",
  "issuer": "costpilot-license-v1"
}
```

**Consumer Behavior:** Detects expiry, runs in Free Edition

### Invalid Signature

```json
{
  "email": "hacker@badactor.com",
  "license_key": "CP-PREMIUM-FAKE",
  "expires": "2099-12-31T23:59:59Z",
  "signature": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  "issuer": "costpilot-license-v1"
}
```

**Consumer Behavior:** Signature verification fails, runs in Free Edition

---

## Compliance Checklist

**For Issuer (External Repository):**
- [ ] Generate Ed25519 signatures with correct private key
- [ ] Use RFC3339 datetime format for `expires`
- [ ] Include all 5 required fields
- [ ] Base64-encode signature
- [ ] Test with CostPilot binary before customer delivery

**For Consumer (This Repository):**
- [ ] Embed correct public key at compile time
- [ ] Validate all required fields present
- [ ] Parse and check expiry
- [ ] Verify Ed25519 signature
- [ ] Fail gracefully to Free Edition on any error

---

## Troubleshooting

### License Not Recognized

**Checklist:**
1. File at correct path? (`~/.costpilot/license.json`)
2. Valid JSON syntax?
3. All required fields present?
4. `expires` in future and RFC3339 format?
5. Signature generated with correct private key?
6. Public key matches issuer's keypair?

**Debug Mode:**
```bash
COSTPILOT_DEBUG=1 costpilot edition
```

### Rate Limit Exceeded

**Solution:** Wait 5 minutes or delete `~/.costpilot/rate_limit.json`

**Prevention:** Don't repeatedly validate invalid licenses

---

## References

**Consumer Implementation:**
- `src/pro_engine/license.rs` - License validation logic
- `src/edition/mod.rs` - Edition detection and activation
- `src/keys.rs` - Embedded public keys

**Issuer Implementation:**
- External repository (not in this repo)
- AWS Lambda deployment
- Email delivery system

---

**Contract Status:** STABLE (v1.0.0)
**Last Updated:** 2026-01-10
**Breaking Changes:** None planned
