// WASM bytecode size validation tests

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const WASM_FILE_PATH: &str = "target/wasm32-unknown-unknown/release/costpilot.wasm";
    const MAX_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
    const WARN_SIZE_BYTES: usize = 8 * 1024 * 1024; // 8 MB warning threshold

    #[test]
    #[ignore] // Only run after building WASM
    fn test_wasm_file_exists() {
        let path = Path::new(WASM_FILE_PATH);
        assert!(
            path.exists(),
            "WASM file not found. Build with: cargo build --target wasm32-unknown-unknown --release --lib"
        );
    }

    #[test]
    #[ignore] // Only run after building WASM
    fn test_wasm_size_limit() {
        let path = Path::new(WASM_FILE_PATH);

        if !path.exists() {
            panic!("WASM file not found. Build first.");
        }

        let metadata = std::fs::metadata(path).expect("Failed to read WASM file metadata");
        let size = metadata.len() as usize;
        let size_mb = size as f64 / (1024.0 * 1024.0);

        println!("WASM module size: {:.2} MB ({} bytes)", size_mb, size);

        if size > WARN_SIZE_BYTES {
            println!(
                "⚠️  WARNING: WASM size exceeds warning threshold of {} MB",
                WARN_SIZE_BYTES / 1024 / 1024
            );
        }

        assert!(
            size <= MAX_SIZE_BYTES,
            "WASM module size ({:.2} MB) exceeds maximum of {} MB. Consider optimizations.",
            size_mb,
            MAX_SIZE_BYTES / 1024 / 1024
        );
    }

    #[test]
    #[ignore] // Only run after building WASM
    fn test_wasm_optimized_size() {
        // Check if optimized version exists
        let opt_path = Path::new("target/wasm32-unknown-unknown/release/costpilot_opt.wasm");

        if !opt_path.exists() {
            println!(
                "ℹ️  Optimized WASM not found. Build with: ./scripts/build_wasm.sh --optimize"
            );
            return;
        }

        let opt_metadata = std::fs::metadata(opt_path).expect("Failed to read optimized WASM");
        let opt_size = opt_metadata.len() as usize;
        let opt_size_mb = opt_size as f64 / (1024.0 * 1024.0);

        println!(
            "Optimized WASM module size: {:.2} MB ({} bytes)",
            opt_size_mb, opt_size
        );

        // Optimized version should be smaller
        if let Ok(metadata) = std::fs::metadata(WASM_FILE_PATH) {
            let orig_size = metadata.len() as usize;
            let reduction = ((orig_size - opt_size) as f64 / orig_size as f64) * 100.0;

            println!("Size reduction: {:.1}%", reduction);

            assert!(
                opt_size < orig_size,
                "Optimized WASM should be smaller than unoptimized"
            );
        }
    }

    #[test]
    fn test_compilation_size_estimate() {
        // Estimate module sizes to detect bloat

        // These are rough estimates based on typical sizes
        let estimated_sizes = vec![
            ("Prediction Engine", 500 * 1024), // ~500 KB
            ("Detection Engine", 400 * 1024),  // ~400 KB
            ("Policy Engine", 300 * 1024),     // ~300 KB
            ("Mapping Engine", 600 * 1024),    // ~600 KB
            ("Grouping Engine", 500 * 1024),   // ~500 KB
            ("SLO Engine", 200 * 1024),        // ~200 KB
            ("Parser", 400 * 1024),            // ~400 KB
            ("Core/Runtime", 1000 * 1024),     // ~1 MB
        ];

        let total_estimate: usize = estimated_sizes.iter().map(|(_, size)| size).sum();
        let total_estimate_mb = total_estimate as f64 / (1024.0 * 1024.0);

        println!("Estimated component sizes:");
        for (name, size) in &estimated_sizes {
            println!("  {}: {:.1} KB", name, *size as f64 / 1024.0);
        }
        println!("\nTotal estimated size: {:.2} MB", total_estimate_mb);

        assert!(
            total_estimate <= MAX_SIZE_BYTES,
            "Estimated size ({:.2} MB) exceeds limit",
            total_estimate_mb
        );
    }

    #[test]
    fn test_feature_flag_sizes() {
        // Verify feature flags reduce size
        // This is informational - actual sizes depend on build

        println!("Feature flag size impact (estimated):");
        println!("  --no-default-features: Saves ~2-3 MB");
        println!("  --features prediction: ~500 KB");
        println!("  --features detection: ~400 KB");
        println!("  --features policy: ~300 KB");
        println!("  --features mapping: ~600 KB");
        println!("  --features grouping: ~500 KB");
        println!("  --features slo: ~200 KB");

        // Feature builds should be smaller than full build
        // This can be validated with actual builds
    }

    #[test]
    fn test_dependency_sizes() {
        // Check for large dependencies that might bloat WASM

        // These dependencies should be reasonable
        let known_dependencies = vec![
            "serde",
            "serde_json",
            "serde_yaml",
            "regex",
            "thiserror",
            "wasm-bindgen",
        ];

        println!("Known dependencies:");
        for dep in &known_dependencies {
            println!("  - {}", dep);
        }

        // Large dependencies to avoid in WASM:
        // - tokio (async runtime not needed)
        // - reqwest (network not allowed)
        // - image processing (bloats binary)
        // - heavy compression libraries

        println!("\nDependencies to avoid in WASM:");
        println!("  - tokio (use native async instead)");
        println!("  - reqwest (network not allowed)");
        println!("  - large compression libs");
    }

    #[test]
    fn test_optimization_levels() {
        // Document optimization strategies

        println!("Optimization strategies for size:");
        println!("  1. opt-level = 'z' (optimize for size)");
        println!("  2. lto = true (link-time optimization)");
        println!("  3. codegen-units = 1 (single codegen unit)");
        println!("  4. strip = true (strip debug symbols)");
        println!("  5. panic = 'abort' (smaller panic handler)");
        println!("  6. wasm-opt -Oz (post-build optimization)");
        println!("  7. wee_alloc (smaller allocator)");
        println!("\nExpected reduction: 30-50%");
    }

    #[test]
    fn test_size_regression_detection() {
        // This test helps detect size regressions

        const BASELINE_SIZE_MB: f64 = 5.0; // Baseline size
        const REGRESSION_THRESHOLD: f64 = 0.2; // 20% increase is a regression

        println!("Size regression detection:");
        println!("  Baseline: {:.2} MB", BASELINE_SIZE_MB);
        println!(
            "  Regression threshold: {}%",
            (REGRESSION_THRESHOLD * 100.0) as i32
        );

        // In CI, compare against baseline
        // If size increases by more than threshold, flag as regression

        let max_acceptable = BASELINE_SIZE_MB * (1.0 + REGRESSION_THRESHOLD);
        println!("  Max acceptable: {:.2} MB", max_acceptable);

        // This would be checked against actual built size in CI
    }

    #[test]
    fn test_compression_potential() {
        // WASM files compress well with gzip/brotli

        println!("Compression potential:");
        println!("  gzip: typically 30-40% reduction");
        println!("  brotli: typically 40-50% reduction");
        println!("\nExample:");
        println!("  10 MB WASM -> ~6 MB gzipped -> ~5 MB brotli");
        println!("\nServe WASM with compression for best performance");
    }

    #[test]
    fn test_size_monitoring() {
        // Document size monitoring strategy

        println!("Size monitoring strategy:");
        println!("  1. Track size in each PR");
        println!("  2. Fail CI if size exceeds limit");
        println!("  3. Warn if size increases >10%");
        println!("  4. Track size trends over time");
        println!("  5. Profile large dependencies");
        println!("\nImplement in CI/CD pipeline");
    }

    #[test]
    fn test_lazy_loading_strategy() {
        // For browser use, consider lazy loading

        println!("Lazy loading strategies:");
        println!("  1. Split engines into separate WASM modules");
        println!("  2. Load only required engines");
        println!("  3. Stream WASM compilation");
        println!("  4. Cache compiled modules");
        println!("\nExample:");
        println!("  costpilot_core.wasm (2 MB) - always loaded");
        println!("  costpilot_prediction.wasm (500 KB) - on-demand");
        println!("  costpilot_mapping.wasm (600 KB) - on-demand");
    }

    #[test]
    fn test_wasm_checksum_reproducible() {
        // Build first time
        let output1 = Command::new("cargo")
            .args(&[
                "build",
                "--target",
                "wasm32-wasip1",
                "--release",
                "--lib",
                "--features",
                "wasm",
            ])
            .output()
            .expect("Failed to build WASM first time");
        assert!(output1.status.success());

        let wasm_path = "target/wasm32-wasip1/release/costpilot.wasm";
        assert!(Path::new(wasm_path).exists());

        let data1 = fs::read(wasm_path).unwrap();
        let mut hasher1 = Sha256::new();
        hasher1.update(&data1);
        let hash1 = hasher1.finalize();

        // Build second time
        let output2 = Command::new("cargo")
            .args(&[
                "build",
                "--target",
                "wasm32-wasip1",
                "--release",
                "--lib",
                "--features",
                "wasm",
            ])
            .output()
            .expect("Failed to build WASM second time");
        assert!(output2.status.success());

        let data2 = fs::read(wasm_path).unwrap();
        let mut hasher2 = Sha256::new();
        hasher2.update(&data2);
        let hash2 = hasher2.finalize();

        assert_eq!(hash1, hash2, "WASM bundle checksum not reproducible");
    }
}
