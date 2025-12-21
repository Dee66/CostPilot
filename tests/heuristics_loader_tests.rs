use costpilot::engines::prediction::heuristics_loader::{HeuristicsLoader, HeuristicsStats};
use costpilot::engines::prediction::prediction_engine::CostHeuristics;
use costpilot::engines::shared::error_model::{CostPilotError, ErrorCategory};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

// ===== BASIC LOADER TESTS =====

#[test]
fn test_heuristics_loader_new() {
    let loader = HeuristicsLoader::new();
    // Basic construction test - should not panic
    assert!(!loader.search_paths().is_empty());
}

#[test]
fn test_heuristics_loader_default() {
    let loader = HeuristicsLoader::default();
    // Default should work without panicking
    assert!(!loader.search_paths().is_empty());
}

#[test]
fn test_heuristics_loader_with_paths() {
    let custom_paths = vec![
        PathBuf::from("/custom/path/heuristics.json"),
        PathBuf::from("/another/path/heuristics.json"),
    ];

    let loader = HeuristicsLoader::with_paths(custom_paths.clone());
    assert_eq!(loader.search_paths(), custom_paths);
}

// ===== SEARCH PATH TESTS =====

#[test]
fn test_default_search_paths_contains_expected_locations() {
    let paths = HeuristicsLoader::default_search_paths();

    // Should include current directory paths
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("cost_heuristics.json")));

    // Should include system paths
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("/etc/")));
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("/usr/local/")));
}

#[test]
fn test_default_search_paths_includes_home_directory() {
    let paths = HeuristicsLoader::default_search_paths();

    if let Some(home) = std::env::var_os("HOME") {
        let home_str = home.to_string_lossy();
        assert!(paths.iter().any(|p| p.to_string_lossy().contains(&*home_str)));
    }
}

#[test]
fn test_default_search_paths_includes_current_directory() {
    let paths = HeuristicsLoader::default_search_paths();

    // Should include relative paths for current directory
    assert!(paths.iter().any(|p| p.is_relative()));
}

// ===== FILE LOADING TESTS =====

#[test]
fn test_load_from_file_nonexistent() {
    let loader = HeuristicsLoader::new();
    let result = loader.load_from_file(Path::new("nonexistent_file.json"));
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::FileSystemError);
    assert!(error.message.contains("Failed to read heuristics file"));
}

#[test]
fn test_load_from_file_invalid_json() {
    let loader = HeuristicsLoader::new();

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"invalid json content").unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ParseError);
    assert!(error.message.contains("Failed to parse heuristics JSON"));
}

#[test]
fn test_load_from_file_valid_heuristics() {
    let loader = HeuristicsLoader::new();

    let valid_heuristics = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(valid_heuristics.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    if let Err(e) = &result { println!("Error: {:?}", e); } assert!(result.is_ok());

    let heuristics = result.unwrap();
    assert_eq!(heuristics.version, "1.0.0");
    assert_eq!(heuristics.compute.ec2.len(), 1);
}

// ===== LOAD METHOD TESTS =====

#[test]
fn test_load_no_files_found() {
    let loader = HeuristicsLoader::with_paths(vec![
        PathBuf::from("/definitely/does/not/exist/heuristics.json"),
        PathBuf::from("/another/nonexistent/path/heuristics.json"),
    ]);

    let result = loader.load();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::FileSystemError);
    assert!(error.message.contains("Could not find cost_heuristics.json"));
}

#[test]
fn test_load_with_valid_file() {
    let loader = HeuristicsLoader::new();

    let valid_heuristics = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(valid_heuristics.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let custom_paths = vec![temp_file.path().to_path_buf()];
    let loader = HeuristicsLoader::with_paths(custom_paths);

    let result = loader.load();
    if let Err(e) = &result { println!("Error: {:?}", e); } assert!(result.is_ok());

    let heuristics = result.unwrap();
    assert_eq!(heuristics.version, "1.0.0");
}

// ===== VALIDATION TESTS =====

#[test]
fn test_validate_missing_version() {
    let loader = HeuristicsLoader::new();

    let invalid_heuristics = CostHeuristics {
        version: "no-dots".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute: Default::default(),
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid version format"));
}

#[test]
fn test_validate_invalid_version_major() {
    let loader = HeuristicsLoader::new();

    let invalid_heuristics = CostHeuristics {
        version: "abc.1.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute: Default::default(),
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid major version number"));
}

#[test]
fn test_validate_invalid_version_minor() {
    let loader = HeuristicsLoader::new();

    let invalid_heuristics = CostHeuristics {
        version: "1.abc.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute: Default::default(),
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid minor version number"));
}

#[test]
fn test_validate_invalid_version_patch() {
    let loader = HeuristicsLoader::new();

    let invalid_heuristics = CostHeuristics {
        version: "1.0.abc".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute: Default::default(),
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid patch version number"));
}

#[test]
fn test_validate_empty_ec2_instances() {
    let loader = HeuristicsLoader::new();

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute: Default::default(), // Empty EC2
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("No EC2 instance types defined"));
}

#[test]
fn test_validate_invalid_ec2_hourly_cost() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: -1.0, // Invalid negative cost
        monthly: 7.488,
    });

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute,
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid hourly cost"));
}

#[test]
fn test_validate_invalid_lambda_price() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.0104,
        monthly: 7.488,
    });
    compute.lambda.price_per_gb_second = -1.0; // Invalid negative price

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute,
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid Lambda price_per_gb_second"));
}

