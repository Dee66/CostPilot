// Group resources by AWS service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A group of resources organized by AWS service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroup {
    /// AWS service name (e.g., "EC2", "S3", "RDS")
    pub service_name: String,
    /// Resource addresses in this service
    pub resources: Vec<String>,
    /// Total monthly cost for this service
    pub monthly_cost: f64,
    /// Number of resources
    pub resource_count: usize,
    /// Cost breakdown by resource type
    pub cost_by_type: HashMap<String, f64>,
    /// Service category (compute, storage, database, networking, etc.)
    pub category: ServiceCategory,
}

/// AWS service category for high-level grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceCategory {
    Compute,
    Storage,
    Database,
    Networking,
    Security,
    Analytics,
    ApplicationIntegration,
    Management,
    MachineLearning,
    Other,
}

impl ServiceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compute => "Compute",
            Self::Storage => "Storage",
            Self::Database => "Database",
            Self::Networking => "Networking",
            Self::Security => "Security",
            Self::Analytics => "Analytics",
            Self::ApplicationIntegration => "Application Integration",
            Self::Management => "Management",
            Self::MachineLearning => "Machine Learning",
            Self::Other => "Other",
        }
    }
}

impl ServiceGroup {
    pub fn new(service_name: String, category: ServiceCategory) -> Self {
        Self {
            service_name,
            resources: Vec::new(),
            monthly_cost: 0.0,
            resource_count: 0,
            cost_by_type: HashMap::new(),
            category,
        }
    }

    pub fn add_resource(&mut self, address: String, resource_type: String, cost: f64) {
        self.resources.push(address);
        self.monthly_cost += cost;
        self.resource_count += 1;
        *self.cost_by_type.entry(resource_type).or_insert(0.0) += cost;
    }

    pub fn average_cost_per_resource(&self) -> f64 {
        if self.resource_count == 0 {
            0.0
        } else {
            self.monthly_cost / self.resource_count as f64
        }
    }
}

/// Group resources by their AWS service
pub fn group_by_service(
    resources: &[(String, String, f64)], // (address, type, cost)
) -> Vec<ServiceGroup> {
    let mut groups: HashMap<String, ServiceGroup> = HashMap::new();

    for (address, resource_type, cost) in resources {
        let (service_name, category) = extract_service_info(resource_type);
        let group = groups
            .entry(service_name.clone())
            .or_insert_with(|| ServiceGroup::new(service_name, category));
        group.add_resource(address.clone(), resource_type.clone(), *cost);
    }

    let mut result: Vec<ServiceGroup> = groups.into_values().collect();
    result.sort_by(|a, b| b.monthly_cost.partial_cmp(&a.monthly_cost).unwrap());
    result
}

