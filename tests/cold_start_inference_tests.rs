use costpilot::engines::prediction::cold_start::ColdStartInference;
use costpilot::engines::prediction::prediction_engine::ColdStartDefaults;

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
fn test_cold_start_new() {
    let defaults = get_test_defaults();
    let _inference = ColdStartInference::new(&defaults);
    // Just test that it creates successfully
}

#[test]
fn test_estimate_ec2_t2_micro() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t2.micro");
    assert!(cost > 0.0);
    assert!(cost == 7.6); // Base cost
}

#[test]
fn test_estimate_ec2_t2_small() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t2.small");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 2.0); // Base * size multiplier
}

#[test]
fn test_estimate_ec2_t3_micro() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.micro");
    assert!(cost > 0.0);
    assert!(cost == 7.6);
}

#[test]
fn test_estimate_ec2_t3_small() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.small");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 2.0);
}

#[test]
fn test_estimate_ec2_t3_medium() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.medium");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 4.0);
}

#[test]
fn test_estimate_ec2_t3_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 8.0);
}

#[test]
fn test_estimate_ec2_t3_xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 16.0);
}

#[test]
fn test_estimate_ec2_t3_2xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.2xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 32.0);
}

#[test]
fn test_estimate_ec2_m5_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("m5.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.3 * 8.0); // Base * family * size
}

#[test]
fn test_estimate_ec2_m5_xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("m5.xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.3 * 16.0);
}

#[test]
fn test_estimate_ec2_m5_2xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("m5.2xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.3 * 32.0);
}

#[test]
fn test_estimate_ec2_m5_4xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("m5.4xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.3 * 64.0);
}

#[test]
fn test_estimate_ec2_c5_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("c5.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.2 * 8.0);
}

#[test]
fn test_estimate_ec2_c5_xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("c5.xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.2 * 16.0);
}

#[test]
fn test_estimate_ec2_c5_2xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("c5.2xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.2 * 32.0);
}

#[test]
fn test_estimate_ec2_c5_4xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("c5.4xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.2 * 64.0);
}

#[test]
fn test_estimate_ec2_r5_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("r5.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.5 * 8.0);
}

#[test]
fn test_estimate_ec2_r5_xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("r5.xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.5 * 16.0);
}

#[test]
fn test_estimate_ec2_r5_2xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("r5.2xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.5 * 32.0);
}

#[test]
fn test_estimate_ec2_r5_4xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("r5.4xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.5 * 64.0);
}

#[test]
fn test_estimate_ec2_i3_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("i3.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.8 * 8.0);
}

#[test]
fn test_estimate_ec2_i3_xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("i3.xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.8 * 16.0);
}

#[test]
fn test_estimate_ec2_p3_2xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("p3.2xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 10.0 * 32.0); // GPU instances are expensive
}

#[test]
fn test_estimate_ec2_p3_8xlarge() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("p3.8xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 10.0 * 128.0);
}

#[test]
fn test_estimate_ec2_unknown_family() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("x5.large");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 1.2 * 8.0); // Default family multiplier
}

#[test]
fn test_estimate_ec2_unknown_size() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.5xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 8.0); // Default size multiplier
}

#[test]
fn test_estimate_ec2_malformed_type() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("invalid");
    assert!(cost == 50.0); // Fallback for malformed types
}

#[test]
fn test_estimate_ec2_empty_type() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("");
    assert!(cost == 50.0); // Fallback
}

#[test]
fn test_estimate_ec2_single_part() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3");
    assert!(cost == 50.0); // Fallback
}

#[test]
fn test_estimate_ec2_three_parts() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.micro.extra");
    assert!(cost == 50.0); // Fallback
}

// RDS Cost Estimation Tests
#[test]
fn test_estimate_rds_t3_micro_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4); // Base cost
}

#[test]
fn test_estimate_rds_t3_micro_postgres() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "postgres");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 1.1); // Base * engine multiplier
}

#[test]
fn test_estimate_rds_t3_micro_oracle() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "oracle");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 3.0); // Expensive engine
}

#[test]
fn test_estimate_rds_t3_micro_sqlserver() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "sqlserver");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 2.5);
}

#[test]
fn test_estimate_rds_t3_small_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.small", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 2.0); // Size multiplier
}

#[test]
fn test_estimate_rds_t3_medium_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.medium", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 4.0);
}

#[test]
fn test_estimate_rds_t3_large_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.large", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 8.0);
}

#[test]
fn test_estimate_rds_t3_xlarge_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.xlarge", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 16.0);
}

#[test]
fn test_estimate_rds_t3_2xlarge_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.2xlarge", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 32.0);
}

#[test]
fn test_estimate_rds_m5_large_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.m5.large", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 8.0); // Same as t3.large
}

#[test]
fn test_estimate_rds_r5_large_mysql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.r5.large", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 8.0);
}

