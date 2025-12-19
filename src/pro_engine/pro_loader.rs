// ProEngine loader - handles license, decryption, and WASM loading

use super::{crypto, instantiate, license::License};
use crate::edition::{EditionContext, ProEngineHandle as EditionProEngineHandle};

/// Load ProEngine if available and valid
#[cfg(not(target_arch = "wasm32"))]
pub fn load_pro_engine(edition: &mut EditionContext) -> Result<(), String> {
    // 1. Locate artifacts
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let base = home.join(".costpilot");
    let wasm_enc = base.join("pro-engine.wasm.enc");
    let license_file = base.join("license.json");
    let sig_file = base.join("pro-engine.sig");

    // If files don't exist, stay in Free mode
    if !wasm_enc.exists() || !license_file.exists() || !sig_file.exists() {
        return Ok(()); // Free mode - no error
    }

    // 2. Load license
    let lic = License::load_from_file(&license_file)?;

    // 3. Validate license
    lic.validate()?;

    // 4. Verify license signature
    crypto::verify_license_signature(&lic)?;

    // 5. Derive AES-GCM key from license via HKDF
    let key = crypto::derive_key(&lic.license_key);

    // 6. Decrypt WASM (AES-GCM)
    let ciphertext =
        std::fs::read(&wasm_enc).map_err(|e| format!("Failed to read encrypted WASM: {}", e))?;
    let plaintext = crypto::decrypt_aes_gcm(&ciphertext, &key)?;

    // 7. Verify decrypted WASM signature (Ed25519)
    let sig =
        std::fs::read(&sig_file).map_err(|e| format!("Failed to read WASM signature: {}", e))?;
    crypto::verify_wasm_signature(&plaintext, &sig)?;

    // 8. Instantiate WASM module (returns pro_engine::ProEngineHandle)
    let engine_internal = instantiate::instantiate_wasm(&plaintext)?;

    // 9. Convert to edition::ProEngineHandle (wraps the internal handle)
    let engine_edition = EditionProEngineHandle::with_executor(
        wasm_enc.clone(),
        Some(plaintext),
        Box::new(WrapperExecutor {
            inner: engine_internal,
        }),
    );

    // 10. Attach to edition context
    edition.pro = Some(engine_edition);
    edition.mode = crate::edition::EditionMode::Premium;
    edition.capabilities = crate::edition::Capabilities::from_edition(edition);

    Ok(())
}

/// Wrapper to bridge pro_engine::ProEngineHandle to edition::ProEngineHandle
struct WrapperExecutor {
    inner: crate::pro_engine::ProEngineHandle,
}

impl crate::pro_engine::ProEngineExecutor for WrapperExecutor {
    fn execute(
        &self,
        req: crate::pro_engine::ProEngineRequest,
    ) -> Result<crate::pro_engine::ProEngineResponse, String> {
        self.inner.execute(req)
    }
}
