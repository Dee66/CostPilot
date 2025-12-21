#![cfg(not(target_arch = "wasm32"))]

// WASM runtime with strict sandboxing and resource limits

use wasmtime::*;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct WasmSandboxConfig {
    pub time_budget_ms: u64,
    pub memory_limit_bytes: usize,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            time_budget_ms: 300,
            memory_limit_bytes: 64 * 1024 * 1024, // 64MB
        }
    }
}

#[derive(Debug)]
pub enum WasmError {
    CompileError(String),
    InstantiateError(String),
    Timeout,
    MemoryLimitExceeded,
    HostImportDenied,
    CallError(String),
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmError::CompileError(e) => write!(f, "Compile error: {}", e),
            WasmError::InstantiateError(e) => write!(f, "Instantiate error: {}", e),
            WasmError::Timeout => write!(f, "Timeout"),
            WasmError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            WasmError::HostImportDenied => write!(f, "Host import denied"),
            WasmError::CallError(e) => write!(f, "Call error: {}", e),
        }
    }
}

impl std::error::Error for WasmError {}

pub struct WasmRuntime {
    engine: Engine,
}

pub struct SandboxInstance {
    store: Store<ResourceState>,
    instance: Instance,
}

struct ResourceState {
    memory_limit_bytes: usize,
}

impl ResourceLimiter for ResourceState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(desired <= self.memory_limit_bytes)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        Ok(desired <= 1000)
    }
}

impl WasmRuntime {
    pub fn new() -> Result<Self, WasmError> {
        let mut config = Config::new();
        config.epoch_interruption(true);
        config.wasm_threads(false);
        config.wasm_multi_memory(false);
        config.wasm_bulk_memory(true);
        config.static_memory_maximum_size(256 * 1024 * 1024); // 256MB absolute max

        let engine = Engine::new(&config)
            .map_err(|e| WasmError::CompileError(e.to_string()))?;

        Ok(Self { engine })
    }

    pub fn instantiate(
        &self,
        wasm_bytes: &[u8],
        config: &WasmSandboxConfig,
    ) -> Result<SandboxInstance, WasmError> {
        // Compile module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::CompileError(e.to_string()))?;

        // Check imports - deny all by default
        for _import in module.imports() {
            return Err(WasmError::HostImportDenied);
        }

        // Create store with resource limits
        let state = ResourceState {
            memory_limit_bytes: config.memory_limit_bytes,
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|s| s);

        // Setup epoch for timeout
        store.set_epoch_deadline(1);

        // Instantiate with no imports
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| WasmError::InstantiateError(e.to_string()))?;

        Ok(SandboxInstance {
            store,
            instance,
        })
    }
}

impl SandboxInstance {
    pub fn call_export(
        &mut self,
        func_name: &str,
        _input: &[u8],
        timeout_ms: u64,
    ) -> Result<Vec<u8>, WasmError> {
        // Get exported function
        let func = self
            .instance
            .get_func(&mut self.store, func_name)
            .ok_or_else(|| WasmError::CallError(format!("Function '{}' not found", func_name)))?;

        // Setup timeout using epoch
        let engine = self.store.engine().clone();
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms);

        // Spawn timeout thread
        let timeout_handle = std::thread::spawn(move || {
            std::thread::sleep(timeout_duration);
            engine.increment_epoch();
        });

        // Call function (assume no-arg i32 return for tests)
        let mut results = vec![Val::I32(0)];
        let call_result = func.call(&mut self.store, &[], &mut results);

        // Check if we timed out
        let elapsed = start.elapsed();
        drop(timeout_handle);

        match call_result {
            Ok(_) => {
                if let Val::I32(val) = results[0] {
                    Ok(val.to_le_bytes().to_vec())
                } else {
                    Ok(vec![])
                }
            }
            Err(trap) => {
                let trap_str = trap.to_string();
                if trap_str.contains("interrupt")
                    || trap_str.contains("epoch")
                    || elapsed >= timeout_duration {
                    Err(WasmError::Timeout)
                } else {
                    Err(WasmError::CallError(trap_str))
                }
            }
        }
    }
}
