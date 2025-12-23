/// Test data generators for property-based and fuzz testing
///
/// Provides functions to generate realistic test data for various AWS resources,
/// plans, and configurations.

use serde_json::json;

/// Generate a random EC2 instance type from common options
pub fn random_ec2_instance_type(seed: usize) -> &'static str {
    let types = [
        "t3.micro", "t3.small", "t3.medium", "t3.large",
        "t3.xlarge", "t3.2xlarge",
        "m5.large", "m5.xlarge", "m5.2xlarge", "m5.4xlarge",
        "c5.large", "c5.xlarge", "c5.2xlarge",
        "r5.large", "r5.xlarge", "r5.2xlarge",
    ];
    types[seed % types.len()]
}

/// Generate a random RDS instance class
pub fn random_rds_instance_class(seed: usize) -> &'static str {
    let classes = [
        "db.t3.micro", "db.t3.small", "db.t3.medium",
        "db.r5.large", "db.r5.xlarge", "db.r5.2xlarge",
        "db.m5.large", "db.m5.xlarge", "db.m5.2xlarge",
    ];
    classes[seed % classes.len()]
}

/// Generate a random RDS engine
pub fn random_rds_engine(seed: usize) -> &'static str {
    let engines = ["mysql", "postgres", "mariadb"];
    engines[seed % engines.len()]
}

/// Generate a Terraform plan with N EC2 instances
pub fn generate_terraform_plan_with_n_ec2(count: usize) -> serde_json::Value {
    let mut resources = Vec::new();

    for i in 0..count {
        let instance_type = random_ec2_instance_type(i);
        resources.push(json!({
            "address": format!("aws_instance.web[{}]", i),
            "mode": "managed",
            "type": "aws_instance",
            "name": "web",
            "index": i,
            "change": {
                "actions": ["create"],
                "after": {
                    "instance_type": instance_type,
                    "ami": "ami-12345678",
                    "tags": {
                        "Name": format!("web-{}", i),
                        "Environment": if i % 3 == 0 { "production" } else { "development" }
                    }
                }
            }
        }));
    }

    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": resources
    })
}

/// Generate a Terraform plan with mixed resource types
pub fn generate_mixed_terraform_plan(
    ec2_count: usize,
    rds_count: usize,
    lambda_count: usize,
) -> serde_json::Value {
    let mut resources = Vec::new();

    // EC2 instances
    for i in 0..ec2_count {
        resources.push(json!({
            "address": format!("aws_instance.app[{}]", i),
            "type": "aws_instance",
            "change": {
                "actions": ["create"],
                "after": {
                    "instance_type": random_ec2_instance_type(i),
                    "ami": "ami-12345678"
                }
            }
        }));
    }

    // RDS instances
    for i in 0..rds_count {
        resources.push(json!({
            "address": format!("aws_db_instance.db[{}]", i),
            "type": "aws_db_instance",
            "change": {
                "actions": ["create"],
                "after": {
                    "instance_class": random_rds_instance_class(i),
                    "engine": random_rds_engine(i),
                    "allocated_storage": 100 + (i * 50)
                }
            }
        }));
    }

    // Lambda functions
    for i in 0..lambda_count {
        resources.push(json!({
            "address": format!("aws_lambda_function.func[{}]", i),
            "type": "aws_lambda_function",
            "change": {
                "actions": ["create"],
                "after": {
                    "function_name": format!("func-{}", i),
                    "runtime": "python3.11",
                    "memory_size": 128 + (i * 128),
                    "timeout": 30
                }
            }
        }));
    }

    json!({
        "format_version": "1.1",
        "terraform_version": "1.5.0",
        "resource_changes": resources
    })
}

/// Generate a large Terraform plan for stress testing
pub fn generate_large_terraform_plan(resource_count: usize) -> serde_json::Value {
    let resources_per_type = resource_count / 6;
    generate_mixed_terraform_plan(
        resources_per_type * 2,  // EC2
        resources_per_type,      // RDS
        resources_per_type * 3,  // Lambda
    )
}

/// Generate a policy with N rules
pub fn generate_policy_with_n_rules(rule_count: usize) -> serde_json::Value {
    let mut rules = Vec::new();

    for i in 0..rule_count {
        rules.push(json!({
            "id": format!("rule-{}", i),
            "description": format!("Test rule {}", i),
            "severity": if i % 3 == 0 { "Critical" } else if i % 3 == 1 { "High" } else { "Medium" },
            "conditions": [{
                "type": "MonthlyCost",
                "operator": "GreaterThan",
                "value": 1000.0 + (i as f64 * 100.0)
            }],
            "actions": ["Warn"]
        }));
    }

    json!({
        "id": "generated-policy",
        "name": "Generated Test Policy",
        "version": "1.0.0",
        "rules": rules
    })
}