/// Extract service name and category from resource type
/// Examples:
/// - "aws_instance" -> ("EC2", Compute)
/// - "aws_s3_bucket" -> ("S3", Storage)
/// - "aws_rds_cluster" -> ("RDS", Database)
pub fn extract_service_info(resource_type: &str) -> (String, ServiceCategory) {
    let service_map: HashMap<&str, (&str, ServiceCategory)> = [
        // Compute
        ("aws_instance", ("EC2", ServiceCategory::Compute)),
        ("aws_launch_template", ("EC2", ServiceCategory::Compute)),
        ("aws_autoscaling_group", ("EC2", ServiceCategory::Compute)),
        ("aws_lambda_function", ("Lambda", ServiceCategory::Compute)),
        ("aws_ecs_cluster", ("ECS", ServiceCategory::Compute)),
        ("aws_ecs_service", ("ECS", ServiceCategory::Compute)),
        ("aws_ecs_task_definition", ("ECS", ServiceCategory::Compute)),
        ("aws_eks_cluster", ("EKS", ServiceCategory::Compute)),
        (
            "aws_batch_compute_environment",
            ("Batch", ServiceCategory::Compute),
        ),
        // Storage
        ("aws_s3_bucket", ("S3", ServiceCategory::Storage)),
        ("aws_ebs_volume", ("EBS", ServiceCategory::Storage)),
        ("aws_efs_file_system", ("EFS", ServiceCategory::Storage)),
        (
            "aws_fsx_windows_file_system",
            ("FSx", ServiceCategory::Storage),
        ),
        ("aws_glacier_vault", ("Glacier", ServiceCategory::Storage)),
        // Database
        ("aws_db_instance", ("RDS", ServiceCategory::Database)),
        ("aws_rds_cluster", ("RDS", ServiceCategory::Database)),
        (
            "aws_dynamodb_table",
            ("DynamoDB", ServiceCategory::Database),
        ),
        (
            "aws_elasticache_cluster",
            ("ElastiCache", ServiceCategory::Database),
        ),
        (
            "aws_redshift_cluster",
            ("Redshift", ServiceCategory::Database),
        ),
        (
            "aws_neptune_cluster",
            ("Neptune", ServiceCategory::Database),
        ),
        (
            "aws_docdb_cluster",
            ("DocumentDB", ServiceCategory::Database),
        ),
        // Networking
        ("aws_vpc", ("VPC", ServiceCategory::Networking)),
        ("aws_subnet", ("VPC", ServiceCategory::Networking)),
        ("aws_nat_gateway", ("VPC", ServiceCategory::Networking)),
        ("aws_internet_gateway", ("VPC", ServiceCategory::Networking)),
        ("aws_lb", ("ELB", ServiceCategory::Networking)),
        ("aws_alb", ("ALB", ServiceCategory::Networking)),
        ("aws_elb", ("ELB", ServiceCategory::Networking)),
        ("aws_route53_zone", ("Route53", ServiceCategory::Networking)),
        (
            "aws_cloudfront_distribution",
            ("CloudFront", ServiceCategory::Networking),
        ),
        (
            "aws_api_gateway_rest_api",
            ("API Gateway", ServiceCategory::Networking),
        ),
        ("aws_vpn_gateway", ("VPN", ServiceCategory::Networking)),
        (
            "aws_dx_connection",
            ("Direct Connect", ServiceCategory::Networking),
        ),
        // Security
        ("aws_security_group", ("VPC", ServiceCategory::Security)),
        ("aws_iam_role", ("IAM", ServiceCategory::Security)),
        ("aws_iam_policy", ("IAM", ServiceCategory::Security)),
        ("aws_kms_key", ("KMS", ServiceCategory::Security)),
        ("aws_wafv2_web_acl", ("WAF", ServiceCategory::Security)),
        (
            "aws_guardduty_detector",
            ("GuardDuty", ServiceCategory::Security),
        ),
        (
            "aws_secretsmanager_secret",
            ("Secrets Manager", ServiceCategory::Security),
        ),
        // Analytics
        (
            "aws_kinesis_stream",
            ("Kinesis", ServiceCategory::Analytics),
        ),
        (
            "aws_kinesis_firehose_delivery_stream",
            ("Kinesis", ServiceCategory::Analytics),
        ),
        (
            "aws_athena_workgroup",
            ("Athena", ServiceCategory::Analytics),
        ),
        (
            "aws_glue_catalog_database",
            ("Glue", ServiceCategory::Analytics),
        ),
        ("aws_emr_cluster", ("EMR", ServiceCategory::Analytics)),
        // Application Integration
        (
            "aws_sqs_queue",
            ("SQS", ServiceCategory::ApplicationIntegration),
        ),
        (
            "aws_sns_topic",
            ("SNS", ServiceCategory::ApplicationIntegration),
        ),
        (
            "aws_mq_broker",
            ("MQ", ServiceCategory::ApplicationIntegration),
        ),
        (
            "aws_step_functions_state_machine",
            ("Step Functions", ServiceCategory::ApplicationIntegration),
        ),
        // Management
        (
            "aws_cloudwatch_log_group",
            ("CloudWatch", ServiceCategory::Management),
        ),
        (
            "aws_cloudwatch_metric_alarm",
            ("CloudWatch", ServiceCategory::Management),
        ),
        ("aws_ssm_parameter", ("SSM", ServiceCategory::Management)),
        (
            "aws_config_config_rule",
            ("Config", ServiceCategory::Management),
        ),
        // Machine Learning
        (
            "aws_sagemaker_notebook_instance",
            ("SageMaker", ServiceCategory::MachineLearning),
        ),
        (
            "aws_sagemaker_endpoint",
            ("SageMaker", ServiceCategory::MachineLearning),
        ),
    ]
    .iter()
    .cloned()
    .collect();

    service_map
        .get(resource_type)
        .map(|(name, cat)| (name.to_string(), *cat))
        .unwrap_or_else(|| {
            // Fallback: try to extract from prefix (aws_xxx_yyy -> XXX)
            if let Some(service) = resource_type.strip_prefix("aws_") {
                let parts: Vec<&str> = service.split('_').collect();
                if !parts.is_empty() {
                    let name = parts[0].to_uppercase();
                    return (name, ServiceCategory::Other);
                }
            }
            ("Unknown".to_string(), ServiceCategory::Other)
        })
}

