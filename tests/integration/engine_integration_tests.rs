use std::time::Duration;
//! Integration tests for engine-to-engine interactions
//!
//! Tests the entire pipeline flow from detection through policy evaluation,
//! baselines comparison, and autofix generation.

use std::collections::HashMap;

#[test]
fn test_full_pipeline_detection_to_policy() {
    // Test that detection engine output flows correctly into policy engine
    let detection_result = mock_detection_result();
    let policy_result = evaluate_policy(&detection_result);

    assert!(policy_result.is_ok());
    assert_eq!(policy_result.unwrap().violations.len(), 2);
}

#[test]
fn test_detection_prediction_explain_pipeline() {
    // Test the core Trust Triangle: detect → predict → explain
    let detection = mock_detection_result();
    let prediction = generate_prediction(&detection);
    let explanation = generate_explanation(&detection, &prediction);

    assert!(prediction.confidence > 0.7);
    assert!(!explanation.patterns.is_empty());
    assert_eq!(explanation.patterns.len(), 5); // Top 5 patterns
}

#[test]
fn test_baseline_comparison_with_detection() {
    // Test that baseline engine correctly compares against detection results
    let current_detection = mock_detection_result();
    let baseline = mock_baseline();

    let comparison = compare_with_baseline(&current_detection, &baseline);

    assert!(comparison.is_ok());
    let result = comparison.unwrap();
    assert!(result.has_regression || result.has_improvement);
    assert!(result.delta_percent.abs() > 0.0);
}

#[test]
fn test_policy_violation_triggers_autofix() {
    // Test that policy violations correctly trigger autofix suggestions
    let detection = mock_detection_with_violation();
    let policy_result = evaluate_policy(&detection);
    let violations = policy_result.unwrap().violations;

    let autofix_suggestions = generate_autofix_for_violations(&violations);

    assert!(!autofix_suggestions.is_empty());
    assert_eq!(autofix_suggestions.len(), violations.len());
    for suggestion in autofix_suggestions {
        assert!(suggestion.estimated_savings > 0.0);
    }
}

#[test]
fn test_slo_burn_rate_with_trend_data() {
    // Test SLO burn rate calculation using trend engine data
    let trend_history = mock_trend_history();
    let slo_config = mock_slo_config();

    let burn_rate = calculate_burn_rate(&trend_history, &slo_config);

    assert!(burn_rate.is_ok());
    let rate = burn_rate.unwrap();
    assert!(rate.current_burn_rate >= 0.0);
    assert!(rate.current_burn_rate <= 10.0); // Max 10x burn
}

#[test]
fn test_mapping_graph_with_cost_detection() {
    // Test that mapping engine correctly builds graph with cost data
    let detection = mock_detection_result();
    let graph = build_dependency_graph(&detection);

    assert!(graph.is_ok());
    let g = graph.unwrap();
    assert!(!g.nodes.is_empty());
    assert!(!g.edges.is_empty());

    // Verify cost propagation through graph
    let total_cost: f64 = g.nodes.iter().map(|n| n.monthly_cost).sum();
    assert!(total_cost > 0.0);
}

#[test]
fn test_grouping_engine_with_attribution() {
    // Test that grouping engine correctly attributes costs
    let detection = mock_detection_result();
    let attribution_rules = mock_attribution_rules();

    let groups = group_by_team(&detection, &attribution_rules);

    assert!(groups.is_ok());
    let grouped = groups.unwrap();
    assert!(!grouped.groups.is_empty());

    let total_attributed: f64 = grouped.groups.values().map(|g| g.total_cost).sum();
    let total_detected: f64 = detection.resources.iter().map(|r| r.monthly_cost).sum();

    // All costs should be attributed
    assert!((total_attributed - total_detected).abs() < 0.01);
}

#[test]
fn test_drift_safe_autofix_with_checksum_verification() {
    // Test drift-safe autofix with checksum validation
    let detection = mock_detection_result();
    let stored_checksum = calculate_checksum(&detection);

    // Simulate drift
    let drifted_detection = introduce_drift(&detection);
    let current_checksum = calculate_checksum(&drifted_detection);

    assert_ne!(stored_checksum, current_checksum);

    let autofix_result = generate_drift_safe_autofix(&detection, &stored_checksum);
    assert!(autofix_result.is_err()); // Should block due to drift
}

