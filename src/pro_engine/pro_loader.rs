use super::{crypto, instantiate, license::License};
use crate::edition::{EditionContext, ProEngineHandle as EditionProEngineHandle};

#[cfg(not(target_arch = "wasm32"))]
pub fn load_pro_engine(edition: &mut EditionContext) -> Result<(), String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    let base = home.join(".costpilot");
    let wasm_enc = base.join("pro-engine.wasm.enc");
    let license_file = base.join("license.json");
    let sig_file = base.join("pro-engine.sig");

    if !wasm_enc.exists() || !license_file.exists() || !sig_file.exists() {
        return Ok(());
    }

    let lic = License::load_from_file(&license_file)?;

    lic.validate()?;

    crypto::verify_license_signature(&lic)?;

    edition.license = Some(lic.clone());

    let key = crypto::derive_key(&lic.license_key);

    let ciphertext =
        std::fs::read(&wasm_enc).map_err(|e| format!("Failed to read encrypted WASM: {}", e))?;
    let plaintext = crypto::decrypt_aes_gcm(&ciphertext, &key)?;

    let sig =
        std::fs::read(&sig_file).map_err(|e| format!("Failed to read WASM signature: {}", e))?;
    crypto::verify_wasm_signature(&plaintext, &sig)?;

    let engine_internal = instantiate::instantiate_wasm(&plaintext)?;

    let engine_edition = EditionProEngineHandle::with_executor(
        wasm_enc.clone(),
        Some(plaintext),
        Box::new(WrapperExecutor {
            inner: engine_internal,
        }),
    );

    edition.pro = Some(engine_edition);
    edition.mode = crate::edition::EditionMode::Premium;
    edition.capabilities = crate::edition::Capabilities::from_edition(edition);

    Ok(())
}

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
