// ProEngineHandle wrapper for WASM runtime

use super::wasm_runtime::{SandboxInstance, WasmError, WasmRuntime, WasmSandboxConfig};

pub struct ProEngineHandle {
    instance: SandboxInstance,
    config: WasmSandboxConfig,
}

impl ProEngineHandle {
    pub fn from_bytes(wasm_bytes: Vec<u8>, config: WasmSandboxConfig) -> Result<Self, WasmError> {
        let runtime = WasmRuntime::new()?;
        let instance = runtime.instantiate(&wasm_bytes, &config)?;
        
        Ok(Self { instance, config })
    }

    pub fn call_method(&mut self, method: &str, payload: &[u8]) -> Result<Vec<u8>, WasmError> {
        self.instance.call_export(method, payload, self.config.time_budget_ms)
    }
}