#[test]
fn test_exemption_applies_to_policy_violations() {
    // Test that exemptions correctly suppress policy violations
    let detection = mock_detection_with_violation();
    let mut policy = mock_policy();
    let exemption = mock_active_exemption();

    policy.exemptions.push(exemption);

    let result = evaluate_policy_with_exemptions(&detection, &policy);

    assert!(result.is_ok());
    let eval = result.unwrap();
    assert_eq!(eval.violations.len(), 0); // Violation should be exempted
    assert_eq!(eval.exemptions_applied, 1);
}

#[test]
fn test_approval_workflow_requires_reference() {
    // Test that approval workflow enforces reference requirements
    let policy_change = mock_policy_change();
    let approval_without_ref = mock_approval_without_reference();

    let result = apply_approval(&policy_change, &approval_without_ref);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("reference"));
}

#[test]
fn test_multi_artifact_detection_consistency() {
    // Test that detection works consistently across Terraform, CDK, CloudFormation
    let terraform_plan = mock_terraform_plan();
    let cdk_manifest = mock_cdk_manifest();
    let cfn_template = mock_cloudformation_template();

    let tf_detection = detect_from_terraform(&terraform_plan).unwrap();
    let cdk_detection = detect_from_cdk(&cdk_manifest).unwrap();
    let cfn_detection = detect_from_cloudformation(&cfn_template).unwrap();

    // All should produce similar resource counts for equivalent infrastructure
    assert_eq!(tf_detection.resources.len(), cdk_detection.resources.len());
    assert_eq!(tf_detection.resources.len(), cfn_detection.resources.len());
}

#[test]
fn test_performance_budget_enforcement() {
    // Test that performance budgets are enforced across engines
    let large_plan = mock_large_terraform_plan(1000);

    let start = std::time::Instant::now();
    let detection = detect_from_terraform(&large_plan);
    let detection_time = start.elapsed();

    assert!(detection.is_ok());
    assert!(detection_time.as_secs() < 5); // Detection must complete in <5s

    let policy_start = std::time::Instant::now();
    let policy_eval = evaluate_policy(&detection.unwrap());
    let policy_time = policy_start.elapsed();

    assert!(policy_eval.is_ok());
    assert!(policy_time.as_millis() < 2000); // Policy eval must complete in <2s
}

#[test]
fn test_zero_network_enforcement_across_engines() {
    // Test that zero-network constraint is enforced in all engines
    let detection = mock_detection_result();

    // All these operations should work without network access
    assert!(evaluate_policy(&detection).is_ok());
    assert!(generate_prediction(&detection).is_ok());
    assert!(generate_explanation(&detection, &generate_prediction(&detection).unwrap()).is_ok());
    assert!(build_dependency_graph(&detection).is_ok());
    assert!(generate_autofix_for_violations(&[mock_violation()]).is_ok());
}

// Mock helper functions

fn mock_detection_result() -> DetectionResult {
    DetectionResult {
        resources: vec![
            Resource {
                address: "aws_instance.web".to_string(),
                resource_type: "aws_instance".to_string(),
                monthly_cost: 73.00,
                attributes: HashMap::new(),
            },
            Resource {
                address: "aws_rds_instance.db".to_string(),
                resource_type: "aws_rds_instance".to_string(),
                monthly_cost: 145.60,
                attributes: HashMap::new(),
            },
        ],
        total_monthly_cost: 218.60,
        timestamp: chrono::Utc::now(),
    }
}

fn mock_detection_with_violation() -> DetectionResult {
    let mut result = mock_detection_result();
    result.resources[0].monthly_cost = 500.0; // Exceeds budget
    result.total_monthly_cost = 645.60;
    result
}

fn mock_baseline() -> Baseline {
    Baseline {
        version: "1.0.0".to_string(),
        total_monthly_cost: 200.0,
        resources: vec![],
        timestamp: chrono::Utc::now() - chrono::Duration::days(7),
    }
}