#[test]
fn test_validate_empty_rds_mysql() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.0104,
        monthly: 7.488,
    });
    compute.lambda.price_per_gb_second = 0.0000166667; // Valid lambda price

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute,
        database: Default::default(), // Empty RDS MySQL
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("No RDS MySQL instance types defined"));
}

// ===== VERSION COMPATIBILITY TESTS =====

#[test]
fn test_check_version_compatibility_too_new_major() {
    let loader = HeuristicsLoader::new();

    let result = loader.check_version_compatibility("2.0.0");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("not compatible"));
}

#[test]
fn test_check_version_compatibility_too_old() {
    let loader = HeuristicsLoader::new();

    let result = loader.check_version_compatibility("0.9.0");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("too old"));
}

#[test]
fn test_check_version_compatibility_valid_versions() {
    let loader = HeuristicsLoader::new();

    // These should all pass
    assert!(loader.check_version_compatibility("1.0.0").is_ok());
    assert!(loader.check_version_compatibility("1.0.1").is_ok());
    assert!(loader.check_version_compatibility("1.1.0").is_ok());
    assert!(loader.check_version_compatibility("1.1.5").is_ok());
}

#[test]
fn test_check_version_compatibility_invalid_format() {
    let loader = HeuristicsLoader::new();

    let result = loader.check_version_compatibility("1.0");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid version format"));
}

#[test]
fn test_check_version_compatibility_non_numeric() {
    let loader = HeuristicsLoader::new();

    let result = loader.check_version_compatibility("a.b.c");
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid major version number"));
}

// ===== STATISTICS TESTS =====

#[test]
fn test_get_statistics() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.0104,
        monthly: 7.488,
    });
    compute.ec2.insert("t3.small".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.0208,
        monthly: 14.976,
    });

    let mut database = costpilot::engines::prediction::prediction_engine::DatabaseHeuristics::default();
    database.rds.mysql.insert("db.t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.017,
        monthly: 12.41,
    });
    database.rds.postgres.insert("db.t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.018,
        monthly: 13.14,
    });

    let mut storage = costpilot::engines::prediction::prediction_engine::StorageHeuristics::default();
    storage.ebs.insert("gp3".to_string(), costpilot::engines::prediction::prediction_engine::EbsCost {
        per_gb: 0.08,
    });

    let heuristics = CostHeuristics {
        version: "1.2.3".to_string(),
        last_updated: "2024-12-19".to_string(),
        compute,
        database,
        storage,
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let stats = loader.get_statistics(&heuristics);

    assert_eq!(stats.version, "1.2.3");
    assert_eq!(stats.last_updated, "2024-12-19");
    assert_eq!(stats.ec2_instance_count, 2);
    assert_eq!(stats.rds_mysql_count, 1);
    assert_eq!(stats.rds_postgres_count, 1);
    assert_eq!(stats.ebs_types_count, 1);
    assert!(stats.lambda_configured);
    assert!(stats.dynamodb_configured);
    assert!(stats.nat_gateway_configured);
}

#[test]
fn test_heuristics_stats_format_text() {
    let stats = HeuristicsStats {
        version: "1.2.3".to_string(),
        last_updated: "2024-12-19".to_string(),
        ec2_instance_count: 5,
        rds_mysql_count: 3,
        rds_postgres_count: 2,
        ebs_types_count: 4,
        lambda_configured: true,
        dynamodb_configured: true,
        nat_gateway_configured: false,
    };

    let formatted = stats.format_text();

    assert!(formatted.contains("Version: 1.2.3"));
    assert!(formatted.contains("Last Updated: 2024-12-19"));
    assert!(formatted.contains("EC2 Instance Types: 5"));
    assert!(formatted.contains("RDS MySQL: 3"));
    assert!(formatted.contains("RDS Postgres: 2"));
    assert!(formatted.contains("EBS Types: 4"));
    assert!(formatted.contains("Lambda: ✓"));
    assert!(formatted.contains("DynamoDB: ✓"));
    assert!(formatted.contains("NAT Gateway: ✗"));
}

