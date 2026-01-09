# CostPilot License Issuer - Integration Guide

This guide shows how to integrate the license issuer into your API or billing system.

## Table of Contents

1. [License Format](#license-format)
2. [Setup](#setup)
3. [REST API Example](#rest-api-example)
4. [Webhook Integration](#webhook-integration)
5. [Security Best Practices](#security-best-practices)

---

## License Format

### Issued License JSON Structure

```json
{
  "email": "customer@example.com",
  "license_key": "PREMIUM-2596341E",
  "expires": "2027-01-07T08:26:46.056456389+00:00",
  "issued_at": "2026-01-07T08:26:46.056456389+00:00",
  "signature": "a87221177e36270b83e3364ede0a59a8...",
  "version": "1.0",
  "issuer": "costpilot-v1"
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `email` | String | Customer email address (used as unique identifier) |
| `license_key` | String | License key with edition prefix (FREE-, PREMIUM-, ENTERPRISE-) |
| `expires` | String | Expiration date in RFC3339 format (ISO 8601) |
| `issued_at` | String | Issue date in RFC3339 format (ISO 8601) |
| `signature` | String | Ed25519 signature (hex-encoded, 128 characters) |
| `version` | String | License format version (currently "1.0") |
| `issuer` | String | Issuer identifier (e.g., "costpilot-v1") |

### Signature Algorithm

The signature is computed using Ed25519 over the canonical message:

```
canonical_message = "{email}|{license_key}|{expires}|{issuer}"
signature = Ed25519.sign(private_key, canonical_message)
```

Example:
```
Message: "customer@example.com|PREMIUM-2596341E|2027-01-07T08:26:46Z|costpilot-v1"
Signature: a87221177e36270b83e3364ede0a59a8f3fbb70426189436a3ad54c3e217fc749c615d2338d9811038d79e69bdf35bee9353c69b4c3b3e3bd3d06a2b89d80907
```

---

## Setup

### 1. Add Dependency

```toml
[dependencies]
costpilot-license-issuer = { path = "../costpilot-license-issuer" }
```

### 2. Generate and Store Keypair

```rust
use costpilot_license_issuer::LicenseIssuer;
use std::env;

// Generate once during initial setup
let keypair = LicenseIssuer::generate_keypair()?;

// Store private key in environment variable or secrets manager
// Example: AWS Secrets Manager, HashiCorp Vault, etc.
env::set_var("LICENSE_PRIVATE_KEY", hex::encode(&keypair.private_key_bytes));

// Share public key with CostPilot application
println!("Public Key: {}", keypair.public_key_hex);
```

### 3. Load Private Key Securely

```rust
use hex;
use std::env;

fn load_private_key() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let key_hex = env::var("LICENSE_PRIVATE_KEY")
        .expect("LICENSE_PRIVATE_KEY not set");
    let key_bytes = hex::decode(key_hex)?;
    Ok(key_bytes)
}
```

---

## REST API Example

### Using Axum Framework

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use costpilot_license_issuer::{EditionTier, IssuedLicense, LicenseIssuer, LicenseRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Application state
struct AppState {
    license_issuer: LicenseIssuer,
}

// Request body for license creation
#[derive(Deserialize)]
struct CreateLicenseRequest {
    email: String,
    license_key: String,
    edition: String, // "free", "premium", "enterprise"
    expires_days: i64,
}

// Response body
#[derive(Serialize)]
struct CreateLicenseResponse {
    success: bool,
    license: Option<IssuedLicense>,
    error: Option<String>,
}

// Handler function
async fn create_license(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateLicenseRequest>,
) -> impl IntoResponse {
    // Map string to EditionTier
    let edition = match payload.edition.to_lowercase().as_str() {
        "free" => EditionTier::Free,
        "premium" => EditionTier::Premium,
        "enterprise" => EditionTier::Enterprise,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CreateLicenseResponse {
                    success: false,
                    license: None,
                    error: Some("Invalid edition tier".to_string()),
                }),
            );
        }
    };

    let request = LicenseRequest {
        email: payload.email,
        license_key: payload.license_key,
        edition,
        expires_days: payload.expires_days,
    };

    match state.license_issuer.issue_license(request) {
        Ok(license) => (
            StatusCode::CREATED,
            Json(CreateLicenseResponse {
                success: true,
                license: Some(license),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateLicenseResponse {
                success: false,
                license: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

#[tokio::main]
async fn main() {
    // Load private key from environment
    let private_key_hex = std::env::var("LICENSE_PRIVATE_KEY")
        .expect("LICENSE_PRIVATE_KEY not set");
    let private_key_bytes = hex::decode(private_key_hex).expect("Invalid private key");

    // Create issuer
    let issuer = LicenseIssuer::from_private_key_bytes(&private_key_bytes)
        .expect("Failed to create issuer");

    // Create app state
    let state = Arc::new(AppState {
        license_issuer: issuer,
    });

    // Build router
    let app = Router::new()
        .route("/api/licenses", post(create_license))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("License API server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### Example cURL Request

```bash
curl -X POST http://localhost:3000/api/licenses \
  -H "Content-Type: application/json" \
  -d '{
    "email": "customer@example.com",
    "license_key": "PREMIUM-1234-5678",
    "edition": "premium",
    "expires_days": 365
  }'
```

### Example Response

```json
{
  "success": true,
  "license": {
    "email": "customer@example.com",
    "license_key": "PREMIUM-1234-5678",
    "expires": "2027-01-07T12:00:00Z",
    "issued_at": "2026-01-07T12:00:00Z",
    "signature": "a87221177e36270b...",
    "version": "1.0",
    "issuer": "costpilot-v1"
  },
  "error": null
}
```

---

## Webhook Integration

### Stripe Webhook Example

```rust
use costpilot_license_issuer::{EditionTier, LicenseIssuer};
use stripe::{Event, EventType, CheckoutSession};

async fn handle_stripe_webhook(
    event: Event,
    issuer: &LicenseIssuer,
) -> Result<(), Box<dyn std::error::Error>> {
    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            let session: CheckoutSession = event.data.deserialize()?;

            let email = session.customer_email.ok_or("No email")?;
            let license_key = format!("PREMIUM-{}", uuid::Uuid::new_v4());

            let request = LicenseRequest {
                email: email.clone(),
                license_key: license_key.clone(),
                edition: EditionTier::Premium,
                expires_days: 365,
            };

            let license = issuer.issue_license(request)?;

            // Send license via email
            send_license_email(&email, &license).await?;

            // Store in database
            store_license_in_db(&license).await?;

            println!("License issued for {}: {}", email, license_key);
        }
        _ => {}
    }

    Ok(())
}
```

---

## Security Best Practices

### 1. Private Key Storage

**❌ Never do this:**
```rust
const PRIVATE_KEY: &str = "5d06a1afc2937a29..."; // Hardcoded!
```

**✅ Do this instead:**
```rust
// Use environment variables
let key = env::var("LICENSE_PRIVATE_KEY")?;

// Or use secrets manager
let key = aws_secrets_manager::get_secret("license-private-key").await?;
```

### 2. Key Rotation

Plan for periodic key rotation:

```rust
struct MultiKeyIssuer {
    current_key: LicenseIssuer,
    old_keys: Vec<LicenseIssuer>, // For verification of old licenses
}
```

### 3. Rate Limiting

Implement rate limiting on license issuance:

```rust
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_hour(100));
if limiter.check().is_err() {
    return Err("Rate limit exceeded");
}
```

### 4. Audit Logging

Log all license issuance:

```rust
log::info!(
    "License issued: email={}, key={}, edition={:?}, expires={}",
    email, license_key, edition, expires
);
```

### 5. HTTPS Only

Always serve license issuance APIs over HTTPS in production.

---

## Testing

### Test with curl

```bash
# Generate test keypair and licenses
cd costpilot-license-issuer
cargo run --example generate_test_license

