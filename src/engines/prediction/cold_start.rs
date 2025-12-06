// Cold start inference for unknown values

use super::prediction_engine::ColdStartDefaults;

/// Cold start inference engine
pub struct ColdStartInference {
    defaults: ColdStartDefaults,
}

impl ColdStartInference {
    /// Create a new cold start inference engine
    pub fn new(defaults: &ColdStartDefaults) -> Self {
        Self {
            defaults: defaults.clone(),
        }
    }

    /// Estimate EC2 cost for unknown instance type
    pub fn estimate_ec2_cost(&self, instance_type: &str) -> f64 {
        // Parse instance family and size
        let parts: Vec<&str> = instance_type.split('.').collect();
        
        if parts.len() != 2 {
            return 50.0; // Default fallback
        }

        let family = parts[0];
        let size = parts[1];

        // Base cost multipliers by family
        let family_multiplier = match family {
            "t3" | "t2" => 1.0,
            "m5" | "m6" => 1.3,
            "c5" | "c6" => 1.2,
            "r5" | "r6" => 1.5,
            "i3" | "i4" => 1.8,
            "p3" | "p4" => 10.0, // GPU instances are expensive
            _ => 1.2, // Conservative default
        };

        // Size multipliers
        let size_multiplier = match size {
            "nano" => 0.5,
            "micro" => 1.0,
            "small" => 2.0,
            "medium" => 4.0,
            "large" => 8.0,
            "xlarge" => 16.0,
            "2xlarge" => 32.0,
            "4xlarge" => 64.0,
            "8xlarge" => 128.0,
            "16xlarge" => 256.0,
            _ => 8.0, // Default to large equivalent
        };

        // Base cost for t3.micro
        let base_cost = 7.6;
        
        base_cost * family_multiplier * size_multiplier
    }

    /// Estimate RDS cost for unknown configuration
    pub fn estimate_rds_cost(&self, instance_class: &str, engine: &str) -> f64 {
        // Extract size from instance class (e.g., db.t3.micro -> micro)
        let parts: Vec<&str> = instance_class.split('.').collect();
        
        if parts.len() < 3 {
            return 100.0; // Conservative default for RDS
        }

        let size = parts[2];

        // Base multipliers
        let engine_multiplier = match engine {
            "postgres" | "postgresql" => 1.1,
            "mysql" | "mariadb" => 1.0,
            "oracle" => 3.0,
            "sqlserver" => 2.5,
            _ => 1.0,
        };

        let size_multiplier = match size {
            "micro" => 1.0,
            "small" => 2.0,
            "medium" => 4.0,
            "large" => 8.0,
            "xlarge" => 16.0,
            "2xlarge" => 32.0,
            "4xlarge" => 64.0,
            _ => 4.0,
        };

        // Base cost for db.t3.micro MySQL
        let base_cost = 12.4;
        
        base_cost * engine_multiplier * size_multiplier
    }

    /// Estimate storage cost
    pub fn estimate_storage_cost(&self, storage_type: &str, size_gb: f64) -> f64 {
        let per_gb_cost = match storage_type {
            "gp3" | "gp2" => 0.10,
            "io1" | "io2" => 0.125,
            "st1" => 0.045,
            "sc1" => 0.015,
            "s3_standard" => 0.023,
            "s3_glacier" => 0.004,
            _ => 0.10,
        };

        size_gb * per_gb_cost
    }

    /// Get default DynamoDB RCU
    pub fn default_dynamodb_rcu(&self) -> u32 {
        self.defaults.dynamodb_unknown_rcu
    }

    /// Get default DynamoDB WCU
    pub fn default_dynamodb_wcu(&self) -> u32 {
        self.defaults.dynamodb_unknown_wcu
    }

    /// Get default Lambda invocations
    pub fn default_lambda_invocations(&self) -> u64 {
        self.defaults.lambda_default_invocations
    }

    /// Get default NAT Gateway data transfer GB
    pub fn default_nat_gateway_gb(&self) -> u32 {
        self.defaults.nat_gateway_default_gb
    }

    /// Get default S3 storage GB
    pub fn default_s3_storage_gb(&self) -> u32 {
        self.defaults.s3_default_gb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_defaults() -> ColdStartDefaults {
        ColdStartDefaults {
            dynamodb_unknown_rcu: 15,
            dynamodb_unknown_wcu: 15,
            lambda_default_invocations: 10000,
            nat_gateway_default_gb: 10,
            s3_default_gb: 50,
            ec2_default_utilization: 0.4,
        }
    }

    #[test]
    fn test_ec2_cost_estimation() {
        let cold_start = ColdStartInference::new(&get_test_defaults());

        // Test known patterns
        let t3_micro_cost = cold_start.estimate_ec2_cost("t3.micro");
        assert!(t3_micro_cost > 0.0);

        let t3_large_cost = cold_start.estimate_ec2_cost("t3.large");
        assert!(t3_large_cost > t3_micro_cost);

        // Larger instance should cost more
        let m5_xlarge_cost = cold_start.estimate_ec2_cost("m5.xlarge");
        assert!(m5_xlarge_cost > t3_large_cost);
    }

    #[test]
    fn test_rds_cost_estimation() {
        let cold_start = ColdStartInference::new(&get_test_defaults());

        let mysql_cost = cold_start.estimate_rds_cost("db.t3.micro", "mysql");
        let postgres_cost = cold_start.estimate_rds_cost("db.t3.micro", "postgres");
        
        // Postgres is slightly more expensive
        assert!(postgres_cost >= mysql_cost);

        // Larger instance should cost more
        let large_cost = cold_start.estimate_rds_cost("db.t3.large", "mysql");
        assert!(large_cost > mysql_cost);
    }

    #[test]
    fn test_storage_cost_estimation() {
        let cold_start = ColdStartInference::new(&get_test_defaults());

        let gp2_cost = cold_start.estimate_storage_cost("gp2", 100.0);
        let glacier_cost = cold_start.estimate_storage_cost("s3_glacier", 100.0);

        // Glacier should be cheaper than GP2
        assert!(glacier_cost < gp2_cost);
    }
}
