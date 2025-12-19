# Security Architecture

## Pro Engine Loader

### Encryption & Signing

- **Algorithm**: AES-256-GCM with authenticated encryption
- **Key Derivation**: HKDF-SHA256 from license key + subject + optional machine binding
- **Signature**: Ed25519 over metadata + nonce + ciphertext
- **AAD**: Metadata JSON included as Additional Authenticated Data

### Bundle Format

```
[4-byte BE length][metadata JSON][12-byte nonce][ciphertext][64-byte Ed25519 signature]
```

### Security Properties

1. **Signature Verification**: Ed25519 strict verification prevents tampering
2. **Key Zeroization**: Sensitive key material zeroized after use (via `zeroize` crate)
3. **No Plaintext Leaks**: Keys, tokens, and plaintext WASM never logged
4. **Fail-Closed**: Any validation failure returns error; no partial decryption
5. **WASM Integrity**: Magic header (`\0asm`) validated post-decryption

### Key Rotation

Public keys for signature verification can be rotated via:
- Embedded default key (compile-time)
- Config override: `config/pro_engine_pubkey.pem`
- Environment variable: `COSTPILOT_PRO_PUBKEY_PATH`

### Assumptions

- License key material has sufficient entropy (128+ bits recommended)
- Machine binding (if used) uniquely identifies the authorized host
- Ed25519 public key authenticity verified via separate channel (release signatures, checksum verification)
- WASM bundle encrypted at rest and in transit; decrypted only in-memory

### Threat Model

**Protected Against:**
- Bundle tampering (signature verification)
- Unauthorized decryption (license-bound keys)
- Key reuse attacks (unique nonces per bundle)
- Partial decryption (AAD + integrity checks)

**Not Protected Against:**
- Memory dumps of running process (plaintext WASM in memory post-decryption)
- Side-channel attacks on AES-GCM implementation (relies on constant-time crypto libraries)
- Compromised license keys (revocation not implemented)

### Recommendations

1. Rotate Ed25519 signing keys annually
2. Use machine binding for enterprise deployments
3. Monitor license usage for anomalies
4. Implement license revocation for compromised keys (future enhancement)