#[test]
fn test_estimate_rds_unknown_engine() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "unknown");
    assert!(cost > 0.0);
    assert!(cost == 12.4); // Default engine multiplier
}

#[test]
fn test_estimate_rds_unknown_size() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.5xlarge", "mysql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 4.0); // Default size multiplier
}

#[test]
fn test_estimate_rds_malformed_class() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("invalid", "mysql");
    assert!(cost == 100.0); // Conservative fallback
}

#[test]
fn test_estimate_rds_empty_class() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("", "mysql");
    assert!(cost == 100.0);
}

#[test]
fn test_estimate_rds_two_parts() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3", "mysql");
    assert!(cost == 100.0);
}

// Storage Cost Estimation Tests
#[test]
fn test_estimate_storage_gp3() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp3", 100.0);
    assert!(cost == 100.0 * 0.10);
}

#[test]
fn test_estimate_storage_gp2() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp2", 100.0);
    assert!(cost == 100.0 * 0.10);
}

#[test]
fn test_estimate_storage_io1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("io1", 100.0);
    assert!(cost == 100.0 * 0.125);
}

#[test]
fn test_estimate_storage_io2() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("io2", 100.0);
    assert!(cost == 100.0 * 0.125);
}

#[test]
fn test_estimate_storage_st1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("st1", 100.0);
    assert!(cost == 100.0 * 0.045);
}

#[test]
fn test_estimate_storage_sc1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("sc1", 100.0);
    assert!(cost == 100.0 * 0.015);
}

#[test]
fn test_estimate_storage_s3_standard() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("s3_standard", 100.0);
    assert!(cost == 100.0 * 0.023);
}

#[test]
fn test_estimate_storage_s3_glacier() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("s3_glacier", 100.0);
    assert!(cost == 100.0 * 0.004);
}

#[test]
fn test_estimate_storage_unknown_type() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("unknown", 100.0);
    assert!(cost == 100.0 * 0.10); // Default to gp3 cost
}

#[test]
fn test_estimate_storage_zero_size() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp3", 0.0);
    assert!(cost == 0.0);
}

#[test]
fn test_estimate_storage_large_size() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp3", 10000.0);
    assert!(cost == 10000.0 * 0.10);
}

#[test]
fn test_estimate_storage_negative_size() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp3", -100.0);
    assert!(cost == -100.0 * 0.10); // Negative size gives negative cost
}

// Default Value Tests
#[test]
fn test_default_dynamodb_rcu() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let rcu = inference.default_dynamodb_rcu();
    assert_eq!(rcu, 15);
}

#[test]
fn test_default_dynamodb_wcu() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let wcu = inference.default_dynamodb_wcu();
    assert_eq!(wcu, 15);
}

#[test]
fn test_default_lambda_invocations() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let invocations = inference.default_lambda_invocations();
    assert_eq!(invocations, 10000);
}

#[test]
fn test_default_nat_gateway_gb() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let gb = inference.default_nat_gateway_gb();
    assert_eq!(gb, 10);
}

#[test]
fn test_default_s3_storage_gb() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let gb = inference.default_s3_storage_gb();
    assert_eq!(gb, 50);
}

// Edge Cases and Boundary Conditions
#[test]
fn test_estimate_ec2_very_large_instance() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("p3.16xlarge");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 10.0 * 256.0); // Very expensive
}

#[test]
fn test_estimate_rds_very_large_instance() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.r5.16xlarge", "oracle");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 3.0 * 4.0); // Large with expensive engine
}

#[test]
fn test_estimate_storage_very_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("gp3", 1000000.0);
    assert!(cost == 1000000.0 * 0.10);
}

#[test]
fn test_estimate_ec2_nano() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_ec2_cost("t3.nano");
    assert!(cost > 0.0);
    assert!(cost == 7.6 * 0.5);
}

#[test]
fn test_estimate_rds_mariadb() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "mariadb");
    assert!(cost > 0.0);
    assert!(cost == 12.4); // Same as mysql
}

#[test]
fn test_estimate_rds_postgresql() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_rds_cost("db.t3.micro", "postgresql");
    assert!(cost > 0.0);
    assert!(cost == 12.4 * 1.1);
}

#[test]
fn test_estimate_storage_magnetic() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("magnetic", 100.0);
    assert!(cost == 100.0 * 0.10); // Default
}

#[test]
fn test_estimate_storage_empty_type() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let cost = inference.estimate_storage_cost("", 100.0);
    assert!(cost == 100.0 * 0.10);
}

// Comparative Tests
#[test]
fn test_ec2_family_comparison_t_vs_m() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let t_cost = inference.estimate_ec2_cost("t3.large");
    let m_cost = inference.estimate_ec2_cost("m5.large");
    assert!(m_cost > t_cost); // M5 should be more expensive
}