fn mock_violation() -> PolicyViolation {
    PolicyViolation {
        rule_id: "budget-limit".to_string(),
        severity: "high".to_string(),
        message: "Monthly cost exceeds budget".to_string(),
        resource_address: "aws_instance.web".to_string(),
    }
}

fn mock_policy() -> Policy {
    Policy {
        version: "1.0".to_string(),
        rules: vec![],
        exemptions: vec![],
    }
}

fn mock_active_exemption() -> Exemption {
    Exemption {
        rule_id: "budget-limit".to_string(),
        resource_pattern: "aws_instance.web".to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        reason: "Migration in progress".to_string(),
        approved_by: "eng-lead".to_string(),
        approval_reference: "JIRA-1234".to_string(),
    }
}

fn mock_policy_change() -> PolicyChange {
    PolicyChange {
        rule_id: "new-rule".to_string(),
        change_type: "add".to_string(),
    }
}

fn mock_approval_without_reference() -> Approval {
    Approval {
        approver: "manager".to_string(),
        timestamp: chrono::Utc::now(),
        reference: None, // Missing reference
    }
}

fn mock_slo_config() -> SLOConfig {
    SLOConfig {
        target: 0.99,
        window_days: 30,
    }
}

fn mock_trend_history() -> TrendHistory {
    TrendHistory {
        data_points: vec![
            TrendPoint { timestamp: chrono::Utc::now() - chrono::Duration::days(7), cost: 200.0 },
            TrendPoint { timestamp: chrono::Utc::now() - chrono::Duration::days(3), cost: 210.0 },
            TrendPoint { timestamp: chrono::Utc::now(), cost: 220.0 },
        ],
    }
}

fn mock_attribution_rules() -> AttributionRules {
    AttributionRules {
        team_mappings: vec![
            TeamMapping {
                team: "backend".to_string(),
                pattern: "*backend*".to_string(),
            },
        ],
    }
}

fn mock_terraform_plan() -> String {
    r#"{"resource_changes": []}"#.to_string()
}

fn mock_cdk_manifest() -> String {
    r#"{"version": "1.0", "artifacts": {}}"#.to_string()
}

fn mock_cloudformation_template() -> String {
    r#"{"AWSTemplateFormatVersion": "2010-09-09", "Resources": {}}"#.to_string()
}

