# CostPilot License Validation Contract

**Version:** 1.0.0
**Date:** 2026-01-10
**Status:** ✅ Stable and Test-Backed

---

## Consumer Cryptographic Contract

This document defines the exact contract that CostPilot enforces when validating licenses. This is the **consumer-side specification** only.

### Embedded Public Keys

**Location:** Generated at compile time in `build.rs` (lines 242-245)

**License Validation Public Key:**
```
Hex: db52fc95fe7ccbd5e55ecfd357d8271d1b2d4a9f608e68db3e7f869d54dba5df
Fingerprint: db52fc95fe7ccbd5 (first 8 bytes)
SHA256: c4dc407deb10e489 (first 16 chars)
Length: 32 bytes (Ed25519)
```

**Key Rotation History:**
- **2026-01-08:** Current keys activated (fingerprint `db52fc95`)
- **Pre-2026-01-08:** OLD keys revoked (fingerprints `23837ac5`, `10f8798e`)

**Test Public Key:**
```
Hex: 197f6b23e16c8532c6abc838facd5ea789be0c76b29203340396fa8b3d368d61
Seed: [42u8; 32] (ed25519-dalek)
Issuer: "test-costpilot"
```

---

## License JSON Structure

### Required Fields

**Location:** `src/pro_engine/license.rs` (lines 155-161)

```json
{
  "email": "string (non-empty)",
  "license_key": "string (non-empty)",
  "expires": "RFC3339 datetime",
  "signature": "hex-encoded Ed25519 signature (128 chars)",
  "issuer": "string (non-empty)"
}
```

**All 5 fields are REQUIRED.** Missing or empty fields result in validation failure.

### Optional Fields

The consumer **ignores** extra fields. These are safe to include:
- `issued_at` - Issuer timestamp (informational)
- `version` - License format version (informational)
- Any other custom metadata

---

## Signature Verification Algorithm

### Canonical Message Format

**Location:** `src/pro_engine/crypto.rs` (lines 166-169)

**Format:** Pipe-delimited concatenation
```
{email}|{license_key}|{expires}|{issuer}
```

**Example:**
```
customer@example.com|PREMIUM-12345|2026-12-31T23:59:59Z|costpilot-v1
```

**Critical:** No spaces, no quotes, exact pipe-delimited format.

### Signature Algorithm

- **Algorithm:** Ed25519 (RFC 8032)
- **Library:** `ring` crate (`signature::ED25519`)
- **Encoding:** Hex string (128 characters for 64-byte signature)
- **Decoding:** `hex::decode()` before verification

### Verification Process

**Location:** `src/pro_engine/crypto.rs` (lines 163-179)

```rust
// 1. Construct canonical message
let message = format!("{}|{}|{}|{}", email, license_key, expires, issuer);

// 2. Decode hex signature
let sig_bytes = hex::decode(signature)?;

// 3. Select public key by issuer
let public_key_bytes = match issuer {
    "costpilot-v1" => LICENSE_PUBLIC_KEY,
    "test-costpilot" => TEST_LICENSE_PUBLIC_KEY,
    _ => return Err("Unknown license issuer"),
};

// 4. Verify Ed25519 signature
ring::signature::UnparsedPublicKey::new(&ED25519, public_key_bytes)
    .verify(message.as_bytes(), &sig_bytes)
```

---

## Duration Validation

### Expiry Enforcement

**Location:** `src/pro_engine/license.rs` (lines 206-212)

**Rule:** License is valid if `expires > now()` (strict inequality)

**Implementation:**
```rust
pub fn is_expired(&self) -> bool {
    match chrono::DateTime::parse_from_rfc3339(&self.expires) {
        Ok(expiry) => expiry < chrono::Utc::now(),
        Err(_) => true, // Invalid date = expired
    }
}
```

### Date Format

**Format:** RFC 3339 (ISO 8601)
**Parser:** `chrono::DateTime::parse_from_rfc3339()`

