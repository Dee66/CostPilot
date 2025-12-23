#!/usr/bin/env python3
"""
CostPilot Synthetic Data Generator

Generates anonymized production-mirroring datasets for comprehensive testing.
Creates realistic Terraform plans and CloudFormation templates that represent
typical production infrastructure patterns.
"""

import json
import random
import uuid
from typing import Dict, Any, List
import argparse

class ProductionDataGenerator:
    def __init__(self, seed: int = 42):
        random.seed(seed)
        self.regions = [
            "us-east-1", "us-west-2", "eu-west-1", "eu-central-1",
            "ap-southeast-1", "ap-northeast-1"
        ]
        self.instance_types = {
            "web": ["t3.medium", "t3.large", "m5.large", "m5.xlarge", "c5.large"],
            "db": ["db.t3.medium", "db.t3.large", "db.r5.large", "db.r5.xlarge"],
            "cache": ["cache.t3.medium", "cache.t3.large", "cache.r5.large"],
            "compute": ["c5.large", "c5.xlarge", "m5.large", "m5.xlarge"]
        }
        self.industries = ["ecommerce", "financial", "healthcare", "gaming", "media"]

    def generate_anonymized_id(self) -> str:
        """Generate an anonymized identifier."""
        return str(uuid.uuid4())[:8]

    def generate_terraform_plan(self, industry: str, scale: str) -> Dict[str, Any]:
        """Generate a synthetic Terraform plan."""

        plan = {
            "version": "1.0",
            "terraform_version": "1.5.0",
            "planned_values": {
                "root_module": {
                    "resources": []
                }
            },
            "resource_changes": [],
            "configuration": {
                "root_module": {
                    "resources": []
                }
            }
        }

        # Generate resources based on industry and scale
        resources = self._generate_resources_for_industry(industry, scale)

        for resource in resources:
            plan["planned_values"]["root_module"]["resources"].append(resource)
            plan["resource_changes"].append(self._create_resource_change(resource))
            plan["configuration"]["root_module"]["resources"].append(
                self._create_resource_config(resource)
            )

        return plan

    def _generate_resources_for_industry(self, industry: str, scale: str) -> List[Dict[str, Any]]:
        """Generate resources appropriate for the industry and scale."""
        resources = []

        if scale == "small":
            num_instances = random.randint(1, 3)
            num_db = 1
        elif scale == "medium":
            num_instances = random.randint(3, 8)
            num_db = random.randint(1, 2)
        else:  # large/enterprise
            num_instances = random.randint(8, 20)
            num_db = random.randint(2, 5)

        # Generate EC2 instances
        for i in range(num_instances):
            resources.append(self._generate_ec2_instance(i, industry))

        # Generate RDS instances
        for i in range(num_db):
            resources.append(self._generate_rds_instance(i))

        # Generate S3 buckets
        resources.append(self._generate_s3_bucket())

        # Industry-specific resources
        if industry == "ecommerce":
            resources.append(self._generate_cloudfront_distribution())
            resources.append(self._generate_elasticache_cluster())
        elif industry == "financial":
            resources.append(self._generate_lambda_function())
            resources.append(self._generate_api_gateway())
        elif industry == "healthcare":
            resources.append(self._generate_emr_cluster())
            resources.append(self._generate_redshift_cluster())

        return resources

    def _generate_ec2_instance(self, index: int, industry: str) -> Dict[str, Any]:
        """Generate an EC2 instance resource."""
        instance_type = random.choice(self.instance_types["web"])
        ami = f"ami-{self.generate_anonymized_id()}"

        return {
            "address": f"aws_instance.{industry}_app[{index}]",
            "mode": "managed",
            "type": "aws_instance",
            "name": f"{industry}_app",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "instance_type": instance_type,
                "ami": ami,
                "tags": {
                    "Name": f"{industry}-app-prod-{index:03d}",
                    "Environment": "production",
                    "Application": f"{industry}-platform"
                }
            }
        }

    def _generate_rds_instance(self, index: int) -> Dict[str, Any]:
        """Generate an RDS instance resource."""
        instance_class = random.choice(self.instance_types["db"])
        db_name = f"prod_db_{index}"

        return {
            "address": f"aws_db_instance.database[{index}]",
            "mode": "managed",
            "type": "aws_db_instance",
            "name": "database",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "instance_class": instance_class,
                "engine": "mysql",
                "engine_version": "8.0",
                "database_name": db_name,
                "username": "admin",
                "password": "***",
                "allocated_storage": 100,
                "backup_retention_period": 7
            }
        }

    def _generate_s3_bucket(self) -> Dict[str, Any]:
        """Generate an S3 bucket resource."""
        bucket_name = f"prod-{self.generate_anonymized_id()}-assets"

        return {
            "address": "aws_s3_bucket.assets",
            "mode": "managed",
            "type": "aws_s3_bucket",
            "name": "assets",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "bucket": bucket_name,
                "acl": "private",
                "versioning": {
                    "enabled": True
                }
            }
        }

    def _generate_cloudfront_distribution(self) -> Dict[str, Any]:
        """Generate a CloudFront distribution."""
        return {
            "address": "aws_cloudfront_distribution.cdn",
            "mode": "managed",
            "type": "aws_cloudfront_distribution",
            "name": "cdn",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "enabled": True,
                "default_cache_behavior": {
                    "target_origin_id": "assets-origin",
                    "viewer_protocol_policy": "redirect-to-https",
                    "allowed_methods": ["GET", "HEAD", "OPTIONS"],
                    "cached_methods": ["GET", "HEAD"]
                }
            }
        }

    def _generate_elasticache_cluster(self) -> Dict[str, Any]:
        """Generate an ElastiCache cluster."""
        node_type = random.choice(self.instance_types["cache"])

        return {
            "address": "aws_elasticache_cluster.redis",
            "mode": "managed",
            "type": "aws_elasticache_cluster",
            "name": "redis",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "cluster_id": f"prod-redis-{self.generate_anonymized_id()}",
                "engine": "redis",
                "node_type": node_type,
                "num_cache_nodes": random.randint(1, 3)
            }
        }

    def _generate_lambda_function(self) -> Dict[str, Any]:
        """Generate a Lambda function."""
        return {
            "address": "aws_lambda_function.processor",
            "mode": "managed",
            "type": "aws_lambda_function",
            "name": "processor",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "function_name": f"prod-processor-{self.generate_anonymized_id()}",
                "runtime": "python3.9",
                "handler": "lambda_function.lambda_handler",
                "memory_size": random.choice([512, 1024, 2048]),
                "timeout": random.randint(30, 300)
            }
        }

    def _generate_api_gateway(self) -> Dict[str, Any]:
        """Generate an API Gateway."""
        return {
            "address": "aws_api_gateway_rest_api.api",
            "mode": "managed",
            "type": "aws_api_gateway_rest_api",
            "name": "api",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "name": f"prod-api-{self.generate_anonymized_id()}",
                "description": "Production API Gateway",
                "endpoint_configuration": {
                    "types": ["REGIONAL"]
                }
            }
        }

    def _generate_emr_cluster(self) -> Dict[str, Any]:
        """Generate an EMR cluster."""
        return {
            "address": "aws_emr_cluster.analytics",
            "mode": "managed",
            "type": "aws_emr_cluster",
            "name": "analytics",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "name": f"prod-analytics-{self.generate_anonymized_id()}",
                "release_label": "emr-6.4.0",
                "applications": ["Spark", "Hadoop"],
                "master_instance_type": "m5.xlarge",
                "core_instance_type": "m5.large",
                "core_instance_count": random.randint(2, 6)
            }
        }

    def _generate_redshift_cluster(self) -> Dict[str, Any]:
        """Generate a Redshift cluster."""
        return {
            "address": "aws_redshift_cluster.data_warehouse",
            "mode": "managed",
            "type": "aws_redshift_cluster",
            "name": "data_warehouse",
            "provider_name": "registry.terraform.io/hashicorp/aws",
            "schema_version": 1,
            "values": {
                "cluster_identifier": f"prod-dw-{self.generate_anonymized_id()}",
                "database_name": "prod_dw",
                "master_username": "admin",
                "master_password": "***",
                "node_type": "dc2.large",
                "number_of_nodes": random.randint(2, 8)
            }
        }

    def _create_resource_change(self, resource: Dict[str, Any]) -> Dict[str, Any]:
        """Create a resource change entry."""
        return {
            "address": resource["address"],
            "change": {
                "actions": ["create"],
                "before": None,
                "after": resource["values"]
            }
        }

    def _create_resource_config(self, resource: Dict[str, Any]) -> Dict[str, Any]:
        """Create a resource configuration entry."""
        config = {
            "address": resource["address"],
            "mode": "managed",
            "type": resource["type"],
            "name": resource["name"],
            "expressions": {}
        }

        # Convert values to expressions (simplified)
        for key, value in resource["values"].items():
            if isinstance(value, str):
                config["expressions"][key] = {"constant_value": value}
            else:
                config["expressions"][key] = {"constant_value": value}

        return config

    def generate_cloudformation_template(self, industry: str, scale: str) -> Dict[str, Any]:
        """Generate a synthetic CloudFormation template."""
        template = {
            "AWSTemplateFormatVersion": "2010-09-09",
            "Description": f"Synthetic {industry} {scale} infrastructure template",
            "Resources": {}
        }

        # Generate resources based on industry and scale
        resources = self._generate_cf_resources_for_industry(industry)

        for resource_name, resource_def in resources.items():
            template["Resources"][resource_name] = resource_def

        return template

    def _generate_cf_resources_for_industry(self, industry: str) -> Dict[str, Any]:
        """Generate CloudFormation resources."""
        resources = {}

        # Basic EC2 setup
        resources["EC2Instance"] = {
            "Type": "AWS::EC2::Instance",
            "Properties": {
                "ImageId": f"ami-{self.generate_anonymized_id()}",
                "InstanceType": random.choice(self.instance_types["web"])
            }
        }

        # S3 bucket
        resources["S3Bucket"] = {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BucketName": f"prod-{industry}-{self.generate_anonymized_id()}"
            }
        }

        # Industry-specific resources
        if industry == "ecommerce":
            resources["CloudFrontDistribution"] = {
                "Type": "AWS::CloudFront::Distribution",
                "Properties": {
                    "DistributionConfig": {
                        "Enabled": True,
                        "DefaultCacheBehavior": {
                            "TargetOriginId": "S3Origin",
                            "ViewerProtocolPolicy": "redirect-to-https"
                        }
                    }
                }
            }
        elif industry == "financial":
            resources["LambdaFunction"] = {
                "Type": "AWS::Lambda::Function",
                "Properties": {
                    "FunctionName": f"prod-{industry}-processor",
                    "Runtime": "python3.9",
                    "Handler": "lambda_function.lambda_handler",
                    "Code": {"ZipFile": "def lambda_handler(event, context): return 'hello'"}
                }
            }

        return resources

