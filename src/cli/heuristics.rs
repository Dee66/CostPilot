// CLI commands for managing cost heuristics

use crate::engines::prediction::HeuristicsLoader;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum HeuristicsCommand {
    /// Show loaded heuristics statistics
    Stats {
        /// Path to heuristics file (optional, will auto-discover if not provided)
        #[arg(long)]
        file: Option<PathBuf>,
    },

    /// Show search paths for heuristics discovery
    Paths,

    /// Validate heuristics file format
    Validate {
        /// Path to heuristics file to validate
        file: PathBuf,
    },

    /// Show heuristics for specific service
    Show {
        /// Service type: ec2, rds, lambda, s3, dynamodb, etc.
        service: String,

        /// Path to heuristics file (optional)
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

pub fn execute_heuristics_command(command: HeuristicsCommand) -> Result<String, String> {
    match command {
        HeuristicsCommand::Stats { file } => execute_stats(file),
        HeuristicsCommand::Paths => execute_paths(),
        HeuristicsCommand::Validate { file } => execute_validate(file),
        HeuristicsCommand::Show { service, file } => execute_show(service, file),
    }
}

fn execute_stats(file: Option<PathBuf>) -> Result<String, String> {
    let loader = HeuristicsLoader::new();

    let heuristics = if let Some(path) = file {
        loader
            .load_from_file(&path)
            .map_err(|e| format!("Failed to load heuristics: {}", e))?
    } else {
        loader
            .load()
            .map_err(|e| format!("Failed to load heuristics: {}", e))?
    };

    let stats = loader.get_statistics(&heuristics);
    Ok(stats.format_text())
}

fn execute_paths() -> Result<String, String> {
    let search_paths = get_search_paths_for_display();

    let mut output = String::from("🔍 Heuristics Search Paths\n");
    output.push_str("==========================\n\n");

    for (idx, path) in search_paths.iter().enumerate() {
        let status = if path.exists() {
            "✓ Found"
        } else {
            "✗ Not found"
        };
        output.push_str(&format!("{}. {} - {}\n", idx + 1, path.display(), status));
    }

    output.push_str("\n💡 Tip: Place cost_heuristics.json in any of these locations\n");

    Ok(output)
}

fn execute_validate(file: PathBuf) -> Result<String, String> {
    let loader = HeuristicsLoader::new();

    match loader.load_from_file(&file) {
        Ok(heuristics) => {
            let stats = loader.get_statistics(&heuristics);

            let mut output = String::from("✅ Heuristics file is valid\n\n");
            output.push_str(&stats.format_text());
            Ok(output)
        }
        Err(e) => Err(format!("❌ Validation failed: {}", e)),
    }
}

fn execute_show(service: String, file: Option<PathBuf>) -> Result<String, String> {
    let loader = HeuristicsLoader::new();

    let heuristics = if let Some(path) = file {
        loader
            .load_from_file(&path)
            .map_err(|e| format!("Failed to load heuristics: {}", e))?
    } else {
        loader
            .load()
            .map_err(|e| format!("Failed to load heuristics: {}", e))?
    };

    let mut output = format!("💰 {} Cost Heuristics\n", service.to_uppercase());
    output.push_str("====================\n\n");

    match service.to_lowercase().as_str() {
        "ec2" => {
            output.push_str("Instance Types:\n");
            for (instance_type, cost) in &heuristics.compute.ec2 {
                output.push_str(&format!(
                    "  {} - ${:.4}/hour (${:.2}/month)\n",
                    instance_type, cost.hourly, cost.monthly
                ));
            }
        }
        "lambda" => {
            output.push_str(&format!(
                "Price per GB-second: ${:.10}\n",
                heuristics.compute.lambda.price_per_gb_second
            ));
            output.push_str(&format!(
                "Price per request: ${:.10}\n",
                heuristics.compute.lambda.price_per_request
            ));
            output.push_str(&format!(
                "Free tier requests: {}\n",
                heuristics.compute.lambda.free_tier_requests
            ));
            output.push_str(&format!(
                "Free tier compute: {} GB-seconds\n",
                heuristics.compute.lambda.free_tier_compute_gb_seconds
            ));
        }
        "rds" => {
            output.push_str("MySQL Instance Types:\n");
            for (instance_type, cost) in &heuristics.database.rds.mysql {
                output.push_str(&format!(
                    "  {} - ${:.4}/hour (${:.2}/month)\n",
                    instance_type, cost.hourly, cost.monthly
                ));
            }
            output.push_str("\nPostgreSQL Instance Types:\n");
            for (instance_type, cost) in &heuristics.database.rds.postgres {
                output.push_str(&format!(
                    "  {} - ${:.4}/hour (${:.2}/month)\n",
                    instance_type, cost.hourly, cost.monthly
                ));
            }
            output.push_str(&format!(
                "\nStorage (GP2): ${:.3}/GB/month\n",
                heuristics.database.rds.storage_gp2_per_gb
            ));
            output.push_str(&format!(
                "Storage (GP3): ${:.3}/GB/month\n",
                heuristics.database.rds.storage_gp3_per_gb
            ));
        }
        "s3" => {
            if let Some(price) = heuristics.storage.s3.standard.first_50tb_per_gb {
                output.push_str(&format!("Standard (first 50 TB): ${:.3}/GB/month\n", price));
            }
            if let Some(price) = heuristics.storage.s3.glacier.per_gb {
                output.push_str(&format!("Glacier: ${:.4}/GB/month\n", price));
            }
            output.push_str(&format!(
                "PUT/COPY/POST/LIST: ${:.4}/1000 requests\n",
                heuristics.storage.s3.requests.put_copy_post_list_per_1000
            ));
            output.push_str(&format!(
                "GET/SELECT: ${:.4}/1000 requests\n",
                heuristics.storage.s3.requests.get_select_per_1000
            ));
        }
        "dynamodb" => {
            output.push_str("On-Demand:\n");
            output.push_str(&format!(
                "  Write request unit: ${:.8}\n",
                heuristics.database.dynamodb.on_demand.write_request_unit
            ));
            output.push_str(&format!(
                "  Read request unit: ${:.8}\n",
                heuristics.database.dynamodb.on_demand.read_request_unit
            ));
            output.push_str(&format!(
                "  Storage: ${:.2}/GB/month\n",
                heuristics.database.dynamodb.on_demand.storage_per_gb
            ));

            output.push_str("\nProvisioned:\n");
            output.push_str(&format!(
                "  Write capacity unit: ${:.5}/hour\n",
                heuristics
                    .database
                    .dynamodb
                    .provisioned
                    .write_capacity_unit_hourly
            ));
            output.push_str(&format!(
                "  Read capacity unit: ${:.5}/hour\n",
                heuristics
                    .database
                    .dynamodb
                    .provisioned
                    .read_capacity_unit_hourly
            ));
        }
        "nat" | "nat_gateway" => {
            output.push_str(&format!(
                "Hourly: ${:.4} (${:.2}/month)\n",
                heuristics.networking.nat_gateway.hourly, heuristics.networking.nat_gateway.monthly
            ));
            output.push_str(&format!(
                "Data processing: ${:.3}/GB\n",
                heuristics.networking.nat_gateway.data_processing_per_gb
            ));
        }
        "alb" | "load_balancer" => {
            output.push_str(&format!(
                "Hourly: ${:.4} (${:.2}/month)\n",
                heuristics.networking.load_balancer.alb.hourly,
                heuristics.networking.load_balancer.alb.monthly
            ));
            output.push_str(&format!(
                "LCU: ${:.3}/hour\n",
                heuristics.networking.load_balancer.alb.lcu_hourly
            ));
        }
        "ebs" => {
            output.push_str("EBS Volume Types:\n");
            for (volume_type, cost) in &heuristics.storage.ebs {
                output.push_str(&format!(
                    "  {} - ${:.3}/GB/month\n",
                    volume_type, cost.per_gb
                ));
            }
        }
        _ => {
            return Err(format!(
                "Unknown service: {}. Supported: ec2, rds, lambda, s3, dynamodb, nat, alb, ebs",
                service
            ));
        }
    }

    Ok(output)
}

// Helper to get search paths for CLI display
fn get_search_paths_for_display() -> Vec<PathBuf> {
    let _loader = HeuristicsLoader::new();
    // Access the field directly since it's a public method
    let mut paths = Vec::new();

    // Replicate default paths logic for display
    paths.push(PathBuf::from("heuristics/cost_heuristics.json"));
    paths.push(PathBuf::from("cost_heuristics.json"));

    if let Ok(current_dir) = std::env::current_dir() {
        paths.push(current_dir.join("heuristics/cost_heuristics.json"));
    }

    if let Some(home) = std::env::var_os("HOME") {
        let home_path = PathBuf::from(home);
        paths.push(home_path.join(".costpilot/cost_heuristics.json"));
        paths.push(home_path.join(".config/costpilot/cost_heuristics.json"));
    }

    paths.push(PathBuf::from("/etc/costpilot/cost_heuristics.json"));
    paths.push(PathBuf::from(
        "/usr/local/share/costpilot/cost_heuristics.json",
    ));

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_paths() {
        let result = execute_paths();
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Heuristics Search Paths"));
        assert!(output.contains("cost_heuristics.json"));
    }
}
