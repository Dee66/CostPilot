// Anti-pattern detection - MVP top 5 patterns + advanced optimization detection

use crate::engines::shared::models::{CostEstimate, ResourceChange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anti-pattern detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub pattern_id: String,
    pub pattern_name: String,
    pub description: String,
    pub severity: String,
    pub detected_in: String,
    pub evidence: Vec<String>,
    pub suggested_fix: Option<String>,
    pub cost_impact: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumptions: Option<Vec<String>>,
}

// ============================================================================
// STATIC DATA TABLES
// ============================================================================

/// Instance vCPU counts by type (us-east-1 on-demand pricing basis)
fn get_instance_vcpu(instance_type: &str) -> Option<u32> {
    let map: HashMap<&str, u32> = [
        // t3 family
        ("t3.nano", 2),
        ("t3.micro", 2),
        ("t3.small", 2),
        ("t3.medium", 2),
        ("t3.large", 2),
        ("t3.xlarge", 4),
        ("t3.2xlarge", 8),
        // m5 family
        ("m5.large", 2),
        ("m5.xlarge", 4),
        ("m5.2xlarge", 8),
        ("m5.4xlarge", 16),
        ("m5.8xlarge", 32),
        ("m5.12xlarge", 48),
        ("m5.16xlarge", 64),
        ("m5.24xlarge", 96),
        // c5 family
        ("c5.large", 2),
        ("c5.xlarge", 4),
        ("c5.2xlarge", 8),
        ("c5.4xlarge", 16),
        ("c5.9xlarge", 36),
        ("c5.12xlarge", 48),
        ("c5.18xlarge", 72),
        ("c5.24xlarge", 96),
        // r5 family
        ("r5.large", 2),
        ("r5.xlarge", 4),
        ("r5.2xlarge", 8),
        ("r5.4xlarge", 16),
        ("r5.8xlarge", 32),
        ("r5.12xlarge", 48),
        ("r5.16xlarge", 64),
        ("r5.24xlarge", 96),
    ]
    .iter()
    .cloned()
    .collect();

    map.get(instance_type).copied()
}

/// On-demand pricing per hour (us-east-1, documented 2026-01-06)
fn get_instance_hourly_price(instance_type: &str) -> Option<f64> {
    let map: HashMap<&str, f64> = [
        // t3 family
        ("t3.nano", 0.0052),
        ("t3.micro", 0.0104),
        ("t3.small", 0.0208),
        ("t3.medium", 0.0416),
        ("t3.large", 0.0832),
        ("t3.xlarge", 0.1664),
        ("t3.2xlarge", 0.3328),
        // m5 family
        ("m5.large", 0.096),
        ("m5.xlarge", 0.192),
        ("m5.2xlarge", 0.384),
        ("m5.4xlarge", 0.768),
        ("m5.8xlarge", 1.536),
        ("m5.12xlarge", 2.304),
        ("m5.16xlarge", 3.072),
        ("m5.24xlarge", 4.608),
        // c5 family
        ("c5.large", 0.085),
        ("c5.xlarge", 0.17),
        ("c5.2xlarge", 0.34),
        ("c5.4xlarge", 0.68),
        ("c5.9xlarge", 1.53),
        ("c5.12xlarge", 2.04),
        ("c5.18xlarge", 3.06),
        ("c5.24xlarge", 4.08),
        // r5 family
        ("r5.large", 0.126),
        ("r5.xlarge", 0.252),
        ("r5.2xlarge", 0.504),
        ("r5.4xlarge", 1.008),
        ("r5.8xlarge", 2.016),
        ("r5.12xlarge", 3.024),
        ("r5.16xlarge", 4.032),
        ("r5.24xlarge", 6.048),
    ]
    .iter()
    .cloned()
    .collect();

    map.get(instance_type).copied()
}

/// Extract instance family prefix (e.g., "c5.4xlarge" -> "c5")
fn extract_instance_family(instance_type: &str) -> Option<&str> {
    instance_type.split('.').next()
}

/// Maximum reasonable vCPU by environment tag
fn get_max_reasonable_vcpu_by_environment(env: &str) -> u32 {
    match env.to_lowercase().as_str() {
        "dev" | "development" => 16,
        "test" | "testing" => 32,
        "staging" | "stage" => 64,
        "prod" | "production" => 128,
        _ => 32, // default to test threshold
    }
}

/// Instance family classifications
fn get_compute_families() -> Vec<&'static str> {
    vec!["c5", "c6i", "c7g", "c6g"]
}

fn get_memory_families() -> Vec<&'static str> {
    vec!["r5", "r6i", "r7g", "x1", "x2"]
}

fn get_burstable_families() -> Vec<&'static str> {
    vec!["t3", "t4g", "t2"]
}

fn get_general_families() -> Vec<&'static str> {
    vec!["m5", "m6i", "m7g"]
}

/// Tag keyword to recommended instance family mapping
fn get_workload_to_family_hints() -> HashMap<&'static str, Vec<&'static str>> {
    let mut map = HashMap::new();
    map.insert("light-processing", vec!["t3", "t4g"]);
    map.insert("light", vec!["t3", "t4g"]);
    map.insert("compute-intensive", vec!["c5", "c6i", "c7g"]);
    map.insert("compute", vec!["c5", "c6i"]);
    map.insert("cpu-intensive", vec!["c5", "c6i"]);
    map.insert("memory-intensive", vec!["r5", "r6i"]);
    map.insert("memory", vec!["r5", "r6i"]);
    map.insert("database", vec!["r5", "m5"]);
    map.insert("batch-processing", vec!["c5", "m5"]);
    map.insert("batch", vec!["c5", "m5"]);
    map.insert("web-server", vec!["t3", "m5"]);
    map.insert("web", vec!["t3", "m5"]);
    map
}

/// Storage pricing (us-east-1, $/GB/month, documented 2026-01-06)
const GP2_COST_PER_GB: f64 = 0.10;
const GP3_COST_PER_GB: f64 = 0.08;
const GP3_BASE_IOPS: u32 = 3000;
const GP3_MAX_IOPS: u32 = 16000;
const GP3_IOPS_COST: f64 = 0.005; // $/IOPS/month above 3000
const IO1_IOPS_COST: f64 = 0.065; // $/IOPS/month
#[allow(dead_code)]
const IO2_IOPS_COST: f64 = 0.065; // $/IOPS/month

/// Thresholds for gross over-provisioning
const MAX_REASONABLE_STORAGE_GB: u32 = 10_000; // 10TB
const MAX_REASONABLE_IOPS: u32 = 64_000;
const RDS_LARGE_STORAGE_THRESHOLD_GB: u32 = 5_000; // 5TB - flag for review
#[allow(dead_code)]
const RDS_GP2_TO_GP3_MIN_GB: u32 = 1_000; // Only flag gp2 migrations for databases >1TB
const LAMBDA_EXCESSIVE_TIMEOUT_SEC: u32 = 120; // 2 minutes
const LAMBDA_CRITICAL_TIMEOUT_SEC: u32 = 300; // 5 minutes
const SECURITY_GROUP_COMPLEX_THRESHOLD: u32 = 50; // Rules count
const LARGE_EBS_VOLUME_GB: u32 = 1_000; // 1TB
const NAT_GATEWAY_HOURLY_COST: f64 = 0.045; // $/hour
const NAT_GATEWAY_MONTHLY_HOURS: f64 = 730.0; // Average hours per month

/// Phase 2 thresholds
const MICROSERVICES_SMALL_INSTANCE_THRESHOLD: u32 = 5; // 5+ small instances
const ASG_MIN_CAPACITY_THRESHOLD: u32 = 10; // Review ASGs with min_size >= 10

/// Phase 3 thresholds
#[allow(dead_code)]
const S3_LARGE_BUCKET_OBJECTS: u64 = 100_000; // Flag buckets with >100K objects without lifecycle
#[allow(dead_code)]
const EFS_LARGE_SIZE_GB: u32 = 1000; // 1TB - flag for IA storage class review
const WAF_RULE_THRESHOLD: u32 = 50; // Flag WAF ACLs with >50 rules
const IAM_POLICY_SIZE_THRESHOLD: u32 = 3000; // Flag large IAM policies (characters)

/// ECS Fargate thresholds
const FARGATE_HIGH_CPU_THRESHOLD: u32 = 2048; // 2 vCPU
const FARGATE_HIGH_MEMORY_THRESHOLD: u32 = 8192; // 8GB
#[allow(dead_code)]
const FARGATE_OPTIMAL_CPU_MEMORY_RATIO: f64 = 4.0; // 1:4 CPU:Memory ratio is typical

// ============================================================================
// BATCH DETECTION (NEW) - Processes multiple resources
// ============================================================================

/// Detect anti-patterns across multiple resources (batch analysis)
/// This enables consolidation detection and cross-resource pattern matching
pub fn detect_anti_patterns_batch(
    changes: &[ResourceChange],
    estimates: &HashMap<String, CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Multi-resource consolidation detection
    patterns.extend(detect_multi_instance_consolidation(changes));

    // Phase 2: Microservices consolidation (cross-resource analysis)
    let microservices_patterns = detect_microservices_consolidation(changes, estimates);
    patterns.extend(microservices_patterns);

    // Per-resource advanced detection (with full context)
    for change in changes {
        let estimate = estimates.get(&change.resource_id);

        // Phase 2: Environment detection for context-aware filtering
        let _environment = detect_environment(change);

        // Over-provisioning detection
        if let Some(pattern) = detect_gross_overprovisioning(change, estimate) {
            patterns.push(pattern);
        }

        // Instance family mismatch
        if let Some(pattern) = detect_instance_family_mismatch(change) {
            patterns.push(pattern);
        }

        // Storage inefficiency
        patterns.extend(detect_storage_inefficiency(change));

        // Phase 1 expansions
        patterns.extend(detect_rds_storage_optimization(change, estimate));
        patterns.extend(detect_nat_gateway_optimization(change));
        patterns.extend(detect_lambda_timeout_optimization(change));
        patterns.extend(detect_security_group_complexity(change));
        patterns.extend(detect_elasticache_rightsizing(change, estimate));

        // Phase 2 additions
        if let Some(pattern) = detect_bastion_host_rightsizing(change, estimate) {
            patterns.push(pattern);
        }
        patterns.extend(detect_development_environment_rightsizing(change, estimate));
        patterns.extend(detect_large_ebs_volume_optimization(change));
        patterns.extend(detect_regional_optimization(change));
        patterns.extend(detect_asg_optimization(change));

        // Phase 3 additions
        patterns.extend(detect_api_gateway_optimization(change));
        patterns.extend(detect_cloudfront_optimization(change));
        patterns.extend(detect_efs_optimization(change));
        patterns.extend(detect_s3_lifecycle_optimization(change));
        patterns.extend(detect_vpc_endpoint_opportunity(change));
        patterns.extend(detect_waf_optimization(change));
        patterns.extend(detect_iam_optimization(change));
        patterns.extend(detect_reserved_instance_opportunity(change, estimate));
        patterns.extend(detect_spot_instance_opportunity(change));
        patterns.extend(detect_ecs_fargate_oversized(change, estimate));
        patterns.extend(detect_elasticache_reserved_instance(change, estimate));
        patterns.extend(detect_opensearch_optimization(change, estimate));
        patterns.extend(detect_kinesis_optimization(change));
        patterns.extend(detect_ec2_rightsizing(change, estimate));
        patterns.extend(detect_lambda_memory_optimization(change, estimate));
        patterns.extend(detect_s3_storage_class_optimization(change));
    }

    // Phase 2: Apply environment-aware filtering
    patterns = apply_environment_filters(patterns, changes);

    // Limit output to avoid noise (max 3 per category)
    limit_patterns_by_category(patterns)
}

/// Limit patterns to max 3 per category, prioritizing by cost impact
fn limit_patterns_by_category(patterns: Vec<AntiPattern>) -> Vec<AntiPattern> {
    let mut by_category: HashMap<String, Vec<AntiPattern>> = HashMap::new();

    for pattern in patterns {
        let category = pattern.pattern_id.split('_').next().unwrap_or("OTHER").to_string();
        by_category.entry(category).or_insert_with(Vec::new).push(pattern);
    }

    let mut result = Vec::new();
    for (_, mut category_patterns) in by_category {
        // Sort by cost impact descending
        category_patterns.sort_by(|a, b| {
            b.cost_impact.unwrap_or(0.0).partial_cmp(&a.cost_impact.unwrap_or(0.0)).unwrap()
        });
        result.extend(category_patterns.into_iter().take(3));
    }

    result
}

/// Detect environment from resource tags (Phase 2)
fn detect_environment(change: &ResourceChange) -> &str {
    change.tags.get("Environment")
        .or_else(|| change.tags.get("Env"))
        .or_else(|| change.tags.get("environment"))
        .map(|s| s.to_lowercase())
        .map(|s| match s.as_str() {
            "dev" | "development" => "dev",
            "test" | "testing" | "qa" => "test",
            "staging" | "stage" | "stg" => "staging",
            "prod" | "production" | "prd" => "production",
            _ => "unknown"
        })
        .unwrap_or("unknown")
}

/// Apply environment-aware filters (Phase 2)
fn apply_environment_filters(patterns: Vec<AntiPattern>, changes: &[ResourceChange]) -> Vec<AntiPattern> {
    patterns.into_iter().filter(|pattern| {
        // Find the resource this pattern applies to
        let resource = changes.iter().find(|c| c.resource_id == pattern.detected_in);
        let env = resource.map(|r| detect_environment(r)).unwrap_or("unknown");

        // Filter HA recommendations for dev/test
        if env == "dev" || env == "test" {
            if pattern.pattern_id.contains("SINGLE_NODE") ||
               pattern.pattern_id.contains("SINGLE_AZ") ||
               pattern.pattern_id.contains("NO_HA") {
                return false; // Skip HA recommendations for non-prod
            }
        }

        // Keep all other recommendations
        true
    }).collect()
}

// ============================================================================
// HEURISTIC 1: Multi-Resource Consolidation
// ============================================================================

