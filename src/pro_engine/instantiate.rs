// WASM instantiation for ProEngine

use crate::pro_engine::{ProEngineExecutor, ProEngineHandle, ProEngineRequest, ProEngineResponse};
use std::sync::Mutex;

/// Instantiate WASM module and return executor handle
pub fn instantiate_wasm(bytes: &[u8]) -> Result<ProEngineHandle, String> {
    // Verify bytes are valid WASM
    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        return Err("Invalid WASM magic number".to_string());
    }

    // Use wasmtime to instantiate the WASM module
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_binary(&engine, bytes)
        .map_err(|e| format!("WASM compilation failed: {}", e))?;

    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new(&mut store, &module, &[])
        .map_err(|e| format!("WASM instantiation failed: {}", e))?;

    // Get the exported functions from WASM
    // For wasm-bindgen, functions take &str and return Result<String, String>
    // But wasmtime expects different signatures. For now, we'll use a simplified approach.
    let predict_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "predict")
        .map_err(|e| format!("Function 'predict' not found: {}", e))?;

    let explain_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "explain")
        .map_err(|e| format!("Function 'explain' not found: {}", e))?;

    let autofix_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "autofix")
        .map_err(|e| format!("Function 'autofix' not found: {}", e))?;

    let mapdeep_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "mapdeep")
        .map_err(|e| format!("Function 'mapdeep' not found: {}", e))?;

    let trend_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "trend")
        .map_err(|e| format!("Function 'trend' not found: {}", e))?;

    let enforce_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "enforce")
        .map_err(|e| format!("Function 'enforce' not found: {}", e))?;

    let slo_enforce_fn = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "slo_enforce")
        .map_err(|e| format!("Function 'slo_enforce' not found: {}", e))?;

    // Get memory export for string passing
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or("WASM module must export 'memory'")?;

    let wasm_executor = WasmExecutor {
        store: Mutex::new(store),
        memory,
        predict_fn,
        explain_fn,
        autofix_fn,
        mapdeep_fn,
        trend_fn,
        enforce_fn,
        slo_enforce_fn,
    };

    Ok(ProEngineHandle::new(Box::new(wasm_executor)))
}

struct WasmExecutor {
    store: Mutex<wasmtime::Store<()>>,
    memory: wasmtime::Memory,
    predict_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    explain_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    autofix_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    mapdeep_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    trend_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    enforce_fn: wasmtime::TypedFunc<(i32, i32), i32>,
    slo_enforce_fn: wasmtime::TypedFunc<(i32, i32), i32>,
}

impl ProEngineExecutor for WasmExecutor {
    fn execute(&self, req: ProEngineRequest) -> Result<ProEngineResponse, String> {
        // Serialize request to JSON
        let json_input = serde_json::to_string(&req)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Call appropriate WASM function based on request type
        let result = match req {
            ProEngineRequest::Predict { .. } => {
                self.call_wasm_function(&json_input, &self.predict_fn)
            }
            ProEngineRequest::Explain { .. } => {
                self.call_wasm_function(&json_input, &self.explain_fn)
            }
            ProEngineRequest::Autofix { .. } => {
                self.call_wasm_function(&json_input, &self.autofix_fn)
            }
            ProEngineRequest::MapDeep { .. } => {
                self.call_wasm_function(&json_input, &self.mapdeep_fn)
            }
            ProEngineRequest::TrendSnapshot { .. } => {
                self.call_wasm_function(&json_input, &self.trend_fn)
            }
            ProEngineRequest::PolicyEnforce { .. } => {
                self.call_wasm_function(&json_input, &self.enforce_fn)
            }
            ProEngineRequest::SloEnforce { .. } => {
                self.call_wasm_function(&json_input, &self.slo_enforce_fn)
            }
        }?;

        // Deserialize response from JSON
        let response: ProEngineResponse = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to deserialize response: {}", e))?;

        Ok(response)
    }
}

impl WasmExecutor {
    fn call_wasm_function(
        &self,
        input: &str,
        func: &wasmtime::TypedFunc<(i32, i32), i32>,
    ) -> Result<String, String> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| format!("Mutex lock failed: {}", e))?;

        // Write input directly to WASM memory at fixed location (simulating INPUT_BUFFER)
        let input_bytes = input.as_bytes();
        let input_len = input_bytes.len() as i32;

        if input_bytes.len() > 4096 {
            return Err("Input too large for WASM buffer".to_string());
        }

        // Write input to fixed location in WASM memory (offset 0 for input buffer)
        self.memory
            .write(&mut *store, 0, input_bytes)
            .map_err(|e| format!("Failed to write input to WASM memory: {}", e))?;

        // Call the WASM function with buffer location
        let packed_result = func
            .call(&mut *store, (0, input_len))
            .map_err(|e| format!("WASM function call failed: {}", e))?;

        if packed_result < 0 {
            return Err("WASM function returned error".to_string());
        }

        // Unpack result: upper 16 bits = length, lower 16 bits = offset
        let result_len = (packed_result >> 16) as usize;
        let result_offset = (packed_result & 0xFFFF) as usize;

        // Read result from WASM memory (from OUTPUT_BUFFER at offset 4096)
        let mut result_bytes = vec![0u8; result_len];
        self.memory
            .read(&*store, 4096 + result_offset, &mut result_bytes)
            .map_err(|e| format!("Failed to read result data: {}", e))?;

        String::from_utf8(result_bytes).map_err(|e| format!("Invalid UTF-8 in WASM result: {}", e))
    }

    #[allow(dead_code)]
    fn allocate_memory(&self, _size: i32) -> Result<i32, String> {
        // For this simple implementation, we'll use a fixed memory location
        // In a real implementation, the WASM module should export an allocate function
        // For now, assume memory starts at offset 1024 and we have enough space
        Ok(1024) // Fixed allocation for simplicity
    }
}
