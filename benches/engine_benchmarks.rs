// CostPilot Benchmark Suite
//
// Performance benchmarks for all engines using Criterion
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// CLI End-to-End Benchmarks (No Cost)
// ============================================================================

fn bench_cli_scan_basic(c: &mut Criterion) {
    // Create a temporary terraform plan file for benchmarking
    let temp_dir = tempfile::TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("bench_plan.json");

    // Use the sample terraform plan from e2e tests
    let plan_content = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-0c55b159cbfafe1f0"
                    }
                }
            }
        ],
        "configuration": {
            "root_module": {}
        }
    }"#;

    std::fs::write(&plan_path, plan_content).unwrap();

    c.bench_function("cli_scan_basic", |b| {
        b.iter(|| {
            let mut cmd = Command::new("cargo");
            cmd.args([
                "run",
                "--bin",
                "costpilot",
                "--quiet",
                "--",
                "scan",
                &plan_path.to_string_lossy(),
                "--format",
                "json",
            ]);

            let output = cmd.output().unwrap();
            assert!(output.status.success());

            // Verify we get expected JSON output
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("aws_instance.web"));
            black_box(stdout);
        });
    });
}

fn bench_cli_scan_multi_resource(c: &mut Criterion) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("bench_multi_plan.json");

    // Multi-resource plan for more complex benchmarking
    let plan_content = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-0c55b159cbfafe1f0"
                    }
                }
            },
            {
                "address": "aws_nat_gateway.main",
                "mode": "managed",
                "type": "aws_nat_gateway",
                "name": "main",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "subnet_id": "subnet-12345",
                        "connectivity_type": "public"
                    }
                }
            },
            {
                "address": "aws_s3_bucket.data",
                "mode": "managed",
                "type": "aws_s3_bucket",
                "name": "data",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "bucket": "my-data-bucket"
                    }
                }
            }
        ],
        "configuration": {
            "root_module": {}
        }
    }"#;

    std::fs::write(&plan_path, plan_content).unwrap();

    c.bench_function("cli_scan_multi_resource", |b| {
        b.iter(|| {
            let mut cmd = Command::new("cargo");
            cmd.args([
                "run",
                "--bin",
                "costpilot",
                "--quiet",
                "--",
                "scan",
                &plan_path.to_string_lossy(),
                "--format",
                "json",
            ]);

            let output = cmd.output().unwrap();
            assert!(output.status.success());

            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("aws_instance.web"));
            assert!(stdout.contains("aws_nat_gateway.main"));
            assert!(stdout.contains("aws_s3_bucket.data"));
            black_box(stdout);
        });
    });
}

fn bench_cli_scan_with_policy(c: &mut Criterion) {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let plan_path = temp_dir.path().join("bench_plan.json");
    let policy_path = temp_dir.path().join("bench_policy.yml");

    // Simple plan
    let plan_content = r#"{
        "format_version": "1.0",
        "terraform_version": "1.5.0",
        "resource_changes": [
            {
                "address": "aws_instance.web",
                "mode": "managed",
                "type": "aws_instance",
                "name": "web",
                "change": {
                    "actions": ["create"],
                    "before": null,
                    "after": {
                        "instance_type": "t3.medium",
                        "ami": "ami-0c55b159cbfafe1f0"
                    }
                }
            }
        ],
        "configuration": {
            "root_module": {}
        }
    }"#;

    // Simple policy
    let policy_content = r#"version: "1.0"
policies:
  - name: "Instance Type Restrictions"
    rule: "instance_type in ['t3.micro', 't3.small', 't3.medium']"
    action: warn
    severity: MEDIUM
    resources:
      - aws_instance
"#;

    std::fs::write(&plan_path, plan_content).unwrap();
    std::fs::write(&policy_path, policy_content).unwrap();

    c.bench_function("cli_scan_with_policy", |b| {
        b.iter(|| {
            let mut cmd = Command::new("cargo");
            cmd.args([
                "run",
                "--bin",
                "costpilot",
                "--quiet",
                "--",
                "scan",
                &plan_path.to_string_lossy(),
                "--policy",
                &policy_path.to_string_lossy(),
                "--format",
                "json",
            ]);

            let output = cmd.output().unwrap();
            assert!(output.status.success());

            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("aws_instance.web"));
            black_box(stdout);
        });
    });
}

fn bench_cli_init(c: &mut Criterion) {
    c.bench_function("cli_init", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let init_path = temp_dir.path().join("bench_project");

            let mut cmd = Command::new("cargo");
            cmd.args([
                "run",
                "--bin",
                "costpilot",
                "--quiet",
                "--",
                "init",
                "--path",
                &init_path.to_string_lossy(),
                "--no-ci",
            ]);

            let output = cmd.output().unwrap();
            assert!(output.status.success());

            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("CostPilot initialized"));
            black_box(stdout);
        });
    });
}