def main():
    parser = argparse.ArgumentParser(description="Generate synthetic production-mirroring datasets")
    parser.add_argument("--industry", choices=["ecommerce", "financial", "healthcare", "gaming", "media"],
                       default="ecommerce", help="Industry type")
    parser.add_argument("--scale", choices=["small", "medium", "large"],
                       default="medium", help="Infrastructure scale")
    parser.add_argument("--format", choices=["terraform", "cloudformation"], default="terraform",
                       help="Output format")
    parser.add_argument("--count", type=int, default=1, help="Number of datasets to generate")
    parser.add_argument("--output-dir", default="tests/test_data/production_mirroring/generated",
                       help="Output directory")

    args = parser.parse_args()

    generator = ProductionDataGenerator()

    import os
    os.makedirs(args.output_dir, exist_ok=True)

    for i in range(args.count):
        if args.format == "terraform":
            dataset = generator.generate_terraform_plan(args.industry, args.scale)
            filename = f"{args.industry}_{args.scale}_terraform_{i+1}.json"
            with open(os.path.join(args.output_dir, filename), 'w', encoding='utf-8') as f:
                json.dump(dataset, f, indent=2)
        else:
            dataset = generator.generate_cloudformation_template(args.industry, args.scale)
            filename = f"{args.industry}_{args.scale}_cloudformation_{i+1}.json"
            with open(os.path.join(args.output_dir, filename), 'w', encoding='utf-8') as f:
                json.dump(dataset, f, indent=2)

        print(f"Generated {filename}")

if __name__ == "__main__":
    main()