#[test]
fn test_ec2_family_comparison_m_vs_c() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let m_cost = inference.estimate_ec2_cost("m5.large");
    let c_cost = inference.estimate_ec2_cost("c5.large");
    assert!(m_cost > c_cost); // M5 should be more expensive than C5
}

#[test]
fn test_ec2_family_comparison_c_vs_r() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let c_cost = inference.estimate_ec2_cost("c5.large");
    let r_cost = inference.estimate_ec2_cost("r5.large");
    assert!(r_cost > c_cost); // R5 should be more expensive than C5
}

#[test]
fn test_ec2_family_comparison_r_vs_i() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let r_cost = inference.estimate_ec2_cost("r5.large");
    let i_cost = inference.estimate_ec2_cost("i3.large");
    assert!(i_cost > r_cost); // I3 should be more expensive than R5
}

#[test]
fn test_ec2_family_comparison_i_vs_p() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let i_cost = inference.estimate_ec2_cost("i3.large");
    let p_cost = inference.estimate_ec2_cost("p3.large");
    assert!(p_cost > i_cost); // P3 should be much more expensive
}

#[test]
fn test_rds_engine_comparison_mysql_vs_postgres() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let mysql_cost = inference.estimate_rds_cost("db.t3.micro", "mysql");
    let postgres_cost = inference.estimate_rds_cost("db.t3.micro", "postgres");
    assert!(postgres_cost > mysql_cost);
}

#[test]
fn test_rds_engine_comparison_postgres_vs_oracle() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let postgres_cost = inference.estimate_rds_cost("db.t3.micro", "postgres");
    let oracle_cost = inference.estimate_rds_cost("db.t3.micro", "oracle");
    assert!(oracle_cost > postgres_cost);
}

#[test]
fn test_rds_engine_comparison_oracle_vs_sqlserver() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let oracle_cost = inference.estimate_rds_cost("db.t3.micro", "oracle");
    let sqlserver_cost = inference.estimate_rds_cost("db.t3.micro", "sqlserver");
    assert!(oracle_cost > sqlserver_cost); // Oracle is most expensive
}

#[test]
fn test_storage_type_comparison_gp3_vs_io1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let gp3_cost = inference.estimate_storage_cost("gp3", 100.0);
    let io1_cost = inference.estimate_storage_cost("io1", 100.0);
    assert!(io1_cost > gp3_cost);
}

#[test]
fn test_storage_type_comparison_io1_vs_st1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let io1_cost = inference.estimate_storage_cost("io1", 100.0);
    let st1_cost = inference.estimate_storage_cost("st1", 100.0);
    assert!(io1_cost > st1_cost);
}

#[test]
fn test_storage_type_comparison_st1_vs_sc1() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let st1_cost = inference.estimate_storage_cost("st1", 100.0);
    let sc1_cost = inference.estimate_storage_cost("sc1", 100.0);
    assert!(st1_cost > sc1_cost);
}

#[test]
fn test_storage_type_comparison_sc1_vs_glacier() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let sc1_cost = inference.estimate_storage_cost("sc1", 100.0);
    let glacier_cost = inference.estimate_storage_cost("s3_glacier", 100.0);
    assert!(sc1_cost > glacier_cost);
}

// Size Scaling Tests
#[test]
fn test_ec2_size_scaling_micro_to_small() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let micro_cost = inference.estimate_ec2_cost("t3.micro");
    let small_cost = inference.estimate_ec2_cost("t3.small");
    assert!(small_cost == micro_cost * 2.0);
}

#[test]
fn test_ec2_size_scaling_small_to_medium() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let small_cost = inference.estimate_ec2_cost("t3.small");
    let medium_cost = inference.estimate_ec2_cost("t3.medium");
    assert!(medium_cost == small_cost * 2.0);
}

#[test]
fn test_ec2_size_scaling_medium_to_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let medium_cost = inference.estimate_ec2_cost("t3.medium");
    let large_cost = inference.estimate_ec2_cost("t3.large");
    assert!(large_cost == medium_cost * 2.0);
}

#[test]
fn test_rds_size_scaling_micro_to_small() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let micro_cost = inference.estimate_rds_cost("db.t3.micro", "mysql");
    let small_cost = inference.estimate_rds_cost("db.t3.small", "mysql");
    assert!(small_cost == micro_cost * 2.0);
}

#[test]
fn test_rds_size_scaling_small_to_medium() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let small_cost = inference.estimate_rds_cost("db.t3.small", "mysql");
    let medium_cost = inference.estimate_rds_cost("db.t3.medium", "mysql");
    assert!(medium_cost == small_cost * 2.0);
}

#[test]
fn test_rds_size_scaling_medium_to_large() {
    let inference = ColdStartInference::new(&get_test_defaults());
    let medium_cost = inference.estimate_rds_cost("db.t3.medium", "mysql");
    let large_cost = inference.estimate_rds_cost("db.t3.large", "mysql");
    assert!(large_cost == medium_cost * 2.0);
}
