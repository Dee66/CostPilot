/// CostPilot License Issuer Library
///
/// This library provides license generation and signing capabilities for CostPilot.
/// It can be used as a dependency in other projects (e.g., API servers) to issue licenses.
///
/// # Example Usage
///
/// ```no_run
/// use costpilot_license_issuer::{LicenseIssuer, LicenseRequest, EditionTier};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Generate a keypair (do this once, securely store the private key)
/// let keypair = LicenseIssuer::generate_keypair()?;
/// println!("Public key: {}", keypair.public_key_hex);
///
/// // Issue a license
/// let request = LicenseRequest {
///     email: "customer@example.com".to_string(),
///     license_key: "PREMIUM-1234-5678-9ABC".to_string(),
///     edition: EditionTier::Premium,
///     expires_days: 365,
/// };
///
/// let issuer = LicenseIssuer::from_private_key_bytes(&keypair.private_key_bytes)?;
/// let license = issuer.issue_license(request)?;
///
/// // The license can now be sent to the customer
/// println!("License JSON:\n{}", serde_json::to_string_pretty(&license)?);
/// # Ok(())
/// # }
/// ```

use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Invalid key length: expected 32 bytes")]
    InvalidKeyLength,
}

/// Edition tiers for CostPilot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditionTier {
    Free,
    Premium,
    Enterprise,
}

impl EditionTier {
    pub fn as_license_key_prefix(&self) -> &'static str {
        match self {
            EditionTier::Free => "FREE",
            EditionTier::Premium => "PREMIUM",
            EditionTier::Enterprise => "ENTERPRISE",
        }
    }
}

/// A license that can be issued to customers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedLicense {
    pub email: String,
    pub license_key: String,
    pub expires: String,
    pub issued_at: String,
    pub signature: String,
    pub version: String,
    pub issuer: String,
}

/// Request structure for issuing a new license
#[derive(Debug, Clone)]
pub struct LicenseRequest {
    pub email: String,
    pub license_key: String,
    pub edition: EditionTier,
    pub expires_days: i64,
}

/// Keypair structure returned when generating keys
#[derive(Debug, Clone)]
pub struct Keypair {
    pub private_key_bytes: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
    pub public_key_hex: String,
    pub public_key_base64: String,
    pub fingerprint: String,
}

/// Main license issuer that holds the signing key
pub struct LicenseIssuer {
    signing_key: SigningKey,
    issuer_name: String,
}

impl LicenseIssuer {
    /// Create a new issuer from raw private key bytes (32 bytes)
    pub fn from_private_key_bytes(key_bytes: &[u8]) -> Result<Self, LicenseError> {
        if key_bytes.len() != 32 {
            return Err(LicenseError::InvalidKeyLength);
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(key_bytes);
        let signing_key = SigningKey::from_bytes(&bytes);

        Ok(Self {
            signing_key,
            issuer_name: "costpilot-v1".to_string(),
        })
    }

    /// Create a new issuer with a custom issuer name
    pub fn from_private_key_bytes_with_issuer(
        key_bytes: &[u8],
        issuer: String,
    ) -> Result<Self, LicenseError> {
        if key_bytes.len() != 32 {
            return Err(LicenseError::InvalidKeyLength);
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(key_bytes);
        let signing_key = SigningKey::from_bytes(&bytes);

        Ok(Self {
            signing_key,
            issuer_name: issuer,
        })
    }

    /// Generate a new Ed25519 keypair for license signing
    pub fn generate_keypair() -> Result<Keypair, LicenseError> {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        let public_bytes = verifying_key.to_bytes();
        let fingerprint = hex::encode(&public_bytes[..8]);

        Ok(Keypair {
            private_key_bytes: secret_bytes.to_vec(),
            public_key_bytes: public_bytes.to_vec(),
            public_key_hex: hex::encode(public_bytes),
            public_key_base64: base64::engine::general_purpose::STANDARD.encode(public_bytes),
            fingerprint,
        })
    }

    /// Get the public key for this issuer
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the public key as hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key().to_bytes())
    }

    /// Get the fingerprint (first 8 bytes of public key)
    pub fn fingerprint(&self) -> String {
        hex::encode(&self.public_key().to_bytes()[..8])
    }

    /// Issue a new license based on the request
    pub fn issue_license(&self, request: LicenseRequest) -> Result<IssuedLicense, LicenseError> {
        // Calculate expiration date
        let now: DateTime<Utc> = Utc::now();
        let expires = now + Duration::days(request.expires_days);
        let expires_str = expires.to_rfc3339();
        let issued_at = now.to_rfc3339();

        // Create canonical message (matches the format expected by CostPilot)
        let canonical_message = format!(
            "{}|{}|{}|{}",
            request.email, request.license_key, expires_str, self.issuer_name
        );

        // Sign the message
        let signature = self.signing_key.sign(canonical_message.as_bytes());

        // Create the license
        Ok(IssuedLicense {
            email: request.email,
            license_key: request.license_key,
            expires: expires_str,
            issued_at,
            signature: hex::encode(signature.to_bytes()),
            version: "1.0".to_string(),
            issuer: self.issuer_name.clone(),
        })
    }

    /// Issue a license with a specific expiration date (RFC3339 format)
    pub fn issue_license_with_expiry(
        &self,
        email: String,
        license_key: String,
        expires: String,
    ) -> Result<IssuedLicense, LicenseError> {
        let issued_at = Utc::now().to_rfc3339();

        // Create canonical message
        let canonical_message = format!(
            "{}|{}|{}|{}",
            email, license_key, expires, self.issuer_name
        );

        // Sign the message
        let signature = self.signing_key.sign(canonical_message.as_bytes());

        Ok(IssuedLicense {
            email,
            license_key,
            expires,
            issued_at,
            signature: hex::encode(signature.to_bytes()),
            version: "1.0".to_string(),
            issuer: self.issuer_name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = LicenseIssuer::generate_keypair().unwrap();
        assert_eq!(keypair.private_key_bytes.len(), 32);
        assert_eq!(keypair.public_key_bytes.len(), 32);
        assert!(!keypair.fingerprint.is_empty());
    }

    #[test]
    fn test_issue_license() {
        let keypair = LicenseIssuer::generate_keypair().unwrap();
        let issuer = LicenseIssuer::from_private_key_bytes(&keypair.private_key_bytes).unwrap();

        let request = LicenseRequest {
            email: "test@example.com".to_string(),
            license_key: "PREMIUM-TEST-1234".to_string(),
            edition: EditionTier::Premium,
            expires_days: 365,
        };

        let license = issuer.issue_license(request).unwrap();

        assert_eq!(license.email, "test@example.com");
        assert_eq!(license.license_key, "PREMIUM-TEST-1234");
        assert_eq!(license.version, "1.0");
        assert_eq!(license.issuer, "costpilot-v1");
        assert!(!license.signature.is_empty());
    }

    #[test]
    fn test_custom_issuer_name() {
        let keypair = LicenseIssuer::generate_keypair().unwrap();
        let issuer = LicenseIssuer::from_private_key_bytes_with_issuer(
            &keypair.private_key_bytes,
            "my-custom-issuer".to_string(),
        )
        .unwrap();

        let request = LicenseRequest {
            email: "test@example.com".to_string(),
            license_key: "PREMIUM-TEST-1234".to_string(),
            edition: EditionTier::Premium,
            expires_days: 365,
        };

        let license = issuer.issue_license(request).unwrap();
        assert_eq!(license.issuer, "my-custom-issuer");
    }
}