# Files created:
# - license_free.json
# - license_premium.json
# - license_enterprise.json
# - keypair_info.json

# Use the license
cp license_premium.json ~/.costpilot/license.json
costpilot scan plan.json
```

### Verify License

To verify the license in CostPilot, you need to configure the public key:

1. Get the public key from `keypair_info.json`
2. Add to CostPilot's crypto module as a trusted issuer
3. CostPilot will verify the signature on load

---

## Example: Complete Billing Integration

```rust
// billing_service.rs
use costpilot_license_issuer::{EditionTier, LicenseIssuer, LicenseRequest};

pub struct BillingService {
    license_issuer: LicenseIssuer,
    email_service: EmailService,
    database: Database,
}

impl BillingService {
    pub async fn handle_subscription_created(
        &self,
        customer_email: String,
        plan_tier: String,
    ) -> Result<(), Error> {
        // 1. Generate license key
        let license_key = self.generate_license_key(&plan_tier);

        // 2. Determine edition
        let edition = match plan_tier.as_str() {
            "free" => EditionTier::Free,
            "premium" => EditionTier::Premium,
            "enterprise" => EditionTier::Enterprise,
            _ => return Err(Error::InvalidPlan),
        };

        // 3. Issue license
        let request = LicenseRequest {
            email: customer_email.clone(),
            license_key: license_key.clone(),
            edition,
            expires_days: 365,
        };

        let license = self.license_issuer.issue_license(request)?;

        // 4. Store in database
        self.database.store_license(&license).await?;

        // 5. Send to customer
        self.email_service.send_license_email(&customer_email, &license).await?;

        Ok(())
    }

    fn generate_license_key(&self, plan: &str) -> String {
        format!("{}-{}",
            plan.to_uppercase(),
            uuid::Uuid::new_v4().to_string()[..13].to_uppercase()
        )
    }
}
```

---

## Support

For questions or issues:
- Open an issue on GitHub
- Check the main CostPilot documentation
- Review the example code in the `examples/` directory
