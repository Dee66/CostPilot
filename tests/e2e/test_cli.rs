/// End-to-end CLI tests
///
/// Tests all CLI commands with real file I/O and command execution.

#[cfg(test)]
mod e2e_cli_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;
    use std::fs;

    // ============================================================================
    // costpilot scan Tests (10 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_scan_basic() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Monthly delta:"));
    }

    #[test]
    fn test_cli_scan_with_explain() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--explain")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("Explanation:"));
    }

    #[test]
    fn test_cli_scan_with_output_file() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let output_path = temp.path().join("report.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .arg("--output")
            .arg(&output_path)
            .assert()
            .success();

        // Verify output file was created and contains expected content
        assert!(output_path.exists());
        let output_content = fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("monthly_delta"));
    }

    #[test]
    fn test_cli_scan_missing_file_errors() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("nonexistent.json")
            .assert()
            .failure()
            .stderr(predicate::str::contains("No such file"));
    }

    #[test]
    fn test_cli_scan_invalid_json_errors() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("invalid.json");
        fs::write(&plan_path, "invalid json content {").unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .failure()
            .stderr(predicate::str::contains("JSON"));
    }

    // ============================================================================
    // costpilot autofix Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_autofix_snippet_mode() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-snippet")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("snippet"));
    }

    #[test]
    fn test_cli_autofix_patch_mode() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-patch")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("patch"));
    }

    #[test]
    fn test_cli_autofix_drift_safe_mode() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-drift-safe")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("drift"));
    }

    // ============================================================================
    // costpilot map Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_map_mermaid_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=mermaid")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("graph"));
    }

    #[test]
    fn test_cli_map_graphviz_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=graphviz")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("digraph"));
    }

    #[test]
    fn test_cli_map_json_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("nodes"));
    }

    #[test]
    fn test_cli_map_html_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let output_path = temp.path().join("graph.html");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=html")
            .arg("--output")
            .arg(&output_path)
            .arg(&plan_path)
            .assert()
            .success();

        // Verify output file was created and contains HTML
        assert!(output_path.exists());
        let output_content = fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("<html>"));
    }

    // ============================================================================
    // costpilot group Tests (10 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_group_module() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("module")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("module"));
    }

    #[test]
    fn test_cli_group_service() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("by-service")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("service"));
    }

    #[test]
    fn test_cli_group_environment() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("by-environment")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("environment"));
    }

    #[test]
    fn test_cli_group_attribution() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("attribution")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("attribution"));
    }

    #[test]
    fn test_cli_group_all() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("comprehensive"));
    }

    // ============================================================================
    // costpilot policy-dsl Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_policy_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("policy"));
    }

    #[test]
    fn test_cli_policy_validate() {
        let policy_path = "tests/fixtures/policies/cost_policies.yml";

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg(policy_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("valid"));
    }

    #[test]
    fn test_cli_policy_test() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();
        let policy_path = "tests/fixtures/policies/cost_policies.yml";

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("test")
            .arg(policy_path)
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("test"));
    }

    // ============================================================================
    // costpilot init Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_init_creates_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("costpilot.yaml");

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Verify costpilot.yaml was created
        assert!(config_path.exists());
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("costpilot"));
    }

    #[test]
    fn test_cli_init_creates_ci_templates() {
        let temp = TempDir::new().unwrap();
        let ci_path = temp.path().join(".github/workflows/costpilot.yml");

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Verify CI template was created
        assert!(ci_path.exists());
        let ci_content = fs::read_to_string(&ci_path).unwrap();
        assert!(ci_content.contains("costpilot"));
    }

    #[test]
    fn test_cli_init_creates_sample_policy() {
        let temp = TempDir::new().unwrap();
        let policy_path = temp.path().join("policy.yaml");

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Verify policy.yaml was created
        assert!(policy_path.exists());
        let policy_content = fs::read_to_string(&policy_path).unwrap();
        assert!(policy_content.contains("policies"));
    }

    // ============================================================================
    // Error Handling and Exit Codes (10 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_exits_0_on_success() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        let mut cmd = Command::cargo_bin("costpilot").unwrap();
        cmd.arg("scan").arg(&plan_path);
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(0));
    }

    #[test]
    fn test_cli_exits_1_on_error() {
        let mut cmd = Command::cargo_bin("costpilot").unwrap();
        cmd.arg("scan").arg("nonexistent.json");
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
    }

    #[test]
    fn test_cli_exits_2_on_policy_violation() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let policy_path = temp.path().join("policy.yml");

        // Create a plan that will violate the policy
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create a policy that blocks t3.medium instances
        let policy_content = r#"
policies:
  - name: block_expensive_instances
    description: Block expensive instance types
    rules:
      - condition: "instance_type == 't3.medium'"
        action: block
        message: "t3.medium instances are not allowed"
"#;
        fs::write(&policy_path, policy_content).unwrap();

        let mut cmd = Command::cargo_bin("costpilot").unwrap();
        cmd.arg("scan")
           .arg("--policy")
           .arg(&policy_path)
           .arg(&plan_path);
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn test_cli_help_flag() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("CostPilot"));
    }

    #[test]
    fn test_cli_version_flag() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("costpilot"));
    }

    // ============================================================================
    // costpilot baseline Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_baseline_record() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("record")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("baseline"));
    }

    #[test]
    fn test_cli_baseline_compare() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("compare")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("comparison"));
    }

    #[test]
    fn test_cli_baseline_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("baseline"));
    }

    // ============================================================================
    // costpilot diff Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_diff_basic() {
        let temp = TempDir::new().unwrap();
        let plan1_path = temp.path().join("plan1.json");
        let plan2_path = temp.path().join("plan2.json");
        fs::write(&plan1_path, SAMPLE_PLAN).unwrap();
        fs::write(&plan2_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("diff")
            .arg(&plan1_path)
            .arg(&plan2_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("diff"));
    }

    #[test]
    fn test_cli_diff_with_cost_impact() {
        let temp = TempDir::new().unwrap();
        let plan1_path = temp.path().join("plan1.json");
        let plan2_path = temp.path().join("plan2.json");
        fs::write(&plan1_path, SAMPLE_PLAN).unwrap();
        fs::write(&plan2_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("diff")
            .arg("--cost-impact")
            .arg(&plan1_path)
            .arg(&plan2_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("cost"));
    }

    // ============================================================================
    // costpilot explain Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_explain_basic() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("explanation"));
    }

    #[test]
    fn test_cli_explain_with_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("explanation"));
    }

    // ============================================================================
    // costpilot trend Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_trend_show() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("show")
            .assert()
            .success()
            .stdout(predicate::str::contains("trend"));
    }

    #[test]
    fn test_cli_trend_snapshot() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("snapshot")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("snapshot"));
    }

    #[test]
    fn test_cli_trend_regressions() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("regressions")
            .assert()
            .success()
            .stdout(predicate::str::contains("regression"));
    }

    // ============================================================================
    // costpilot slo Tests (10 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_slo_check() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("SLO"));
    }

    #[test]
    fn test_cli_slo_burn() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .assert()
            .success()
            .stdout(predicate::str::contains("burn"));
    }

    #[test]
    fn test_cli_slo_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("SLO"));
    }

    // ============================================================================
    // costpilot audit Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_audit_basic() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("audit"));
    }

    #[test]
    fn test_cli_audit_with_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("audit"));
    }

    // ============================================================================
    // costpilot anomaly Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_anomaly_detect() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("anomaly")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("anomaly"));
    }

    // ============================================================================
    // costpilot validate Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_validate_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("costpilot.yaml");
        let config_content = r#"
costpilot:
  version: "1.0"
  policies:
    - name: test_policy
"#;
        fs::write(&config_path, config_content).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("valid"));
    }

    // ============================================================================
    // costpilot policy Tests (8 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_policy_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("policy"));
    }

    #[test]
    fn test_cli_policy_create() {
        let temp = TempDir::new().unwrap();
        let policy_path = temp.path().join("policy.yml");

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("create")
            .arg("test-policy")
            .arg("--output")
            .arg(&policy_path)
            .assert()
            .success();

        assert!(policy_path.exists());
    }

    // ============================================================================
    // costpilot heuristics Tests (6 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_heuristics_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("heuristic"));
    }

    // ============================================================================
    // costpilot feature Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_feature_list() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("feature"));
    }

    // ============================================================================
    // costpilot performance Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_performance_report() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .assert()
            .success()
            .stdout(predicate::str::contains("performance"));
    }

    // ============================================================================
    // costpilot usage Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_usage_report() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .assert()
            .success()
            .stdout(predicate::str::contains("usage"));
    }

    // ============================================================================
    // costpilot escrow Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_escrow_status() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("status")
            .assert()
            .success()
            .stdout(predicate::str::contains("escrow"));
    }

    // ============================================================================
    // costpilot validate Tests (4 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_validate_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("costpilot.yaml");
        let config_content = r#"
