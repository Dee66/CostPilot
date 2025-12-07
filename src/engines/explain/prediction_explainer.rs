// Prediction explainer - generates reasoning chains for cost predictions

use crate::engines::shared::models::{ResourceChange, CostEstimate};
use crate::engines::prediction::prediction_engine::{CostHeuristics, PredictionEngine};
use crate::engines::explain::stepwise::{ReasoningChain, ReasoningChainBuilder, CostComponent};
use serde_json::Value;

pub struct PredictionExplainer<'a> {
    heuristics: &'a CostHeuristics,
}

impl<'a> PredictionExplainer<'a> {
    /// Create new explainer with heuristics
    pub fn new(heuristics: &'a CostHeuristics) -> Self {
        Self { heuristics }
    }

    /// Create from prediction engine
    pub fn from_engine(engine: &'a PredictionEngine) -> Self {
        Self::new(engine.heuristics())
    }

    /// Explain a cost prediction with full reasoning chain
    pub fn explain(
        &self,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) -> ReasoningChain {
        let mut builder = ReasoningChainBuilder::new(
            change.resource_id.clone(),
            change.resource_type.clone(),
        );

        // Step 1: Resource identification
        builder.add_resource_identification(&change.resource_id, &change.resource_type);

        // Step 2-N: Resource-specific reasoning
        match change.resource_type.as_str() {
            "aws_instance" => self.explain_ec2(&mut builder, change, estimate),
            "aws_rds_instance" => self.explain_rds(&mut builder, change, estimate),
            "aws_lambda_function" => self.explain_lambda(&mut builder, change, estimate),
            "aws_dynamodb_table" => self.explain_dynamodb(&mut builder, change, estimate),
            "aws_nat_gateway" => self.explain_nat_gateway(&mut builder, change, estimate),
            "aws_lb" | "aws_alb" => self.explain_load_balancer(&mut builder, change, estimate),
            "aws_s3_bucket" => self.explain_s3(&mut builder, change, estimate),
            _ => self.explain_generic(&mut builder, change, estimate),
        }

        // Final steps: confidence and intervals
        self.add_confidence_reasoning(&mut builder, estimate);
        self.add_interval_reasoning(&mut builder, estimate);

        builder.build()
    }