**Valid Examples:**
```
2026-12-31T23:59:59Z                (UTC with Z)
2026-12-31T23:59:59+00:00           (UTC with offset)
2026-12-31T23:59:59-08:00           (PST)
2026-12-31T23:59:59.123456789Z      (Subsecond precision OK)
```

**Invalid Examples:**
```
2026-12-31                          (Missing time)
2026-13-01T00:00:00Z                (Invalid month)
not-a-date                           (Garbage)
```

**Invalid dates are treated as expired.**

### Supported Durations

**Consumer does NOT enforce specific durations.** Any future date is valid.

**Test-Backed Durations:**
- ✅ 1 day
- ✅ 7 days
- ✅ 29 days
- ✅ **30 days (monthly)**
- ✅ 31 days
- ✅ 90 days
- ✅ 364 days
- ✅ **365 days (yearly)**
- ✅ 366 days
- ✅ 730 days (2 years)
- ✅ 1825 days (5 years)

**Duration is issuer-defined.** The consumer only checks `expires` against current time.

### Boundary Conditions

**Test-Backed Boundaries:**
- `expires = now - 1 second` → ❌ Expired
- `expires = now + 1 second` → ✅ Valid
- `expires = now - 1 hour` → ❌ Expired
- `expires = now + 1 hour` → ✅ Valid
- `expires = now - 1 day` → ❌ Expired
- `expires = epoch (1970-01-01)` → ❌ Expired
- `expires = 2099-12-31` → ✅ Valid

---

## Validation Pipeline

### Full Validation Sequence

**Location:** `src/pro_engine/license.rs` (lines 215-252)

```
1. Rate Limit Check
   └─▶ If blocked: Reject ("Rate limit exceeded")

2. Field Presence Check
   └─▶ If any field empty: Reject ("Field is empty")

3. Expiry Check
   └─▶ If expired: Reject ("License expired")

4. Signature Verification
   └─▶ If invalid: Reject ("Signature verification failed")

5. Success
   └─▶ Return Ok(())
```

### Rate Limiting

**File:** `~/.costpilot/rate_limit.json`
**Limits:**
- 5 attempts per minute
- 5-minute block after exceeding limit
- HMAC integrity protection (SHA256)

**Purpose:** Prevent brute-force signature attacks

---

## Edition Detection

### Detection Logic

**Location:** `src/edition/mod.rs` (lines 20-60)

```
1. Check: ~/.costpilot/license.json exists?
   ├─ No  → Free Edition (no error)
   └─ Yes → Proceed

2. Load license from file
   └─▶ If parse error: Free Edition (silent unless DEBUG)

3. Validate license
   ├─ Success → Premium Edition
   └─ Failure → Free Edition (silent unless DEBUG)

4. Load ProEngine WASM (Premium only)
```

### Error Handling

**Default Behavior (COSTPILOT_DEBUG=0):**
- Missing license → Silent Free Edition
- Invalid license → Silent Free Edition
- Expired license → Silent Free Edition
- Signature failure → Silent Free Edition

**Debug Mode (COSTPILOT_DEBUG=1):**
- Prints validation errors to stderr
- Still returns Free Edition (does not panic)

---

## Test Coverage

### Duration Tests

**File:** `tests/license_duration_tests.rs`
**Coverage:** 25 tests

- ✅ Monthly licenses (30 days)
- ✅ Yearly licenses (365 days)
- ✅ Boundary conditions (±1 second, ±1 hour, ±1 day)
- ✅ Arbitrary durations (7, 90, 730, 1825 days)
- ✅ Invalid date formats
- ✅ Timezone handling (UTC, +offset, -offset)

**All tests pass.**

### Real License E2E Tests

**File:** `tests/license_e2e_real_tests.rs`
**Coverage:** 12 tests