fn bench_cli_validate(c: &mut Criterion) {
    c.bench_function("cli_validate", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let config_path = temp_dir.path().join("bench_config.yml");

            let config_content = r#"version: 1.0.0
detection:
  enabled: true
prediction:
  enabled: true
"#;

            std::fs::write(&config_path, config_content).unwrap();

            let mut cmd = Command::new("cargo");
            cmd.args([
                "run",
                "--bin",
                "costpilot",
                "--quiet",
                "--",
                "validate",
                &config_path.to_string_lossy(),
            ]);

            let output = cmd.output().unwrap();
            // Validation may fail but command should run
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(stdout.contains("Validation Report"));
            black_box(stdout);
        });
    });
}

// ============================================================================
// Prediction Engine Benchmarks
// ============================================================================

fn bench_prediction_single_ec2(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();
    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();

    c.bench_function("predict_ec2_t3_medium", |b| {
        b.iter(|| {
            let _total_cost = prediction_engine
                .predict_total_cost(black_box(&changes))
                .unwrap();
        });
    });
}

fn bench_prediction_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("prediction_batch");

    // Create multiple copies of the same change for batch testing
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let single_change = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();
    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();

    for size in [10, 100, 1000].iter() {
        let changes: Vec<_> = (0..*size).flat_map(|_| single_change.clone()).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _size| {
            b.iter(|| {
                let _total_cost = prediction_engine
                    .predict_total_cost(black_box(&changes))
                    .unwrap();
            });
        });
    }

    group.finish();
}

// ============================================================================
// Detection Engine Benchmarks
// ============================================================================

fn bench_detection_parse_plan(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");

    c.bench_function("parse_terraform_plan", |b| {
        b.iter(|| {
            let detection_engine = costpilot::engines::detection::DetectionEngine::new();
            let _changes = detection_engine
                .detect_from_terraform_plan(black_box(&plan_path))
                .unwrap();
        });
    });
}

fn create_large_plan(base_plan: serde_json::Value, multiplier: usize) -> serde_json::Value {
    let mut large_plan = base_plan.clone();

    if let Some(resources) = base_plan
        .get("planned_values")
        .and_then(|pv| pv.get("root_module"))
        .and_then(|rm| rm.get("resources"))
        .and_then(|r| r.as_array())
    {
        let mut large_resources = Vec::new();
        for i in 0..multiplier {
            for resource in resources {
                let mut new_resource = resource.clone();
                if let Some(obj) = new_resource.as_object_mut() {
                    if let Some(address) = obj.get_mut("address") {
                        if let Some(addr_str) = address.as_str() {
                            *address = serde_json::Value::String(format!("{}_{}", addr_str, i));
                        }
                    }
                }
                large_resources.push(new_resource);
            }
        }

        if let Some(planned_values) = large_plan.get_mut("planned_values") {
            if let Some(root_module) = planned_values.get_mut("root_module") {
                if let Some(resources) = root_module.get_mut("resources") {
                    *resources = serde_json::Value::Array(large_resources);
                }
            }
        }
    }

    large_plan
}

fn bench_detection_large_plan(c: &mut Criterion) {
    // Create a synthetic large plan by duplicating resources
    let base_plan: serde_json::Value = {
        let content = std::fs::read_to_string("tests/fixtures/terraform/ec2_create.json").unwrap();
        serde_json::from_str(&content).unwrap()
    };

    let large_plan = create_large_plan(base_plan, 10); // Reduced multiplier for benchmark
    let large_plan_json = serde_json::to_string(&large_plan).unwrap();

    c.bench_function("parse_100_resources", |b| {
        b.iter(|| {
            let detection_engine = costpilot::engines::detection::DetectionEngine::new();
            let _changes = detection_engine
                .detect_from_terraform_json(black_box(&large_plan_json))
                .unwrap();
        });
    });
}

// ============================================================================
// Policy Engine Benchmarks
// ============================================================================

fn bench_policy_evaluation(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();
    let mut prediction_engine = costpilot::engines::prediction::PredictionEngine::new().unwrap();
    let total_cost_summary = prediction_engine.predict_total_cost(&changes).unwrap();

    let total_cost = costpilot::engines::shared::models::CostEstimate {
        resource_id: "total".to_string(),
        monthly_cost: total_cost_summary.monthly,
        prediction_interval_low: total_cost_summary.prediction_interval_low,
        prediction_interval_high: total_cost_summary.prediction_interval_high,
        confidence_score: total_cost_summary.confidence_score,
        heuristic_reference: None,
        cold_start_inference: false,
        one_time: None,
        breakdown: None,
        hourly: None,
        daily: None,
    };

    // Create a simple policy config for benchmarking
    let policy_config = costpilot::engines::policy::PolicyConfig {
        version: "1.0".to_string(),
        metadata: Default::default(),
        budgets: Default::default(),
        resources: Default::default(),
        slos: vec![],
        enforcement: costpilot::engines::policy::EnforcementConfig {
            mode: "advisory".to_string(),
            fail_on_violation: false,
        },
    };

    let edition = costpilot::edition::EditionContext::free();
    let policy_engine = costpilot::engines::policy::PolicyEngine::new(policy_config, &edition);

    c.bench_function("policy_evaluation", |b| {
        b.iter(|| {
            let _result = policy_engine.evaluate(black_box(&changes), black_box(&total_cost));
        });
    });
}