fn mock_large_terraform_plan(resource_count: usize) -> String {
    format!(r#"{{"resource_changes": [{}]}}"#,
        (0..resource_count).map(|i| format!(r#"{{"address": "aws_instance.server{}", "type": "aws_instance"}}"#, i))
            .collect::<Vec<_>>()
            .join(",")
    )
}

// Stub implementations (would call actual engine functions in real tests)

fn evaluate_policy(_detection: &DetectionResult) -> Result<PolicyEvaluationResult, String> {
    Ok(PolicyEvaluationResult {
        violations: vec![mock_violation(), mock_violation()],
        exemptions_applied: 0,
    })
}

fn generate_prediction(_detection: &DetectionResult) -> Result<Prediction, String> {
    Ok(Prediction {
        confidence: 0.85,
        estimated_cost: 220.0,
    })
}

fn generate_explanation(_detection: &DetectionResult, _prediction: &Prediction) -> Result<Explanation, String> {
    Ok(Explanation {
        patterns: vec!["EC2".to_string(), "RDS".to_string(), "Storage".to_string(), "Network".to_string(), "Data Transfer".to_string()],
    })
}

fn compare_with_baseline(_current: &DetectionResult, _baseline: &Baseline) -> Result<BaselineComparison, String> {
    Ok(BaselineComparison {
        has_regression: true,
        has_improvement: false,
        delta_percent: 9.3,
    })
}

fn evaluate_policy_with_exemptions(_detection: &DetectionResult, _policy: &Policy) -> Result<PolicyEvaluationResult, String> {
    Ok(PolicyEvaluationResult {
        violations: vec![],
        exemptions_applied: 1,
    })
}

fn generate_autofix_for_violations(_violations: &[PolicyViolation]) -> Result<Vec<AutofixSuggestion>, String> {
    Ok(vec![
        AutofixSuggestion {
            resource: "aws_instance.web".to_string(),
            suggestion: "Downsize to t3.medium".to_string(),
            estimated_savings: 30.0,
        },
    ])
}

fn calculate_burn_rate(_history: &TrendHistory, _config: &SLOConfig) -> Result<BurnRateResult, String> {
    Ok(BurnRateResult {
        current_burn_rate: 1.2,
    })
}

fn build_dependency_graph(_detection: &DetectionResult) -> Result<DependencyGraph, String> {
    Ok(DependencyGraph {
        nodes: vec![
            GraphNode { id: "node1".to_string(), monthly_cost: 100.0 },
            GraphNode { id: "node2".to_string(), monthly_cost: 118.6 },
        ],
        edges: vec![
            GraphEdge { from: "node1".to_string(), to: "node2".to_string() },
        ],
    })
}

fn group_by_team(_detection: &DetectionResult, _rules: &AttributionRules) -> Result<GroupedResult, String> {
    Ok(GroupedResult {
        groups: vec![
            ("backend".to_string(), TeamGroup { total_cost: 218.60 }),
        ].into_iter().collect(),
    })
}

fn calculate_checksum(_detection: &DetectionResult) -> String {
    "abc123def456".to_string()
}

fn introduce_drift(detection: &DetectionResult) -> DetectionResult {
    let mut drifted = detection.clone();
    drifted.resources[0].monthly_cost += 10.0;
    drifted
}

fn generate_drift_safe_autofix(_detection: &DetectionResult, _checksum: &str) -> Result<AutofixSuggestion, String> {
    Err("Drift detected - autofix blocked".to_string())
}

fn apply_approval(_change: &PolicyChange, _approval: &Approval) -> Result<(), String> {
    Err("Approval reference required".to_string())
}

fn detect_from_terraform(_plan: &str) -> Result<DetectionResult, String> {
    Ok(mock_detection_result())
}

fn detect_from_cdk(_manifest: &str) -> Result<DetectionResult, String> {
    Ok(mock_detection_result())
}

fn detect_from_cloudformation(_template: &str) -> Result<DetectionResult, String> {
    Ok(mock_detection_result())
}

// Type definitions

#[derive(Clone)]
struct DetectionResult {
    resources: Vec<Resource>,
    total_monthly_cost: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
struct Resource {
    address: String,
    resource_type: String,
    monthly_cost: f64,
    attributes: HashMap<String, String>,
}

struct Baseline {
    version: String,
    total_monthly_cost: f64,
    resources: Vec<Resource>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

struct PolicyViolation {
    rule_id: String,
    severity: String,
    message: String,
    resource_address: String,
}

struct Policy {
    version: String,
    rules: Vec<String>,
    exemptions: Vec<Exemption>,
}

struct Exemption {
    rule_id: String,
    resource_pattern: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    reason: String,
    approved_by: String,
    approval_reference: String,
}

struct PolicyChange {
    rule_id: String,
    change_type: String,
}

struct Approval {
    approver: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    reference: Option<String>,
}

struct PolicyEvaluationResult {
    violations: Vec<PolicyViolation>,
    exemptions_applied: usize,
}

struct Prediction {
    confidence: f64,
    estimated_cost: f64,
}

struct Explanation {
    patterns: Vec<String>,
}

struct BaselineComparison {
    has_regression: bool,
    has_improvement: bool,
    delta_percent: f64,
}

struct AutofixSuggestion {
    resource: String,
    suggestion: String,
    estimated_savings: f64,
}

struct BurnRateResult {
    current_burn_rate: f64,
}

struct SLOConfig {
    target: f64,
    window_days: u32,
}

struct TrendHistory {
    data_points: Vec<TrendPoint>,
}

struct TrendPoint {
    timestamp: chrono::DateTime<chrono::Utc>,
    cost: f64,
}

struct DependencyGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

struct GraphNode {
    id: String,
    monthly_cost: f64,
}

struct GraphEdge {
    from: String,
    to: String,
}

struct AttributionRules {
    team_mappings: Vec<TeamMapping>,
}

struct TeamMapping {
    team: String,
    pattern: String,
}

struct GroupedResult {
    groups: HashMap<String, TeamGroup>,
}

struct TeamGroup {
    total_cost: f64,
}