// ===== INTEGRATION TESTS =====

#[test]
fn test_load_from_file_with_validation() {
    let loader = HeuristicsLoader::new();

    let valid_heuristics = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(valid_heuristics.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    if let Err(e) = &result { println!("Error: {:?}", e); } assert!(result.is_ok());

    let heuristics = result.unwrap();
    let stats = loader.get_statistics(&heuristics);

    assert_eq!(stats.ec2_instance_count, 1);
    assert_eq!(stats.rds_mysql_count, 1);
    assert_eq!(stats.rds_postgres_count, 0);
    assert_eq!(stats.ebs_types_count, 1);
}

// ===== ERROR HANDLING TESTS =====

#[test]
fn test_load_from_file_permission_denied() {
    let loader = HeuristicsLoader::new();

    // Create a file and remove read permissions (if possible on this system)
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"test content").unwrap();
    temp_file.flush().unwrap();

    // On Unix systems, we can try to remove read permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_file.path()).unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(temp_file.path(), perms).unwrap();

        let result = loader.load_from_file(temp_file.path());
        assert!(result.is_err());
    }

    // On other systems, just test that the file exists and is readable
    #[cfg(not(unix))]
    {
        let result = loader.load_from_file(temp_file.path());
        // Should fail because it's not valid JSON, not because of permissions
        assert!(result.is_err());
    }
}

#[test]
fn test_load_fallback_behavior() {
    let loader = HeuristicsLoader::new();

    let valid_heuristics = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(valid_heuristics.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Create loader with multiple paths, first nonexistent, second valid
    let custom_paths = vec![
        PathBuf::from("/definitely/does/not/exist.json"),
        temp_file.path().to_path_buf(),
    ];
    let loader = HeuristicsLoader::with_paths(custom_paths);

    let result = loader.load();
    if let Err(e) = &result { println!("Error: {:?}", e); } assert!(result.is_ok()); // Should find the valid file
}

// ===== EDGE CASE TESTS =====

#[test]
fn test_validate_extremely_high_cost() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 1000.1, // Above maximum allowed
        monthly: 7.488,
    });

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute,
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid hourly cost"));
}

#[test]
fn test_validate_zero_cost() {
    let loader = HeuristicsLoader::new();

    let mut compute = costpilot::engines::prediction::prediction_engine::ComputeHeuristics::default();
    compute.ec2.insert("t3.micro".to_string(), costpilot::engines::prediction::prediction_engine::InstanceCost {
        hourly: 0.0, // Zero cost should be invalid
        monthly: 7.488,
    });

    let invalid_heuristics = CostHeuristics {
        version: "1.0.0".to_string(),
        last_updated: "2024-01-01".to_string(),
        compute,
        database: Default::default(),
        storage: Default::default(),
        networking: Default::default(),
        prediction_intervals: Default::default(),
        cold_start_defaults: Default::default(),
    };

    let result = loader.validate(&invalid_heuristics);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ValidationError);
    assert!(error.message.contains("Invalid hourly cost"));
}

#[test]
fn test_version_compatibility_edge_cases() {
    let loader = HeuristicsLoader::new();

    // Test exact minimum version
    assert!(loader.check_version_compatibility("1.0.0").is_ok());

    // Test versions just above minimum
    assert!(loader.check_version_compatibility("1.0.1").is_ok());
    assert!(loader.check_version_compatibility("1.1.0").is_ok());

    // Test maximum allowed major version
    assert!(loader.check_version_compatibility("1.999.999").is_ok());
}

#[test]
fn test_search_paths_with_special_characters() {
    // Test that paths with spaces, unicode, etc. are handled
    let custom_paths = vec![
        PathBuf::from("/path with spaces/heuristics.json"),
        PathBuf::from("/path/with/unicode/测试.json"),
        PathBuf::from("/path/with/dots.../file.json"),
    ];

    let loader = HeuristicsLoader::with_paths(custom_paths.clone());
    assert_eq!(loader.search_paths(), custom_paths);
}

#[test]
fn test_load_from_file_empty_file() {
    let loader = HeuristicsLoader::new();

    let mut temp_file = NamedTempFile::new().unwrap();
    // Write empty content
    temp_file.write_all(b"").unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ParseError);
}

#[test]
fn test_load_from_file_partial_json() {
    let loader = HeuristicsLoader::new();

    let partial_json = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            }
        }"#; // Missing closing braces

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(partial_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::ParseError);
}