/// Generate a graph with N nodes for cycle detection testing
pub fn generate_graph_nodes(node_count: usize) -> Vec<(String, Vec<String>)> {
    let mut nodes = Vec::new();

    for i in 0..node_count {
        let node_id = format!("node_{}", i);
        let mut dependencies = Vec::new();

        // Create some dependencies (but avoid cycles in this basic generator)
        if i > 0 {
            dependencies.push(format!("node_{}", i - 1));
        }
        if i > 1 && i % 3 == 0 {
            dependencies.push(format!("node_{}", i - 2));
        }

        nodes.push((node_id, dependencies));
    }

    nodes
}

/// Generate a graph with a deliberate cycle for testing
pub fn generate_cyclic_graph() -> Vec<(String, Vec<String>)> {
    vec![
        ("A".to_string(), vec!["B".to_string()]),
        ("B".to_string(), vec!["C".to_string()]),
        ("C".to_string(), vec!["A".to_string()]), // Cycle: A -> B -> C -> A
        ("D".to_string(), vec!["B".to_string()]),
    ]
}

/// Generate realistic module paths
pub fn generate_module_path(depth: usize, index: usize) -> String {
    let components = ["vpc", "compute", "database", "storage", "network"];
    let mut path = "root".to_string();

    for d in 0..depth {
        let component = components[(index + d) % components.len()];
        path = format!("{}.{}", path, component);
    }

    path
}

/// Generate resource tags with common patterns
pub fn generate_resource_tags(environment: &str, cost_center: Option<&str>) -> serde_json::Value {
    let mut tags = json!({
        "Environment": environment,
        "ManagedBy": "Terraform",
        "Application": "costpilot-test"
    });

    if let Some(cc) = cost_center {
        tags["CostCenter"] = json!(cc);
    }

    tags
}

/// Generate a baseline file with multiple modules
pub fn generate_baseline_with_modules(module_count: usize) -> serde_json::Value {
    let mut baselines = Vec::new();

    for i in 0..module_count {
        baselines.push(json!({
            "module_path": generate_module_path(2, i),
            "expected_monthly_cost": 100.0 + (i as f64 * 50.0),
            "last_updated": "2025-12-06T00:00:00Z",
            "justification": format!("Baseline for module {}", i)
        }));
    }

    json!({
        "version": "1.0.0",
        "baselines": baselines
    })
}

/// Generate SLO file with multiple SLOs
pub fn generate_slo_file(slo_count: usize) -> serde_json::Value {
    let mut slos = Vec::new();
    let slo_types = ["MonthlyCost", "ModuleCost", "ServiceCost", "ResourceCount"];

    for i in 0..slo_count {
        let slo_type = slo_types[i % slo_types.len()];
        slos.push(json!({
            "id": format!("slo-{}", i),
            "name": format!("Test SLO {}", i),
            "type": slo_type,
            "threshold": 1000.0 + (i as f64 * 200.0),
            "enforcement": if i % 2 == 0 { "Block" } else { "Warn" }
        }));
    }

    json!({
        "version": "1.0.0",
        "slos": slos
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_terraform_plan_with_n_ec2() {
        let plan = generate_terraform_plan_with_n_ec2(5);
        let resources = plan["resource_changes"].as_array().unwrap();
        assert_eq!(resources.len(), 5);
    }

    #[test]
    fn test_generate_mixed_terraform_plan() {
        let plan = generate_mixed_terraform_plan(2, 1, 3);
        let resources = plan["resource_changes"].as_array().unwrap();
        assert_eq!(resources.len(), 6); // 2 + 1 + 3
    }

    #[test]
    fn test_generate_cyclic_graph() {
        let graph = generate_cyclic_graph();
        assert_eq!(graph.len(), 4);
        // Node C depends on A, creating a cycle
        assert!(graph[2].1.contains(&"A".to_string()));
    }

    #[test]
    fn test_generate_module_path() {
        let path = generate_module_path(3, 0);
        assert!(path.starts_with("root."));
        assert_eq!(path.matches('.').count(), 3);
    }

    #[test]
    fn test_generate_policy_with_n_rules() {
        let policy = generate_policy_with_n_rules(10);
        let rules = policy["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 10);
    }
}