/// Detect consolidation opportunities for multiple small instances
fn detect_multi_instance_consolidation(changes: &[ResourceChange]) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Filter EC2 instances being created or updated
    let instances: Vec<&ResourceChange> = changes
        .iter()
        .filter(|c| {
            c.resource_type == "aws_instance"
                && matches!(
                    c.action,
                    crate::engines::shared::models::ChangeAction::Create
                        | crate::engines::shared::models::ChangeAction::Update
                        | crate::engines::shared::models::ChangeAction::Replace
                )
        })
        .collect();

    if instances.len() < 3 {
        return patterns; // Need at least 3 instances to suggest consolidation
    }

    // Group by module_path and instance family
    let mut groups: HashMap<(Option<String>, String), Vec<&ResourceChange>> = HashMap::new();

    for instance in instances {
        if let Some(config) = &instance.new_config {
            if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
                if let Some(family) = extract_instance_family(instance_type) {
                    let key = (instance.module_path.clone(), family.to_string());
                    groups.entry(key).or_insert_with(Vec::new).push(instance);
                }
            }
        }
    }

    // Analyze each group for consolidation opportunities
    for ((module_path, family), group_instances) in groups {
        if group_instances.len() < 3 {
            continue;
        }

        // Check if all instances have the same instance_type
        let instance_types: Vec<String> = group_instances
            .iter()
            .filter_map(|i| {
                i.new_config
                    .as_ref()
                    .and_then(|c| c.get("instance_type").and_then(|v| v.as_str()))
                    .map(|s| s.to_string())
            })
            .collect();

        if instance_types.is_empty() {
            continue;
        }

        let first_type = &instance_types[0];
        let all_same = instance_types.iter().all(|t| t == first_type);

        if !all_same {
            continue; // Only suggest consolidation for identical instance types
        }

        // Calculate total vCPU
        let vcpu_per_instance = get_instance_vcpu(first_type).unwrap_or(2);
        let total_vcpu = vcpu_per_instance * group_instances.len() as u32;

        // Calculate current monthly cost
        let hourly_price = get_instance_hourly_price(first_type).unwrap_or(0.0);
        let current_monthly_cost = hourly_price * 730.0 * group_instances.len() as f64;

        // Find potential consolidated instance type
        if let Some((consolidated_type, consolidated_monthly_cost)) =
            find_consolidated_instance(&family, total_vcpu)
        {
            let savings = current_monthly_cost - consolidated_monthly_cost;
            let savings_percent = (savings / current_monthly_cost) * 100.0;

            // Only suggest if savings >= 15%
            if savings_percent >= 15.0 {
                let confidence = if has_common_service_tag(&group_instances) {
                    "HIGH"
                } else {
                    "MEDIUM"
                };

                let severity = if savings_percent >= 25.0 { "MEDIUM" } else { "LOW" };

                let resource_ids: Vec<String> =
                    group_instances.iter().map(|i| i.resource_id.clone()).collect();

                let mut evidence = vec![
                    format!(
                        "{}× {} instances{}",
                        group_instances.len(),
                        first_type,
                        module_path
                            .as_ref()
                            .map(|m| format!(" in {}", m))
                            .unwrap_or_default()
                    ),
                    format!("Total: {} vCPU", total_vcpu),
                    format!(
                        "Current cost: ${:.2}/month ({}× ${:.2})",
                        current_monthly_cost,
                        group_instances.len(),
                        hourly_price * 730.0
                    ),
                    format!(
                        "Consolidated option: 1× {} (${:.2}/month)",
                        consolidated_type, consolidated_monthly_cost
                    ),
                    format!(
                        "Potential savings: ${:.2}/month ({:.0}%)",
                        savings, savings_percent
                    ),
                ];

                if confidence == "HIGH" {
                    evidence.push("All instances share common Service tag".to_string());
                }

                let mut thresholds = HashMap::new();
                thresholds.insert(
                    "min_instances".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(3)),
                );
                thresholds.insert(
                    "min_savings_percent".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(15.0).unwrap()
                    ),
                );

                patterns.push(AntiPattern {
                    pattern_id: "MULTI_INSTANCE_CONSOLIDATION".to_string(),
                    pattern_name: "Multiple Small Instances - Consolidation Opportunity"
                        .to_string(),
                    description: format!(
                        "Multiple {} instances could potentially be consolidated into larger instance type",
                        first_type
                    ),
                    severity: severity.to_string(),
                    detected_in: resource_ids.join(", "),
                    evidence,
                    suggested_fix: Some(format!(
                        "Consider consolidating into 1× {} or using Auto Scaling Group with dynamic sizing. \
                        Verify workload characteristics support consolidation.",
                        consolidated_type
                    )),
                    cost_impact: Some(savings),
                    confidence: Some(confidence.to_string()),
                    thresholds: Some(thresholds),
                    assumptions: Some(vec![
                        "Assumes similar workload characteristics across instances".to_string(),
                        "Does not account for availability zone distribution requirements".to_string(),
                        "Pricing based on us-east-1 on-demand rates".to_string(),
                    ]),
                });
            }
        }
    }

    patterns
}

/// Check if instances share a common Service, App, or Role tag
fn has_common_service_tag(instances: &[&ResourceChange]) -> bool {
    let service_tags: Vec<Option<String>> = instances
        .iter()
        .map(|i| {
            i.tags
                .get("Service")
                .or_else(|| i.tags.get("App"))
                .or_else(|| i.tags.get("Role"))
                .cloned()
        })
        .collect();

    if service_tags.is_empty() {
        return false;
    }

    let first = match &service_tags[0] {
        Some(s) => s,
        None => return false,
    };

    service_tags.iter().all(|t| t.as_ref() == Some(first))
}

/// Find consolidated instance type for given vCPU requirement
fn find_consolidated_instance(family: &str, total_vcpu: u32) -> Option<(String, f64)> {
    // Instance sizes in ascending order
    let sizes = ["large", "xlarge", "2xlarge", "4xlarge", "8xlarge", "12xlarge", "16xlarge", "24xlarge"];

    for size in &sizes {
        let instance_type = format!("{}.{}", family, size);
        if let Some(vcpu) = get_instance_vcpu(&instance_type) {
            if vcpu >= total_vcpu {
                if let Some(hourly_price) = get_instance_hourly_price(&instance_type) {
                    let monthly_cost = hourly_price * 730.0;
                    return Some((instance_type, monthly_cost));
                }
            }
        }
    }

    None
}

// ============================================================================
// HEURISTIC 2: Gross Over-Provisioning
// ============================================================================

/// Detect gross over-provisioning based on static thresholds
fn detect_gross_overprovisioning(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    // EC2 over-provisioning
    if change.resource_type == "aws_instance" {
        return detect_ec2_overprovisioning(change, estimate);
    }

    // RDS over-provisioning
    if change.resource_type == "aws_db_instance" {
        return detect_rds_overprovisioning(change, estimate);
    }

    None
}

/// Detect EC2 over-provisioning based on vCPU thresholds and environment
fn detect_ec2_overprovisioning(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    let config = change.new_config.as_ref()?;
    let instance_type = config.get("instance_type")?.as_str()?;
    let vcpu = get_instance_vcpu(instance_type)?;

    // Extract environment tag
    let environment = change
        .tags
        .get("Environment")
        .or_else(|| change.tags.get("Env"))
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    let max_vcpu = get_max_reasonable_vcpu_by_environment(environment);

    if vcpu > max_vcpu {
        let confidence = if environment != "unknown" { "HIGH" } else { "LOW" };
        let severity = if vcpu > max_vcpu * 2 { "HIGH" } else { "MEDIUM" };

        let mut evidence = vec![
            format!("Instance type: {} ({} vCPU)", instance_type, vcpu),
            format!(
                "Environment: {} (typical maximum: {} vCPU)",
                environment, max_vcpu
            ),
            format!(
                "{} vCPU exceeds typical {} environment threshold by {}×",
                vcpu,
                environment,
                (vcpu as f64 / max_vcpu as f64)
            ),
        ];

        if let Some(est) = estimate {
            evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
        }

        let mut thresholds = HashMap::new();
        thresholds.insert(
            "max_vcpu".to_string(),
            serde_json::Value::Number(serde_json::Number::from(max_vcpu)),
        );
        thresholds.insert(
            "environment".to_string(),
            serde_json::Value::String(environment.to_string()),
        );

        return Some(AntiPattern {
            pattern_id: "GROSS_EC2_OVERPROVISIONING".to_string(),
            pattern_name: "EC2 Instance Exceeds Typical Environment Size".to_string(),
            description: format!(
                "{} vCPU instance exceeds typical {} environment maximum of {} vCPU",
                vcpu, environment, max_vcpu
            ),
            severity: severity.to_string(),
            detected_in: change.resource_id.clone(),
            evidence,
            suggested_fix: Some(format!(
                "Verify that {} vCPU is necessary for this {} workload. \
                Typical {} environments use {} vCPU or less. \
                Consider rightsizing or adding justification in tags.",
                vcpu, environment, environment, max_vcpu
            )),
            cost_impact: estimate.map(|e| e.monthly_cost),
            confidence: Some(confidence.to_string()),
            thresholds: Some(thresholds),
            assumptions: Some(vec![
                format!("Assumes {} environment based on tags", environment),
                "Thresholds are conservative guidelines, not hard limits".to_string(),
            ]),
        });
    }

    None
}

/// Detect RDS over-provisioning based on storage and IOPS thresholds
fn detect_rds_overprovisioning(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    let config = change.new_config.as_ref()?;

    // Check allocated_storage
    if let Some(allocated_storage) = config.get("allocated_storage").and_then(|v| v.as_u64()) {
        if allocated_storage > MAX_REASONABLE_STORAGE_GB as u64 {
            let mut evidence = vec![
                format!("Allocated storage: {} GB", allocated_storage),
                format!(
                    "Exceeds {} TB threshold ({}×)",
                    MAX_REASONABLE_STORAGE_GB / 1000,
                    allocated_storage / (MAX_REASONABLE_STORAGE_GB as u64)
                ),
            ];

            if let Some(est) = estimate {
                evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
            }

            let mut thresholds = HashMap::new();
            thresholds.insert(
                "max_storage_gb".to_string(),
                serde_json::Value::Number(serde_json::Number::from(MAX_REASONABLE_STORAGE_GB)),
            );

            return Some(AntiPattern {
                pattern_id: "GROSS_RDS_STORAGE_OVERPROVISIONING".to_string(),
                pattern_name: "RDS Allocated Storage Exceeds 10TB".to_string(),
                description: format!(
                    "{} GB allocated storage exceeds typical 10TB threshold",
                    allocated_storage
                ),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some(
                    "Verify that this storage allocation is correct. \
                    Consider using storage autoscaling if growth pattern is uncertain."
                        .to_string(),
                ),
                cost_impact: estimate.map(|e| e.monthly_cost),
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some(thresholds),
                assumptions: Some(vec![
                    "10TB threshold is conservative guideline".to_string(),
                ]),
            });
        }
    }

    // Check provisioned IOPS
    if let Some(iops) = config.get("iops").and_then(|v| v.as_u64()) {
        if iops > MAX_REASONABLE_IOPS as u64 {
            let evidence = vec![
                format!("Provisioned IOPS: {}", iops),
                format!(
                    "Exceeds {} IOPS threshold",
                    MAX_REASONABLE_IOPS
                ),
            ];

            let mut thresholds = HashMap::new();
            thresholds.insert(
                "max_iops".to_string(),
                serde_json::Value::Number(serde_json::Number::from(MAX_REASONABLE_IOPS)),
            );

            return Some(AntiPattern {
                pattern_id: "GROSS_RDS_IOPS_OVERPROVISIONING".to_string(),
                pattern_name: "RDS Provisioned IOPS Exceeds 64K".to_string(),
                description: format!("{} IOPS exceeds typical threshold", iops),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some(
                    "Verify that this IOPS level is necessary for your workload. \
                    Consider starting with lower IOPS and monitoring performance."
                        .to_string(),
                ),
                cost_impact: None,
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some(thresholds),
                assumptions: None,
            });
        }
    }

    None
}

/// Estimate RDS instance memory in GB based on instance class
#[allow(dead_code)]
fn estimate_instance_memory_gb(instance_class: &str) -> Option<f64> {
    // Extract size from instance class (e.g., db.t3.medium -> medium)
    let size = instance_class.split('.').last()?;

    match size {
        // t3/t4g family
        "micro" => Some(1.0),
        "small" => Some(2.0),
        "medium" => Some(4.0),
        "large" => Some(8.0),
        "xlarge" => Some(16.0),
        "2xlarge" => Some(32.0),
        "4xlarge" => Some(64.0),
        "8xlarge" => Some(128.0),
        "12xlarge" => Some(192.0),
        "16xlarge" => Some(256.0),
        "24xlarge" => Some(384.0),
        _ => None,
    }
}

// ============================================================================
// PHASE 1 EXPANSIONS: High-Priority Optimizations
// ============================================================================

/// Detect RDS storage optimization opportunities (Phase 1)
fn detect_rds_storage_optimization(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_db_instance" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // 1. Large RDS storage (>5TB) - flag for review
    if let Some(allocated_storage) = config.get("allocated_storage").and_then(|v| v.as_u64()) {
        if allocated_storage >= RDS_LARGE_STORAGE_THRESHOLD_GB as u64 && allocated_storage < MAX_REASONABLE_STORAGE_GB as u64 {
            let mut evidence = vec![
                format!("Allocated storage: {} GB ({:.1} TB)", allocated_storage, allocated_storage as f64 / 1000.0),
                "Large storage allocation may indicate over-provisioning".to_string(),
                format!("Exceeds review threshold of {} GB", RDS_LARGE_STORAGE_THRESHOLD_GB),
            ];

            if let Some(est) = estimate {
                let storage_cost = allocated_storage as f64 * GP2_COST_PER_GB;
                evidence.push(format!("Estimated storage cost: ${:.2}/month", storage_cost));
                evidence.push(format!("Total instance cost: ${:.2}/month", est.monthly_cost));
            }

            let mut thresholds = HashMap::new();
            thresholds.insert("review_threshold_gb".to_string(), serde_json::Value::Number(serde_json::Number::from(RDS_LARGE_STORAGE_THRESHOLD_GB)));

            patterns.push(AntiPattern {
                pattern_id: "RDS_LARGE_STORAGE".to_string(),
                pattern_name: "RDS Database Storage Over-Provisioning".to_string(),
                description: format!("{} GB RDS storage allocation exceeds typical database needs", allocated_storage),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some("Review actual database size and growth rate. Consider storage autoscaling. Verify this allocation matches actual needs.".to_string()),
                cost_impact: estimate.map(|e| e.monthly_cost * 0.3), // Storage typically 30% of cost
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some(thresholds),
                assumptions: Some(vec![
                    format!("{} GB threshold based on typical database sizing", RDS_LARGE_STORAGE_THRESHOLD_GB),
                    "Storage cost estimated at $0.10/GB/month for gp2".to_string(),
                ]),
            });
        }
    }

    // 2. RDS gp2 storage - already handled by storage inefficiency but add RDS-specific context
    if let Some(storage_type) = config.get("storage_type").and_then(|v| v.as_str()) {
        if storage_type == "gp2" {
            if let Some(allocated_storage) = config.get("allocated_storage").and_then(|v| v.as_u64()) {
                let gp2_cost = allocated_storage as f64 * GP2_COST_PER_GB;
                let gp3_cost = allocated_storage as f64 * GP3_COST_PER_GB;
                let savings = gp2_cost - gp3_cost;

                if savings >= 10.0 { // Higher threshold for RDS (significant databases)
                    patterns.push(AntiPattern {
                        pattern_id: "RDS_GP2_MIGRATION".to_string(),
                        pattern_name: "RDS gp2 to gp3 Migration Opportunity".to_string(),
                        description: format!("RDS database using gp2 storage ({} GB) - gp3 offers 20% savings", allocated_storage),
                        severity: "MEDIUM".to_string(),
                        detected_in: change.resource_id.clone(),
                        evidence: vec![
                            format!("Storage type: gp2 ({} GB)", allocated_storage),
                            format!("Current storage cost: ${:.2}/month", gp2_cost),
                            format!("gp3 storage cost: ${:.2}/month", gp3_cost),
                            format!("Potential savings: ${:.2}/month ({:.0}%)", savings, (savings / gp2_cost) * 100.0),
                            "gp3 provides same baseline performance with lower cost".to_string(),
                        ],
                        suggested_fix: Some("Migrate RDS storage from gp2 to gp3 for cost savings with same baseline performance.".to_string()),
                        cost_impact: Some(savings),
                        confidence: Some("HIGH".to_string()),
                        thresholds: Some({
                            let mut t = HashMap::new();
                            t.insert("min_savings".to_string(), serde_json::Value::Number(serde_json::Number::from(10)));
                            t
                        }),
                        assumptions: Some(vec!["Pricing based on us-east-1 rates".to_string()]),
                    });
                }
            }
        }
    }

    patterns
}

