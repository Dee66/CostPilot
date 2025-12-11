// Minimal heuristics for Free edition - static values, no file loading

use super::prediction_engine::{
    ColdStartDefaults, ComputeHeuristics, CostHeuristics, DatabaseHeuristics, DynamoDbCost,
    DynamoDbOnDemand, DynamoDbProvisioned, EbsCost, InstanceCost, LambdaCost, LoadBalancerCost,
    LoadBalancerType, NatGatewayCost, NetworkingHeuristics, PredictionIntervals, RdsCost, S3Cost,
    S3Requests, S3Tier, StorageHeuristics,
};
use std::collections::HashMap;

/// Minimal static heuristics for Free edition
#[derive(Debug, Clone)]
pub struct MinimalHeuristics;

impl MinimalHeuristics {
    pub fn to_cost_heuristics() -> CostHeuristics {
        // EC2 instances
        let mut ec2_map = HashMap::new();
        ec2_map.insert(
            "t3.micro".to_string(),
            InstanceCost {
                hourly: 0.0104,
                monthly: 7.59,
            },
        );
        ec2_map.insert(
            "t3.small".to_string(),
            InstanceCost {
                hourly: 0.0208,
                monthly: 15.18,
            },
        );
        ec2_map.insert(
            "t3.medium".to_string(),
            InstanceCost {
                hourly: 0.0416,
                monthly: 30.37,
            },
        );
        ec2_map.insert(
            "m5.large".to_string(),
            InstanceCost {
                hourly: 0.096,
                monthly: 70.08,
            },
        );

        // RDS instances
        let mut rds_mysql = HashMap::new();
        rds_mysql.insert(
            "db.t3.micro".to_string(),
            InstanceCost {
                hourly: 0.017,
                monthly: 12.41,
            },
        );
        rds_mysql.insert(
            "db.t3.small".to_string(),
            InstanceCost {
                hourly: 0.034,
                monthly: 24.82,
            },
        );

        let mut rds_postgres = HashMap::new();
        rds_postgres.insert(
            "db.t3.micro".to_string(),
            InstanceCost {
                hourly: 0.018,
                monthly: 13.14,
            },
        );
        rds_postgres.insert(
            "db.t3.small".to_string(),
            InstanceCost {
                hourly: 0.036,
                monthly: 26.28,
            },
        );

        // EBS volumes
        let mut ebs_map = HashMap::new();
        ebs_map.insert("gp2".to_string(), EbsCost { per_gb: 0.10 });
        ebs_map.insert("gp3".to_string(), EbsCost { per_gb: 0.08 });

        CostHeuristics {
            version: "1.0.0-minimal".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            compute: ComputeHeuristics {
                ec2: ec2_map,
                lambda: LambdaCost {
                    price_per_gb_second: 0.0000166667,
                    price_per_request: 0.0000002,
                    free_tier_requests: 1_000_000,
                    free_tier_compute_gb_seconds: 400_000,
                    default_memory_mb: 128,
                    default_duration_ms: 1000,
                },
            },
            storage: StorageHeuristics {
                s3: S3Cost {
                    standard: S3Tier {
                        per_gb: Some(0.023),
                        first_50tb_per_gb: Some(0.023),
                    },
                    glacier: S3Tier {
                        per_gb: Some(0.004),
                        first_50tb_per_gb: None,
                    },
                    requests: S3Requests {
                        put_copy_post_list_per_1000: 0.005,
                        get_select_per_1000: 0.0004,
                    },
                },
                ebs: ebs_map,
            },
            database: DatabaseHeuristics {
                rds: RdsCost {
                    mysql: rds_mysql,
                    postgres: rds_postgres,
                    storage_gp2_per_gb: 0.115,
                    storage_gp3_per_gb: 0.115,
                    backup_per_gb: 0.095,
                },
                dynamodb: DynamoDbCost {
                    on_demand: DynamoDbOnDemand {
                        write_request_unit: 0.00000125,
                        read_request_unit: 0.00000025,
                        storage_per_gb: 0.25,
                    },
                    provisioned: DynamoDbProvisioned {
                        write_capacity_unit_hourly: 0.00065,
                        read_capacity_unit_hourly: 0.00013,
                        storage_per_gb: 0.25,
                    },
                },
            },
            networking: NetworkingHeuristics {
                nat_gateway: NatGatewayCost {
                    hourly: 0.045,
                    monthly: 32.85,
                    data_processing_per_gb: 0.045,
                },
                load_balancer: LoadBalancerCost {
                    alb: LoadBalancerType {
                        hourly: 0.0225,
                        monthly: 16.43,
                        lcu_hourly: 0.008,
                    },
                },
            },
            cold_start_defaults: ColdStartDefaults {
                dynamodb_unknown_rcu: 5,
                dynamodb_unknown_wcu: 5,
                lambda_default_invocations: 1_000_000,
                nat_gateway_default_gb: 100,
                s3_default_gb: 100,
                ec2_default_utilization: 0.7,
            },
            prediction_intervals: PredictionIntervals { range_factor: 0.3 },
        }
    }
}

impl Default for MinimalHeuristics {
    fn default() -> Self {
        Self
    }
}
