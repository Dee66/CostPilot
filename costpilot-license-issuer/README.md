# CostPilot License Issuer

A standalone library for issuing CostPilot licenses. This can be used as a dependency in other projects (e.g., API servers, billing systems) to generate signed licenses.

## Features

- **Ed25519 Signing**: Cryptographically secure license signing
- **Standalone Library**: Can be used independently of the main CostPilot application
- **Simple API**: Easy to integrate into existing systems
- **Multiple Edition Tiers**: Support for Free, Premium, and Enterprise editions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
costpilot-license-issuer = { path = "../costpilot-license-issuer" }
# or when published:
# costpilot-license-issuer = "1.0"
```

## Usage

### 1. Generate a Keypair (One-Time Setup)

```rust
use costpilot_license_issuer::LicenseIssuer;

// Generate keypair - do this once and securely store the private key
let keypair = LicenseIssuer::generate_keypair()?;

println!("Private key (keep secret): {:?}", keypair.private_key_bytes);
println!("Public key: {}", keypair.public_key_hex);
println!("Fingerprint: {}", keypair.fingerprint);

// Store private_key_bytes securely (e.g., in environment variable, secrets manager)
// Share public_key_hex with the CostPilot application for verification
```

### 2. Issue a License

```rust
use costpilot_license_issuer::{LicenseIssuer, LicenseRequest, EditionTier};

// Load your private key (stored securely)
let private_key_bytes: Vec<u8> = get_private_key_from_secure_storage();

// Create issuer
let issuer = LicenseIssuer::from_private_key_bytes(&private_key_bytes)?;

// Create license request
let request = LicenseRequest {
    email: "customer@example.com".to_string(),
    license_key: "PREMIUM-1234-5678-9ABC".to_string(),
    edition: EditionTier::Premium,
    expires_days: 365,
};

// Issue the license
let license = issuer.issue_license(request)?;

// Serialize to JSON
let license_json = serde_json::to_string_pretty(&license)?;
println!("{}", license_json);
```

### 3. Example API Integration

```rust
// In your API server (e.g., Axum, Actix, Rocket)
use axum::{Json, extract::State};
use costpilot_license_issuer::{LicenseIssuer, LicenseRequest, IssuedLicense};

struct AppState {
    license_issuer: LicenseIssuer,
}

async fn issue_license_handler(
    State(state): State<AppState>,
    Json(request): Json<LicenseRequest>,
) -> Json<IssuedLicense> {
    let license = state.license_issuer.issue_license(request).unwrap();
    Json(license)
}
```

## License Format

The issued license is a JSON object with the following structure:

```json
{
  "email": "customer@example.com",
  "license_key": "PREMIUM-1234-5678-9ABC",
  "expires": "2027-01-07T12:00:00Z",
  "issued_at": "2026-01-07T12:00:00Z",
  "signature": "a1b2c3d4...",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

### Fields:
- **email**: Customer email address
- **license_key**: Unique license key (should include edition prefix like `PREMIUM-`)
- **expires**: Expiration date in RFC3339 format
- **issued_at**: Issue date in RFC3339 format
- **signature**: Ed25519 signature (hex-encoded) of the canonical message
- **version**: License format version
- **issuer**: Issuer identifier

## Security Notes

1. **Private Key Storage**: Never commit private keys to version control. Use environment variables or secrets management services.
2. **Signature Verification**: The CostPilot application verifies signatures using the corresponding public key.
3. **Key Rotation**: Plan for key rotation by supporting multiple issuers in your deployment.

## Testing

Run the tests:

```bash
cd costpilot-license-issuer
cargo test
```

## License

MIT License - See LICENSE file for details