/// Detect NAT Gateway optimization opportunities (Phase 1)
fn detect_nat_gateway_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_nat_gateway" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // 1. Public NAT gateway without explicit private specification
    let connectivity_type = config.get("connectivity_type").and_then(|v| v.as_str()).unwrap_or("public");

    if connectivity_type == "public" {
        let hourly_cost = NAT_GATEWAY_HOURLY_COST;
        let monthly_base_cost = hourly_cost * NAT_GATEWAY_MONTHLY_HOURS;
        let data_transfer_cost_per_gb = 0.045;

        // Estimate data transfer (assume 100GB/month as conservative baseline)
        let estimated_data_gb = 100.0;
        let estimated_data_cost = estimated_data_gb * data_transfer_cost_per_gb;
        let total_monthly_cost = monthly_base_cost + estimated_data_cost;

        patterns.push(AntiPattern {
            pattern_id: "NAT_GATEWAY_PUBLIC".to_string(),
            pattern_name: "Public NAT Gateway Cost Optimization".to_string(),
            description: format!("Public NAT Gateway costs ${:.2}/month base + data transfer fees", monthly_base_cost),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Connectivity type: {}", connectivity_type),
                format!("Hourly charge: ${:.4}/hour", hourly_cost),
                format!("Monthly base cost: ${:.2}/month ({:.0} hours × ${:.4})", monthly_base_cost, NAT_GATEWAY_MONTHLY_HOURS, hourly_cost),
                format!("Data transfer: ${:.4}/GB processed", data_transfer_cost_per_gb),
                format!("Estimated total: ${:.2}/month (base + ~{}GB data)", total_monthly_cost, estimated_data_gb as u32),
                "VPC Gateway Endpoints: FREE for S3 and DynamoDB traffic".to_string(),
                "VPC Interface Endpoints: $7.30/month + $0.01/GB (cheaper than NAT for high traffic)".to_string(),
            ],
            suggested_fix: Some("High-impact cost reduction: (1) Replace with VPC Gateway Endpoints for S3/DynamoDB (FREE - saves $32.85/month base), (2) Use VPC Interface Endpoints for other AWS services (cheaper for >2.5TB/month), (3) Review if private NAT Gateway sufficient (no hourly charge), (4) Consolidate multi-AZ NAT gateways if HA not critical. Implementation difficulty: EASY for VPC endpoints.".to_string()),
            cost_impact: Some(monthly_base_cost), // Conservative: base cost only
            confidence: Some("HIGH".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("hourly_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(hourly_cost).unwrap()));
                t.insert("data_transfer_cost_per_gb".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(data_transfer_cost_per_gb).unwrap()));
                t
            }),
            assumptions: Some(vec![
                "Pricing based on us-east-1 rates".to_string(),
                "Data transfer estimated at 100GB/month (actual varies by usage)".to_string(),
                "VPC endpoints eliminate NAT Gateway dependency for supported services".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect Lambda timeout optimization opportunities (Phase 1)
fn detect_lambda_timeout_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_lambda_function" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    if let Some(timeout) = config.get("timeout").and_then(|v| v.as_u64()) {
        let timeout_sec = timeout as u32;

        // Critical: timeout > 300 seconds (5 minutes)
        if timeout_sec > LAMBDA_CRITICAL_TIMEOUT_SEC {
            patterns.push(AntiPattern {
                pattern_id: "LAMBDA_CRITICAL_TIMEOUT".to_string(),
                pattern_name: "Lambda Function Excessive Timeout".to_string(),
                description: format!("Lambda timeout {} seconds exceeds recommended maximum", timeout_sec),
                severity: "HIGH".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Timeout: {} seconds ({} minutes)", timeout_sec, timeout_sec / 60),
                    format!("Exceeds {} second (5 minute) threshold", LAMBDA_CRITICAL_TIMEOUT_SEC),
                    "Long timeouts increase cost risk and indicate potential architectural issues".to_string(),
                    "Lambda is optimized for short-duration tasks".to_string(),
                ],
                suggested_fix: Some("Review function execution patterns. Consider: (1) Refactoring into smaller functions, (2) Moving to ECS/Fargate for long-running tasks, (3) Using Step Functions for orchestration, (4) Optimizing code to reduce execution time.".to_string()),
                cost_impact: None,
                confidence: Some("HIGH".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("critical_timeout_sec".to_string(), serde_json::Value::Number(serde_json::Number::from(LAMBDA_CRITICAL_TIMEOUT_SEC)));
                    t
                }),
                assumptions: Some(vec!["Lambda best suited for sub-5-minute executions".to_string()]),
            });
        }
        // Warning: timeout > 120 seconds (2 minutes)
        else if timeout_sec > LAMBDA_EXCESSIVE_TIMEOUT_SEC {
            patterns.push(AntiPattern {
                pattern_id: "LAMBDA_EXCESSIVE_TIMEOUT".to_string(),
                pattern_name: "Lambda Function Timeout May Be Excessive".to_string(),
                description: format!("Lambda timeout {} seconds - review if necessary", timeout_sec),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Timeout: {} seconds", timeout_sec),
                    format!("Exceeds typical {} second threshold", LAMBDA_EXCESSIVE_TIMEOUT_SEC),
                    "Most Lambda functions complete in < 60 seconds".to_string(),
                ],
                suggested_fix: Some("Review actual execution time. Consider reducing timeout to match typical execution patterns plus buffer.".to_string()),
                cost_impact: None,
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("excessive_timeout_sec".to_string(), serde_json::Value::Number(serde_json::Number::from(LAMBDA_EXCESSIVE_TIMEOUT_SEC)));
                    t
                }),
                assumptions: None,
            });
        }
    }

    patterns
}

/// Detect security group complexity issues (Phase 1)
fn detect_security_group_complexity(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_security_group" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Count ingress and egress rules
    let ingress_count = config.get("ingress").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let egress_count = config.get("egress").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
    let total_rules = ingress_count + egress_count;

    if total_rules > SECURITY_GROUP_COMPLEX_THRESHOLD as usize {
        patterns.push(AntiPattern {
            pattern_id: "SECURITY_GROUP_COMPLEX".to_string(),
            pattern_name: "Security Group Complexity Exceeds Threshold".to_string(),
            description: format!("Security group has {} rules (ingress: {}, egress: {})", total_rules, ingress_count, egress_count),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Total rules: {} (threshold: {})", total_rules, SECURITY_GROUP_COMPLEX_THRESHOLD),
                format!("Ingress rules: {}", ingress_count),
                format!("Egress rules: {}", egress_count),
                "Complex security groups are harder to audit and maintain".to_string(),
            ],
            suggested_fix: Some("Consider: (1) Consolidating similar rules, (2) Using security group references instead of CIDR blocks, (3) Splitting into multiple focused security groups, (4) Using prefix lists for common CIDR ranges.".to_string()),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("max_rules".to_string(), serde_json::Value::Number(serde_json::Number::from(SECURITY_GROUP_COMPLEX_THRESHOLD)));
                t
            }),
            assumptions: None,
        });
    }

    // Check for overly broad rules (0.0.0.0/0)
    if let Some(ingress) = config.get("ingress").and_then(|v| v.as_array()) {
        for rule in ingress {
            if let Some(cidr_blocks) = rule.get("cidr_blocks").and_then(|v| v.as_array()) {
                for cidr in cidr_blocks {
                    if let Some(cidr_str) = cidr.as_str() {
                        if cidr_str == "0.0.0.0/0" {
                            patterns.push(AntiPattern {
                                pattern_id: "SECURITY_GROUP_OVERLY_BROAD".to_string(),
                                pattern_name: "Security Group With Overly Broad Rules".to_string(),
                                description: "Security group allows ingress from 0.0.0.0/0 (entire internet)".to_string(),
                                severity: "MEDIUM".to_string(),
                                detected_in: change.resource_id.clone(),
                                evidence: vec![
                                    "Ingress rule allows 0.0.0.0/0 CIDR block".to_string(),
                                    "This exposes resources to the entire internet".to_string(),
                                    "Consider restricting to specific IP ranges".to_string(),
                                ],
                                suggested_fix: Some("Replace 0.0.0.0/0 with specific IP ranges, VPN CIDR blocks, or other security groups. Use AWS WAF or CloudFront for public-facing services.".to_string()),
                                cost_impact: None,
                                confidence: Some("HIGH".to_string()),
                                thresholds: None,
                                assumptions: Some(vec!["0.0.0.0/0 is appropriate only for public load balancers and similar services".to_string()]),
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    patterns
}

/// Detect ElastiCache rightsizing opportunities (Phase 1)
fn detect_elasticache_rightsizing(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_elasticache_cluster" && change.resource_type != "aws_elasticache_replication_group" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check for large instance types
    let node_type = config.get("node_type").or_else(|| config.get("cache_node_type")).and_then(|v| v.as_str());

    if let Some(node_type_str) = node_type {
        // Flag large instance types (2xlarge and above)
        if node_type_str.contains(".2xlarge") || node_type_str.contains(".4xlarge") ||
           node_type_str.contains(".8xlarge") || node_type_str.contains(".12xlarge") {
            let mut evidence = vec![
                format!("Node type: {}", node_type_str),
                "Large ElastiCache instance type detected".to_string(),
            ];

            if let Some(est) = estimate {
                evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
            }

            patterns.push(AntiPattern {
                pattern_id: "ELASTICACHE_LARGE_INSTANCE".to_string(),
                pattern_name: "ElastiCache Large Instance Type".to_string(),
                description: format!("ElastiCache using large instance type: {}", node_type_str),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some("Review actual cache utilization. Consider: (1) Starting with smaller instances and scaling up, (2) Using multiple smaller nodes for better distribution, (3) Reviewing cache hit rates to ensure caching is effective.".to_string()),
                cost_impact: estimate.map(|e| e.monthly_cost),
                confidence: Some("MEDIUM".to_string()),
                thresholds: None,
                assumptions: Some(vec!["Large instances should match actual workload requirements".to_string()]),
            });
        }
    }

    // Check for single-node clusters (if num_cache_nodes is 1)
    if change.resource_type == "aws_elasticache_cluster" {
        if let Some(num_nodes) = config.get("num_cache_nodes").and_then(|v| v.as_u64()) {
            if num_nodes == 1 {
                patterns.push(AntiPattern {
                    pattern_id: "ELASTICACHE_SINGLE_NODE".to_string(),
                    pattern_name: "ElastiCache Single-Node Cluster".to_string(),
                    description: "Single-node ElastiCache cluster lacks redundancy".to_string(),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        "Cluster configuration: 1 node".to_string(),
                        "Single node provides no redundancy".to_string(),
                        "Consider multi-node for production workloads".to_string(),
                    ],
                    suggested_fix: Some("For production workloads, consider multi-node configuration with automatic failover. For dev/test, single-node may be appropriate.".to_string()),
                    cost_impact: None,
                    confidence: Some("MEDIUM".to_string()),
                    thresholds: None,
                    assumptions: Some(vec!["Multi-node recommended for production".to_string()]),
                });
            }
        }
    }

    patterns
}

// ============================================================================
// PHASE 2: Additional Optimizations
// ============================================================================

/// Detect bastion host rightsizing (Phase 2)
fn detect_bastion_host_rightsizing(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    if change.resource_type != "aws_instance" {
        return None;
    }

    // Look for bastion host indicators in tags or name
    let is_bastion = change.tags.get("Name")
        .or_else(|| change.tags.get("Role"))
        .or_else(|| change.tags.get("Purpose"))
        .map(|s| {
            let lower = s.to_lowercase();
            lower.contains("bastion") || lower.contains("jumpbox") || lower.contains("jump-box")
        })
        .unwrap_or(false);

    if !is_bastion {
        return None;
    }

    let config = change.new_config.as_ref()?;
    let instance_type = config.get("instance_type")?.as_str()?;

    // Bastion hosts should use small, burstable instances
    // Flag if using compute-optimized, memory-optimized, or large general-purpose
    let family = extract_instance_family(instance_type)?;

    let inappropriate = get_compute_families().contains(&family) ||
                        get_memory_families().contains(&family) ||
                        instance_type.contains("xlarge");

    if inappropriate {
        let mut evidence = vec![
            format!("Instance type: {} (bastion host)", instance_type),
            "Bastion hosts typically need minimal resources".to_string(),
            "Recommended: t3.micro, t3.small, or t3.medium".to_string(),
        ];

        if let Some(est) = estimate {
            evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
            // Estimate savings with t3.small ($15.18/month)
            let t3_small_cost = 15.18;
            if est.monthly_cost > t3_small_cost {
                evidence.push(format!("Potential savings with t3.small: ${:.2}/month", est.monthly_cost - t3_small_cost));
            }
        }

        return Some(AntiPattern {
            pattern_id: "BASTION_HOST_OVERSIZED".to_string(),
            pattern_name: "Bastion Host Using Oversized Instance".to_string(),
            description: format!("Bastion host using {} - typically only needs t3.micro/small", instance_type),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence,
            suggested_fix: Some("Bastion hosts typically only need t3.micro, t3.small, or t3.medium for SSH/RDP access. Consider downsizing.".to_string()),
            cost_impact: estimate.map(|e| (e.monthly_cost - 15.18).max(0.0)),
            confidence: Some("HIGH".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Bastion hosts identified by tags/naming (bastion, jumpbox)".to_string(),
                "t3.small sufficient for typical bastion workload".to_string(),
            ]),
        });
    }

    None
}

/// Detect development environment rightsizing (Phase 2)
fn detect_development_environment_rightsizing(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Only check EC2 and RDS
    if change.resource_type != "aws_instance" && change.resource_type != "aws_db_instance" {
        return patterns;
    }

    // Check if this is dev/staging environment
    let environment = change.tags.get("Environment")
        .or_else(|| change.tags.get("Env"))
        .map(|s| s.to_lowercase());

    let is_dev = match &environment {
        Some(env) => env == "dev" || env == "development" || env == "staging" || env == "stage" || env == "test",
        None => false,
    };

    if !is_dev {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // For EC2: Flag production-grade instances in dev
    if change.resource_type == "aws_instance" {
        if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
            // Production-grade: 2xlarge and above, or r5/c5 families xlarge and above
            let is_production_grade = instance_type.contains(".2xlarge") ||
                                       instance_type.contains(".4xlarge") ||
                                       instance_type.contains(".8xlarge") ||
                                       (instance_type.starts_with("r5.") && instance_type.contains("xlarge")) ||
                                       (instance_type.starts_with("c5.") && instance_type.contains("xlarge"));

            if is_production_grade {
                let mut evidence = vec![
                    format!("Environment: {} (non-production)", environment.as_ref().unwrap()),
                    format!("Instance type: {} (production-grade)", instance_type),
                    "Development environments typically don't need production-grade resources".to_string(),
                ];

                if let Some(est) = estimate {
                    evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
                }

                patterns.push(AntiPattern {
                    pattern_id: "DEV_ENVIRONMENT_PRODUCTION_INSTANCE".to_string(),
                    pattern_name: "Development Environment Using Production-Grade Instance".to_string(),
                    description: format!("{} environment using production-grade instance: {}", environment.as_ref().unwrap(), instance_type),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence,
                    suggested_fix: Some("Consider using smaller instance types for development/staging. Reserve production-grade instances for production workloads.".to_string()),
                    cost_impact: estimate.map(|e| e.monthly_cost * 0.5), // Estimate 50% savings potential
                    confidence: Some("HIGH".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        format!("Environment classification based on '{}' tag", environment.as_ref().unwrap()),
                    ]),
                });
            }
        }
    }

    // For RDS: Flag large storage allocations in dev
    if change.resource_type == "aws_db_instance" {
        if let Some(allocated_storage) = config.get("allocated_storage").and_then(|v| v.as_u64()) {
            if allocated_storage > 1000 { // >1TB in dev is unusual
                patterns.push(AntiPattern {
                    pattern_id: "DEV_ENVIRONMENT_LARGE_DATABASE".to_string(),
                    pattern_name: "Development Database With Production-Grade Storage".to_string(),
                    description: format!("{} database with {} GB storage", environment.as_ref().unwrap(), allocated_storage),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        format!("Environment: {} (non-production)", environment.as_ref().unwrap()),
                        format!("Allocated storage: {} GB", allocated_storage),
                        "Development databases typically use smaller datasets".to_string(),
                    ],
                    suggested_fix: Some("Consider using database snapshots or subset of production data for development. Typical dev databases: 100-500 GB.".to_string()),
                    cost_impact: Some((allocated_storage as f64 - 500.0) * GP2_COST_PER_GB), // Savings vs 500GB baseline
                    confidence: Some("MEDIUM".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "Development databases typically < 1TB".to_string(),
                    ]),
                });
            }
        }
    }

    patterns
}

/// Detect large EBS volume optimization (Phase 3, early implementation)
fn detect_large_ebs_volume_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_ebs_volume" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    if let Some(size) = config.get("size").and_then(|v| v.as_u64()) {
        if size > LARGE_EBS_VOLUME_GB as u64 {
            let volume_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("gp2");

            patterns.push(AntiPattern {
                pattern_id: "LARGE_EBS_VOLUME".to_string(),
                pattern_name: "Large EBS Volume - Review Type and Sizing".to_string(),
                description: format!("EBS volume {} GB - review if appropriate type", size),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Volume size: {} GB ({} TB)", size, size / 1000),
                    format!("Volume type: {}", volume_type),
                    format!("Exceeds {} GB review threshold", LARGE_EBS_VOLUME_GB),
                ],
                suggested_fix: Some("For large volumes: (1) Verify gp3 is used instead of gp2, (2) Consider Throughput Optimized HDD (st1) for big data/log processing, (3) Review if all allocated storage is actually needed.".to_string()),
                cost_impact: None,
                confidence: Some("LOW".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("large_volume_gb".to_string(), serde_json::Value::Number(serde_json::Number::from(LARGE_EBS_VOLUME_GB)));
                    t
                }),
                assumptions: None,
            });
        }
    }

    patterns
}