- ✅ No license file → Free edition
- ✅ Valid 30-day license → Premium edition
- ✅ Valid 365-day license → Premium edition
- ✅ Expired license → Free edition
- ✅ Invalid signature → Free edition
- ✅ Tampered data → Free edition
- ✅ Unknown issuer → Free edition
- ✅ Silent failure without DEBUG
- ✅ Boundary: expires in 1 second → Valid
- ✅ Boundary: expired 1 second ago → Invalid
- ✅ Test keypair verification
- ✅ Canonical message format verification

**All tests pass.**

---

## Security Properties

### Guarantees

✅ **Signature Authenticity:** Ed25519 ensures license issued by holder of private key
✅ **Tamper Resistance:** Any modification invalidates signature
✅ **Expiry Enforcement:** Time-based validity checked on every validation
✅ **Rate Limiting:** Brute-force protection (5 attempts/min)

### Non-Guarantees

❌ **Revocation:** No real-time revocation (valid until expiry)
❌ **License Sharing Prevention:** Files are copyable (offline-first design)
❌ **Network Validation:** No phone-home or online checks

---

## Issuer Integration Requirements

**To issue compatible licenses, the issuer must:**

1. **Use correct signing key**
   - Private key matching consumer's embedded public key
   - For production: `LICENSE_PUBLIC_KEY` (fingerprint `db52fc95`)
   - For testing: `TEST_LICENSE_PUBLIC_KEY` (seed `[42u8; 32]`)

2. **Generate canonical message**
   - Format: `{email}|{license_key}|{expires}|{issuer}`
   - No spaces, no quotes, exact pipe separation

3. **Sign with Ed25519**
   - Algorithm: Ed25519 (RFC 8032)
   - Encode signature as hex (128 characters)

4. **Use RFC3339 datetime**
   - Format: `2026-12-31T23:59:59Z`
   - Must be parseable by `chrono::DateTime::parse_from_rfc3339()`

5. **Set issuer to recognized value**
   - Production: `"costpilot-v1"`
   - Testing: `"test-costpilot"`

6. **Include all required fields**
   - `email`, `license_key`, `expires`, `signature`, `issuer`

---

## File Locations

**Consumer Reads:**
- `~/.costpilot/license.json` (primary)
- `./.costpilot/license.json` (fallback, not tested)

**Consumer Writes:**
- `~/.costpilot/rate_limit.json` (rate limiting state)

**Consumer Does NOT Write:**
- Never modifies `license.json`
- Never creates license files

---

## Change Policy

**This contract is STABLE.**

**Breaking Changes:**
- Require major version bump
- Advance notice to license holders
- Backward compatibility period

**Non-Breaking Changes:**
- Adding optional JSON fields (ignored by consumer)
- Adding new issuer names to whitelist
- Key rotation (maintains old keys temporarily)

---

## Verification Commands

**Run Duration Tests:**
```bash
cargo test --test license_duration_tests
```

**Run E2E Tests:**
```bash
cargo test --test license_e2e_real_tests
```

**Check Embedded Keys:**
```bash
cargo build 2>&1 | grep "key fingerprint"
```

**Expected Output:**
```
License key fingerprint: db52fc95
WASM key fingerprint: 8db250f6
```

---

## References

**Implementation Files:**
- `build.rs` (lines 235-302) - Key generation
- `src/pro_engine/crypto.rs` (lines 163-201) - Signature verification
- `src/pro_engine/license.rs` (lines 155-261) - License structure and validation
- `src/edition/mod.rs` (lines 1-244) - Edition detection

**Test Files:**
- `tests/license_duration_tests.rs` - Duration validation
- `tests/license_e2e_real_tests.rs` - End-to-end integration

---

**Contract Status:** ✅ **STABLE AND TEST-BACKED**
**Monthly Licenses:** ✅ Supported (30 days tested)
**Yearly Licenses:** ✅ Supported (365 days tested)
**Breaking Changes:** None planned
**Last Verified:** 2026-01-10
