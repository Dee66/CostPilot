# CostPilot License Issuer - Summary

## Overview

The `costpilot-license-issuer` is now a **standalone library** that can be used in other projects (like an API server) to issue CostPilot licenses.

---

## 1. License Format

### JSON Structure

```json
{
  "email": "customer@example.com",
  "license_key": "PREMIUM-2596341E",
  "expires": "2027-01-07T08:26:46.056456389+00:00",
  "issued_at": "2026-01-07T08:26:46.056456389+00:00",
  "signature": "a87221177e36270b83e3364ede0a59a8f3fbb70426189436a3ad54c3e217fc749c615d2338d9811038d79e69bdf35bee9353c69b4c3b3e3bd3d06a2b89d80907",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

### Key Points

- **email**: Customer identifier
- **license_key**: Unique key with edition prefix (FREE-, PREMIUM-, ENTERPRISE-)
- **expires**: RFC3339 timestamp (ISO 8601)
- **signature**: Ed25519 signature (hex, 128 chars)
- **issuer**: Identifies the signing authority

---

## 2. Test License (Ready to Use)

### Premium License

**File**: `costpilot-license-issuer/license_premium.json`

```json
{
  "email": "test-premium@example.com",
  "license_key": "PREMIUM-2596341E",
  "expires": "2027-01-07T08:26:46.056456389+00:00",
  "issued_at": "2026-01-07T08:26:46.056456389+00:00",
  "signature": "a87221177e36270b83e3364ede0a59a8f3fbb70426189436a3ad54c3e217fc749c615d2338d9811038d79e69bdf35bee9353c69b4c3b3e3bd3d06a2b89d80907",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

### Keypair for This License

**File**: `costpilot-license-issuer/keypair_info.json`

```json
{
  "private_key_hex": "5d06a1afc2937a29e128f34a1041234b2b60bc61b50098114b781276b344dfd7",
  "public_key_hex": "45f9c85f9c70b5d51902a30cace13835994bb5c10e8dcd496689ab69b5bb4439",
  "public_key_base64": "RfnIX5xwtdUZAqMMrOE4NZlLtcEOjc1JZomrabW7RDk=",
  "fingerprint": "45f9c85f9c70b5d5"
}
```

⚠️ **Important**: Keep the private key secure. The public key is used by CostPilot to verify licenses.

---

## 3. Using the Standalone Library

### Location

```
CostPilot/
└── costpilot-license-issuer/
    ├── Cargo.toml              # Standalone library manifest
    ├── src/
    │   └── lib.rs              # Library code
    ├── examples/
    │   └── generate_test_license.rs
    ├── README.md               # Library documentation
    └── INTEGRATION_GUIDE.md    # API integration guide
```

### Add as Dependency

In your API project's `Cargo.toml`:

```toml
[dependencies]
costpilot-license-issuer = { path = "../costpilot-license-issuer" }
# or when published to crates.io:
# costpilot-license-issuer = "1.0"
```

### Basic Usage

```rust
use costpilot_license_issuer::{LicenseIssuer, LicenseRequest, EditionTier};

// Load private key (from env or secrets manager)
let private_key_hex = std::env::var("LICENSE_PRIVATE_KEY")?;
let private_key_bytes = hex::decode(private_key_hex)?;

// Create issuer
let issuer = LicenseIssuer::from_private_key_bytes(&private_key_bytes)?;

// Issue license
let request = LicenseRequest {
    email: "customer@example.com".to_string(),
    license_key: "PREMIUM-1234-5678".to_string(),
    edition: EditionTier::Premium,
    expires_days: 365,
};

let license = issuer.issue_license(request)?;

// Convert to JSON
let license_json = serde_json::to_string_pretty(&license)?;
```

---

## 4. API Integration Example

### REST API Endpoint (using Axum)

```rust
use axum::{Router, routing::post, Json};
use costpilot_license_issuer::{LicenseIssuer, IssuedLicense};

async fn issue_license(
    Json(request): Json<LicenseRequest>
) -> Json<IssuedLicense> {
    let issuer = get_issuer(); // Your issuer instance
    let license = issuer.issue_license(request).unwrap();
    Json(license)
}

let app = Router::new()
    .route("/api/licenses", post(issue_license));
```

### cURL Request

```bash
curl -X POST http://localhost:3000/api/licenses \
  -H "Content-Type: application/json" \
  -d '{
    "email": "customer@example.com",
    "license_key": "PREMIUM-1234",
    "edition": "premium",
    "expires_days": 365
  }'
```

---

## 5. Testing the License

### Generate Test Licenses

```bash
cd costpilot-license-issuer
cargo run --example generate_test_license
```

This creates:
- ✅ `license_free.json`
- ✅ `license_premium.json`
- ✅ `license_enterprise.json`
- ✅ `keypair_info.json`

### Use in CostPilot

```bash
# Copy license to CostPilot directory
cp costpilot-license-issuer/license_premium.json ~/.costpilot/license.json

# Test with CostPilot
costpilot scan plan.json
```

⚠️ **Note**: For CostPilot to verify this license, you need to configure it with the public key from `keypair_info.json`.

---

## 6. Architecture

```
┌─────────────────────────────────────────────────┐
│           Your API/Billing System               │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │  costpilot-license-issuer (library)      │  │
│  │                                          │  │
│  │  • Load private key                      │  │
│  │  • Issue licenses                        │  │
│  │  • Sign with Ed25519                     │  │
│  └──────────────────────────────────────────┘  │
│                      │                          │
│                      │ Generates                │
│                      ▼                          │
│            ┌──────────────────┐                 │
│            │  License JSON    │                 │
│            └──────────────────┘                 │
│                      │                          │
└──────────────────────┼──────────────────────────┘
                       │
                       │ Send to customer
                       ▼
              ┌─────────────────┐
              │   Customer      │
              │  (.costpilot/   │
              │  license.json)  │
              └─────────────────┘
                       │
                       │ Uses
                       ▼
              ┌─────────────────┐
              │   CostPilot     │
              │  (verifies with │
              │   public key)   │
              └─────────────────┘
```

---

## 7. Security Checklist

- ✅ Private key stored securely (env vars or secrets manager)
- ✅ Never commit private keys to version control
- ✅ Use HTTPS for license issuance endpoints
- ✅ Implement rate limiting
- ✅ Log all license issuance for audit
- ✅ Plan for key rotation
- ✅ Validate input (email, license key format)

---

## 8. Next Steps

### For API Integration

1. Add `costpilot-license-issuer` as dependency to your project
2. Generate keypair and store private key securely
3. Create REST endpoint using the library
4. Integrate with your billing system (Stripe, etc.)
5. Configure CostPilot with the public key for verification

### For Testing

1. Use the pre-generated test licenses in `costpilot-license-issuer/`
2. Copy to `~/.costpilot/license.json`
3. Test with CostPilot commands

---

## 9. File Locations

All generated files are in: `costpilot-license-issuer/`

```
costpilot-license-issuer/
├── license_free.json          # Ready-to-use Free license
├── license_premium.json       # Ready-to-use Premium license
├── license_enterprise.json    # Ready-to-use Enterprise license
├── keypair_info.json          # Keypair (public + private)
├── README.md                  # Library documentation
├── INTEGRATION_GUIDE.md       # Complete API integration guide
└── examples/
    └── generate_test_license.rs
```

---

## 10. Quick Reference

### Issue a License

```rust
let license = issuer.issue_license(LicenseRequest {
    email: "user@example.com".to_string(),
    license_key: "PREMIUM-1234".to_string(),
    edition: EditionTier::Premium,
    expires_days: 365,
})?;
```

### With Custom Expiry

```rust
let license = issuer.issue_license_with_expiry(
    "user@example.com".to_string(),
    "PREMIUM-1234".to_string(),
    "2027-12-31T23:59:59Z".to_string(),
)?;
```

### Generate Keypair

```rust
let keypair = LicenseIssuer::generate_keypair()?;
println!("Private: {}", hex::encode(&keypair.private_key_bytes));
println!("Public:  {}", keypair.public_key_hex);
```

---

## Questions?

- 📖 Read: `INTEGRATION_GUIDE.md` for detailed API examples
- 🔍 Check: `examples/generate_test_license.rs` for working code
- 🧪 Test: `cargo test` in the `costpilot-license-issuer/` directory