    /// Explain EC2 instance cost
    fn explain_ec2(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let config = change.new_config.as_ref().unwrap();

        // Extract instance type
        let instance_type = config
            .get("instance_type")
            .and_then(|v| v.as_str())
            .unwrap_or("t3.micro");

        let from_plan = config.get("instance_type").is_some();
        builder.add_configuration_extraction("instance_type", instance_type, from_plan);

        // Lookup or infer pricing
        if let Some(cost) = self.heuristics.compute.ec2.get(instance_type) {
            builder.add_heuristic_lookup(
                instance_type,
                cost.hourly,
                "$/hour",
                &self.heuristics.version,
            );

            builder.add_calculation(
                "Monthly Instance Cost",
                &format!("{:.4} $/hour × 730 hours/month", cost.hourly),
                cost.monthly,
                "$/month",
            );

            let mut components = vec![
                CostComponent {
                    name: "EC2 Instance".to_string(),
                    cost: cost.monthly,
                    percentage: 100.0,
                },
            ];

            builder.set_final_estimate(cost.monthly, estimate.prediction_interval_low, estimate.prediction_interval_high, components);
        } else {
            // Cold start inference
            let inferred_cost = self.infer_ec2_cost(instance_type);
            builder.add_cold_start_inference(
                instance_type,
                &format!("${:.2}/month", inferred_cost),
                &format!("Instance type not in heuristics; inferred from family/size patterns"),
            );

            builder.add_calculation(
                "Monthly Instance Cost",
                "Based on cold-start inference model",
                inferred_cost,
                "$/month",
            );

            builder.set_final_estimate(inferred_cost, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
                CostComponent {
                    name: "EC2 Instance (inferred)".to_string(),
                    cost: inferred_cost,
                    percentage: 100.0,
                },
            ]);
        }
    }

    /// Explain RDS instance cost
    fn explain_rds(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let config = change.new_config.as_ref().unwrap();

        // Extract configuration
        let instance_class = config
            .get("instance_class")
            .and_then(|v| v.as_str())
            .unwrap_or("db.t3.micro");
        let engine = config
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("mysql");
        let storage_gb = config
            .get("allocated_storage")
            .and_then(|v| v.as_f64())
            .unwrap_or(20.0);

        builder.add_configuration_extraction("instance_class", instance_class, config.get("instance_class").is_some());
        builder.add_configuration_extraction("engine", engine, config.get("engine").is_some());
        builder.add_configuration_extraction(
            "allocated_storage",
            &format!("{} GB", storage_gb),
            config.get("allocated_storage").is_some(),
        );

        // Lookup instance pricing
        let instances = match engine {
            "postgres" | "postgresql" => &self.heuristics.database.rds.postgres,
            _ => &self.heuristics.database.rds.mysql,
        };

        let instance_cost = if let Some(cost) = instances.get(instance_class) {
            builder.add_heuristic_lookup(
                instance_class,
                cost.hourly,
                "$/hour",
                &self.heuristics.version,
            );
            builder.add_calculation(
                "Monthly Instance Cost",
                &format!("{:.4} $/hour × 730 hours/month", cost.hourly),
                cost.monthly,
                "$/month",
            );
            cost.monthly
        } else {
            let inferred = 50.0;
            builder.add_cold_start_inference(
                instance_class,
                &format!("${:.2}/month", inferred),
                "RDS instance class not in heuristics",
            );
            inferred
        };

        // Storage cost
        builder.add_heuristic_lookup(
            "gp2_storage",
            self.heuristics.database.rds.storage_gp2_per_gb,
            "$/GB/month",
            &self.heuristics.version,
        );

        let storage_cost = storage_gb * self.heuristics.database.rds.storage_gp2_per_gb;
        builder.add_calculation(
            "Monthly Storage Cost",
            &format!("{} GB × ${:.3}/GB/month", storage_gb, self.heuristics.database.rds.storage_gp2_per_gb),
            storage_cost,
            "$/month",
        );

        let total = instance_cost + storage_cost;
        let instance_pct = (instance_cost / total) * 100.0;
        let storage_pct = (storage_cost / total) * 100.0;

        builder.set_final_estimate(total, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
            CostComponent {
                name: "RDS Instance".to_string(),
                cost: instance_cost,
                percentage: instance_pct,
            },
            CostComponent {
                name: "Storage (GP2)".to_string(),
                cost: storage_cost,
                percentage: storage_pct,
            },
        ]);
    }

    /// Explain Lambda function cost
    fn explain_lambda(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let config = change.new_config.as_ref().unwrap();

        let memory_mb = config
            .get("memory_size")
            .and_then(|v| v.as_f64())
            .unwrap_or(self.heuristics.compute.lambda.default_memory_mb as f64);

        builder.add_configuration_extraction(
            "memory_size",
            &format!("{} MB", memory_mb),
            config.get("memory_size").is_some(),
        );

        // Use defaults for invocations and duration
        let invocations = self.heuristics.compute.lambda.default_memory_mb as f64 / 1000.0;
        let duration_ms = self.heuristics.compute.lambda.default_duration_ms as f64;

        builder.add_configuration_extraction(
            "estimated_invocations",
            &format!("{} per month", invocations as u64),
            false,
        );
        builder.add_configuration_extraction(
            "estimated_duration",
            &format!("{} ms", duration_ms),
            false,
        );

        // Request cost
        builder.add_heuristic_lookup(
            "lambda_request_price",
            self.heuristics.compute.lambda.price_per_request,
            "$/request",
            &self.heuristics.version,
        );

        let request_cost = invocations * self.heuristics.compute.lambda.price_per_request;
        builder.add_calculation(
            "Monthly Request Cost",
            &format!("{} requests × ${:.10}/request", invocations, self.heuristics.compute.lambda.price_per_request),
            request_cost,
            "$/month",
        );

        // Compute cost
        builder.add_heuristic_lookup(
            "lambda_compute_price",
            self.heuristics.compute.lambda.price_per_gb_second,
            "$/GB-second",
            &self.heuristics.version,
        );

        let duration_seconds = duration_ms / 1000.0;
        let gb_seconds = (memory_mb / 1024.0) * duration_seconds * invocations;
        let compute_cost = gb_seconds * self.heuristics.compute.lambda.price_per_gb_second;

        builder.add_calculation(
            "Monthly Compute Cost",
            &format!("({} MB / 1024) × ({} ms / 1000) × {} invocations × ${:.10}/GB-second",
                memory_mb, duration_ms, invocations, self.heuristics.compute.lambda.price_per_gb_second),
            compute_cost,
            "$/month",
        );

        let total = request_cost + compute_cost;
        let request_pct = (request_cost / total) * 100.0;
        let compute_pct = (compute_cost / total) * 100.0;

        builder.set_final_estimate(total, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
            CostComponent {
                name: "Lambda Requests".to_string(),
                cost: request_cost,
                percentage: request_pct,
            },
            CostComponent {
                name: "Lambda Compute".to_string(),
                cost: compute_cost,
                percentage: compute_pct,
            },
        ]);

        builder.add_assumption(
            format!("Assuming {} invocations per month (update with actual usage)", invocations)
        );
        builder.add_assumption(
            format!("Assuming {}ms average duration (measure in production)", duration_ms)
        );
    }

    /// Explain DynamoDB table cost
    fn explain_dynamodb(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let config = change.new_config.as_ref().unwrap();

        let billing_mode = config
            .get("billing_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("PAY_PER_REQUEST");

        builder.add_configuration_extraction("billing_mode", billing_mode, config.get("billing_mode").is_some());

        if billing_mode == "PROVISIONED" {
            let read_capacity = config
                .get("read_capacity")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.heuristics.cold_start_defaults.dynamodb_unknown_rcu as f64);
            let write_capacity = config
                .get("write_capacity")
                .and_then(|v| v.as_f64())
                .unwrap_or(self.heuristics.cold_start_defaults.dynamodb_unknown_wcu as f64);

            builder.add_configuration_extraction(
                "read_capacity",
                &format!("{} RCU", read_capacity),
                config.get("read_capacity").is_some(),
            );
            builder.add_configuration_extraction(
                "write_capacity",
                &format!("{} WCU", write_capacity),
                config.get("write_capacity").is_some(),
            );

            let read_cost = read_capacity * self.heuristics.database.dynamodb.provisioned.read_capacity_unit_hourly * 730.0;
            let write_cost = write_capacity * self.heuristics.database.dynamodb.provisioned.write_capacity_unit_hourly * 730.0;

            builder.add_calculation(
                "Monthly Read Cost",
                &format!("{} RCU × ${:.5}/hour × 730 hours", read_capacity, self.heuristics.database.dynamodb.provisioned.read_capacity_unit_hourly),
                read_cost,
                "$/month",
            );

            builder.add_calculation(
                "Monthly Write Cost",
                &format!("{} WCU × ${:.5}/hour × 730 hours", write_capacity, self.heuristics.database.dynamodb.provisioned.write_capacity_unit_hourly),
                write_cost,
                "$/month",
            );

            let total = read_cost + write_cost;
            builder.set_final_estimate(total, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
                CostComponent {
                    name: "DynamoDB Reads".to_string(),
                    cost: read_cost,
                    percentage: (read_cost / total) * 100.0,
                },
                CostComponent {
                    name: "DynamoDB Writes".to_string(),
                    cost: write_cost,
                    percentage: (write_cost / total) * 100.0,
                },
            ]);
        } else {
            // On-demand: use conservative estimate
            let estimate_cost = 25.0;
            builder.add_cold_start_inference(
                "on_demand_cost",
                &format!("${:.2}/month", estimate_cost),
                "On-demand billing requires actual usage metrics",
            );

            builder.set_final_estimate(estimate_cost, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
                CostComponent {
                    name: "DynamoDB On-Demand".to_string(),
                    cost: estimate_cost,
                    percentage: 100.0,
                },
            ]);

            builder.add_assumption("On-demand cost highly variable; update with actual usage patterns".to_string());
        }
    }

    /// Explain NAT Gateway cost
    fn explain_nat_gateway(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let base_cost = self.heuristics.networking.nat_gateway.monthly;
        let data_gb = self.heuristics.cold_start_defaults.nat_gateway_default_gb as f64;
        let data_cost = data_gb * self.heuristics.networking.nat_gateway.data_processing_per_gb;

        builder.add_heuristic_lookup(
            "nat_gateway_hourly",
            self.heuristics.networking.nat_gateway.hourly,
            "$/hour",
            &self.heuristics.version,
        );

        builder.add_calculation(
            "Monthly NAT Gateway Cost",
            &format!("{:.4} $/hour × 730 hours", self.heuristics.networking.nat_gateway.hourly),
            base_cost,
            "$/month",
        );

        builder.add_configuration_extraction(
            "estimated_data_transfer",
            &format!("{} GB", data_gb),
            false,
        );

        builder.add_calculation(
            "Monthly Data Processing Cost",
            &format!("{} GB × ${:.3}/GB", data_gb, self.heuristics.networking.nat_gateway.data_processing_per_gb),
            data_cost,
            "$/month",
        );

        let total = base_cost + data_cost;
        builder.set_final_estimate(total, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
            CostComponent {
                name: "NAT Gateway (hourly)".to_string(),
                cost: base_cost,
                percentage: (base_cost / total) * 100.0,
            },
            CostComponent {
                name: "Data Processing".to_string(),
                cost: data_cost,
                percentage: (data_cost / total) * 100.0,
            },
        ]);

        builder.add_assumption(format!("Assuming {}GB monthly data transfer; adjust based on actual usage", data_gb));
    }

    /// Explain Load Balancer cost
    fn explain_load_balancer(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let base_cost = self.heuristics.networking.load_balancer.alb.monthly;
        let lcu_cost = self.heuristics.networking.load_balancer.alb.lcu_hourly * 730.0 * 2.0; // Assume 2 LCUs

        builder.add_heuristic_lookup(
            "alb_hourly",
            self.heuristics.networking.load_balancer.alb.hourly,
            "$/hour",
            &self.heuristics.version,
        );

        builder.add_calculation(
            "Monthly ALB Cost",
            &format!("{:.4} $/hour × 730 hours", self.heuristics.networking.load_balancer.alb.hourly),
            base_cost,
            "$/month",
        );

        builder.add_configuration_extraction("estimated_lcus", "2 LCU", false);

        builder.add_calculation(
            "Monthly LCU Cost",
            &format!("{:.3} $/LCU/hour × 2 LCU × 730 hours", self.heuristics.networking.load_balancer.alb.lcu_hourly),
            lcu_cost,
            "$/month",
        );

        let total = base_cost + lcu_cost;
        builder.set_final_estimate(total, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
            CostComponent {
                name: "ALB (hourly)".to_string(),
                cost: base_cost,
                percentage: (base_cost / total) * 100.0,
            },
            CostComponent {
                name: "LCU Usage".to_string(),
                cost: lcu_cost,
                percentage: (lcu_cost / total) * 100.0,
            },
        ]);

        builder.add_assumption("Assuming 2 LCU average usage; monitor actual consumption".to_string());
    }

    /// Explain S3 bucket cost
    fn explain_s3(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        let storage_gb = self.heuristics.cold_start_defaults.s3_default_gb as f64;
        let price_per_gb = self.heuristics.storage.s3.standard.first_50tb_per_gb.unwrap_or(0.023);
        let storage_cost = storage_gb * price_per_gb;

        builder.add_configuration_extraction(
            "estimated_storage",
            &format!("{} GB", storage_gb),
            false,
        );

        builder.add_heuristic_lookup(
            "s3_standard_storage",
            price_per_gb,
            "$/GB/month",
            &self.heuristics.version,
        );

        builder.add_calculation(
            "Monthly Storage Cost",
            &format!("{} GB × ${:.3}/GB", storage_gb, price_per_gb),
            storage_cost,
            "$/month",
        );

        builder.set_final_estimate(storage_cost, estimate.prediction_interval_low, estimate.prediction_interval_high, vec![
            CostComponent {
                name: "S3 Standard Storage".to_string(),
                cost: storage_cost,
                percentage: 100.0,
            },
        ]);

        builder.add_assumption(format!("Assuming {}GB storage; S3 cost highly depends on actual usage", storage_gb));
        builder.add_assumption("Request costs not included; add based on access patterns".to_string());
    }

    /// Explain generic resource
    fn explain_generic(
        &self,
        builder: &mut ReasoningChainBuilder,
        change: &ResourceChange,
        estimate: &CostEstimate,
    ) {
        builder.add_cold_start_inference(
            &change.resource_type,
            &format!("${:.2}/month", estimate.monthly_cost),
            &format!("Using generic cost model for {}", change.resource_type),
        );

        builder.set_final_estimate(
            estimate.monthly_cost,
            estimate.prediction_interval_low,
            estimate.prediction_interval_high,
            vec![
                CostComponent {
                    name: format!("{} (estimated)", change.resource_type),
                    cost: estimate.monthly_cost,
                    percentage: 100.0,
                },
            ],
        );
    }

    /// Add confidence reasoning
    fn add_confidence_reasoning(
        &self,
        builder: &mut ReasoningChainBuilder,
        estimate: &CostEstimate,
    ) {
        let mut factors = Vec::new();

        if estimate.heuristic_reference.is_some() {
            factors.push("Heuristics available".to_string());
        }
        if estimate.cold_start_inference {
            factors.push("Cold-start inference used".to_string());
        }

        builder.add_confidence_scoring(estimate.confidence_score_score, factors);
    }

    /// Add interval reasoning
    fn add_interval_reasoning(
        &self,
        builder: &mut ReasoningChainBuilder,
        estimate: &CostEstimate,
    ) {
        let range_factor = self.heuristics.prediction_intervals.range_factor;
        builder.add_interval_estimation(
            range_factor,
            estimate.prediction_interval_low,
            estimate.prediction_interval_high,
        );
    }

    /// Infer EC2 cost using cold-start model
    fn infer_ec2_cost(&self, instance_type: &str) -> f64 {
        // Simple inference based on instance family/size
        let parts: Vec<&str> = instance_type.split('.').collect();
        if parts.len() != 2 {
            return 50.0;
        }

        let size_multiplier = match parts[1] {
            "nano" => 0.5,
            "micro" => 1.0,
            "small" => 2.0,
            "medium" => 4.0,
            "large" => 8.0,
            "xlarge" => 16.0,
            "2xlarge" => 32.0,
            _ => 8.0,
        };

        7.6 * size_multiplier // Base on t3.micro
    }
}