// ===== ADDITIONAL VALIDATION TESTS =====

#[test]
fn test_validate_missing_required_fields() {
    let loader = HeuristicsLoader::new();

    // Test with minimal valid heuristics to ensure all required fields are present
    let minimal_heuristics = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {
                "t3.micro": {
                    "hourly": 0.0104,
                    "monthly": 7.488
                }
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(minimal_heuristics.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    if let Err(e) = &result { println!("Error: {:?}", e); } assert!(result.is_ok());
}

#[test]
fn test_statistics_formatting() {
    let stats = HeuristicsStats {
        version: "2.1.0".to_string(),
        last_updated: "2025-01-15".to_string(),
        ec2_instance_count: 0,
        rds_mysql_count: 0,
        rds_postgres_count: 0,
        ebs_types_count: 0,
        lambda_configured: false,
        dynamodb_configured: false,
        nat_gateway_configured: false,
    };

    let formatted = stats.format_text();

    // Test that formatting works even with zero counts
    assert!(formatted.contains("EC2 Instance Types: 0"));
    assert!(formatted.contains("Lambda: ✗"));
    assert!(formatted.contains("DynamoDB: ✗"));
    assert!(formatted.contains("NAT Gateway: ✗"));
}

#[test]
fn test_loader_with_empty_search_paths() {
    let loader = HeuristicsLoader::with_paths(vec![]);

    let result = loader.load();
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.category, ErrorCategory::FileSystemError);
}

#[test]
fn test_validate_version_with_leading_zeros() {
    let loader = HeuristicsLoader::new();

    // Versions with leading zeros should be parsed correctly
    assert!(loader.check_version_compatibility("1.01.001").is_ok());
    assert!(loader.check_version_compatibility("001.000.000").is_ok());
}

#[test]
fn test_load_from_file_large_file() {
    let loader = HeuristicsLoader::new();

    // Create a large but valid JSON file
    let mut large_content = r#"{
        "version": "1.0.0",
        "last_updated": "2024-01-01",
        "compute": {
            "ec2": {"#.to_string();

    // Add many EC2 instances to make it large
    for i in 0..1000 {
        let comma = if i < 999 { "," } else { "" };
        large_content.push_str(&format!(r#"
                "instance{}": {{
                    "hourly": 0.01,
                    "monthly": 7.0
                }}{}"#, i, comma));
    }

    large_content.push_str(r#"
            },
            "lambda": {
                "price_per_gb_second": 0.0000166667,
                "price_per_request": 0.20,
                "free_tier_requests": 1000000,
                "free_tier_compute_gb_seconds": 400000,
                "default_memory_mb": 128,
                "default_duration_ms": 3000
            }
        },
        "database": {
            "rds": {
                "mysql": {
                    "db.t3.micro": {
                        "hourly": 0.017,
                        "monthly": 12.41
                    }
                },
                "postgres": {},
                "storage_gp2_per_gb": 0.115,
                "storage_gp3_per_gb": 0.08,
                "backup_per_gb": 0.095
            },
            "dynamodb": {
                "on_demand": {
                    "write_request_unit": 1.25,
                    "read_request_unit": 0.25,
                    "storage_per_gb": 0.25
                },
                "provisioned": {
                    "write_capacity_unit_hourly": 0.00065,
                    "read_capacity_unit_hourly": 0.00013,
                    "storage_per_gb": 0.25
                }
            }
        },
        "storage": {
            "s3": {
                "standard": {
                    "per_gb": 0.023
                },
                "glacier": {
                    "per_gb": 0.004
                },
                "requests": {
                    "put_copy_post_list_per_1000": 0.005,
                    "get_select_per_1000": 0.0004
                }
            },
            "ebs": {
                "gp3": {
                    "per_gb": 0.08
                }
            }
        },
        "networking": {
            "nat_gateway": {
                "hourly": 0.045,
                "monthly": 32.0,
                "data_processing_per_gb": 0.045
            },
            "load_balancer": {
                "alb": {
                    "hourly": 0.0225,
                    "monthly": 16.2,
                    "lcu_hourly": 0.008
                }
            }
        },
        "cold_start_defaults": {
            "dynamodb_unknown_rcu": 5,
            "dynamodb_unknown_wcu": 5,
            "lambda_default_invocations": 1000000,
            "nat_gateway_default_gb": 100,
            "s3_default_gb": 100,
            "ec2_default_utilization": 0.7
        },
        "prediction_intervals": {
            "range_factor": 0.5
        }
    }"#);

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(large_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = loader.load_from_file(temp_file.path());
    assert!(result.is_ok());

    let heuristics = result.unwrap();
    assert_eq!(heuristics.compute.ec2.len(), 1000);
}