// ============================================================================
// PHASE 2: Advanced Cross-Resource Optimizations
// ============================================================================

/// Detect microservices consolidation opportunities (Phase 2)
fn detect_microservices_consolidation(
    resource_changes: &[ResourceChange],
    estimates: &HashMap<String, CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Group small EC2 instances by module path (microservices pattern)
    let mut module_groups: HashMap<String, Vec<(&ResourceChange, Option<&CostEstimate>)>> = HashMap::new();

    for change in resource_changes {
        if change.resource_type != "aws_instance" {
            continue;
        }

        let config = match &change.new_config {
            Some(c) => c,
            None => continue,
        };

        let instance_type = match config.get("instance_type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => continue,
        };

        // Only consider small instances (micro, small, medium)
        if !instance_type.contains(".micro") &&
           !instance_type.contains(".small") &&
           !instance_type.contains(".medium") {
            continue;
        }

        // Extract module path (everything before the resource name)
        let module_path = if let Some(idx) = change.resource_id.rfind('.') {
            &change.resource_id[..idx]
        } else {
            "root"
        };

        let estimate = estimates.get(&change.resource_id);
        module_groups.entry(module_path.to_string())
            .or_insert_with(Vec::new)
            .push((change, estimate));
    }

    // Find modules with 5+ small instances
    for (module_path, instances) in module_groups {
        if instances.len() >= MICROSERVICES_SMALL_INSTANCE_THRESHOLD as usize {
            let instance_types: Vec<String> = instances.iter()
                .filter_map(|(c, _)| {
                    c.new_config.as_ref()
                        .and_then(|cfg| cfg.get("instance_type"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            let total_cost: f64 = instances.iter()
                .filter_map(|(_, est)| est.map(|e| e.monthly_cost))
                .sum();

            let mut evidence = vec![
                format!("Module: {}", module_path),
                format!("Small instances: {}", instances.len()),
                format!("Instance types: {}", instance_types.join(", ")),
            ];

            if total_cost > 0.0 {
                evidence.push(format!("Combined monthly cost: ${:.2}", total_cost));
                // Estimate 30-40% savings from consolidation
                let estimated_savings = total_cost * 0.35;
                evidence.push(format!("Potential savings from consolidation: ${:.2}/month", estimated_savings));
            }

            patterns.push(AntiPattern {
                pattern_id: "MICROSERVICES_CONSOLIDATION".to_string(),
                pattern_name: "Multiple Small Instances - Consolidation Opportunity".to_string(),
                description: format!("{} small instances in module '{}' - consider containerization", instances.len(), module_path),
                severity: "MEDIUM".to_string(),
                detected_in: module_path.clone(),
                evidence,
                suggested_fix: Some("Consider consolidating microservices using: (1) ECS/Fargate for container orchestration, (2) Kubernetes (EKS) for advanced orchestration, (3) Lambda for event-driven workloads, (4) Larger instance with better resource utilization.".to_string()),
                cost_impact: if total_cost > 0.0 { Some(total_cost * 0.35) } else { None },
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("min_instances".to_string(), serde_json::Value::Number(serde_json::Number::from(MICROSERVICES_SMALL_INSTANCE_THRESHOLD)));
                    t
                }),
                assumptions: Some(vec![
                    "Multiple small instances in same module indicate microservices pattern".to_string(),
                    "Consolidation via containers typically saves 30-40%".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect regional optimization opportunities (Phase 2)
fn detect_regional_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Only check expensive resource types
    if change.resource_type != "aws_instance" &&
       change.resource_type != "aws_db_instance" &&
       change.resource_type != "aws_elasticache_cluster" {
        return patterns;
    }

    // Check if provider specifies an expensive region
    // This is a heuristic based on resource_id patterns
    let expensive_regions = vec![
        "us-east-1",  // Reference region
        "eu-west-1",  // ~5-10% more expensive
        "ap-northeast-1", // Tokyo - significantly more expensive
        "ap-northeast-2", // Seoul
        "ap-southeast-1", // Singapore
        "ap-southeast-2", // Sydney - most expensive
        "sa-east-1",  // São Paulo - very expensive
    ];

    // Check if resource tags or name indicates non-US region usage
    let region_hint = change.tags.get("Region")
        .or_else(|| change.tags.get("AWS_REGION"))
        .map(|s| s.as_str());

    if let Some(region) = region_hint {
        let is_expensive = expensive_regions.iter().any(|r| region.starts_with(r));
        let is_apac_or_latam = region.starts_with("ap-") || region.starts_with("sa-");

        if is_expensive && is_apac_or_latam {
            patterns.push(AntiPattern {
                pattern_id: "REGIONAL_COST_OPTIMIZATION".to_string(),
                pattern_name: "Resource in High-Cost Region".to_string(),
                description: format!("Resource deployed in {} (higher pricing region)", region),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Region: {}", region),
                    "APAC and LATAM regions typically 10-30% more expensive than us-east-1".to_string(),
                    "Tokyo, Sydney, São Paulo are most expensive regions".to_string(),
                ],
                suggested_fix: Some("Review if regional deployment is required. Consider: (1) us-east-1 for cost optimization (reference pricing), (2) us-west-2 for US-based workloads, (3) eu-central-1 for European workloads with better pricing than eu-west-1.".to_string()),
                cost_impact: None,
                confidence: Some("LOW".to_string()),
                thresholds: None,
                assumptions: Some(vec![
                    "Regional pricing varies 10-30% from us-east-1 baseline".to_string(),
                    "Region detection based on tags (may be incomplete)".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect Auto Scaling Group optimization opportunities (Phase 2)
fn detect_asg_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_autoscaling_group" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    let min_size = config.get("min_size").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let max_size = config.get("max_size").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let desired_capacity = config.get("desired_capacity").and_then(|v| v.as_u64()).unwrap_or(min_size as u64) as u32;

    // 1. High minimum capacity
    if min_size >= ASG_MIN_CAPACITY_THRESHOLD {
        patterns.push(AntiPattern {
            pattern_id: "ASG_HIGH_MIN_CAPACITY".to_string(),
            pattern_name: "Auto Scaling Group High Minimum Capacity".to_string(),
            description: format!("ASG with min_size={} prevents scaling down during low traffic", min_size),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Minimum size: {}", min_size),
                format!("Maximum size: {}", max_size),
                format!("Desired capacity: {}", desired_capacity),
                "High minimum prevents cost savings during off-peak hours".to_string(),
            ],
            suggested_fix: Some("Review if high minimum is necessary. Consider: (1) Lowering min_size for off-peak cost savings, (2) Using scheduled scaling for predictable patterns, (3) Target tracking policies for dynamic scaling.".to_string()),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("min_capacity_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(ASG_MIN_CAPACITY_THRESHOLD)));
                t
            }),
            assumptions: Some(vec![
                format!("ASGs with min_size >= {} should be reviewed", ASG_MIN_CAPACITY_THRESHOLD),
            ]),
        });
    }

    // 2. Min = Max (no scaling)
    if min_size > 0 && min_size == max_size {
        patterns.push(AntiPattern {
            pattern_id: "ASG_NO_SCALING_RANGE".to_string(),
            pattern_name: "Auto Scaling Group With No Scaling Range".to_string(),
            description: format!("ASG with min_size=max_size={} - not utilizing auto scaling", min_size),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Minimum size: {}", min_size),
                format!("Maximum size: {}", max_size),
                "ASG cannot scale - functionally equivalent to fixed instance count".to_string(),
            ],
            suggested_fix: Some("If scaling not needed, consider using fixed instance count without ASG overhead. If scaling needed, increase max_size to allow scaling.".to_string()),
            cost_impact: None,
            confidence: Some("HIGH".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "ASGs with min=max are not utilizing auto scaling benefits".to_string(),
            ]),
        });
    }

    // 3. Narrow scaling range (max < 2 * min)
    if min_size > 0 && max_size > 0 && max_size < min_size * 2 {
        patterns.push(AntiPattern {
            pattern_id: "ASG_NARROW_SCALING_RANGE".to_string(),
            pattern_name: "Auto Scaling Group With Limited Scaling Range".to_string(),
            description: format!("ASG scaling range {}-{} may be too narrow for effective scaling", min_size, max_size),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Minimum size: {}", min_size),
                format!("Maximum size: {}", max_size),
                format!("Scaling range: {}% (max/min ratio: {:.1}x)", ((max_size - min_size) as f64 / min_size as f64 * 100.0), max_size as f64 / min_size as f64),
                "Narrow range limits cost optimization during traffic variations".to_string(),
            ],
            suggested_fix: Some("Consider wider scaling range (e.g., max = 2-3x min) to handle traffic spikes while maintaining cost efficiency during low traffic.".to_string()),
            cost_impact: None,
            confidence: Some("LOW".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Effective auto scaling typically requires max >= 2x min".to_string(),
            ]),
        });
    }

    patterns
}

// ============================================================================
// PHASE 3: Advanced Service Optimizations (95% Coverage)
// ============================================================================

/// Detect API Gateway optimization opportunities (Phase 3)
fn detect_api_gateway_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_api_gateway_rest_api" &&
       change.resource_type != "aws_apigatewayv2_api" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check if caching is disabled (default)
    let cache_enabled = config.get("cache_cluster_enabled").and_then(|v| v.as_bool()).unwrap_or(false);

    if !cache_enabled && change.resource_type == "aws_api_gateway_rest_api" {
        patterns.push(AntiPattern {
            pattern_id: "API_GATEWAY_NO_CACHING".to_string(),
            pattern_name: "API Gateway Without Response Caching".to_string(),
            description: "API Gateway without caching may result in unnecessary backend calls".to_string(),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "Cache cluster: disabled".to_string(),
                "Each request hits backend - no response caching".to_string(),
                "Caching reduces backend load and improves latency".to_string(),
            ],
            suggested_fix: Some("Consider enabling API Gateway caching for frequently accessed endpoints. Cache sizes: 0.5GB ($0.020/hr) to 237GB ($3.80/hr). Start small and monitor cache hit rates.".to_string()),
            cost_impact: None,
            confidence: Some("LOW".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Caching beneficial for read-heavy APIs with cacheable responses".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect CloudFront optimization opportunities (Phase 3)
fn detect_cloudfront_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_cloudfront_distribution" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check if price class is PriceClass_All (most expensive)
    let price_class = config.get("price_class").and_then(|v| v.as_str()).unwrap_or("PriceClass_All");

    if price_class == "PriceClass_All" {
        patterns.push(AntiPattern {
            pattern_id: "CLOUDFRONT_EXPENSIVE_PRICE_CLASS".to_string(),
            pattern_name: "CloudFront Using Most Expensive Price Class".to_string(),
            description: "CloudFront PriceClass_All includes all edge locations (highest cost)".to_string(),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "Price class: PriceClass_All (all edge locations globally)".to_string(),
                "Consider if coverage in expensive regions (India, Australia, South America) is needed".to_string(),
                "PriceClass_100: US, Canada, Europe (cheapest)".to_string(),
                "PriceClass_200: + Asia, Africa, Middle East (mid-tier)".to_string(),
            ],
            suggested_fix: Some("Review if global edge coverage is required. PriceClass_100 (US/Canada/Europe) or PriceClass_200 (adds Asia) may provide sufficient coverage at lower cost.".to_string()),
            cost_impact: None,
            confidence: Some("LOW".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Price class selection should match actual user geographic distribution".to_string(),
            ]),
        });
    }

    // Check if compression is disabled
    let compression_enabled = config.get("default_cache_behavior")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|behavior| behavior.get("compress"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !compression_enabled {
        patterns.push(AntiPattern {
            pattern_id: "CLOUDFRONT_NO_COMPRESSION".to_string(),
            pattern_name: "CloudFront Without Automatic Compression".to_string(),
            description: "CloudFront compression disabled - higher data transfer costs".to_string(),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "Automatic compression: disabled".to_string(),
                "Compression reduces data transfer costs".to_string(),
                "Typical savings: 50-70% for text-based content".to_string(),
            ],
            suggested_fix: Some("Enable automatic compression in CloudFront cache behaviors. Reduces data transfer costs for compressible content (HTML, CSS, JS, JSON, XML).".to_string()),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Compression beneficial for text-based content".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect EFS optimization opportunities (Phase 3)
fn detect_efs_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_efs_file_system" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check lifecycle policy (IA storage class)
    let lifecycle_policy = config.get("lifecycle_policy").and_then(|v| v.as_array());
    let has_ia_transition = lifecycle_policy.map(|policies| {
        policies.iter().any(|p| {
            p.get("transition_to_ia").is_some()
        })
    }).unwrap_or(false);

    if !has_ia_transition {
        patterns.push(AntiPattern {
            pattern_id: "EFS_NO_LIFECYCLE_POLICY".to_string(),
            pattern_name: "EFS Without Infrequent Access Lifecycle".to_string(),
            description: "EFS without lifecycle policy - all data in Standard storage class".to_string(),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "Lifecycle policy: not configured".to_string(),
                "All data stored in Standard class ($0.30/GB/month)".to_string(),
                "IA class: $0.025/GB/month (92% savings for infrequently accessed files)".to_string(),
                "IA automatically moves files not accessed for 7/14/30/60/90 days".to_string(),
            ],
            suggested_fix: Some("Configure EFS lifecycle policy to move infrequently accessed files to IA storage class. Start with 30-day transition for conservative savings.".to_string()),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Most file systems have mix of frequently and infrequently accessed files".to_string(),
            ]),
        });
    }

    // Check throughput mode
    let throughput_mode = config.get("throughput_mode").and_then(|v| v.as_str()).unwrap_or("bursting");
    let provisioned_throughput = config.get("provisioned_throughput_in_mibps").and_then(|v| v.as_f64());

    if throughput_mode == "provisioned" {
        if let Some(throughput) = provisioned_throughput {
            if throughput > 100.0 {
                patterns.push(AntiPattern {
                    pattern_id: "EFS_HIGH_PROVISIONED_THROUGHPUT".to_string(),
                    pattern_name: "EFS High Provisioned Throughput".to_string(),
                    description: format!("EFS provisioned throughput {:.0} MiB/s may be over-provisioned", throughput),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        format!("Throughput mode: provisioned ({:.0} MiB/s)", throughput),
                        format!("Cost: ${:.2}/month per MiB/s", 6.00),
                        format!("Monthly cost: ${:.2} for throughput alone", throughput * 6.00),
                        "Bursting mode: free, scales with storage size".to_string(),
                    ],
                    suggested_fix: Some("Review actual throughput utilization. Bursting mode may be sufficient (baseline: 50 MiB/s per TB of storage, burst to 100 MiB/s).".to_string()),
                    cost_impact: Some(throughput * 6.00),
                    confidence: Some("MEDIUM".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "Provisioned throughput at $6/MiB/s/month".to_string(),
                    ]),
                });
            }
        }
    }

    patterns
}

/// Detect S3 lifecycle optimization opportunities (Phase 3)
fn detect_s3_lifecycle_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_s3_bucket" {
        return patterns;
    }

    // Note: lifecycle_rule is typically defined in aws_s3_bucket_lifecycle_configuration
    // This is a heuristic check - absence doesn't guarantee no lifecycle policy
    patterns.push(AntiPattern {
        pattern_id: "S3_LIFECYCLE_REVIEW".to_string(),
        pattern_name: "S3 Bucket - Review Lifecycle Policy".to_string(),
        description: "S3 bucket detected - ensure lifecycle policies are configured".to_string(),
        severity: "LOW".to_string(),
        detected_in: change.resource_id.clone(),
        evidence: vec![
            "S3 storage costs vary significantly by storage class".to_string(),
            "Standard: $0.023/GB, IA: $0.0125/GB (46% savings)".to_string(),
            "Glacier Instant: $0.004/GB (83% savings)".to_string(),
            "Glacier Flexible: $0.0036/GB (84% savings)".to_string(),
            "Glacier Deep Archive: $0.00099/GB (96% savings)".to_string(),
        ],
        suggested_fix: Some("Configure lifecycle policies: (1) Transition to IA after 30-90 days, (2) Transition to Glacier after 180-365 days for archives, (3) Delete old versions/incomplete multipart uploads, (4) Intelligent-Tiering for unpredictable access patterns.".to_string()),
        cost_impact: None,
        confidence: Some("LOW".to_string()),
        thresholds: None,
        assumptions: Some(vec![
            "Most buckets benefit from lifecycle policies for cost optimization".to_string(),
            "Actual policy may be defined in separate lifecycle_configuration resource".to_string(),
        ]),
    });

    patterns
}

/// Detect VPC endpoint optimization opportunities (Phase 3)
fn detect_vpc_endpoint_opportunity(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Look for NAT gateways without corresponding VPC endpoints
    // This is a heuristic - we flag NAT gateways as potential VPC endpoint opportunities
    if change.resource_type == "aws_nat_gateway" {
        patterns.push(AntiPattern {
            pattern_id: "VPC_ENDPOINT_OPPORTUNITY".to_string(),
            pattern_name: "Consider VPC Endpoints to Reduce NAT Gateway Costs".to_string(),
            description: "NAT Gateway detected - VPC endpoints can eliminate NAT costs for AWS services".to_string(),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "NAT Gateway charges: ~$32.85/month + $0.045/GB data transfer".to_string(),
                "VPC Gateway Endpoints (S3, DynamoDB): FREE".to_string(),
                "VPC Interface Endpoints: $7.30/month + $0.01/GB (cheaper for high volume)".to_string(),
                "Common services: S3, DynamoDB, SQS, SNS, Lambda, EC2, CloudWatch".to_string(),
            ],
            suggested_fix: Some("Review AWS service usage. Create VPC endpoints for frequently used services to bypass NAT Gateway and reduce costs.".to_string()),
            cost_impact: None,
            confidence: Some("LOW".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "VPC endpoints eliminate NAT Gateway data transfer for AWS services".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect WAF optimization opportunities (Phase 3)
fn detect_waf_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_wafv2_web_acl" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Count rules
    let rules = config.get("rule").and_then(|v| v.as_array());
    let rule_count = rules.map(|r| r.len()).unwrap_or(0);

    if rule_count > WAF_RULE_THRESHOLD as usize {
        patterns.push(AntiPattern {
            pattern_id: "WAF_COMPLEX_ACL".to_string(),
            pattern_name: "WAF Web ACL With Many Rules".to_string(),
            description: format!("WAF Web ACL has {} rules - increases cost and complexity", rule_count),
            severity: "LOW".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Rule count: {} (threshold: {})", rule_count, WAF_RULE_THRESHOLD),
                "WAF charges per Web ACL ($5/month) + per rule ($1/month) + per million requests ($0.60)".to_string(),
                format!("Rule costs alone: ${}/month", rule_count),
            ],
            suggested_fix: Some("Review WAF rules: (1) Consolidate similar rules, (2) Use AWS Managed Rule Groups instead of custom rules, (3) Remove unused rules, (4) Use rule groups for better organization.".to_string()),
            cost_impact: Some(rule_count as f64 * 1.0),
            confidence: Some("MEDIUM".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("max_rules".to_string(), serde_json::Value::Number(serde_json::Number::from(WAF_RULE_THRESHOLD)));
                t
            }),
            assumptions: Some(vec![
                "Each WAF rule costs $1/month".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect IAM optimization opportunities (Phase 3)
fn detect_iam_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_iam_policy" &&
       change.resource_type != "aws_iam_role_policy" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check policy size (not a cost issue, but impacts manageability)
    if let Some(policy_str) = config.get("policy").and_then(|v| v.as_str()) {
        let policy_len = policy_str.len() as u32;

        if policy_len > IAM_POLICY_SIZE_THRESHOLD {
            patterns.push(AntiPattern {
                pattern_id: "IAM_LARGE_POLICY".to_string(),
                pattern_name: "IAM Policy Exceeds Complexity Threshold".to_string(),
                description: format!("IAM policy {} characters - may be overly complex", policy_len),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Policy size: {} characters", policy_len),
                    format!("Threshold: {} characters", IAM_POLICY_SIZE_THRESHOLD),
                    "Large policies are harder to audit and maintain".to_string(),
                    "AWS limit: 6,144 characters for inline policies".to_string(),
                ],
                suggested_fix: Some("Consider: (1) Breaking into multiple smaller policies, (2) Using AWS managed policies where possible, (3) Consolidating similar permissions, (4) Removing unused permissions.".to_string()),
                cost_impact: None,
                confidence: Some("LOW".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("max_policy_size".to_string(), serde_json::Value::Number(serde_json::Number::from(IAM_POLICY_SIZE_THRESHOLD)));
                    t
                }),
                assumptions: None,
            });
        }
    }

    patterns
}

/// Detect Reserved Instance opportunities (Phase 3)
fn detect_reserved_instance_opportunity(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // Only suggest RIs for production instances
    if change.resource_type != "aws_instance" {
        return patterns;
    }

    let environment = change.tags.get("Environment")
        .or_else(|| change.tags.get("Env"))
        .map(|s| s.to_lowercase());

    let is_production = match &environment {
        Some(env) => env == "prod" || env == "production" || env == "prd",
        None => false,
    };

    if !is_production {
        return patterns;
    }

    if let Some(est) = estimate {
        // Suggest RI if monthly cost > $50 (roughly $600/year)
        if est.monthly_cost > 50.0 {
            let annual_cost = est.monthly_cost * 12.0;
            // 1-year RI: ~40% savings, 3-year: ~60% savings
            let ri_1yr_savings = annual_cost * 0.40;
            let ri_3yr_savings = annual_cost * 0.60;

            patterns.push(AntiPattern {
                pattern_id: "RESERVED_INSTANCE_OPPORTUNITY".to_string(),
                pattern_name: "Production Instance - Reserved Instance Opportunity".to_string(),
                description: format!("Production instance ${:.2}/month - consider Reserved Instance", est.monthly_cost),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Environment: {}", environment.as_ref().unwrap()),
                    format!("Annual on-demand cost: ${:.2}", annual_cost),
                    format!("1-year RI savings: ${:.2}/year (~40% discount)", ri_1yr_savings),
                    format!("3-year RI savings: ${:.2}/year (~60% discount)", ri_3yr_savings),
                    "Reserved Instances require upfront commitment".to_string(),
                ],
                suggested_fix: Some("For stable production workloads, consider Reserved Instances: (1) 1-year convertible RI for flexibility, (2) 3-year standard RI for maximum savings, (3) Start with partial commitment (50-70% of usage).".to_string()),
                cost_impact: Some(ri_1yr_savings / 12.0), // Monthly savings
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("min_monthly_cost".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));
                    t
                }),
                assumptions: Some(vec![
                    "RI savings: 1-year ~40%, 3-year ~60%".to_string(),
                    "Production workloads typically run continuously".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect Spot Instance opportunities (Phase 3)
fn detect_spot_instance_opportunity(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_instance" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check if instance market is "spot" or if using launch template with spot
    let instance_market = config.get("instance_market_options")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|opt| opt.get("market_type"))
        .and_then(|v| v.as_str());

    let is_spot = instance_market == Some("spot");

    if is_spot {
        return patterns; // Already using spot
    }

    // Check for workload tags that indicate spot-suitable workloads
    let workload_hint = change.tags.get("Workload")
        .or_else(|| change.tags.get("Purpose"))
        .or_else(|| change.tags.get("Role"))
        .map(|s| s.to_lowercase());

    let spot_suitable = if let Some(workload) = &workload_hint {
        workload.contains("batch") ||
        workload.contains("processing") ||
        workload.contains("analytics") ||
        workload.contains("etl") ||
        workload.contains("worker") ||
        workload.contains("queue")
    } else {
        false
    };

    if spot_suitable {
        patterns.push(AntiPattern {
            pattern_id: "SPOT_INSTANCE_OPPORTUNITY".to_string(),
            pattern_name: "Fault-Tolerant Workload - Consider Spot Instances".to_string(),
            description: format!("Workload '{}' may be suitable for Spot Instances (70-90% savings)", workload_hint.as_ref().unwrap()),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                format!("Workload type: {}", workload_hint.as_ref().unwrap()),
                "Spot Instances: 70-90% discount vs on-demand".to_string(),
                "Suitable for: batch processing, analytics, stateless workloads, fault-tolerant systems".to_string(),
                "Risk: instances can be interrupted with 2-minute notice".to_string(),
            ],
            suggested_fix: Some("For interruptible workloads, consider Spot Instances: (1) Use Spot for batch/analytics jobs, (2) Implement checkpointing for interruption handling, (3) Mix Spot with on-demand for availability, (4) Use EC2 Fleet or Auto Scaling with mixed instances.".to_string()),
            cost_impact: None,
            confidence: Some("LOW".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Spot identified by workload tags (batch, processing, analytics, etl, worker, queue)".to_string(),
                "Spot savings average 70-90% vs on-demand".to_string(),
            ]),
        });
    }

    patterns
}

/// Detect ECS Fargate oversized tasks (new detection)
fn detect_ecs_fargate_oversized(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_ecs_task_definition" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check if using Fargate
    let requires_compatibilities = config.get("requires_compatibilities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());

    let is_fargate = match &requires_compatibilities {
        Some(compat) => compat.contains(&"FARGATE"),
        None => false,
    };

    if !is_fargate {
        return patterns;
    }

    // Get CPU and memory allocations
    let cpu = config.get("cpu").and_then(|v| v.as_str()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let memory = config.get("memory").and_then(|v| v.as_str()).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

    // Check for high CPU allocation
    if cpu >= FARGATE_HIGH_CPU_THRESHOLD {
        let mut evidence = vec![
            format!("Fargate CPU allocation: {} (threshold: {})", cpu, FARGATE_HIGH_CPU_THRESHOLD),
            "High CPU allocations increase Fargate costs significantly".to_string(),
            "Consider rightsizing based on actual CPU utilization".to_string(),
        ];

        if let Some(est) = estimate {
            evidence.push(format!("Estimated monthly cost: ${:.2}", est.monthly_cost));
        }

        patterns.push(AntiPattern {
            pattern_id: "FARGATE_HIGH_CPU".to_string(),
            pattern_name: "ECS Fargate High CPU Allocation".to_string(),
            description: format!("Fargate task using high CPU allocation: {}", cpu),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence,
            suggested_fix: Some("Review actual CPU utilization metrics. Consider: (1) Reducing CPU allocation to match actual usage, (2) Using Fargate Spot for cost savings (70% discount), (3) Optimizing application code to reduce CPU requirements.".to_string()),
            cost_impact: estimate.map(|e| e.monthly_cost * 0.3), // Potential 30% savings
            confidence: Some("MEDIUM".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("cpu_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(FARGATE_HIGH_CPU_THRESHOLD)));
                t
            }),
            assumptions: Some(vec![
                "Fargate pricing scales with CPU/memory allocation".to_string(),
                "Most applications are over-provisioned on CPU".to_string(),
            ]),
        });
    }

    // Check for high memory allocation
    if memory >= FARGATE_HIGH_MEMORY_THRESHOLD {
        let mut evidence = vec![
            format!("Fargate memory allocation: {}MB (threshold: {}MB)", memory, FARGATE_HIGH_MEMORY_THRESHOLD),
            "High memory allocations increase Fargate costs".to_string(),
            "Review actual memory usage patterns".to_string(),
        ];

        if let Some(est) = estimate {
            evidence.push(format!("Estimated monthly cost: ${:.2}", est.monthly_cost));
        }

        patterns.push(AntiPattern {
            pattern_id: "FARGATE_HIGH_MEMORY".to_string(),
            pattern_name: "ECS Fargate High Memory Allocation".to_string(),
            description: format!("Fargate task using high memory allocation: {}MB", memory),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence,
            suggested_fix: Some("Analyze memory utilization metrics. Consider: (1) Reducing memory allocation to match actual usage, (2) Using memory-optimized instance types on EC2 if consistently high, (3) Implementing memory optimization in application code.".to_string()),
            cost_impact: estimate.map(|e| e.monthly_cost * 0.25), // Potential 25% savings
            confidence: Some("MEDIUM".to_string()),
            thresholds: Some({
                let mut t = HashMap::new();
                t.insert("memory_threshold".to_string(), serde_json::Value::Number(serde_json::Number::from(FARGATE_HIGH_MEMORY_THRESHOLD)));
                t
            }),
            assumptions: Some(vec![
                "Fargate pricing includes memory allocation cost".to_string(),
            ]),
        });
    }

    // Check CPU:Memory ratio
    if cpu > 0 && memory > 0 {
        let ratio = memory as f64 / cpu as f64;

        // Fargate optimal ratios: 256 CPU (0.25 vCPU) → 512-2048 MB (ratio 2-8)
        // Typical is 1:4 (e.g., 1024 CPU = 4096 MB)
        if ratio < 2.0 || ratio > 8.0 {
            patterns.push(AntiPattern {
                pattern_id: "FARGATE_SUBOPTIMAL_RATIO".to_string(),
                pattern_name: "ECS Fargate Suboptimal CPU:Memory Ratio".to_string(),
                description: format!("Fargate task has unusual CPU:Memory ratio: 1:{:.1}", ratio),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("CPU: {}, Memory: {}MB (ratio 1:{:.1})", cpu, memory, ratio),
                    "Fargate supports ratios between 1:2 and 1:8".to_string(),
                    "Optimal ratio is typically 1:4 for balanced workloads".to_string(),
                ],
                suggested_fix: Some("Review workload characteristics. For CPU-intensive: use 1:2 ratio. For memory-intensive: use 1:8. For balanced: use 1:4. Adjust allocations to match Fargate's supported configurations for cost efficiency.".to_string()),
                cost_impact: None,
                confidence: Some("LOW".to_string()),
                thresholds: None,
                assumptions: Some(vec![
                    "Fargate pricing is based on allocated resources".to_string(),
                    "Optimal ratio depends on workload type".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect ElastiCache reserved instance opportunities (new detection)
fn detect_elasticache_reserved_instance(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_elasticache_cluster" &&
       change.resource_type != "aws_elasticache_replication_group" {
        return patterns;
    }

    // Check if production environment
    let environment = change.tags.get("Environment")
        .or_else(|| change.tags.get("Env"))
        .map(|s| s.to_lowercase());

    let is_production = match &environment {
        Some(env) => env == "prod" || env == "production" || env == "prd",
        None => false,
    };

    if !is_production {
        return patterns;
    }

    // Check if monthly cost warrants RI consideration
    if let Some(est) = estimate {
        if est.monthly_cost > 50.0 {
            let annual_cost = est.monthly_cost * 12.0;
            let ri_1yr_savings = annual_cost * 0.37; // 37% typical savings
            let ri_3yr_savings = annual_cost * 0.55; // 55% typical savings

            patterns.push(AntiPattern {
                pattern_id: "ELASTICACHE_RESERVED_INSTANCE".to_string(),
                pattern_name: "ElastiCache Reserved Instance Opportunity".to_string(),
                description: format!("Production ElastiCache ${:.2}/month - consider Reserved Instances", est.monthly_cost),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Environment: {}", environment.as_ref().unwrap()),
                    format!("Annual on-demand cost: ${:.2}", annual_cost),
                    format!("1-year RI savings: ${:.2}/year (~37% discount)", ri_1yr_savings),
                    format!("3-year RI savings: ${:.2}/year (~55% discount)", ri_3yr_savings),
                    "ElastiCache RIs require upfront commitment but provide significant savings".to_string(),
                ],
                suggested_fix: Some("For production caches, consider Reserved Instances: (1) 1-year no-upfront RI for flexibility (~30% savings), (2) 1-year partial-upfront for better discount (~35%), (3) 3-year partial-upfront for maximum savings (~55%). Implementation difficulty: EASY.".to_string()),
                cost_impact: Some(ri_1yr_savings / 12.0), // Monthly savings
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("min_monthly_cost".to_string(), serde_json::Value::Number(serde_json::Number::from(50)));
                    t
                }),
                assumptions: Some(vec![
                    "RI savings: 1-year ~37%, 3-year ~55%".to_string(),
                    "Production caches typically run continuously".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect OpenSearch domain optimization opportunities (new detection)
fn detect_opensearch_optimization(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_opensearch_domain" &&
       change.resource_type != "aws_elasticsearch_domain" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check for large instance types
    if let Some(cluster_config) = config.get("cluster_config").and_then(|v| v.as_array()).and_then(|arr| arr.first()) {
        if let Some(instance_type) = cluster_config.get("instance_type").and_then(|v| v.as_str()) {
            // Flag large instances (4xlarge+)
            if instance_type.contains(".4xlarge") || instance_type.contains(".8xlarge") ||
               instance_type.contains(".12xlarge") || instance_type.contains(".16xlarge") {

                let mut evidence = vec![
                    format!("Instance type: {}", instance_type),
                    "Large OpenSearch instances are expensive".to_string(),
                    "Consider horizontal scaling with smaller instances".to_string(),
                ];

                if let Some(est) = estimate {
                    evidence.push(format!("Estimated monthly cost: ${:.2}", est.monthly_cost));
                }

                patterns.push(AntiPattern {
                    pattern_id: "OPENSEARCH_LARGE_INSTANCE".to_string(),
                    pattern_name: "OpenSearch Large Instance Type".to_string(),
                    description: format!("OpenSearch using large instance: {}", instance_type),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence,
                    suggested_fix: Some("Review cluster sizing: (1) Use horizontal scaling with more smaller nodes instead of few large nodes, (2) Consider UltraWarm for infrequently accessed data (90% cost reduction), (3) Review actual resource utilization, (4) Use Reserved Instances for production (up to 40% savings). Implementation difficulty: MEDIUM.".to_string()),
                    cost_impact: estimate.map(|e| e.monthly_cost * 0.3),
                    confidence: Some("MEDIUM".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "Horizontal scaling provides better cost efficiency than vertical".to_string(),
                    ]),
                });
            }
        }

        // Check storage type
        if let Some(ebs_options) = config.get("ebs_options").and_then(|v| v.as_array()).and_then(|arr| arr.first()) {
            if let Some(volume_type) = ebs_options.get("volume_type").and_then(|v| v.as_str()) {
                if volume_type == "gp2" {
                    patterns.push(AntiPattern {
                        pattern_id: "OPENSEARCH_GP2_STORAGE".to_string(),
                        pattern_name: "OpenSearch Using gp2 Storage".to_string(),
                        description: "OpenSearch domain using gp2 EBS storage - gp3 offers better performance and cost".to_string(),
                        severity: "LOW".to_string(),
                        detected_in: change.resource_id.clone(),
                        evidence: vec![
                            format!("Volume type: {}", volume_type),
                            "gp3 provides 20% cost savings over gp2".to_string(),
                            "gp3 also offers better baseline performance".to_string(),
                        ],
                        suggested_fix: Some("Migrate OpenSearch EBS volumes from gp2 to gp3 for immediate 20% storage cost reduction with better performance. Implementation difficulty: EASY (in-place migration).".to_string()),
                        cost_impact: estimate.map(|e| e.monthly_cost * 0.1), // ~10% of domain cost is storage
                        confidence: Some("HIGH".to_string()),
                        thresholds: None,
                        assumptions: Some(vec![
                            "gp3 provides 20% cost savings over gp2".to_string(),
                        ]),
                    });
                }
            }
        }
    }

    patterns
}

/// Detect Kinesis stream optimization opportunities (new detection)
fn detect_kinesis_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_kinesis_stream" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check shard count
    if let Some(shard_count) = config.get("shard_count").and_then(|v| v.as_u64()) {
        if shard_count >= 10 {
            let hourly_cost_per_shard = 0.015;
            let monthly_cost_per_shard = hourly_cost_per_shard * 730.0;
            let total_monthly_cost = shard_count as f64 * monthly_cost_per_shard;

            patterns.push(AntiPattern {
                pattern_id: "KINESIS_HIGH_SHARD_COUNT".to_string(),
                pattern_name: "Kinesis Stream with High Shard Count".to_string(),
                description: format!("Kinesis stream with {} shards - review if all capacity needed", shard_count),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Shard count: {}", shard_count),
                    format!("Cost per shard: ${:.2}/month", monthly_cost_per_shard),
                    format!("Total stream cost: ${:.2}/month", total_monthly_cost),
                    "Each shard: 1MB/sec ingress, 2MB/sec egress".to_string(),
                    "Over-provisioning shards wastes money".to_string(),
                ],
                suggested_fix: Some("Review actual throughput: (1) Monitor shard metrics (IncomingBytes, IncomingRecords), (2) Right-size shard count based on actual usage, (3) Consider Kinesis Data Streams On-Demand mode for variable traffic (pay per GB ingested), (4) Use enhanced fan-out only when needed. Implementation difficulty: EASY.".to_string()),
                cost_impact: Some(total_monthly_cost * 0.3), // Potential 30% reduction
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("shard_count".to_string(), serde_json::Value::Number(serde_json::Number::from(shard_count)));
                    t.insert("monthly_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(total_monthly_cost).unwrap()));
                    t
                }),
                assumptions: Some(vec![
                    "Shard cost: $0.015/hour = $10.95/month".to_string(),
                    "Many streams are over-provisioned".to_string(),
                ]),
            });
        }
    }

    // Check stream mode (provisioned vs on-demand)
    let stream_mode_details = config.get("stream_mode_details").and_then(|v| v.as_array()).and_then(|arr| arr.first());
    if let Some(mode_details) = stream_mode_details {
        if let Some(stream_mode) = mode_details.get("stream_mode").and_then(|v| v.as_str()) {
            if stream_mode == "PROVISIONED" {
                patterns.push(AntiPattern {
                    pattern_id: "KINESIS_PROVISIONED_MODE".to_string(),
                    pattern_name: "Kinesis Provisioned Mode - Consider On-Demand".to_string(),
                    description: "Kinesis stream in provisioned mode - on-demand may be more cost-effective for variable traffic".to_string(),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        format!("Stream mode: {}", stream_mode),
                        "Provisioned mode: fixed shard costs regardless of usage".to_string(),
                        "On-demand mode: pay per GB ingested/hour retained".to_string(),
                        "On-demand better for unpredictable or spiky traffic".to_string(),
                    ],
                    suggested_fix: Some("For variable traffic, consider On-Demand mode: (1) No shard management, (2) Automatic scaling, (3) Pay $0.04/GB ingested + $0.011/hour, (4) Typically cheaper for <20% utilization. Implementation difficulty: EASY (mode switch).".to_string()),
                    cost_impact: None,
                    confidence: Some("LOW".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "On-demand suitable for variable traffic patterns".to_string(),
                    ]),
                });
            }
        }
    }

    patterns
}

/// Detect EC2 instance rightsizing opportunities (comprehensive detection)
fn detect_ec2_rightsizing(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_instance" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    let instance_type = match config.get("instance_type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return patterns,
    };

    // Extract instance family and size
    let parts: Vec<&str> = instance_type.split('.').collect();
    if parts.len() < 2 {
        return patterns;
    }

    let family = parts[0];
    let size = parts[1];

    // Detect oversized instances (8xlarge, 12xlarge, 16xlarge, 24xlarge)
    if size.contains("8xlarge") || size.contains("12xlarge") ||
       size.contains("16xlarge") || size.contains("24xlarge") {

        let mut evidence = vec![
            format!("Instance type: {} (very large)", instance_type),
            "Large instances often indicate over-provisioning".to_string(),
            "Consider horizontal scaling with smaller instances".to_string(),
        ];

        if let Some(est) = estimate {
            evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));

            // Calculate potential savings with 4xlarge
            let potential_savings = est.monthly_cost * 0.4; // Assume 40% savings
            evidence.push(format!("Potential savings: ${:.2}/month with rightsizing", potential_savings));
        }

        patterns.push(AntiPattern {
            pattern_id: "EC2_OVERSIZED_INSTANCE".to_string(),
            pattern_name: "EC2 Instance May Be Oversized".to_string(),
            description: format!("Instance {} is very large - consider rightsizing", instance_type),
            severity: "HIGH".to_string(),
            detected_in: change.resource_id.clone(),
            evidence,
            suggested_fix: Some(format!(
                "Review actual CPU/memory utilization. Consider: (1) Rightsizing to smaller instance (e.g., {}.4xlarge), \
                (2) Horizontal scaling with multiple smaller instances, \
                (3) Using Auto Scaling for variable workloads, \
                (4) Reserved Instances if consistently utilized. Implementation difficulty: MEDIUM.",
                family
            )),
            cost_impact: estimate.map(|e| e.monthly_cost * 0.4),
            confidence: Some("MEDIUM".to_string()),
            thresholds: None,
            assumptions: Some(vec![
                "Large instances often over-provisioned".to_string(),
                "Horizontal scaling provides better cost efficiency".to_string(),
            ]),
        });
    }

    // Detect expensive compute-optimized instances for non-compute workloads
    if family.starts_with("c5") || family.starts_with("c6") {
        // Check workload tags
        let workload = change.tags.get("Workload")
            .or_else(|| change.tags.get("Purpose"))
            .or_else(|| change.tags.get("Role"))
            .map(|s| s.to_lowercase());

        let is_compute_workload = workload.as_ref().map(|w| {
            w.contains("compute") || w.contains("cpu") || w.contains("processing") ||
            w.contains("batch") || w.contains("encoding") || w.contains("rendering")
        }).unwrap_or(false);

        if !is_compute_workload && (size.contains("large") || size.contains("xlarge")) {
            patterns.push(AntiPattern {
                pattern_id: "EC2_EXPENSIVE_COMPUTE_FAMILY".to_string(),
                pattern_name: "Compute-Optimized Instance for Non-Compute Workload".to_string(),
                description: format!("Instance {} (compute-optimized) may not match workload", instance_type),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Instance family: {} (compute-optimized)", family),
                    "Compute-optimized instances cost more for CPU-intensive workloads".to_string(),
                    "Consider general-purpose (m5, m6i) for balanced workloads".to_string(),
                ],
                suggested_fix: Some(format!(
                    "If workload is not CPU-intensive, consider switching to general-purpose instances (m5.{}, m6i.{}) \
                    for 15-20% cost savings. Implementation difficulty: EASY (instance type change).",
                    size, size
                )),
                cost_impact: estimate.map(|e| e.monthly_cost * 0.15),
                confidence: Some("LOW".to_string()),
                thresholds: None,
                assumptions: Some(vec![
                    "Workload type inferred from tags or absence of compute tags".to_string(),
                ]),
            });
        }
    }

    // Detect expensive memory-optimized instances
    if (family.starts_with("r5") || family.starts_with("r6") || family.starts_with("x1")) &&
       (size.contains("xlarge") || size.contains("large")) {

        let workload = change.tags.get("Workload")
            .or_else(|| change.tags.get("Purpose"))
            .map(|s| s.to_lowercase());

        let is_memory_workload = workload.as_ref().map(|w| {
            w.contains("memory") || w.contains("cache") || w.contains("database") ||
            w.contains("analytics") || w.contains("redis") || w.contains("elastic")
        }).unwrap_or(false);

        if !is_memory_workload {
            patterns.push(AntiPattern {
                pattern_id: "EC2_EXPENSIVE_MEMORY_FAMILY".to_string(),
                pattern_name: "Memory-Optimized Instance Without Memory-Intensive Workload".to_string(),
                description: format!("Instance {} (memory-optimized) may not match workload needs", instance_type),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    format!("Instance family: {} (memory-optimized)", family),
                    "Memory-optimized instances cost premium for high memory workloads".to_string(),
                    "Consider general-purpose (m5, m6i) if memory requirements are standard".to_string(),
                ],
                suggested_fix: Some(format!(
                    "For standard memory workloads, consider general-purpose m5.{} or m6i.{} for 20-25% cost savings. \
                    Implementation difficulty: EASY.",
                    size, size
                )),
                cost_impact: estimate.map(|e| e.monthly_cost * 0.20),
                confidence: Some("LOW".to_string()),
                thresholds: None,
                assumptions: Some(vec![
                    "Workload type inferred from tags".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect Lambda function memory optimization opportunities (comprehensive)
fn detect_lambda_memory_optimization(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_lambda_function" {
        return patterns;
    }

    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    // Check memory allocation
    if let Some(memory) = config.get("memory_size").and_then(|v| v.as_u64()) {
        let memory_mb = memory as u32;

        // Flag very high memory allocations (>3GB)
        if memory_mb >= 3072 {
            let mut evidence = vec![
                format!("Memory allocation: {} MB ({:.1} GB)", memory_mb, memory_mb as f64 / 1024.0),
                "High memory allocation increases Lambda costs".to_string(),
                "Lambda pricing: $0.0000166667 per GB-second".to_string(),
            ];

            if let Some(est) = estimate {
                evidence.push(format!("Estimated monthly cost: ${:.2}", est.monthly_cost));
            }

            patterns.push(AntiPattern {
                pattern_id: "LAMBDA_HIGH_MEMORY".to_string(),
                pattern_name: "Lambda Function with High Memory Allocation".to_string(),
                description: format!("Lambda allocated {} MB memory - review if necessary", memory_mb),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some(
                    "Review actual memory usage: (1) Use CloudWatch Logs to check MaxMemoryUsed, \
                    (2) Start with 1024MB and scale up based on actual needs, \
                    (3) Consider ECS/Fargate for memory-intensive workloads (>3GB), \
                    (4) Optimize code to reduce memory footprint. Implementation difficulty: MEDIUM."
                        .to_string(),
                ),
                cost_impact: estimate.map(|e| e.monthly_cost * 0.3),
                confidence: Some("MEDIUM".to_string()),
                thresholds: Some({
                    let mut t = HashMap::new();
                    t.insert("high_memory_threshold_mb".to_string(), serde_json::Value::Number(serde_json::Number::from(3072)));
                    t
                }),
                assumptions: Some(vec![
                    "Most Lambda functions use <2GB memory".to_string(),
                    "Higher memory = higher cost per execution".to_string(),
                ]),
            });
        }

        // Flag low memory with high timeout (indicates potential under-provisioning)
        if let Some(timeout) = config.get("timeout").and_then(|v| v.as_u64()) {
            if memory_mb <= 512 && timeout >= 60 {
                patterns.push(AntiPattern {
                    pattern_id: "LAMBDA_UNDERPROVISIONED_MEMORY".to_string(),
                    pattern_name: "Lambda May Be Under-Provisioned on Memory".to_string(),
                    description: format!("Lambda with {}MB memory and {}s timeout may be slow", memory_mb, timeout),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        format!("Memory: {} MB (low)", memory_mb),
                        format!("Timeout: {} seconds (high)", timeout),
                        "Low memory with high timeout suggests slow execution".to_string(),
                        "Increasing memory also increases CPU allocation".to_string(),
                    ],
                    suggested_fix: Some(
                        "Consider increasing memory to 1024MB or 1536MB. Lambda allocates proportional CPU to memory, \
                        so more memory = faster execution = potentially lower total cost despite higher per-second rate. \
                        Implementation difficulty: EASY."
                            .to_string(),
                    ),
                    cost_impact: None,
                    confidence: Some("LOW".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "More memory can reduce execution time and total cost".to_string(),
                    ]),
                });
            }
        }

        // Flag default memory (128MB) which is often suboptimal
        if memory_mb == 128 {
            patterns.push(AntiPattern {
                pattern_id: "LAMBDA_DEFAULT_MEMORY".to_string(),
                pattern_name: "Lambda Using Default Memory Allocation".to_string(),
                description: "Lambda using default 128MB memory - likely not optimized".to_string(),
                severity: "LOW".to_string(),
                detected_in: change.resource_id.clone(),
                evidence: vec![
                    "Memory: 128 MB (Lambda default)".to_string(),
                    "Default memory is rarely optimal for production workloads".to_string(),
                    "Most functions benefit from 512MB-1024MB allocation".to_string(),
                ],
                suggested_fix: Some(
                    "Optimize memory allocation: (1) Test with different memory settings (512MB, 1024MB, 1536MB), \
                    (2) Use AWS Lambda Power Tuning tool for automatic optimization, \
                    (3) Monitor CloudWatch metrics for actual memory usage. Implementation difficulty: EASY."
                        .to_string(),
                ),
                cost_impact: None,
                confidence: Some("MEDIUM".to_string()),
                thresholds: None,
                assumptions: Some(vec![
                    "Default memory setting indicates lack of optimization".to_string(),
                ]),
            });
        }
    }

    patterns
}

/// Detect S3 storage class optimization opportunities (comprehensive)
fn detect_s3_storage_class_optimization(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    if change.resource_type != "aws_s3_bucket" {
        return patterns;
    }

    // Always recommend lifecycle policies for S3 buckets
    patterns.push(AntiPattern {
        pattern_id: "S3_NO_LIFECYCLE_POLICY".to_string(),
        pattern_name: "S3 Bucket Without Lifecycle Policy".to_string(),
        description: "S3 bucket lacks lifecycle policy - missing cost optimization opportunity".to_string(),
        severity: "MEDIUM".to_string(),
        detected_in: change.resource_id.clone(),
        evidence: vec![
            "S3 storage costs vary significantly by storage class:".to_string(),
            "• Standard: $0.023/GB (baseline)".to_string(),
            "• Intelligent-Tiering: $0.023/GB + $0.0025/1000 objects (automatic optimization)".to_string(),
            "• Standard-IA: $0.0125/GB (46% savings for infrequent access)".to_string(),
            "• One Zone-IA: $0.01/GB (57% savings, single AZ)".to_string(),
            "• Glacier Instant: $0.004/GB (83% savings)".to_string(),
            "• Glacier Flexible: $0.0036/GB (84% savings)".to_string(),
            "• Glacier Deep Archive: $0.00099/GB (96% savings)".to_string(),
            "Without lifecycle policies, all data remains in Standard storage".to_string(),
        ],
        suggested_fix: Some(
            "Implement lifecycle policies: (1) Transition to Standard-IA after 30-90 days for infrequent access, \
            (2) Use Intelligent-Tiering for unknown access patterns (automatic cost optimization), \
            (3) Move to Glacier Flexible after 180-365 days for archives, \
            (4) Use Glacier Deep Archive for long-term compliance data, \
            (5) Delete incomplete multipart uploads after 7 days, \
            (6) Expire old object versions if versioning enabled. \
            Potential savings: 40-96% depending on access patterns. Implementation difficulty: EASY."
                .to_string(),
        ),
        cost_impact: None, // Unknown without object count/size
        confidence: Some("HIGH".to_string()),
        thresholds: None,
        assumptions: Some(vec![
            "Most S3 buckets benefit from lifecycle policies".to_string(),
            "Typical savings: 40-60% with proper storage class transitions".to_string(),
            "Actual policy depends on access patterns".to_string(),
        ]),
    });

    // Check if versioning is enabled without lifecycle expiration
    let config = match &change.new_config {
        Some(c) => c,
        None => return patterns,
    };

    if let Some(versioning) = config.get("versioning").and_then(|v| v.as_array()).and_then(|arr| arr.first()) {
        if let Some(enabled) = versioning.get("enabled").and_then(|v| v.as_bool()) {
            if enabled {
                patterns.push(AntiPattern {
                    pattern_id: "S3_VERSIONING_WITHOUT_LIFECYCLE".to_string(),
                    pattern_name: "S3 Versioning Without Lifecycle Expiration".to_string(),
                    description: "S3 versioning enabled without lifecycle policy for old versions".to_string(),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        "Versioning: enabled".to_string(),
                        "Old versions accumulate indefinitely without lifecycle policy".to_string(),
                        "Storage costs multiply with each version kept".to_string(),
                        "Old versions are charged at standard storage rates".to_string(),
                    ],
                    suggested_fix: Some(
                        "Add lifecycle policy for noncurrent versions: (1) Expire noncurrent versions after 90 days, \
                        (2) Transition noncurrent versions to Glacier after 30 days if needed for compliance, \
                        (3) Permanently delete after retention period. \
                        Typical savings: 50-80% on versioned objects. Implementation difficulty: EASY."
                            .to_string(),
                    ),
                    cost_impact: None,
                    confidence: Some("HIGH".to_string()),
                    thresholds: None,
                    assumptions: Some(vec![
                        "Versioned objects accumulate rapidly without lifecycle management".to_string(),
                        "Old versions rarely accessed after 90 days".to_string(),
                    ]),
                });
            }
        }
    }

    patterns
}

// ============================================================================
// HEURISTIC 3: Instance Family Mismatch (Unchanged)
// ============================================================================

/// Detect mismatch between instance family and workload tags
fn detect_instance_family_mismatch(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_instance" {
        return None;
    }

    let config = change.new_config.as_ref()?;
    let instance_type = config.get("instance_type")?.as_str()?;
    let family = extract_instance_family(instance_type)?;

    // Extract workload hints from tags
    let workload_hint = change
        .tags
        .get("Workload")
        .or_else(|| change.tags.get("Usage"))
        .or_else(|| change.tags.get("Role"))
        .map(|s| s.to_lowercase())?;

    // Check if workload hint matches known patterns
    let workload_to_family = get_workload_to_family_hints();

    for (keyword, recommended_families) in &workload_to_family {
        if workload_hint.contains(keyword) {
            if !recommended_families.contains(&family) {
                let mut evidence = vec![
                    format!("Instance type: {} (family: {})", instance_type, family),
                    format!("Workload tag indicates: {}", workload_hint),
                    format!(
                        "Recommended families for '{}' workload: {}",
                        keyword,
                        recommended_families.join(", ")
                    ),
                ];

                // Classify family types for explanation
                let current_family_type = if get_compute_families().contains(&family) {
                    "compute-optimized"
                } else if get_memory_families().contains(&family) {
                    "memory-optimized"
                } else if get_burstable_families().contains(&family) {
                    "burstable"
                } else if get_general_families().contains(&family) {
                    "general-purpose"
                } else {
                    "specialized"
                };

                evidence.push(format!(
                    "Current instance uses {} family ({})",
                    family, current_family_type
                ));

                let mut thresholds = HashMap::new();
                thresholds.insert(
                    "workload_keyword".to_string(),
                    serde_json::Value::String(keyword.to_string()),
                );
                thresholds.insert(
                    "recommended_families".to_string(),
                    serde_json::Value::Array(
                        recommended_families
                            .iter()
                            .map(|f| serde_json::Value::String(f.to_string()))
                            .collect(),
                    ),
                );

                return Some(AntiPattern {
                    pattern_id: "INSTANCE_FAMILY_MISMATCH".to_string(),
                    pattern_name: "Instance Family May Not Match Workload".to_string(),
                    description: format!(
                        "{} family instance used for '{}' workload - consider {} family instead",
                        family,
                        keyword,
                        recommended_families.join(" or ")
                    ),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence,
                    suggested_fix: Some(format!(
                        "Consider using {} family instances for this workload. \
                        Current {} family may be over/under-provisioned for '{}' use case. \
                        Verify instance family aligns with actual workload requirements.",
                        recommended_families.join(" or "),
                        family,
                        keyword
                    )),
                    cost_impact: None,
                    confidence: Some("MEDIUM".to_string()),
                    thresholds: Some(thresholds),
                    assumptions: Some(vec![
                        format!("Workload classification based on '{}' tag", workload_hint),
                        "Family recommendation is advisory, not prescriptive".to_string(),
                        "Actual requirements may differ from tag indication".to_string(),
                    ]),
                });
            }
        }
    }

    None
}

// ============================================================================
// HEURISTIC 4: Storage Inefficiency
// ============================================================================

/// Detect storage type inefficiencies (gp2→gp3, over-provisioned IOPS)
fn detect_storage_inefficiency(change: &ResourceChange) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // EBS volume inefficiency
    if change.resource_type == "aws_ebs_volume" {
        if let Some(pattern) = detect_ebs_storage_inefficiency(change) {
            patterns.push(pattern);
        }
    }

    // RDS storage inefficiency
    if change.resource_type == "aws_db_instance" {
        if let Some(pattern) = detect_rds_storage_inefficiency(change) {
            patterns.push(pattern);
        }
    }

    patterns
}

/// Detect EBS gp2→gp3 and IOPS inefficiency
fn detect_ebs_storage_inefficiency(change: &ResourceChange) -> Option<AntiPattern> {
    let config = change.new_config.as_ref()?;
    let volume_type = config.get("type")?.as_str()?;
    let size_gb = config.get("size")?.as_u64()? as f64;

    // Check for gp2→gp3 opportunity
    if volume_type == "gp2" {
        let current_cost = size_gb * GP2_COST_PER_GB;
        let gp3_cost = size_gb * GP3_COST_PER_GB;
        let savings = current_cost - gp3_cost;

        if savings >= 1.0 {
            // Only flag if savings >= $1/month
            let evidence = vec![
                format!("Volume type: gp2 ({} GB)", size_gb),
                format!("Current cost: ${:.2}/month (${:.2}/GB)", current_cost, GP2_COST_PER_GB),
                format!("gp3 cost: ${:.2}/month (${:.2}/GB)", gp3_cost, GP3_COST_PER_GB),
                format!("Potential savings: ${:.2}/month ({:.0}%)", savings, (savings / current_cost) * 100.0),
                "gp3 provides same baseline performance (3000 IOPS, 125 MB/s)".to_string(),
            ];

            let mut thresholds = HashMap::new();
            thresholds.insert(
                "min_savings".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
            );
            thresholds.insert(
                "gp2_cost_per_gb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(GP2_COST_PER_GB).unwrap()),
            );
            thresholds.insert(
                "gp3_cost_per_gb".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(GP3_COST_PER_GB).unwrap()),
            );

            return Some(AntiPattern {
                pattern_id: "STORAGE_GP2_TO_GP3_OPPORTUNITY".to_string(),
                pattern_name: "EBS gp2 Volume - gp3 More Cost-Effective".to_string(),
                description: format!(
                    "gp2 volume costs ${:.2}/month, gp3 offers same performance for ${:.2}",
                    current_cost, gp3_cost
                ),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some(
                    "Migrate to gp3 volume type for 20% cost savings with same baseline performance (3000 IOPS, 125 MB/s). \
                    gp3 also allows independent IOPS/throughput configuration."
                        .to_string(),
                ),
                cost_impact: Some(savings),
                confidence: Some("HIGH".to_string()),
                thresholds: Some(thresholds),
                assumptions: Some(vec![
                    "Pricing based on us-east-1 rates".to_string(),
                    "gp3 baseline performance (3000 IOPS) sufficient for workload".to_string(),
                ]),
            });
        }
    }

    // Check for io1/io2 when gp3 could suffice
    if volume_type == "io1" || volume_type == "io2" {
        if let Some(provisioned_iops) = config.get("iops").and_then(|v| v.as_u64()) {
            if provisioned_iops <= GP3_MAX_IOPS as u64 {
                let io_storage_cost = size_gb * 0.125; // io1/io2 storage cost
                let io_iops_cost = provisioned_iops as f64 * IO1_IOPS_COST;
                let io_total_cost = io_storage_cost + io_iops_cost;

                let gp3_storage_cost = size_gb * GP3_COST_PER_GB;
                let gp3_iops_cost = if provisioned_iops > GP3_BASE_IOPS as u64 {
                    (provisioned_iops - GP3_BASE_IOPS as u64) as f64 * GP3_IOPS_COST
                } else {
                    0.0
                };
                let gp3_total_cost = gp3_storage_cost + gp3_iops_cost;

                let savings = io_total_cost - gp3_total_cost;
                let savings_percent = (savings / io_total_cost) * 100.0;

                if savings_percent >= 20.0 {
                    let evidence = vec![
                        format!("Volume type: {} with {} IOPS", volume_type, provisioned_iops),
                        format!("Current cost: ${:.2}/month", io_total_cost),
                        format!(
                            "  - Storage: ${:.2} ({} GB × $0.125)",
                            io_storage_cost, size_gb
                        ),
                        format!(
                            "  - IOPS: ${:.2} ({} × ${})",
                            io_iops_cost, provisioned_iops, IO1_IOPS_COST
                        ),
                        format!("gp3 can deliver same {} IOPS for ${:.2}/month", provisioned_iops, gp3_total_cost),
                        format!("Potential savings: ${:.2}/month ({:.0}%)", savings, savings_percent),
                    ];

                    let mut thresholds = HashMap::new();
                    thresholds.insert(
                        "gp3_max_iops".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(GP3_MAX_IOPS)),
                    );
                    thresholds.insert(
                        "min_savings_percent".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(20)),
                    );

                    return Some(AntiPattern {
                        pattern_id: "STORAGE_IO_TO_GP3_OPPORTUNITY".to_string(),
                        pattern_name: format!("EBS {} Volume - gp3 More Cost-Effective", volume_type),
                        description: format!(
                            "{} with {} IOPS costs ${:.2}/month, gp3 can deliver same IOPS for ${:.2}",
                            volume_type, provisioned_iops, io_total_cost, gp3_total_cost
                        ),
                        severity: "MEDIUM".to_string(),
                        detected_in: change.resource_id.clone(),
                        evidence,
                        suggested_fix: Some(format!(
                            "Consider migrating to gp3 volume type with {} provisioned IOPS. \
                            gp3 can deliver up to {} IOPS at significantly lower cost than io1/io2.",
                            provisioned_iops, GP3_MAX_IOPS
                        )),
                        cost_impact: Some(savings),
                        confidence: Some("MEDIUM".to_string()),
                        thresholds: Some(thresholds),
                        assumptions: Some(vec![
                            "Pricing based on us-east-1 rates".to_string(),
                            format!("gp3 can support {} IOPS workload", provisioned_iops),
                            "Cost comparison does not include throughput differences".to_string(),
                        ]),
                    });
                }
            }
        }
    }

    None
}

/// Detect RDS storage inefficiency (gp2→gp3)
fn detect_rds_storage_inefficiency(change: &ResourceChange) -> Option<AntiPattern> {
    let config = change.new_config.as_ref()?;
    let storage_type = config.get("storage_type")?.as_str()?;
    let allocated_storage = config.get("allocated_storage")?.as_u64()? as f64;

    if storage_type == "gp2" {
        let current_cost = allocated_storage * GP2_COST_PER_GB;
        let gp3_cost = allocated_storage * GP3_COST_PER_GB;
        let savings = current_cost - gp3_cost;

        if savings >= 1.0 {
            let evidence = vec![
                format!("Storage type: gp2 ({} GB)", allocated_storage),
                format!("Current cost: ${:.2}/month", current_cost),
                format!("gp3 cost: ${:.2}/month", gp3_cost),
                format!("Potential savings: ${:.2}/month ({:.0}%)", savings, (savings / current_cost) * 100.0),
            ];

            let mut thresholds = HashMap::new();
            thresholds.insert(
                "min_savings".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
            );

            return Some(AntiPattern {
                pattern_id: "STORAGE_RDS_GP2_TO_GP3_OPPORTUNITY".to_string(),
                pattern_name: "RDS gp2 Storage - gp3 More Cost-Effective".to_string(),
                description: format!(
                    "RDS gp2 storage costs ${:.2}/month, gp3 offers same performance for ${:.2}",
                    current_cost, gp3_cost
                ),
                severity: "MEDIUM".to_string(),
                detected_in: change.resource_id.clone(),
                evidence,
                suggested_fix: Some(
                    "Migrate RDS instance to gp3 storage for 20% cost savings with same baseline performance."
                        .to_string(),
                ),
                cost_impact: Some(savings),
                confidence: Some("HIGH".to_string()),
                thresholds: Some(thresholds),
                assumptions: Some(vec!["Pricing based on us-east-1 rates".to_string()]),
            });
        }
    }

    None
}

// ============================================================================
// LEGACY PER-RESOURCE DETECTION (Backwards Compatibility)
// ============================================================================

/// Detect anti-patterns in resource change (MVP top 5)
/// This is the original per-resource detection function, maintained for backwards compatibility
pub fn detect_anti_patterns(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Vec<AntiPattern> {
    let mut patterns = Vec::new();

    // 1. NAT Gateway overuse
    if let Some(pattern) = detect_nat_gateway_overuse(change, estimate) {
        patterns.push(pattern);
    }

    // 2. Overprovisioned EC2
    if let Some(pattern) = detect_overprovisioned_ec2(change, estimate) {
        patterns.push(pattern);
    }

    // 3. S3 missing lifecycle
    if let Some(pattern) = detect_s3_missing_lifecycle(change) {
        patterns.push(pattern);
    }

    // 4. Unbounded Lambda concurrency
    if let Some(pattern) = detect_unbounded_lambda_concurrency(change) {
        patterns.push(pattern);
    }

    // 5. DynamoDB pay-per-request default
    if let Some(pattern) = detect_dynamodb_pay_per_request_default(change) {
        patterns.push(pattern);
    }

    patterns
}

/// Pattern 1: NAT Gateway overuse
fn detect_nat_gateway_overuse(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    if change.resource_type != "aws_nat_gateway" {
        return None;
    }

    // NAT Gateways are expensive (~$32.85/month + data transfer)
    let mut evidence =
        vec!["NAT Gateway incurs fixed hourly charges plus data transfer costs".to_string()];

    if let Some(est) = estimate {
        evidence.push(format!("Estimated cost: ${:.2}/month", est.monthly_cost));
    }

    Some(AntiPattern {
        pattern_id: "NAT_GATEWAY_OVERUSE".to_string(),
        pattern_name: "NAT Gateway Overuse".to_string(),
        description: "NAT Gateways are expensive. Consider VPC endpoints or consolidating across AZs.".to_string(),
        severity: "HIGH".to_string(),
        detected_in: change.resource_id.clone(),
        evidence,
        suggested_fix: Some(
            "Use VPC endpoints for AWS services (S3, DynamoDB) to avoid NAT Gateway data transfer. \
            Consider single NAT Gateway if high availability not critical.".to_string()
        ),
        cost_impact: estimate.map(|e| e.monthly_cost),
        confidence: Some("MEDIUM".to_string()),
        thresholds: None,
        assumptions: None,
    })
}

/// Pattern 2: Overprovisioned EC2
fn detect_overprovisioned_ec2(
    change: &ResourceChange,
    estimate: Option<&CostEstimate>,
) -> Option<AntiPattern> {
    if change.resource_type != "aws_instance" {
        return None;
    }

    // Check for large instance types that might be overprovisioned
    if let Some(config) = &change.new_config {
        if let Some(instance_type) = config.get("instance_type").and_then(|v| v.as_str()) {
            // Detect potentially oversized instances
            let is_large = instance_type.contains("large")
                || instance_type.contains("xlarge")
                || instance_type.starts_with("m5.")
                || instance_type.starts_with("c5.")
                || instance_type.starts_with("r5.");

            if is_large {
                let mut evidence = vec![
                    format!("Instance type: {}", instance_type),
                    "Large instance types should be right-sized based on actual workload"
                        .to_string(),
                ];

                if let Some(est) = estimate {
                    evidence.push(format!("Monthly cost: ${:.2}", est.monthly_cost));
                }

                return Some(AntiPattern {
                    pattern_id: "OVERPROVISIONED_EC2".to_string(),
                    pattern_name: "Potentially Overprovisioned EC2".to_string(),
                    description: "Large EC2 instance detected. Verify sizing matches actual workload requirements.".to_string(),
                    severity: "MEDIUM".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence,
                    suggested_fix: Some(
                        "Use AWS Compute Optimizer to analyze utilization patterns and recommend right-sized instances. \
                        Start with smaller instances and scale up as needed.".to_string()
                    ),
                    cost_impact: estimate.map(|e| e.monthly_cost),
                    confidence: Some("LOW".to_string()),
                    thresholds: None,
                    assumptions: None,
                });
            }
        }
    }

    None
}

/// Pattern 3: S3 missing lifecycle
fn detect_s3_missing_lifecycle(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_s3_bucket" {
        return None;
    }

    // Check if lifecycle rules are missing
    let has_lifecycle = if let Some(config) = &change.new_config {
        config.get("lifecycle_rule").is_some() || config.get("lifecycle_configuration").is_some()
    } else {
        false
    };

    if !has_lifecycle {
        return Some(AntiPattern {
            pattern_id: "S3_MISSING_LIFECYCLE".to_string(),
            pattern_name: "S3 Missing Lifecycle Policy".to_string(),
            description: "S3 bucket created without lifecycle rules. Objects will remain in standard storage indefinitely.".to_string(),
            severity: "MEDIUM".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "No lifecycle_rule or lifecycle_configuration found".to_string(),
                "Data will accumulate in standard storage (most expensive tier)".to_string(),
            ],
            suggested_fix: Some(
                "Add lifecycle rules to transition objects to cheaper storage classes: \
                Intelligent-Tiering (automatic), Glacier (archive), or Deep Archive (long-term). \
                Consider expiration rules for temporary/logs data.".to_string()
            ),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: None,
            assumptions: None,
        });
    }

    None
}

/// Pattern 4: Unbounded Lambda concurrency
fn detect_unbounded_lambda_concurrency(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_lambda_function" {
        return None;
    }

    // Check if reserved_concurrent_executions is set
    let has_concurrency_limit = if let Some(config) = &change.new_config {
        config.get("reserved_concurrent_executions").is_some()
    } else {
        false
    };

    if !has_concurrency_limit {
        return Some(AntiPattern {
            pattern_id: "UNBOUNDED_LAMBDA_CONCURRENCY".to_string(),
            pattern_name: "Unbounded Lambda Concurrency".to_string(),
            description:
                "Lambda function without concurrency limits can cause unexpected cost spikes."
                    .to_string(),
            severity: "HIGH".to_string(),
            detected_in: change.resource_id.clone(),
            evidence: vec![
                "No reserved_concurrent_executions configured".to_string(),
                "Function can scale to account-level limits (1000 concurrent by default)"
                    .to_string(),
                "Traffic spikes or bugs can cause runaway costs".to_string(),
            ],
            suggested_fix: Some(
                "Set reserved_concurrent_executions based on expected peak load. \
                Start conservative (e.g., 10-100) and adjust based on CloudWatch metrics. \
                Consider provisioned concurrency for predictable traffic."
                    .to_string(),
            ),
            cost_impact: None,
            confidence: Some("MEDIUM".to_string()),
            thresholds: None,
            assumptions: None,
        });
    }

    None
}

/// Pattern 5: DynamoDB pay-per-request default
fn detect_dynamodb_pay_per_request_default(change: &ResourceChange) -> Option<AntiPattern> {
    if change.resource_type != "aws_dynamodb_table" {
        return None;
    }

    // Check billing mode
    if let Some(config) = &change.new_config {
        let billing_mode = config
            .get("billing_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("PAY_PER_REQUEST"); // Default is on-demand

        if billing_mode == "PAY_PER_REQUEST" {
            // Check if this is intentional or just using defaults
            let has_explicit_config = config.get("billing_mode").is_some();

            if !has_explicit_config {
                return Some(AntiPattern {
                    pattern_id: "DYNAMODB_PAY_PER_REQUEST_DEFAULT".to_string(),
                    pattern_name: "DynamoDB On-Demand by Default".to_string(),
                    description: "DynamoDB table using pay-per-request mode (potentially by default). \
                                 Provisioned mode is often cheaper for predictable workloads.".to_string(),
                    severity: "LOW".to_string(),
                    detected_in: change.resource_id.clone(),
                    evidence: vec![
                        "Billing mode: PAY_PER_REQUEST (on-demand)".to_string(),
                        "On-demand pricing is ~5x more expensive than provisioned for consistent workloads".to_string(),
                    ],
                    suggested_fix: Some(
                        "If workload has predictable traffic patterns, switch to PROVISIONED billing mode \
                        with autoscaling. On-demand is best for unpredictable/spiky workloads or <1M requests/month.".to_string()
                    ),
                    cost_impact: None,
                    confidence: Some("LOW".to_string()),
                    thresholds: None,
                    assumptions: None,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::shared::models::{ChangeAction, CostEstimate, ResourceChange};
    use serde_json::json;

    #[test]
    fn test_nat_gateway_pattern() {
        let change = ResourceChange::builder()
            .resource_id("aws_nat_gateway.test")
            .resource_type("aws_nat_gateway")
            .action(ChangeAction::Create)
            .new_config(json!({}))
            .build();

        let estimate = CostEstimate::builder()
            .resource_id("test")
            .monthly_cost(32.85)
            .prediction_interval_low(24.64)
            .prediction_interval_high(41.06)
            .confidence_score(0.95)
            .build();

        let patterns = detect_anti_patterns(&change, Some(&estimate));
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "NAT_GATEWAY_OVERUSE");
        assert_eq!(patterns[0].severity, "HIGH");
    }

    #[test]
    fn test_overprovisioned_ec2() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "m5.4xlarge"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "OVERPROVISIONED_EC2");
    }

    #[test]
    fn test_s3_missing_lifecycle() {
        let change = ResourceChange::builder()
            .resource_id("aws_s3_bucket.test")
            .resource_type("aws_s3_bucket")
            .action(ChangeAction::Create)
            .new_config(json!({"bucket": "test-bucket"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "S3_MISSING_LIFECYCLE");
    }

    #[test]
    fn test_unbounded_lambda() {
        let change = ResourceChange::builder()
            .resource_id("aws_lambda_function.test")
            .resource_type("aws_lambda_function")
            .action(ChangeAction::Create)
            .new_config(json!({"function_name": "test", "memory_size": 256}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "UNBOUNDED_LAMBDA_CONCURRENCY");
    }

    #[test]
    fn test_dynamodb_pay_per_request() {
        let change = ResourceChange::builder()
            .resource_id("aws_dynamodb_table.test")
            .resource_type("aws_dynamodb_table")
            .action(ChangeAction::Create)
            .new_config(json!({"name": "test-table"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].pattern_id, "DYNAMODB_PAY_PER_REQUEST_DEFAULT");
    }

    #[test]
    fn test_no_patterns() {
        let change = ResourceChange::builder()
            .resource_id("aws_instance.test")
            .resource_type("aws_instance")
            .action(ChangeAction::Create)
            .new_config(json!({"instance_type": "t3.micro"}))
            .build();

        let patterns = detect_anti_patterns(&change, None);
        assert_eq!(patterns.len(), 0);
    }
}