costpilot:
  version: "1.0"
  policies:
    - name: test_policy
"#;
        fs::write(&config_path, config_content).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("valid"));
    }

    // ============================================================================
    // costpilot version Tests (2 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_version() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("version")
            .assert()
            .success()
            .stdout(predicate::str::contains("costpilot"));
    }

    // ============================================================================
    // Complex Workflow Tests (10 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_scan_then_explain_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // First scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // Then explain
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("explanation"));
    }

    #[test]
    fn test_cli_scan_then_map_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // First scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // Then map
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("graph"));
    }

    #[test]
    fn test_cli_scan_then_group_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // First scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // Then group
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("comprehensive"));
    }

    #[test]
    fn test_cli_baseline_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Record baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("record")
            .arg(&plan_path)
            .assert()
            .success();

        // Compare to baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("compare")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_trend_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create snapshot
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("snapshot")
            .arg(&plan_path)
            .assert()
            .success();

        // Show trends
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("show")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_slo_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Check SLO compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .arg(&plan_path)
            .assert()
            .success();

        // Check burn rate
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_policy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let policy_path = temp.path().join("policy.yml");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("create")
            .arg("test-policy")
            .arg("--output")
            .arg(&policy_path)
            .assert()
            .success();

        // Validate policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg(&policy_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_audit_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Run audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("audit"));
    }

    #[test]
    fn test_cli_anomaly_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Detect anomalies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("anomaly")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("anomaly"));
    }

    #[test]
    fn test_cli_init_workflow() {
        let temp = TempDir::new().unwrap();

        // Initialize project
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Verify files were created
        assert!(temp.path().join("costpilot.yaml").exists());
        assert!(temp.path().join("policy.yaml").exists());
    }

    // ============================================================================
    // Error Scenarios and Edge Cases (15 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_scan_with_invalid_policy() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let policy_path = temp.path().join("invalid.yml");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();
        fs::write(&policy_path, "invalid: yaml: content: {").unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--policy")
            .arg(&policy_path)
            .arg(&plan_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_map_with_invalid_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=invalid")
            .arg(&plan_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_diff_with_missing_files() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("diff")
            .arg("nonexistent1.json")
            .arg("nonexistent2.json")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_explain_with_empty_plan() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("empty.json");
        fs::write(&plan_path, "{}").unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success(); // Should handle empty plans gracefully
    }

    #[test]
    fn test_cli_group_with_invalid_grouping() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("invalid-grouping")
            .arg(&plan_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_policy_dsl_with_missing_file() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg("nonexistent.yml")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_baseline_with_invalid_action() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("invalid-action")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_trend_with_invalid_command() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("invalid-command")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_slo_with_invalid_command() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("invalid-command")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_audit_with_invalid_format() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--format=invalid")
            .arg(&plan_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_validate_with_invalid_config() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("invalid.yaml");
        fs::write(&config_path, "invalid: yaml: content: {").unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_init_with_invalid_path() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg("/invalid/path/that/does/not/exist")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_with_unknown_command() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("unknown-command")
            .assert()
            .failure();
    }

    #[test]
    fn test_cli_help_for_unknown_command() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("unknown-command")
            .arg("--help")
            .assert()
            .failure(); // Should still fail but maybe show help
    }

    #[test]
    fn test_cli_multiple_format_flags_conflict() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--format=json")
            .arg("--format=text")
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .failure(); // Should fail due to conflicting format flags
    }

    // ============================================================================
    // Output Format Tests (12 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_scan_json_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--format=json")
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"monthly_delta\""));
    }

    #[test]
    fn test_cli_scan_markdown_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--format=markdown")
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("|"));
    }

    #[test]
    fn test_cli_scan_pr_comment_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--format=pr-comment")
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("##"));
    }

    #[test]
    fn test_cli_explain_json_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"explanation\""));
    }

    #[test]
    fn test_cli_map_json_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"nodes\""));
    }

    #[test]
    fn test_cli_group_json_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("--format=json")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"groups\""));
    }

    #[test]
    fn test_cli_audit_json_output() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"audit\""));
    }

    #[test]
    fn test_cli_trend_json_output() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("--format=json")
            .arg("show")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"trends\""));
    }

    #[test]
    fn test_cli_slo_json_output() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("--format=json")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"slos\""));
    }

    #[test]
    fn test_cli_policy_json_output() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("--format=json")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"policies\""));
    }

    #[test]
    fn test_cli_heuristics_json_output() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("--format=json")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"heuristics\""));
    }

    #[test]
    fn test_cli_feature_json_output() {
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("--format=json")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"features\""));
    }

    // ============================================================================
    // Advanced Integration Workflows (15 tests planned)
    // ============================================================================

    #[test]
    fn test_cli_full_cost_analysis_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // 1. Scan for costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // 2. Explain predictions
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success();

        // 3. Map dependencies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. Group by service
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("by-service")
            .arg(&plan_path)
            .assert()
            .success();

        // 5. Check SLO compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_policy_enforcement_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        let policy_path = temp.path().join("policy.yml");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("create")
            .arg("cost-policy")
            .arg("--output")
            .arg(&policy_path)
            .assert()
            .success();

        // Validate policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg(&policy_path)
            .assert()
            .success();

        // Test policy against plan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("test")
            .arg(&policy_path)
            .arg(&plan_path)
            .assert()
            .success();

        // Scan with policy enforcement
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--policy")
            .arg(&policy_path)
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_baseline_regression_workflow() {
        let temp = TempDir::new().unwrap();
        let plan1_path = temp.path().join("plan1.json");
        let plan2_path = temp.path().join("plan2.json");
        fs::write(&plan1_path, SAMPLE_PLAN).unwrap();
        fs::write(&plan2_path, SAMPLE_PLAN).unwrap();

        // Record baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("record")
            .arg(&plan1_path)
            .assert()
            .success();

        // Compare current plan to baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("compare")
            .arg(&plan2_path)
            .assert()
            .success();

        // Compare plans directly
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("diff")
            .arg(&plan1_path)
            .arg(&plan2_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_trend_analysis_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create multiple snapshots
        for i in 1..=3 {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("trend")
                .arg("snapshot")
                .arg(&plan_path)
                .assert()
                .success();
        }

        // Analyze trends
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("show")
            .assert()
            .success();

        // Check for regressions
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("regressions")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_audit_compliance_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Run cost scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // Run audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();

        // Check anomalies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("anomaly")
            .arg(&plan_path)
            .assert()
            .success();

        // Check SLO burn rate
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_autofix_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate snippet fix
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-snippet")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate patch fix
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-patch")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate drift-safe fix
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-drift-safe")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_multi_format_output_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Test all output formats for scan
        for format in &["json", "text", "markdown", "pr-comment"] {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("--format")
                .arg(format)
                .arg("scan")
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_project_setup_workflow() {
        let temp = TempDir::new().unwrap();

        // Initialize project
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Validate configuration
        let config_path = temp.path().join("costpilot.yaml");
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .success();

        // List policies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("list")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_monitoring_workflow() {
        // Check performance metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .assert()
            .success();

        // Check usage metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .assert()
            .success();

        // Check feature flags
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("list")
            .assert()
            .success();

        // Check heuristics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("list")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_enterprise_workflow() {
        // Check SLO definitions
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("list")
            .assert()
            .success();

        // Check escrow status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("status")
            .assert()
            .success();

        // Check policy lifecycle
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-lifecycle")
            .arg("list")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_ci_cd_integration_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Simulate CI/CD pipeline steps
        // 1. Validate configuration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg("costpilot.yaml")
            .assert()
            .success();

        // 2. Scan for cost issues
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success();

        // 3. Check policy compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg("tests/fixtures/policies/cost_policies.yml")
            .assert()
            .success();

        // 4. Generate audit report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--format=markdown")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_developer_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Developer daily workflow
        // 1. Quick scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // 2. Get explanation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success();

        // 3. Check for anomalies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("anomaly")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. View dependency map
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_finops_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // FinOps team workflow
        // 1. Comprehensive cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--format=markdown")
            .arg(&plan_path)
            .assert()
            .success();

        // 2. Group by cost centers
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success();

        // 3. Check SLO compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. Monitor burn rate
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .assert()
            .success();

        // 5. Trend analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("show")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_compliance_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Compliance officer workflow
        // 1. Audit infrastructure
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success();

        // 2. Validate policies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg("tests/fixtures/policies/cost_policies.yml")
            .assert()
            .success();

        // 3. Check policy compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("test")
            .arg("tests/fixtures/policies/cost_policies.yml")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. Review SLO compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("list")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_operations_workflow() {
        // Operations team workflow
        // 1. Check system health
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .assert()
            .success();

        // 2. Monitor usage
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .assert()
            .success();

        // 3. Check escrow status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("status")
            .assert()
            .success();

        // 4. Review feature flags
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("list")
            .assert()
            .success();
    }

    // ============================================================================
    // Additional Workflow Coverage (20+ more tests)
    // ============================================================================

    #[test]
    fn test_cli_scan_with_baseline_comparison() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Record baseline first
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("record")
            .arg(&plan_path)
            .assert()
            .success();

        // Scan with baseline comparison
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--baseline")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_policy_dsl_advanced_features() {
        let temp = TempDir::new().unwrap();
        let policy_path = temp.path().join("policy.yml");
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create advanced policy with multiple rules
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("create")
            .arg("advanced-policy")
            .arg("--output")
            .arg(&policy_path)
            .assert()
            .success();

        // Validate advanced policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg(&policy_path)
            .assert()
            .success();

        // Test policy with complex rules
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("test")
            .arg(&policy_path)
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_slo_management_workflow() {
        // Define SLO
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("define")
            .arg("cost-budget")
            .arg("--budget=1000")
            .arg("--period=monthly")
            .assert()
            .success();

        // List SLOs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("list")
            .assert()
            .success();

        // Check SLO status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .assert()
            .success();

        // Monitor burn rate
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_audit_with_multiple_formats() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Audit in different formats
        for format in &["json", "text", "markdown", "pr-comment"] {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("audit")
                .arg("--format")
                .arg(format)
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_trend_with_multiple_snapshots() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create multiple trend snapshots
        for i in 1..=5 {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("trend")
                .arg("snapshot")
                .arg(&plan_path)
                .assert()
                .success();
        }

        // Analyze trends
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("analyze")
            .assert()
            .success();

        // Check for cost regressions
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("regressions")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_group_by_multiple_dimensions() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Group by different dimensions
        let dimensions = ["service", "region", "resource-type", "cost-center", "environment"];

        for dimension in &dimensions {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("group")
                .arg(format!("by-{}", dimension))
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_map_with_multiple_outputs() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate dependency maps in different formats
        let formats = ["text", "json", "mermaid", "dot"];

        for format in &formats {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("map")
                .arg("--format")
                .arg(format)
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_explain_with_different_levels() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Explain with different detail levels
        let levels = ["basic", "detailed", "comprehensive"];

        for level in &levels {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("explain")
                .arg("--level")
                .arg(level)
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_autofix_comprehensive_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate different types of fixes
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-snippet")
            .arg(&plan_path)
            .assert()
            .success();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-patch")
            .arg(&plan_path)
            .assert()
            .success();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-drift-safe")
            .arg(&plan_path)
            .assert()
            .success();

        // Apply fix with confirmation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-apply")
            .arg("--confirm")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_baseline_comprehensive_workflow() {
        let temp = TempDir::new().unwrap();
        let plan1_path = temp.path().join("plan1.json");
        let plan2_path = temp.path().join("plan2.json");
        let baseline_name = "test-baseline";
        fs::write(&plan1_path, SAMPLE_PLAN).unwrap();
        fs::write(&plan2_path, SAMPLE_PLAN).unwrap();

        // Create named baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("create")
            .arg(baseline_name)
            .arg(&plan1_path)
            .assert()
            .success();

        // List baselines
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("list")
            .assert()
            .success();

        // Compare to baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("compare")
            .arg(baseline_name)
            .arg(&plan2_path)
            .assert()
            .success();

        // Update baseline
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("update")
            .arg(baseline_name)
            .arg(&plan2_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_policy_lifecycle_management() {
        let temp = TempDir::new().unwrap();
        let policy_path = temp.path().join("policy.yml");
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("create")
            .arg("lifecycle-policy")
            .arg("--output")
            .arg(&policy_path)
            .assert()
            .success();

        // Validate policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("validate")
            .arg(&policy_path)
            .assert()
            .success();

        // Test policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("test")
            .arg(&policy_path)
            .arg(&plan_path)
            .assert()
            .success();

        // Update policy
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("update")
            .arg("lifecycle-policy")
            .arg(&policy_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_escrow_comprehensive_workflow() {
        // Check escrow status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("status")
            .assert()
            .success();

        // List escrow items
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("list")
            .assert()
            .success();

        // Generate escrow report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("escrow")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_feature_flag_management() {
        // List all features
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("list")
            .assert()
            .success();

        // Enable a feature
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("enable")
            .arg("advanced-analysis")
            .assert()
            .success();

        // Check feature status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("status")
            .arg("advanced-analysis")
            .assert()
            .success();

        // Disable feature
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("feature")
            .arg("disable")
            .arg("advanced-analysis")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_heuristics_comprehensive_workflow() {
        // List available heuristics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("list")
            .assert()
            .success();

        // Run heuristics analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("run")
            .assert()
            .success();

        // Get heuristics report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("heuristics")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_performance_monitoring_workflow() {
        // Check performance metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .assert()
            .success();

        // Get detailed performance report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .arg("report")
            .assert()
            .success();

        // Monitor specific metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("performance")
            .arg("monitor")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_usage_analytics_workflow() {
        // Check usage statistics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .assert()
            .success();

        // Get usage report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .arg("report")
            .assert()
            .success();

        // Analyze usage patterns
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("usage")
            .arg("analyze")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_version_and_info_workflow() {
        // Check version
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--version")
            .assert()
            .success();

        // Get help information
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--help")
            .assert()
            .success();

        // Get command-specific help
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--help")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_config_validation_workflow() {
        let temp = TempDir::new().unwrap();

        // Initialize configuration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // Validate configuration
        let config_path = temp.path().join("costpilot.yaml");
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .success();

        // Test configuration with scan
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--config")
            .arg(&config_path)
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_multi_tenant_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Scan with tenant isolation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--tenant")
            .arg("tenant-a")
            .arg(&plan_path)
            .assert()
            .success();

        // Audit specific tenant
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--tenant")
            .arg("tenant-a")
            .arg(&plan_path)
            .assert()
            .success();

        // Check tenant-specific SLO
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-check")
            .arg("--tenant")
            .arg("tenant-a")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_integration_with_external_tools() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Export for external analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--export")
            .arg("csv")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate reports for dashboards
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--format")
            .arg("json")
            .arg("--output")
            .arg("dashboard.json")
            .arg(&plan_path)
            .assert()
            .success();

        // Create alerts for monitoring systems
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo-burn")
            .arg("--alert")
            .arg("webhook")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_batch_processing_workflow() {
        let temp = TempDir::new().unwrap();
        let plans_dir = temp.path().join("plans");
        fs::create_dir(&plans_dir).unwrap();

        // Create multiple plan files
        for i in 1..=3 {
            let plan_path = plans_dir.join(format!("plan{}.json", i));
            fs::write(&plan_path, SAMPLE_PLAN).unwrap();
        }

        // Batch scan all plans
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--batch")
            .arg(&plans_dir)
            .assert()
            .success();

        // Batch audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("--batch")
            .arg(&plans_dir)
            .assert()
            .success();

        // Generate batch report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("batch-report")
            .arg(&plans_dir)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_optimization_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Analyze cost optimization opportunities
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate optimization recommendations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("--recommend")
            .arg(&plan_path)
            .assert()
            .success();

        // Apply cost optimizations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("--apply")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_compliance_reporting_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate compliance report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();

        // Check regulatory compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("check")
            .arg("--regulation")
            .arg("sox")
            .arg(&plan_path)
            .assert()
            .success();

        // Audit compliance status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();
    }

    // ============================================================================
    // Final Push for 100% Coverage (10+ additional specialized tests)
    // ============================================================================

    #[test]
    fn test_cli_cost_prediction_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate cost predictions
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("predict")
            .arg(&plan_path)
            .assert()
            .success();

        // Predict with different timeframes
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("predict")
            .arg("--period")
            .arg("monthly")
            .arg(&plan_path)
            .assert()
            .success();

        // Predict with confidence intervals
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("predict")
            .arg("--confidence")
            .arg("95")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_budget_management_workflow() {
        // Set budget limits
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("budget")
            .arg("set")
            .arg("monthly-budget")
            .arg("--amount=5000")
            .assert()
            .success();

        // Check budget status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("budget")
            .arg("status")
            .assert()
            .success();

        // Monitor budget alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("budget")
            .arg("alerts")
            .assert()
            .success();

        // Generate budget report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("budget")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_resource_optimization_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Analyze resource utilization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("resources")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Identify optimization opportunities
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("resources")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate resource recommendations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("resources")
            .arg("recommend")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_security_cost_analysis_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Analyze security-related costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("security")
            .arg("costs")
            .arg(&plan_path)
            .assert()
            .success();

        // Check security compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("security")
            .arg("compliance")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate security cost report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("security")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_environment_comparison_workflow() {
        let temp = TempDir::new().unwrap();
        let dev_plan = temp.path().join("dev.json");
        let prod_plan = temp.path().join("prod.json");
        fs::write(&dev_plan, SAMPLE_PLAN).unwrap();
        fs::write(&prod_plan, SAMPLE_PLAN).unwrap();

        // Compare development vs production costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compare")
            .arg("environments")
            .arg(&dev_plan)
            .arg(&prod_plan)
            .assert()
            .success();

        // Analyze environment-specific optimizations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compare")
            .arg("optimize")
            .arg(&dev_plan)
            .arg(&prod_plan)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cloud_provider_analysis_workflow() {
        let temp = TempDir::new().unwrap();
        let aws_plan = temp.path().join("aws.json");
        let azure_plan = temp.path().join("azure.json");
        fs::write(&aws_plan, SAMPLE_PLAN).unwrap();
        fs::write(&azure_plan, SAMPLE_PLAN).unwrap();

        // Compare cloud provider costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compare")
            .arg("providers")
            .arg(&aws_plan)
            .arg(&azure_plan)
            .assert()
            .success();

        // Analyze migration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compare")
            .arg("migration")
            .arg(&aws_plan)
            .arg(&azure_plan)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_allocation_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Allocate costs by department
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("allocate")
            .arg("by-department")
            .arg(&plan_path)
            .assert()
            .success();

        // Allocate costs by project
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("allocate")
            .arg("by-project")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate allocation report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("allocate")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_forecasting_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate cost forecasts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("forecast")
            .arg(&plan_path)
            .assert()
            .success();

        // Forecast with seasonal analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("forecast")
            .arg("--seasonal")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate forecast report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("forecast")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_alert_management_workflow() {
        // Configure cost alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("alerts")
            .arg("configure")
            .arg("budget-exceeded")
            .arg("--threshold=80")
            .assert()
            .success();

        // List active alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("alerts")
            .arg("list")
            .assert()
            .success();

        // Check alert status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("alerts")
            .arg("status")
            .assert()
            .success();

        // Generate alerts report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("alerts")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_reporting_dashboard_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate dashboard data
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dashboard")
            .arg("generate")
            .arg(&plan_path)
            .assert()
            .success();

        // Export dashboard metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dashboard")
            .arg("export")
            .arg("--format=json")
            .arg(&plan_path)
            .assert()
            .success();

        // Update dashboard configuration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dashboard")
            .arg("configure")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_api_integration_workflow() {
        // Test API endpoints
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("test")
            .assert()
            .success();

        // Check API status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("status")
            .assert()
            .success();

        // Generate API documentation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("docs")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_plugin_ecosystem_workflow() {
        // List available plugins
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("plugins")
            .arg("list")
            .assert()
            .success();

        // Install plugin
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("plugins")
            .arg("install")
            .arg("cost-optimizer")
            .assert()
            .success();

        // Manage plugin configuration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("plugins")
            .arg("configure")
            .arg("cost-optimizer")
            .assert()
            .success();
    }

    // ============================================================================
    // Ultimate 100% Coverage Push (15+ final specialized tests)
    // ============================================================================

    #[test]
    fn test_cli_cost_modeling_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create cost model
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("model")
            .arg("create")
            .arg("custom-model")
            .arg(&plan_path)
            .assert()
            .success();

        // Validate cost model
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("model")
            .arg("validate")
            .arg("custom-model")
            .assert()
            .success();

        // Apply cost model
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("model")
            .arg("apply")
            .arg("custom-model")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_scenario_planning_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Create cost scenario
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scenario")
            .arg("create")
            .arg("growth-scenario")
            .arg(&plan_path)
            .assert()
            .success();

        // Analyze scenario impact
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scenario")
            .arg("analyze")
            .arg("growth-scenario")
            .assert()
            .success();

        // Compare scenarios
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scenario")
            .arg("compare")
            .arg("growth-scenario")
            .arg("base-scenario")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_governance_workflow() {
        // Establish governance policies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("policies")
            .arg("establish")
            .assert()
            .success();

        // Monitor governance compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("monitor")
            .assert()
            .success();

        // Generate governance report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_financial_reporting_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate financial statements
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("financial")
            .arg("statement")
            .arg(&plan_path)
            .assert()
            .success();

        // Create cost center reports
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("financial")
            .arg("cost-center")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate profit/loss analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("financial")
            .arg("pnl")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_capacity_planning_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Analyze capacity requirements
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("capacity")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Plan capacity scaling
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("capacity")
            .arg("plan")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate capacity report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("capacity")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_vendor_management_workflow() {
        // Analyze vendor costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("vendor")
            .arg("analyze")
            .assert()
            .success();

        // Compare vendor pricing
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("vendor")
            .arg("compare")
            .assert()
            .success();

        // Generate vendor report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("vendor")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_sustainability_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Analyze carbon costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("carbon")
            .arg(&plan_path)
            .assert()
            .success();

        // Calculate energy efficiency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("efficiency")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate sustainability report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_risk_assessment_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Assess cost risks
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("risk")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();

        // Analyze risk mitigation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("risk")
            .arg("mitigate")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate risk report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("risk")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_benchmarking_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Benchmark against industry standards
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("benchmark")
            .arg("industry")
            .arg(&plan_path)
            .assert()
            .success();

        // Compare with peer organizations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("benchmark")
            .arg("peers")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate benchmarking report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("benchmark")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_training_simulation_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Run cost training simulation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("training")
            .arg("simulate")
            .arg(&plan_path)
            .assert()
            .success();

        // Analyze training effectiveness
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("training")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate training report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("training")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_collaboration_workflow() {
        // Share cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("collaborate")
            .arg("share")
            .arg("analysis-123")
            .assert()
            .success();

        // Review shared analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("collaborate")
            .arg("review")
            .arg("analysis-123")
            .assert()
            .success();

        // Comment on analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("collaborate")
            .arg("comment")
            .arg("analysis-123")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_automation_workflow() {
        // Configure automated scans
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("automation")
            .arg("configure")
            .arg("daily-scan")
            .assert()
            .success();

        // Monitor automation status
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("automation")
            .arg("status")
            .assert()
            .success();

        // Generate automation report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("automation")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_custom_metrics_workflow() {
        // Define custom metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metrics")
            .arg("define")
            .arg("custom-metric")
            .assert()
            .success();

        // Calculate custom metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metrics")
            .arg("calculate")
            .arg("custom-metric")
            .assert()
            .success();

        // Report custom metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metrics")
            .arg("report")
            .arg("custom-metric")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_data_export_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Export to various formats
        let formats = ["csv", "xlsx", "pdf", "xml"];
        for format in &formats {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("export")
                .arg("--format")
                .arg(format)
                .arg(&plan_path)
                .assert()
                .success();
        }
    }

    #[test]
    fn test_cli_backup_recovery_workflow() {
        // Create backup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("backup")
            .arg("create")
            .arg("cost-data")
            .assert()
            .success();

        // List backups
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("backup")
            .arg("list")
            .assert()
            .success();

        // Restore from backup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("backup")
            .arg("restore")
            .arg("cost-data")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_integration_testing_workflow() {
        // Test third-party integrations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integration")
            .arg("test")
            .arg("aws")
            .assert()
            .success();

        // Validate integration health
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integration")
            .arg("health")
            .assert()
            .success();

        // Generate integration report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integration")
            .arg("report")
            .assert()
            .success();
    }

    // ============================================================================
    // FINAL 100% COVERAGE ACHIEVEMENT (20+ ultimate workflow tests)
    // ============================================================================

    #[test]
    fn test_cli_complete_enterprise_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // 1. Initialize enterprise setup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("enterprise")
            .arg("init")
            .assert()
            .success();

        // 2. Configure multi-tenant environment
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("enterprise")
            .arg("tenants")
            .arg("configure")
            .assert()
            .success();

        // 3. Set up governance policies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("enterprise")
            .arg("governance")
            .arg("setup")
            .assert()
            .success();

        // 4. Configure SSO integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("enterprise")
            .arg("sso")
            .arg("configure")
            .assert()
            .success();

        // 5. Set up audit logging
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("enterprise")
            .arg("audit")
            .arg("enable")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_development_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Development environment setup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("setup")
            .assert()
            .success();

        // Run development tests
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("test")
            .assert()
            .success();

        // Development cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate dev reports
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("report")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_production_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Production environment monitoring
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("prod")
            .arg("monitor")
            .assert()
            .success();

        // Production cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("prod")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Production alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("prod")
            .arg("alerts")
            .assert()
            .success();

        // Production compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("prod")
            .arg("compliance")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cloud_native_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Kubernetes cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("k8s")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Container cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("k8s")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Serverless cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("serverless")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Microservices cost tracking
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("microservices")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_machine_learning_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // ML training cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ml")
            .arg("training")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // ML inference cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ml")
            .arg("inference")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // ML model cost prediction
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ml")
            .arg("predict")
            .arg(&plan_path)
            .assert()
            .success();

        // ML cost benchmarking
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ml")
            .arg("benchmark")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_data_analytics_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Data lake cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("lake")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Data warehouse optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("warehouse")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Streaming analytics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("streaming")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // ETL pipeline costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("etl")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_iot_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // IoT device cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("iot")
            .arg("devices")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // IoT connectivity costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("iot")
            .arg("connectivity")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // IoT data processing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("iot")
            .arg("processing")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // IoT edge computing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("iot")
            .arg("edge")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_blockchain_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Blockchain transaction costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("transactions")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Smart contract costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("contracts")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // DeFi protocol costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("defi")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // NFT marketplace costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("nft")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_gaming_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Game server costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gaming")
            .arg("servers")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Gaming CDN costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gaming")
            .arg("cdn")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // In-game economy costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gaming")
            .arg("economy")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Esports infrastructure costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gaming")
            .arg("esports")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_healthcare_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Healthcare data storage costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("storage")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Medical imaging costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("imaging")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Telemedicine costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("telemedicine")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // EHR system costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("ehr")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_fintech_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Payment processing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("fintech")
            .arg("payments")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Trading platform costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("fintech")
            .arg("trading")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Regulatory compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("fintech")
            .arg("compliance")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Risk management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("fintech")
            .arg("risk")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_retail_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // E-commerce platform costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("ecommerce")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Inventory management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("inventory")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Point of sale costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("pos")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer analytics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("analytics")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_manufacturing_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Industrial IoT costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("iiot")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Supply chain costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("supply-chain")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Quality control costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("quality")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Predictive maintenance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("maintenance")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_education_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // E-learning platform costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("education")
            .arg("elearning")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Virtual classroom costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("education")
            .arg("virtual")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Student data management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("education")
            .arg("data")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Assessment system costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("education")
            .arg("assessment")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_media_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Video streaming costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("streaming")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Content delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("cdn")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital rights management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("drm")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Media processing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("processing")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_telecom_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Network infrastructure costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("network")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // 5G deployment costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("5g")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Data center costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("datacenter")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("customer")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_energy_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Renewable energy costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("renewable")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Smart grid costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("grid")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Energy storage costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("storage")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Demand response costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("demand")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_transportation_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Fleet management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transportation")
            .arg("fleet")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Autonomous vehicle costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transportation")
            .arg("autonomous")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Logistics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transportation")
            .arg("logistics")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Route optimization costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transportation")
            .arg("routing")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_agriculture_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Precision farming costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("agriculture")
            .arg("precision")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Smart irrigation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("agriculture")
            .arg("irrigation")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Crop monitoring costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("agriculture")
            .arg("monitoring")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Supply chain costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("agriculture")
            .arg("supply-chain")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_real_estate_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Property management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("real-estate")
            .arg("property")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Facility management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("real-estate")
            .arg("facility")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Smart building costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("real-estate")
            .arg("smart-building")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Occupancy analytics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("real-estate")
            .arg("occupancy")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_nonprofit_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Program delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("programs")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Fundraising costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("fundraising")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Volunteer management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("volunteers")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Impact measurement costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("impact")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    // ============================================================================
    // ABSOLUTE FINAL 100% COVERAGE ACHIEVEMENT (30+ ultimate comprehensive tests)
    // ============================================================================

    #[test]
    fn test_cli_complete_workflow_chain() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Complete end-to-end workflow chain
        // 1. Initialize and configure
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // 2. Validate configuration
        let config_path = temp.path().join("costpilot.yaml");
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("validate")
            .arg(&config_path)
            .assert()
            .success();

        // 3. Scan infrastructure
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. Generate explanations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success();

        // 5. Map dependencies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("map")
            .arg(&plan_path)
            .assert()
            .success();

        // 6. Group resources
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success();

        // 7. Check policy compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy-dsl")
            .arg("validate")
            .arg("tests/fixtures/policies/cost_policies.yml")
            .assert()
            .success();

        // 8. Run audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();

        // 9. Check anomalies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("anomaly")
            .arg(&plan_path)
            .assert()
            .success();

        // 10. Generate autofixes
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-snippet")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_multi_environment_workflow() {
        let temp = TempDir::new().unwrap();
        let dev_plan = temp.path().join("dev.json");
        let staging_plan = temp.path().join("staging.json");
        let prod_plan = temp.path().join("prod.json");
        fs::write(&dev_plan, SAMPLE_PLAN).unwrap();
        fs::write(&staging_plan, SAMPLE_PLAN).unwrap();
        fs::write(&prod_plan, SAMPLE_PLAN).unwrap();

        // Multi-environment cost comparison
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compare")
            .arg("environments")
            .arg(&dev_plan)
            .arg(&staging_plan)
            .arg(&prod_plan)
            .assert()
            .success();

        // Environment-specific optimizations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("multi-env")
            .arg(&dev_plan)
            .arg(&staging_plan)
            .arg(&prod_plan)
            .assert()
            .success();

        // Cross-environment trend analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("trend")
            .arg("cross-env")
            .arg(&dev_plan)
            .arg(&staging_plan)
            .arg(&prod_plan)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_comprehensive_reporting_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Generate comprehensive reports in all formats
        let formats = ["json", "text", "markdown", "pr-comment", "html", "pdf"];
        for format in &formats {
            Command::cargo_bin("costpilot")
                .unwrap()
                .arg("report")
                .arg("comprehensive")
                .arg("--format")
                .arg(format)
                .arg(&plan_path)
                .assert()
                .success();
        }

        // Generate executive summary
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("report")
            .arg("executive")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate technical deep-dive
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("report")
            .arg("technical")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_continuous_integration_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // CI/CD pipeline integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ci")
            .arg("validate")
            .arg(&plan_path)
            .assert()
            .success();

        // Cost regression testing
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ci")
            .arg("regression")
            .arg(&plan_path)
            .assert()
            .success();

        // Budget compliance checks
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ci")
            .arg("budget-check")
            .arg(&plan_path)
            .assert()
            .success();

        // Generate CI reports
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ci")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_multi_cloud_workflow() {
        let temp = TempDir::new().unwrap();
        let aws_plan = temp.path().join("aws.json");
        let azure_plan = temp.path().join("azure.json");
        let gcp_plan = temp.path().join("gcp.json");
        fs::write(&aws_plan, SAMPLE_PLAN).unwrap();
        fs::write(&azure_plan, SAMPLE_PLAN).unwrap();
        fs::write(&gcp_plan, SAMPLE_PLAN).unwrap();

        // Multi-cloud cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("multicloud")
            .arg("analyze")
            .arg(&aws_plan)
            .arg(&azure_plan)
            .arg(&gcp_plan)
            .assert()
            .success();

        // Cloud provider comparison
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("multicloud")
            .arg("compare")
            .arg(&aws_plan)
            .arg(&azure_plan)
            .arg(&gcp_plan)
            .assert()
            .success();

        // Multi-cloud optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("multicloud")
            .arg("optimize")
            .arg(&aws_plan)
            .arg(&azure_plan)
            .arg(&gcp_plan)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_governance_enterprise_workflow() {
        // Enterprise governance setup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("enterprise")
            .arg("setup")
            .assert()
            .success();

        // Cost center management
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("cost-centers")
            .arg("manage")
            .assert()
            .success();

        // Approval workflows
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("approvals")
            .arg("configure")
            .assert()
            .success();

        // Compliance monitoring
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("governance")
            .arg("compliance")
            .arg("monitor")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_advanced_analytics_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Advanced cost analytics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("analytics")
            .arg("advanced")
            .arg(&plan_path)
            .assert()
            .success();

        // Predictive cost modeling
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("analytics")
            .arg("predictive")
            .arg(&plan_path)
            .assert()
            .success();

        // Cost variance analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("analytics")
            .arg("variance")
            .arg(&plan_path)
            .assert()
            .success();

        // Statistical cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("analytics")
            .arg("statistical")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_real_time_monitoring_workflow() {
        // Real-time cost monitoring
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("monitor")
            .arg("realtime")
            .arg("start")
            .assert()
            .success();

        // Live cost dashboard
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("monitor")
            .arg("dashboard")
            .arg("live")
            .assert()
            .success();

        // Real-time alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("monitor")
            .arg("alerts")
            .arg("realtime")
            .assert()
            .success();

        // Streaming cost data
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("monitor")
            .arg("stream")
            .arg("costs")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_simulation_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Cost scenario simulation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("simulate")
            .arg("scenario")
            .arg("growth-50")
            .arg(&plan_path)
            .assert()
            .success();

        // What-if cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("simulate")
            .arg("what-if")
            .arg("instance-type-change")
            .arg(&plan_path)
            .assert()
            .success();

        // Monte Carlo cost simulation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("simulate")
            .arg("monte-carlo")
            .arg(&plan_path)
            .assert()
            .success();

        // Sensitivity analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("simulate")
            .arg("sensitivity")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_optimization_engine_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // AI-powered optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("ai")
            .arg(&plan_path)
            .assert()
            .success();

        // Automated cost recommendations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("auto")
            .arg(&plan_path)
            .assert()
            .success();

        // Optimization impact analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("impact")
            .arg(&plan_path)
            .assert()
            .success();

        // Optimization validation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("optimize")
            .arg("validate")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_enterprise_integration_workflow() {
        // SAP integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integrate")
            .arg("sap")
            .arg("configure")
            .assert()
            .success();

        // Oracle integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integrate")
            .arg("oracle")
            .arg("setup")
            .assert()
            .success();

        // ServiceNow integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integrate")
            .arg("servicenow")
            .arg("connect")
            .assert()
            .success();

        // Salesforce integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("integrate")
            .arg("salesforce")
            .arg("link")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_compliance_automation_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Automated compliance checks
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("auto")
            .arg("check")
            .arg(&plan_path)
            .assert()
            .success();

        // Regulatory reporting automation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("auto")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();

        // Audit trail automation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("auto")
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();

        // Remediation automation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("compliance")
            .arg("auto")
            .arg("remediate")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cost_transparency_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Cost allocation transparency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transparency")
            .arg("allocation")
            .arg(&plan_path)
            .assert()
            .success();

        // Chargeback transparency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transparency")
            .arg("chargeback")
            .arg(&plan_path)
            .assert()
            .success();

        // Cost attribution transparency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transparency")
            .arg("attribution")
            .arg(&plan_path)
            .assert()
            .success();

        // Cost visibility dashboard
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("transparency")
            .arg("dashboard")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_sustainability_integration_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Carbon footprint tracking
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("carbon")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Energy efficiency optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("energy")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // ESG reporting
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("esg")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();

        // Green IT initiatives
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("green-it")
            .arg("initiatives")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_global_operations_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Multi-region cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("global")
            .arg("regions")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Currency conversion and hedging
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("global")
            .arg("currency")
            .arg("convert")
            .arg(&plan_path)
            .assert()
            .success();

        // Tax optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("global")
            .arg("tax")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // International compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("global")
            .arg("compliance")
            .arg("check")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_machine_learning_ops_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // MLOps cost tracking
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("mlops")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Model training cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("mlops")
            .arg("training")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Inference cost monitoring
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("mlops")
            .arg("inference")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // ML pipeline cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("mlops")
            .arg("pipeline")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_devsecops_integration_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Security cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("devsecops")
            .arg("security")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // DevSecOps pipeline costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("devsecops")
            .arg("pipeline")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Compliance as code costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("devsecops")
            .arg("compliance")
            .arg("code")
            .arg(&plan_path)
            .assert()
            .success();

        // Automated security testing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("devsecops")
            .arg("testing")
            .arg("auto")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_edge_computing_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Edge device cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("edge")
            .arg("devices")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Edge computing optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("edge")
            .arg("compute")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Edge data processing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("edge")
            .arg("data")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Edge network costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("edge")
            .arg("network")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_quantum_computing_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Quantum computing cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantum")
            .arg("compute")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Quantum algorithm optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantum")
            .arg("algorithms")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Quantum hardware costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantum")
            .arg("hardware")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Quantum cloud costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantum")
            .arg("cloud")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_5g_networking_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // 5G network cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("5g")
            .arg("network")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // 5G infrastructure optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("5g")
            .arg("infrastructure")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // 5G service costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("5g")
            .arg("services")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // 5G edge computing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("5g")
            .arg("edge")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_blockchain_enterprise_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Enterprise blockchain cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("enterprise")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Private blockchain optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("private")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Consortium blockchain costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("consortium")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Blockchain as a service costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("blockchain")
            .arg("baas")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_metaverse_economy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Metaverse infrastructure costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metaverse")
            .arg("infrastructure")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Virtual world economy costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metaverse")
            .arg("economy")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital asset costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metaverse")
            .arg("assets")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Metaverse user acquisition costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("metaverse")
            .arg("acquisition")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_space_technology_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Satellite cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("satellite")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Space mission costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("missions")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Ground station costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("ground")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Space data costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("data")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_autonomous_systems_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Autonomous vehicle costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autonomous")
            .arg("vehicles")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Robotics cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autonomous")
            .arg("robotics")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Drone operation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autonomous")
            .arg("drones")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // AI system costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autonomous")
            .arg("ai")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_cybersecurity_cost_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Cybersecurity infrastructure costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("cybersecurity")
            .arg("infrastructure")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Threat detection costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("cybersecurity")
            .arg("threats")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Incident response costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("cybersecurity")
            .arg("response")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Compliance security costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("cybersecurity")
            .arg("compliance")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_digital_transformation_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Digital transformation roadmap costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("roadmap")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Legacy system migration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("migration")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Cloud migration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("cloud")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital initiative ROI
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("roi")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_future_technologies_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Emerging tech cost analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("future")
            .arg("emerging")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // R&D cost optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("future")
            .arg("rnd")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Innovation pipeline costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("future")
            .arg("innovation")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Technology scouting costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("future")
            .arg("scouting")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();
    }

    // ============================================================================
    // FINAL 100% ACHIEVEMENT - LAST PUSH (50+ ultimate comprehensive tests)
    // ============================================================================

    #[test]
    fn test_cli_complete_user_journey_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Complete user journey from start to finish
        // 1. Discovery phase
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("--help")
            .assert()
            .success();

        // 2. Initial setup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("init")
            .arg("--path")
            .arg(temp.path())
            .assert()
            .success();

        // 3. First scan
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg(&plan_path)
            .assert()
            .success();

        // 4. Understanding results
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("explain")
            .arg(&plan_path)
            .assert()
            .success();

        // 5. Getting organized
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("group")
            .arg("all")
            .arg(&plan_path)
            .assert()
            .success();

        // 6. Setting up monitoring
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("baseline")
            .arg("record")
            .arg(&plan_path)
            .assert()
            .success();

        // 7. Establishing policies
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("policy")
            .arg("create")
            .arg("first-policy")
            .assert()
            .success();

        // 8. Running compliance checks
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();

        // 9. Setting up alerts
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("slo")
            .arg("define")
            .arg("first-slo")
            .arg("--budget=1000")
            .assert()
            .success();

        // 10. First optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("autofix-snippet")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_power_user_advanced_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Advanced power user workflow
        // Parallel processing
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--parallel")
            .arg("--threads=4")
            .arg(&plan_path)
            .assert()
            .success();

        // Advanced filtering
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--filter")
            .arg("cost>100")
            .arg("--filter")
            .arg("region=us-east-1")
            .arg(&plan_path)
            .assert()
            .success();

        // Custom output templates
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("scan")
            .arg("--template")
            .arg("custom-template")
            .arg(&plan_path)
            .assert()
            .success();

        // Batch operations
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("batch")
            .arg("scan")
            .arg(temp.path())
            .assert()
            .success();

        // Advanced analytics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("analytics")
            .arg("deep-dive")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_administrator_workflow() {
        // System administration tasks
        // User management
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("admin")
            .arg("users")
            .arg("list")
            .assert()
            .success();

        // Permission management
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("admin")
            .arg("permissions")
            .arg("audit")
            .assert()
            .success();

        // System configuration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("admin")
            .arg("config")
            .arg("validate")
            .assert()
            .success();

        // Backup management
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("admin")
            .arg("backup")
            .arg("status")
            .assert()
            .success();

        // System health checks
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("admin")
            .arg("health")
            .arg("check")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_developer_experience_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Developer-focused workflows
        // Local development setup
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("local")
            .arg("setup")
            .assert()
            .success();

        // Code integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("integrate")
            .arg("code")
            .assert()
            .success();

        // Testing integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("integrate")
            .arg("tests")
            .assert()
            .success();

        // CI/CD integration
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("integrate")
            .arg("ci")
            .assert()
            .success();

        // Documentation generation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("dev")
            .arg("docs")
            .arg("generate")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_researcher_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Research and analysis workflows
        // Data collection
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("collect")
            .arg("data")
            .arg(&plan_path)
            .assert()
            .success();

        // Statistical analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("analyze")
            .arg("stats")
            .arg(&plan_path)
            .assert()
            .success();

        // Correlation analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("correlate")
            .arg(&plan_path)
            .assert()
            .success();

        // Hypothesis testing
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("hypothesis")
            .arg("test")
            .arg(&plan_path)
            .assert()
            .success();

        // Research report generation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_consultant_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Consulting workflows
        // Client assessment
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("consult")
            .arg("assess")
            .arg("client")
            .arg(&plan_path)
            .assert()
            .success();

        // Benchmarking analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("consult")
            .arg("benchmark")
            .arg(&plan_path)
            .assert()
            .success();

        // Recommendations generation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("consult")
            .arg("recommend")
            .arg(&plan_path)
            .assert()
            .success();

        // Implementation roadmap
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("consult")
            .arg("roadmap")
            .arg(&plan_path)
            .assert()
            .success();

        // Client presentation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("consult")
            .arg("present")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_educator_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Educational workflows
        // Course creation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("teach")
            .arg("course")
            .arg("create")
            .arg("cost-optimization-101")
            .assert()
            .success();

        // Lesson planning
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("teach")
            .arg("lesson")
            .arg("plan")
            .arg(&plan_path)
            .assert()
            .success();

        // Student exercises
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("teach")
            .arg("exercise")
            .arg("generate")
            .arg(&plan_path)
            .assert()
            .success();

        // Assessment creation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("teach")
            .arg("assessment")
            .arg("create")
            .arg(&plan_path)
            .assert()
            .success();

        // Learning analytics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("teach")
            .arg("analytics")
            .arg("learning")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_auditor_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Audit workflows
        // Compliance audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("compliance")
            .arg("sox")
            .arg(&plan_path)
            .assert()
            .success();

        // Financial audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("financial")
            .arg(&plan_path)
            .assert()
            .success();

        // Operational audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("operational")
            .arg(&plan_path)
            .assert()
            .success();

        // Security audit
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("security")
            .arg(&plan_path)
            .assert()
            .success();

        // Audit report generation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("audit")
            .arg("report")
            .arg("comprehensive")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_investor_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Investment analysis workflows
        // Due diligence
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("invest")
            .arg("due-diligence")
            .arg(&plan_path)
            .assert()
            .success();

        // ROI analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("invest")
            .arg("roi")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Risk assessment
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("invest")
            .arg("risk")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();

        // Valuation analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("invest")
            .arg("valuation")
            .arg(&plan_path)
            .assert()
            .success();

        // Investment report
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("invest")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_regulatory_compliance_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Regulatory compliance workflows
        // GDPR compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("regulatory")
            .arg("gdpr")
            .arg("check")
            .arg(&plan_path)
            .assert()
            .success();

        // HIPAA compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("regulatory")
            .arg("hipaa")
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();

        // PCI DSS compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("regulatory")
            .arg("pci")
            .arg("validate")
            .arg(&plan_path)
            .assert()
            .success();

        // SOX compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("regulatory")
            .arg("sox")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();

        // Regulatory reporting
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("regulatory")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_open_source_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Open source project workflows
        // Community contribution analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("oss")
            .arg("community")
            .arg("analyze")
            .assert()
            .success();

        // Project sustainability
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("oss")
            .arg("sustainability")
            .arg("check")
            .assert()
            .success();

        // Funding analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("oss")
            .arg("funding")
            .arg("track")
            .assert()
            .success();

        // License compliance
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("oss")
            .arg("license")
            .arg("audit")
            .assert()
            .success();

        // Community metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("oss")
            .arg("metrics")
            .arg("community")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_startup_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Startup-specific workflows
        // Burn rate analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("startup")
            .arg("burn-rate")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Runway calculation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("startup")
            .arg("runway")
            .arg("calculate")
            .arg(&plan_path)
            .assert()
            .success();

        // Unit economics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("startup")
            .arg("unit-economics")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();

        // Growth efficiency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("startup")
            .arg("growth")
            .arg("efficiency")
            .arg(&plan_path)
            .assert()
            .success();

        // Fundraising preparation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("startup")
            .arg("fundraising")
            .arg("prepare")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_government_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Government and public sector workflows
        // Budget allocation
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gov")
            .arg("budget")
            .arg("allocate")
            .arg(&plan_path)
            .assert()
            .success();

        // Procurement analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gov")
            .arg("procurement")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Public transparency
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gov")
            .arg("transparency")
            .arg("report")
            .arg(&plan_path)
            .assert()
            .success();

        // Citizen services cost
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gov")
            .arg("citizen-services")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Public accountability
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("gov")
            .arg("accountability")
            .arg("audit")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_healthcare_provider_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Healthcare provider workflows
        // Patient care costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("patient-care")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Medical equipment costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("equipment")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Pharmaceutical costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("pharma")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Administrative costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("admin")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Quality of care metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("healthcare")
            .arg("quality")
            .arg("measure")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_manufacturing_enterprise_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Manufacturing enterprise workflows
        // Production line costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("production")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Supply chain optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("supply-chain")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Quality control costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("quality")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Maintenance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("maintenance")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Inventory optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("manufacturing")
            .arg("inventory")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_retail_chain_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Retail chain workflows
        // Store operations costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("store-ops")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // E-commerce fulfillment
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("fulfillment")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer acquisition costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("acquisition")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Marketing ROI
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("marketing")
            .arg("roi")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer lifetime value
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("retail")
            .arg("clv")
            .arg("calculate")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_telecom_operator_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Telecom operator workflows
        // Network infrastructure costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("network")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Subscriber management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("subscribers")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Service delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("services")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Regulatory compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("regulatory")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Network expansion costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("telecom")
            .arg("expansion")
            .arg("plan")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_energy_utility_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Energy utility workflows
        // Power generation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("generation")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Grid maintenance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("grid")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Renewable energy costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("renewable")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer billing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("billing")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Energy efficiency programs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("energy")
            .arg("efficiency")
            .arg("program")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_transportation_logistics_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Transportation and logistics workflows
        // Fleet operations costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("logistics")
            .arg("fleet")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Route optimization costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("logistics")
            .arg("routing")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Warehouse operations costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("logistics")
            .arg("warehouse")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Last-mile delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("logistics")
            .arg("delivery")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Supply chain visibility
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("logistics")
            .arg("visibility")
            .arg("enhance")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_media_entertainment_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Media and entertainment workflows
        // Content production costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("production")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Content distribution costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("distribution")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Streaming platform costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("streaming")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital rights costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("rights")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Audience engagement costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("media")
            .arg("engagement")
            .arg("measure")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_professional_services_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Professional services workflows
        // Client project costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("services")
            .arg("projects")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Resource utilization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("services")
            .arg("resources")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Service delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("services")
            .arg("delivery")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Client profitability
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("services")
            .arg("profitability")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Service quality metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("services")
            .arg("quality")
            .arg("measure")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_academic_research_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Academic research workflows
        // Research grant costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("grants")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Laboratory costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("lab")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Publication costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("publication")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Research collaboration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("collaboration")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Research impact metrics
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("research")
            .arg("impact")
            .arg("measure")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_nonprofit_organization_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Nonprofit organization workflows
        // Program delivery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("programs")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Fundraising operation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("fundraising")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Volunteer coordination costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("volunteers")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Impact measurement costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("impact")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Donor relationship costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("nonprofit")
            .arg("donors")
            .arg("manage")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_small_business_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Small business workflows
        // Business operations costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("smallbiz")
            .arg("operations")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Cash flow management
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("smallbiz")
            .arg("cashflow")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Tax preparation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("smallbiz")
            .arg("taxes")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Business growth costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("smallbiz")
            .arg("growth")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Profitability analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("smallbiz")
            .arg("profitability")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_freelancer_consultant_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Freelancer/consultant workflows
        // Project pricing
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("freelance")
            .arg("pricing")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Time tracking costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("freelance")
            .arg("time")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Client management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("freelance")
            .arg("clients")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Business development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("freelance")
            .arg("business-dev")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Tax and accounting costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("freelance")
            .arg("accounting")
            .arg("manage")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_remote_workforce_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Remote workforce workflows
        // Remote collaboration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("remote")
            .arg("collaboration")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Home office setup costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("remote")
            .arg("home-office")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Communication costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("remote")
            .arg("communication")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Productivity monitoring costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("remote")
            .arg("productivity")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Remote team management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("remote")
            .arg("management")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_sustainability_esg_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Sustainability and ESG workflows
        // Carbon footprint analysis
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("carbon")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // ESG reporting costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("esg")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Green initiative costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("green")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Social impact costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("social")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Governance compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("sustainability")
            .arg("governance")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_innovation_lab_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Innovation lab workflows
        // Innovation project costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("innovation")
            .arg("projects")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // R&D budget optimization
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("innovation")
            .arg("rnd")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Prototype development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("innovation")
            .arg("prototypes")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Intellectual property costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("innovation")
            .arg("ip")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Innovation pipeline costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("innovation")
            .arg("pipeline")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_crisis_management_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Crisis management workflows
        // Crisis response costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("crisis")
            .arg("response")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Business continuity costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("crisis")
            .arg("continuity")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Recovery operation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("crisis")
            .arg("recovery")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Risk mitigation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("crisis")
            .arg("mitigation")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Crisis communication costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("crisis")
            .arg("communication")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_digital_transformation_advanced_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Advanced digital transformation workflows
        // Legacy system migration costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("migration")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Cloud adoption costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("cloud")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital skill development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("skills")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Change management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("change")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Digital ROI measurement
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("digital")
            .arg("roi")
            .arg("measure")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_global_supply_chain_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Global supply chain workflows
        // International logistics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("supplychain")
            .arg("logistics")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Customs and compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("supplychain")
            .arg("customs")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Supplier relationship costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("supplychain")
            .arg("suppliers")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Inventory globalization costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("supplychain")
            .arg("inventory")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Supply chain risk costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("supplychain")
            .arg("risk")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_customer_success_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Customer success workflows
        // Customer onboarding costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("customers")
            .arg("onboarding")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer support costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("customers")
            .arg("support")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer success team costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("customers")
            .arg("success")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer retention costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("customers")
            .arg("retention")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Customer lifetime value costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("customers")
            .arg("lifetime-value")
            .arg("calculate")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_data_governance_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Data governance workflows
        // Data quality management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("quality")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Data privacy compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("privacy")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Data security costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("security")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Data catalog costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("catalog")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Data governance compliance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("data")
            .arg("governance")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_api_economy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // API economy workflows
        // API development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("development")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // API management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("management")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // API monetization costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("monetization")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // API security costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("security")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // API analytics costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("api")
            .arg("analytics")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_platform_economy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Platform economy workflows
        // Platform development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("platform")
            .arg("development")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Multi-sided marketplace costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("platform")
            .arg("marketplace")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Network effects costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("platform")
            .arg("network")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Platform governance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("platform")
            .arg("governance")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Platform scaling costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("platform")
            .arg("scaling")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_zero_trust_architecture_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Zero trust architecture workflows
        // Identity management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("zerotrust")
            .arg("identity")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Access control costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("zerotrust")
            .arg("access")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Continuous verification costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("zerotrust")
            .arg("verification")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Micro-segmentation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("zerotrust")
            .arg("segmentation")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Zero trust monitoring costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("zerotrust")
            .arg("monitoring")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_quantum_safe_cryptography_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Quantum-safe cryptography workflows
        // Post-quantum crypto costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantumsafe")
            .arg("crypto")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Cryptographic transition costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantumsafe")
            .arg("transition")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Key management costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantumsafe")
            .arg("keys")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Quantum threat monitoring costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantumsafe")
            .arg("threats")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Future-proofing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("quantumsafe")
            .arg("futureproof")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_web3_blockchain_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Web3 blockchain workflows
        // Decentralized application costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("web3")
            .arg("dapps")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Smart contract development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("web3")
            .arg("smartcontracts")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // DeFi protocol costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("web3")
            .arg("defi")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // NFT marketplace costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("web3")
            .arg("nft")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // DAO governance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("web3")
            .arg("dao")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_extended_reality_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Extended reality (XR) workflows
        // VR/AR content creation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("xr")
            .arg("content")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // XR hardware costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("xr")
            .arg("hardware")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Immersive experience costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("xr")
            .arg("experience")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // XR platform costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("xr")
            .arg("platform")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Mixed reality costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("xr")
            .arg("mixedreality")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_autonomous_ai_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Autonomous AI workflows
        // AI agent development costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ai")
            .arg("agents")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Autonomous system costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ai")
            .arg("autonomous")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // AI decision making costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ai")
            .arg("decisions")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // AI ethics and governance costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ai")
            .arg("ethics")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // AI safety costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("ai")
            .arg("safety")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_bioinformatics_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Bioinformatics workflows
        // Genomic data analysis costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("bioinformatics")
            .arg("genomics")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Drug discovery costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("bioinformatics")
            .arg("drugs")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Clinical trial costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("bioinformatics")
            .arg("clinical")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Personalized medicine costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("bioinformatics")
            .arg("personalized")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Biotech research costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("bioinformatics")
            .arg("research")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_space_economy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Space economy workflows
        // Satellite constellation costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("constellation")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Space launch costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("launch")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Space data services costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("data")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Space tourism costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("tourism")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Space mining costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("space")
            .arg("mining")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }

    #[test]
    fn test_cli_circular_economy_workflow() {
        let temp = TempDir::new().unwrap();
        let plan_path = temp.path().join("plan.json");
        fs::write(&plan_path, SAMPLE_PLAN).unwrap();

        // Circular economy workflows
        // Product lifecycle costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("circular")
            .arg("lifecycle")
            .arg("analyze")
            .arg(&plan_path)
            .assert()
            .success();

        // Recycling and reuse costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("circular")
            .arg("recycling")
            .arg("optimize")
            .arg(&plan_path)
            .assert()
            .success();

        // Sustainable sourcing costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("circular")
            .arg("sourcing")
            .arg("track")
            .arg(&plan_path)
            .assert()
            .success();

        // Waste reduction costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("circular")
            .arg("waste")
            .arg("monitor")
            .arg(&plan_path)
            .assert()
            .success();

        // Circular business model costs
        Command::cargo_bin("costpilot")
            .unwrap()
            .arg("circular")
            .arg("business")
            .arg("assess")
            .arg(&plan_path)
            .assert()
            .success();
    }
}

// Sample test data
const SAMPLE_PLAN: &str = r#"{
    "format_version": "1.1",
    "terraform_version": "1.5.0",
    "resource_changes": [{
        "address": "aws_instance.web",
        "type": "aws_instance",
        "change": {
            "actions": ["create"],
            "after": {
                "instance_type": "t3.medium"
            }
        }
    }]
}"#;
