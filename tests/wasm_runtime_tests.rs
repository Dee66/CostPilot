// WASM runtime sandbox tests

use costpilot::pro_engine::{WasmRuntime, WasmSandboxConfig, WasmError};

#[test]
fn test_instantiate_simple_module_ok() {
    let wat = r#"
        (module
            (func (export "ping") (result i32)
                i32.const 42
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let runtime = WasmRuntime::new().unwrap();
    let config = WasmSandboxConfig {
        time_budget_ms: 100,
        memory_limit_bytes: 1024 * 1024, // 1MB
    };

    let mut instance = runtime.instantiate(&wasm_bytes, &config).unwrap();
    let result = instance.call_export("ping", &[], 100).unwrap();

    // Decode i32 from little-endian bytes
    let value = i32::from_le_bytes([result[0], result[1], result[2], result[3]]);
    assert_eq!(value, 42);
}

#[test]
fn test_timeout_enforced() {
    let wat = r#"
        (module
            (func (export "busy") (result i32)
                (loop $l
                    br $l
                )
                i32.const 1
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let runtime = WasmRuntime::new().unwrap();
    let config = WasmSandboxConfig {
        time_budget_ms: 1, // Very small budget
        memory_limit_bytes: 1024 * 1024,
    };

    let mut instance = runtime.instantiate(&wasm_bytes, &config).unwrap();
    let result = instance.call_export("busy", &[], 10);

    assert!(matches!(result, Err(WasmError::Timeout)));
}

#[test]
fn test_host_imports_denied() {
    let wat = r#"
        (module
            (import "env" "danger" (func $danger))
            (func (export "call_danger")
                call $danger
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let runtime = WasmRuntime::new().unwrap();
    let config = WasmSandboxConfig::default();

    let result = runtime.instantiate(&wasm_bytes, &config);
    assert!(matches!(result, Err(WasmError::HostImportDenied)));
}

#[test]
fn test_memory_limit_exceeded() {
    let wat = r#"
        (module
            (memory (export "memory") 1)
            (func (export "grow")
                i32.const 1000
                memory.grow
                drop
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat).unwrap();

    let runtime = WasmRuntime::new().unwrap();
    let config = WasmSandboxConfig {
        time_budget_ms: 100,
        memory_limit_bytes: 128 * 1024, // 128KB - too small for growth
    };

    let mut instance = runtime.instantiate(&wasm_bytes, &config).unwrap();
    let result = instance.call_export("grow", &[], 100);

    // Memory growth should fail or trap
    assert!(result.is_err());
}
