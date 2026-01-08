# 🎫 CostPilot License Issuer - Quick Start

## What You Need

### 1. License Format ✅

```json
{
  "email": "customer@example.com",
  "license_key": "PREMIUM-2596341E",
  "expires": "2027-01-07T08:26:46Z",
  "issued_at": "2026-01-07T08:26:46Z",
  "signature": "a87221177e36270b83e3364ede0a59a8...",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

### 2. Test License (Ready to Use) ✅

Located in: `costpilot-license-issuer/license_premium.json`

**To use immediately:**
```bash
cp costpilot-license-issuer/license_premium.json ~/.costpilot/license.json
```

**Keypair info:** `costpilot-license-issuer/keypair_info.json`

- **Public Key**: `45f9c85f9c70b5d51902a30cace13835994bb5c10e8dcd496689ab69b5bb4439`
- **Fingerprint**: `45f9c85f9c70b5d5`

### 3. Separate Library ✅

The license issuer is now a **standalone Rust library** in:
```
costpilot-license-issuer/
```

Can be used in other projects (like an API server) as a dependency.

---

## Quick Usage Examples

### Example 1: Generate More Test Licenses

```bash
cd costpilot-license-issuer
cargo run --example generate_test_license
```

Creates:
- `license_free.json`
- `license_premium.json`
- `license_enterprise.json`
- `keypair_info.json`

### Example 2: Use in Your API (Rust)

Add to your `Cargo.toml`:
```toml
[dependencies]
costpilot-license-issuer = { path = "../costpilot-license-issuer" }
```

In your code:
```rust
use costpilot_license_issuer::{LicenseIssuer, LicenseRequest, EditionTier};

// Load private key
let key_bytes = hex::decode(env::var("LICENSE_PRIVATE_KEY")?)?;
let issuer = LicenseIssuer::from_private_key_bytes(&key_bytes)?;

// Issue license
let license = issuer.issue_license(LicenseRequest {
    email: "customer@example.com".to_string(),
    license_key: "PREMIUM-1234-5678".to_string(),
    edition: EditionTier::Premium,
    expires_days: 365,
})?;

// Send to customer as JSON
let json = serde_json::to_string_pretty(&license)?;
```

### Example 3: Run Simple API Server

```bash
cd costpilot-license-issuer
cargo run --example simple_api_server
```

Then test:
```bash
curl -X POST http://localhost:8080/api/licenses \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "license_key": "PREMIUM-TEST-123",
    "edition": "premium",
    "expires_days": 365
  }'
```

---

## Files Created

All in `costpilot-license-issuer/`:

| File | Description |
|------|-------------|
| `license_premium.json` | ✅ **Ready-to-use Premium license** |
| `license_free.json` | ✅ Ready-to-use Free license |
| `license_enterprise.json` | ✅ Ready-to-use Enterprise license |
| `keypair_info.json` | 🔑 Keypair (public + private keys) |
| `README.md` | 📖 Library documentation |
| `INTEGRATION_GUIDE.md` | 📖 Complete API integration guide |
| `SUMMARY.md` | 📖 This summary document |
| `src/lib.rs` | 💻 Library source code |
| `examples/generate_test_license.rs` | 💻 Generate licenses example |
| `examples/simple_api_server.rs` | 💻 Simple HTTP API example |

---

## For Your Other Application

Since you want to use this in a **different repo/application**, you have options:

### Option A: Publish to crates.io (Recommended)

```bash
cd costpilot-license-issuer
cargo publish
```

Then in your other project:
```toml
[dependencies]
costpilot-license-issuer = "1.0"
```

### Option B: Git Dependency

In your other project's `Cargo.toml`:
```toml
[dependencies]
costpilot-license-issuer = { git = "https://github.com/Dee66/CostPilot", branch = "main" }
```

### Option C: Path Dependency (for local development)

```toml
[dependencies]
costpilot-license-issuer = { path = "../CostPilot/costpilot-license-issuer" }
```

### Option D: Copy the Library

Simply copy the `costpilot-license-issuer/` folder to your other project.

---

## Architecture

```
Your API Server (different repo)
    ↓
[Add costpilot-license-issuer as dependency]
    ↓
Use LicenseIssuer::issue_license()
    ↓
Returns IssuedLicense JSON
    ↓
Send to customer
    ↓
Customer puts in ~/.costpilot/license.json
    ↓
CostPilot verifies signature
```

---

## Next Steps

1. ✅ **You have the license format** (see above)
2. ✅ **You have test licenses** (in `costpilot-license-issuer/`)
3. ✅ **You have a standalone library** (in `costpilot-license-issuer/`)

### To use in your API:

1. Choose integration option (A, B, C, or D above)
2. Add as dependency to your API project
3. Load private key securely (env var or secrets manager)
4. Call `issuer.issue_license()` when customer subscribes
5. Send license JSON to customer

### For Production:

- Store private key in secrets manager (AWS Secrets, Vault, etc.)
- Configure CostPilot with your public key for verification
- Implement rate limiting on license issuance
- Log all license creation for audit

---

## Documentation

- **Library docs**: `costpilot-license-issuer/README.md`
- **API integration**: `costpilot-license-issuer/INTEGRATION_GUIDE.md`
- **Examples**: `costpilot-license-issuer/examples/`

---

## Support

Questions? Check:
1. `INTEGRATION_GUIDE.md` - Complete API examples
2. `examples/` directory - Working code samples
3. Run tests: `cd costpilot-license-issuer && cargo test`
