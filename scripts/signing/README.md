# Release Signing Tools

Ed25519-based signing for release artifacts.

## Usage

### Generate keypair
```bash
./generate_keypair.sh build/keys release_key
# Output: KEYPAIR: build/keys/release_key.pem,build/keys/release_key.pub.pem
```

### Sign checksums
```bash
./sign_checksum_ed25519.sh dist/sha256sum.txt build/keys/release_key.pem dist/sha256sum.txt.sig
# Output: SIGN: dist/sha256sum.txt.sig
```

### Verify signature
```bash
./verify_checksum_ed25519.sh dist/sha256sum.txt dist/sha256sum.txt.sig build/keys/release_key.pub.pem
# Output: VERIFY: dist/sha256sum.txt OK
```

### Rotate keys
```bash
./rotate_keys.sh build/keys new_release_key build/rotation.json
# Output: ROTATE: build/rotation.json
```

## CI Integration

- Ephemeral keys generated per-build for internal testing
- Publisher public key injected via GitHub secrets for release verification
- Private keys never committed (see .gitignore)

## Requirements

- OpenSSL with Ed25519 support (OpenSSL 1.1.1+)

- `verify.sh` - Verify signatures

## Usage

### Generate Keys (First Time Setup)

```bash
cd packaging/signing
./gen_keys.sh
```

**IMPORTANT:** Never commit `private.key` to version control. It should be:
- Generated locally for development builds
- Injected via GitHub Actions secrets for CI builds

### Sign Artifacts

```bash
./sign.sh <artifact_file> private.key
```

This creates `<artifact_file>.sig` containing the Ed25519 signature.

### Verify Signatures

```bash
./verify.sh <artifact_file> public.key <artifact_file>.sig
```

Exit code 0 = verified successfully
Exit code 1 = verification failed

## CI Integration

In GitHub Actions, the private key is injected from secrets:

```yaml
- name: Setup signing key
  run: |
    echo "${{ secrets.SIGNING_PRIVATE_KEY }}" > packaging/signing/private.key
    chmod 600 packaging/signing/private.key
```

## Security Model

- **Free Edition:** Uses repository-committed public key for verification
- **Premium Edition:** Uses customer-specific keypairs (future implementation)
- **Signature Format:** Detached Ed25519 signatures in base64 encoding
- **Deterministic:** Same artifact + same key = same signature

## Key Rotation

To rotate keys:
1. Generate new keypair with `gen_keys.sh`
2. Update GitHub Actions secret `SIGNING_PRIVATE_KEY`
3. Commit new `public.key` to repository
4. Re-sign all release artifacts