/// Group services by category
pub fn group_by_category(groups: &[ServiceGroup]) -> HashMap<ServiceCategory, Vec<ServiceGroup>> {
    let mut category_groups: HashMap<ServiceCategory, Vec<ServiceGroup>> = HashMap::new();

    for group in groups {
        category_groups
            .entry(group.category)
            .or_default()
            .push(group.clone());
    }

    category_groups
}

/// Calculate total cost by category
pub fn cost_by_category(groups: &[ServiceGroup]) -> HashMap<ServiceCategory, f64> {
    let mut costs: HashMap<ServiceCategory, f64> = HashMap::new();

    for group in groups {
        *costs.entry(group.category).or_insert(0.0) += group.monthly_cost;
    }

    costs
}

/// Generate a service cost summary report
pub fn generate_service_report(groups: &[ServiceGroup]) -> String {
    let mut report = String::new();
    report.push_str("Service Cost Summary\n");
    report.push_str("===================\n\n");

    let total_cost: f64 = groups.iter().map(|g| g.monthly_cost).sum();
    report.push_str(&format!("Total Monthly Cost: ${:.2}\n\n", total_cost));

    // Group by category
    let category_costs = cost_by_category(groups);
    let mut categories: Vec<(ServiceCategory, f64)> = category_costs.into_iter().collect();
    categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    report.push_str("By Category:\n");
    for (category, cost) in categories {
        let percentage = if total_cost > 0.0 {
            (cost / total_cost) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!(
            "  {} ${:.2}/mo ({:.1}%)\n",
            category.as_str(),
            cost,
            percentage
        ));
    }

    report.push_str("\nTop Services:\n");
    for (i, group) in groups.iter().take(10).enumerate() {
        let percentage = if total_cost > 0.0 {
            (group.monthly_cost / total_cost) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!(
            "  {}. {} ${:.2}/mo ({:.1}%, {} resources)\n",
            i + 1,
            group.service_name,
            group.monthly_cost,
            percentage,
            group.resource_count
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service_info() {
        let (service, category) = extract_service_info("aws_instance");
        assert_eq!(service, "EC2");
        assert_eq!(category, ServiceCategory::Compute);

        let (service, category) = extract_service_info("aws_s3_bucket");
        assert_eq!(service, "S3");
        assert_eq!(category, ServiceCategory::Storage);

        let (service, category) = extract_service_info("aws_db_instance");
        assert_eq!(service, "RDS");
        assert_eq!(category, ServiceCategory::Database);
    }

    #[test]
    fn test_group_by_service() {
        let resources = vec![
            (
                "aws_instance.web".to_string(),
                "aws_instance".to_string(),
                100.0,
            ),
            (
                "aws_instance.app".to_string(),
                "aws_instance".to_string(),
                150.0,
            ),
            (
                "aws_s3_bucket.data".to_string(),
                "aws_s3_bucket".to_string(),
                50.0,
            ),
        ];

        let groups = group_by_service(&resources);
        assert_eq!(groups.len(), 2);

        // EC2 should have highest cost
        assert_eq!(groups[0].service_name, "EC2");
        assert_eq!(groups[0].monthly_cost, 250.0);
        assert_eq!(groups[0].resource_count, 2);

        // S3
        assert_eq!(groups[1].service_name, "S3");
        assert_eq!(groups[1].monthly_cost, 50.0);
    }

    #[test]
    fn test_cost_by_category() {
        let groups = vec![
            ServiceGroup {
                service_name: "EC2".to_string(),
                resources: vec![],
                monthly_cost: 100.0,
                resource_count: 1,
                cost_by_type: HashMap::new(),
                category: ServiceCategory::Compute,
            },
            ServiceGroup {
                service_name: "Lambda".to_string(),
                resources: vec![],
                monthly_cost: 50.0,
                resource_count: 1,
                cost_by_type: HashMap::new(),
                category: ServiceCategory::Compute,
            },
            ServiceGroup {
                service_name: "S3".to_string(),
                resources: vec![],
                monthly_cost: 75.0,
                resource_count: 1,
                cost_by_type: HashMap::new(),
                category: ServiceCategory::Storage,
            },
        ];

        let costs = cost_by_category(&groups);
        assert_eq!(costs.get(&ServiceCategory::Compute), Some(&150.0));
        assert_eq!(costs.get(&ServiceCategory::Storage), Some(&75.0));
    }
}