// ============================================================================
// Mapping Engine Benchmarks
// ============================================================================

fn bench_mapping_build_graph(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();

    let edition = costpilot::edition::EditionContext::free();

    c.bench_function("build_dependency_graph", |b| {
        b.iter(|| {
            let mut mapping_engine =
                costpilot::engines::mapping::MappingEngine::new(black_box(&edition));
            let _graph = mapping_engine.build_graph(black_box(&changes)).unwrap();
        });
    });
}

fn bench_mapping_graph_analysis(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");
    let detection_engine = costpilot::engines::detection::DetectionEngine::new();
    let changes = detection_engine
        .detect_from_terraform_plan(&plan_path)
        .unwrap();

    let edition = costpilot::edition::EditionContext::free();
    let mut mapping_engine = costpilot::engines::mapping::MappingEngine::new(&edition);
    let graph = mapping_engine.build_graph(&changes).unwrap();

    c.bench_function("graph_node_analysis", |b| {
        b.iter(|| {
            // Simple graph analysis benchmark
            let _node_count = black_box(&graph).nodes.len();
            let _edge_count = black_box(&graph).edges.len();
        });
    });
}

// ============================================================================
// Full Scan Pipeline Benchmarks
// ============================================================================

fn bench_full_scan_pipeline(c: &mut Criterion) {
    let plan_path = PathBuf::from("tests/fixtures/terraform/ec2_create.json");

    c.bench_function("full_scan_pipeline", |b| {
        b.iter(|| {
            // Simulate the full scan pipeline: detection + prediction + policy evaluation
            let detection_engine = costpilot::engines::detection::DetectionEngine::new();
            let changes = detection_engine
                .detect_from_terraform_plan(black_box(&plan_path))
                .unwrap();

            let mut prediction_engine =
                costpilot::engines::prediction::PredictionEngine::new().unwrap();
            let total_cost_summary = prediction_engine
                .predict_total_cost(black_box(&changes))
                .unwrap();

            let total_cost = costpilot::engines::shared::models::CostEstimate {
                resource_id: "total".to_string(),
                monthly_cost: total_cost_summary.monthly,
                prediction_interval_low: total_cost_summary.prediction_interval_low,
                prediction_interval_high: total_cost_summary.prediction_interval_high,
                confidence_score: total_cost_summary.confidence_score,
                heuristic_reference: None,
                cold_start_inference: false,
                one_time: None,
                breakdown: None,
                hourly: None,
                daily: None,
            };

            // Simple policy evaluation
            let policy_config = costpilot::engines::policy::PolicyConfig {
                version: "1.0".to_string(),
                metadata: Default::default(),
                budgets: Default::default(),
                resources: Default::default(),
                slos: vec![],
                enforcement: costpilot::engines::policy::EnforcementConfig {
                    mode: "advisory".to_string(),
                    fail_on_violation: false,
                },
            };

            let edition = costpilot::edition::EditionContext::free();
            let policy_engine =
                costpilot::engines::policy::PolicyEngine::new(policy_config, &edition);

            let _policy_result =
                policy_engine.evaluate(black_box(&changes), black_box(&total_cost));
        });
    });
}

// ============================================================================
// Test Benchmark
// ============================================================================

fn bench_test(c: &mut Criterion) {
    c.bench_function("test_benchmark", |b| {
        b.iter(|| {
            let _result = black_box(42 + 1);
        });
    });
}

criterion_group!(test_benches, bench_test);

criterion_group!(
    prediction_benches,
    bench_prediction_single_ec2,
    bench_prediction_batch
);

criterion_group!(
    detection_benches,
    bench_detection_parse_plan,
    bench_detection_large_plan
);

criterion_group!(policy_benches, bench_policy_evaluation);

criterion_group!(
    mapping_benches,
    bench_mapping_build_graph,
    bench_mapping_graph_analysis
);

criterion_group!(pipeline_benches, bench_full_scan_pipeline);

criterion_group!(
    cli_benches,
    bench_cli_scan_basic,
    bench_cli_scan_multi_resource,
    bench_cli_scan_with_policy,
    bench_cli_init,
    bench_cli_validate
);

criterion_main!(
    cli_benches,
    test_benches,
    prediction_benches,
    detection_benches,
    policy_benches,
    mapping_benches,
    pipeline_benches
);
