# Implementation Checklist: Remove Private Key Embedding

## Overview
Client binaries are verification-only artifacts. License signing is an external, offline or CI-bound responsibility and must never occur in the client build pipeline. License issuer private keys must be stable across releases. Rebuilding CostPilot must not invalidate existing licenses.

## Steps
- [ ] Clarify trust boundary: Client binaries verification-only; signing external/offline/CI.
- [ ] Define key stability: Issuer keys stable across releases; document location (CI secret/HSM/encrypted file) and rotation policy (manual/versioned).
- [ ] Audit repo-wide: Use ripgrep to search for "LICENSE_PRIVATE_KEY" and "WASM_PRIVATE_KEY"; confirm zero references outside build.rs, no test fixtures, no debug/logging paths.
- [ ] Verify no runtime references or assumptions in code/tests.
- [ ] Confirm license signing external, no build.rs dependency; note regeneration invalidates licenses.
- [ ] Plan responsibility split: Client embeds public keys only; issuer uses persistent private keys from secure storage.
- [ ] Open build.rs and locate the format! macro starting around line 23.
- [ ] Within the raw string literal, remove the comment lines about private keys (lines 29-31).
- [ ] Remove the two constant declarations for LICENSE_PRIVATE_KEY and WASM_PRIVATE_KEY (lines 32-33).
- [ ] Add comment in raw string: "// Public keys only. Private keys never shipped."
- [ ] Update the format! parameters to remove license_keypair.private and wasm_keypair.private (lines 36-39).
- [ ] Save the modified build.rs file.
- [ ] Run cargo build --release to rebuild and verify no compilation errors.
- [ ] Execute cargo test to ensure license and WASM verification tests pass: test_load_pro_engine_success (accepts valid signatures), test_load_pro_engine_verify_error (rejects tampered/corrupted), test_load_pro_engine_expired_license (rejects expired).
- [ ] Check generated keys.rs in target/: contains only public keys, no private-key-length material (64-byte blobs), comment present.
- [ ] Update limited documentation: docs/architecture.md (lines 160-175) and docs/SECURITY.md (lines 25-35) to reflect public-key-only embedding.
- [x] Complete non-functional acceptance checklist: Existing licenses validate after rebuild; new licenses issued without repo changes; no private key material in artifacts; public key rotation intentional.

## Further Considerations
- Implement secure license issuer with persistent private keys to avoid regeneration issues.
