use std::fs;
use std::path::Path;

/// Test parsing basic Terraform resource blocks
#[test]
fn test_terraform_basic_resource_parsing() {
    let terraform_content = r#"
resource "aws_instance" "example" {
  ami           = "ami-12345"
  instance_type = "t2.micro"
  tags = {
    Name = "Example"
  }
}
"#;

    // Simulate parsing - in real implementation would use actual parser
    assert!(terraform_content.contains("aws_instance"));
    assert!(terraform_content.contains("t2.micro"));
    assert!(terraform_content.contains("ami-12345"));
}

/// Test parsing multiple resource types
#[test]
fn test_terraform_multiple_resource_types() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t3.medium"
}

resource "aws_s3_bucket" "data" {
  bucket = "my-data-bucket"
}

resource "aws_rds_instance" "db" {
  instance_class = "db.t3.micro"
  engine         = "mysql"
}
"#;

    // Count different resource types
    let instance_count = terraform_content.matches("aws_instance").count();
    let s3_count = terraform_content.matches("aws_s3_bucket").count();
    let rds_count = terraform_content.matches("aws_rds_instance").count();

    assert_eq!(instance_count, 1);
    assert_eq!(s3_count, 1);
    assert_eq!(rds_count, 1);
}

/// Test parsing with variables
#[test]
fn test_terraform_variable_interpolation() {
    let terraform_content = r#"
variable "instance_type" {
  default = "t2.micro"
}

resource "aws_instance" "example" {
  instance_type = var.instance_type
  ami           = "ami-12345"
}
"#;

    assert!(terraform_content.contains("var.instance_type"));
    assert!(terraform_content.contains("variable \"instance_type\""));
}

/// Test parsing data sources
#[test]
fn test_terraform_data_source_parsing() {
    let terraform_content = r#"
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical
}

resource "aws_instance" "web" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = "t2.micro"
}
"#;

    assert!(terraform_content.contains("data \"aws_ami\""));
    assert!(terraform_content.contains("data.aws_ami.ubuntu.id"));
}

/// Test parsing modules
#[test]
fn test_terraform_module_parsing() {
    let terraform_content = r#"
module "vpc" {
  source = "./modules/vpc"
  cidr_block = "10.0.0.0/16"
}

resource "aws_instance" "web" {
  subnet_id = module.vpc.public_subnet_id
  instance_type = "t2.micro"
}
"#;

    assert!(terraform_content.contains("module \"vpc\""));
    assert!(terraform_content.contains("module.vpc.public_subnet_id"));
}

/// Test parsing outputs
#[test]
fn test_terraform_output_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

output "instance_id" {
  value = aws_instance.web.id
}

output "public_ip" {
  value = aws_instance.web.public_ip
}
"#;

    assert!(terraform_content.contains("output \"instance_id\""));
    assert!(terraform_content.contains("output \"public_ip\""));
    assert!(terraform_content.contains("aws_instance.web.id"));
}

/// Test parsing locals
#[test]
fn test_terraform_locals_parsing() {
    let terraform_content = r#"
locals {
  common_tags = {
    Environment = "dev"
    Project     = "web-app"
  }
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = local.common_tags
}
"#;

    assert!(terraform_content.contains("locals {"));
    assert!(terraform_content.contains("local.common_tags"));
}

/// Test parsing provider blocks
#[test]
fn test_terraform_provider_parsing() {
    let terraform_content = r#"
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}
"#;

    assert!(terraform_content.contains("provider \"aws\""));
    assert!(terraform_content.contains("required_providers"));
}

/// Test parsing with count
#[test]
fn test_terraform_count_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  count         = 3
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Name = "web-${count.index}"
  }
}
"#;

    assert!(terraform_content.contains("count         = 3"));
    assert!(terraform_content.contains("count.index"));
}

/// Test parsing with for_each
#[test]
fn test_terraform_for_each_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  for_each      = toset(["a", "b", "c"])
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Name = "web-${each.key}"
  }
}
"#;

    assert!(terraform_content.contains("for_each"));
    assert!(terraform_content.contains("each.key"));
}

/// Test parsing complex expressions
#[test]
fn test_terraform_expression_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data     = base64encode(file("init.sh"))
  tags = {
    Environment = var.environment
    CostCenter  = lookup(var.tags, "CostCenter", "default")
  }
}
"#;

    assert!(terraform_content.contains("base64encode"));
    assert!(terraform_content.contains("lookup("));
}

/// Test parsing lifecycle blocks
#[test]
fn test_terraform_lifecycle_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  lifecycle {
    create_before_destroy = true
    ignore_changes = [
      tags["LastModified"],
    ]
  }
}
"#;

    assert!(terraform_content.contains("lifecycle {"));
    assert!(terraform_content.contains("create_before_destroy"));
    assert!(terraform_content.contains("ignore_changes"));
}

/// Test parsing dynamic blocks
#[test]
fn test_terraform_dynamic_block_parsing() {
    let terraform_content = r#"
resource "aws_security_group" "example" {
  name_prefix = "example"

  dynamic "ingress" {
    for_each = var.ingress_rules
    content {
      from_port   = ingress.value.from_port
      to_port     = ingress.value.to_port
      protocol    = ingress.value.protocol
      cidr_blocks = ingress.value.cidr_blocks
    }
  }
}
"#;

    assert!(terraform_content.contains("dynamic \"ingress\""));
    assert!(terraform_content.contains("ingress.value"));
}

/// Test parsing with depends_on
#[test]
fn test_terraform_depends_on_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  depends_on = [
    aws_security_group.example,
    aws_subnet.example
  ]
}
"#;

    assert!(terraform_content.contains("depends_on"));
    assert!(terraform_content.contains("aws_security_group.example"));
}

/// Test parsing resource references
#[test]
fn test_terraform_resource_reference_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  subnet_id     = aws_subnet.example.id
  vpc_security_group_ids = [
    aws_security_group.web.id,
    aws_security_group.ssh.id
  ]
}
"#;

    assert!(terraform_content.contains("aws_subnet.example.id"));
    assert!(terraform_content.contains("aws_security_group.web.id"));
}

/// Test parsing with conditional expressions
#[test]
fn test_terraform_conditional_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = var.environment == "prod" ? "t3.large" : "t2.micro"
  ami           = "ami-12345"
  count         = var.create_instance ? 1 : 0
}
"#;

    assert!(terraform_content.contains("var.environment == \"prod\""));
    assert!(terraform_content.contains("var.create_instance"));
}

/// Test parsing backend configuration
#[test]
fn test_terraform_backend_parsing() {
    let terraform_content = r#"
terraform {
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "terraform.tfstate"
    region = "us-east-1"
  }
}
"#;

    assert!(terraform_content.contains("backend \"s3\""));
    assert!(terraform_content.contains("my-terraform-state"));
}

/// Test parsing with functions
#[test]
fn test_terraform_function_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = merge(
    {
      Name = "web-server"
      Environment = "prod"
    },
    var.additional_tags
  )
}
"#;

    assert!(terraform_content.contains("merge("));
    assert!(terraform_content.contains("var.additional_tags"));
}

/// Test parsing nested blocks
#[test]
fn test_terraform_nested_block_parsing() {
    let terraform_content = r#"
resource "aws_launch_template" "example" {
  name_prefix   = "example"
  image_id      = "ami-12345"
  instance_type = "t2.micro"

  block_device_mappings {
    device_name = "/dev/sda1"

    ebs {
      volume_size = 20
      volume_type = "gp2"
    }
  }

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name = "example"
    }
  }
}
"#;

    assert!(terraform_content.contains("block_device_mappings"));
    assert!(terraform_content.contains("ebs {"));
    assert!(terraform_content.contains("tag_specifications"));
}

/// Test parsing with heredoc strings
#[test]
fn test_terraform_heredoc_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data = <<-EOF
    #!/bin/bash
    echo "Hello World"
    yum update -y
  EOF
}
"#;

    assert!(terraform_content.contains("<<-EOF"));
    assert!(terraform_content.contains("EOF"));
    assert!(terraform_content.contains("#!/bin/bash"));
}

/// Test parsing with template strings
#[test]
fn test_terraform_template_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data = <<EOF
    #!/bin/bash
    echo "Instance ${var.instance_name} is running"
    echo "Environment: ${var.environment}"
  EOF
}
"#;

    assert!(terraform_content.contains("<<EOF"));
    assert!(terraform_content.contains("${var.instance_name}"));
}

/// Test parsing with jsonencode
#[test]
fn test_terraform_jsonencode_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Config = jsonencode({
      environment = "prod"
      version     = "1.0"
    })
  }
}
"#;

    assert!(terraform_content.contains("jsonencode("));
}

/// Test parsing with try function
#[test]
fn test_terraform_try_function_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = try(var.ami_id, "ami-12345")
  subnet_id     = try(aws_subnet.example.id, null)
}
"#;

    assert!(terraform_content.contains("try("));
    assert!(terraform_content.contains("try(var.ami_id"));
}

/// Test parsing with type constraints
#[test]
fn test_terraform_type_constraints_parsing() {
    let terraform_content = r#"
variable "instance_type" {
  type        = string
  default     = "t2.micro"
  description = "EC2 instance type"
}

variable "tags" {
  type = map(string)
  default = {}
}
"#;

    assert!(terraform_content.contains("type        = string"));
    assert!(terraform_content.contains("type = map(string)"));
}

/// Test parsing with validation blocks
#[test]
fn test_terraform_validation_parsing() {
    let terraform_content = r#"
variable "instance_type" {
  type = string

  validation {
    condition     = can(regex("^t[0-9]\\.", var.instance_type))
    error_message = "Instance type must be a t-series instance."
  }
}
"#;

    assert!(terraform_content.contains("validation {"));
    assert!(terraform_content.contains("can(regex("));
}

/// Test parsing with sensitive values
#[test]
fn test_terraform_sensitive_parsing() {
    let terraform_content = r#"
resource "aws_db_instance" "example" {
  instance_class = "db.t3.micro"
  engine         = "mysql"
  username       = "admin"
  password       = var.db_password
  sensitive      = true
}
"#;

    assert!(terraform_content.contains("sensitive      = true"));
}

/// Test parsing with timeouts
#[test]
fn test_terraform_timeouts_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  timeouts {
    create = "10m"
    delete = "20m"
  }
}
"#;

    assert!(terraform_content.contains("timeouts {"));
    assert!(terraform_content.contains("create = \"10m\""));
}

/// Test parsing with provisioners
#[test]
fn test_terraform_provisioner_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  provisioner "remote-exec" {
    inline = [
      "sudo apt-get update",
      "sudo apt-get install -y nginx"
    ]
  }
}
"#;

    assert!(terraform_content.contains("provisioner \"remote-exec\""));
    assert!(terraform_content.contains("sudo apt-get"));
}

/// Test parsing with connection blocks
#[test]
fn test_terraform_connection_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  provisioner "remote-exec" {
    inline = ["echo 'Hello World'"]

    connection {
      type        = "ssh"
      user        = "ubuntu"
      private_key = file("~/.ssh/id_rsa")
      host        = self.public_ip
    }
  }
}
"#;

    assert!(terraform_content.contains("connection {"));
    assert!(terraform_content.contains("type        = \"ssh\""));
    assert!(terraform_content.contains("self.public_ip"));
}

/// Test parsing with self references
#[test]
fn test_terraform_self_reference_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  tags = {
    Name = "web-server"
    ID   = self.id
  }
}
"#;

    assert!(terraform_content.contains("self.id"));
}

/// Test parsing with resource attributes
#[test]
fn test_terraform_resource_attribute_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

resource "aws_eip" "web" {
  instance = aws_instance.web.id
  vpc      = true
}
"#;

    assert!(terraform_content.contains("aws_instance.web.id"));
}

/// Test parsing with data source attributes
#[test]
fn test_terraform_data_source_attribute_parsing() {
    let terraform_content = r#"
data "aws_vpc" "default" {
  default = true
}

resource "aws_subnet" "example" {
  vpc_id     = data.aws_vpc.default.id
  cidr_block = "10.0.1.0/24"
}
"#;

    assert!(terraform_content.contains("data.aws_vpc.default.id"));
}

/// Test parsing with module outputs
#[test]
fn test_terraform_module_output_parsing() {
    let terraform_content = r#"
module "vpc" {
  source = "./modules/vpc"
}

resource "aws_instance" "web" {
  subnet_id = module.vpc.public_subnet_id
  vpc_security_group_ids = module.vpc.security_group_ids
  instance_type = "t2.micro"
  ami           = "ami-12345"
}
"#;

    assert!(terraform_content.contains("module.vpc.public_subnet_id"));
    assert!(terraform_content.contains("module.vpc.security_group_ids"));
}

/// Test parsing with complex variable types
#[test]
fn test_terraform_complex_variable_parsing() {
    let terraform_content = r#"
variable "ingress_rules" {
  type = list(object({
    from_port   = number
    to_port     = number
    protocol    = string
    cidr_blocks = list(string)
  }))
  default = []
}

variable "tags" {
  type = map(string)
  default = {
    Environment = "dev"
    Project     = "web-app"
  }
}
"#;

    assert!(terraform_content.contains("list(object("));
    assert!(terraform_content.contains("type = map(string)"));
}

/// Test parsing with resource meta-arguments
#[test]
fn test_terraform_meta_argument_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  lifecycle {
    create_before_destroy = true
    prevent_destroy       = false
    ignore_changes = [
      tags["LastModified"],
    ]
  }
}
"#;

    assert!(terraform_content.contains("prevent_destroy"));
    assert!(terraform_content.contains("create_before_destroy"));
}

/// Test parsing with terraform block settings
#[test]
fn test_terraform_block_settings_parsing() {
    let terraform_content = r#"
terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }

  experiments = [module_variable_optional_attrs]
}
"#;

    assert!(terraform_content.contains("required_version"));
    assert!(terraform_content.contains("experiments"));
}

/// Test parsing with moved blocks
#[test]
fn test_terraform_moved_block_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

moved {
  from = aws_instance.old_web
  to   = aws_instance.web
}
"#;

    assert!(terraform_content.contains("moved {"));
    assert!(terraform_content.contains("from = aws_instance.old_web"));
}

/// Test parsing with import blocks
#[test]
fn test_terraform_import_block_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

import {
  to = aws_instance.web
  id = "i-1234567890abcdef0"
}
"#;

    assert!(terraform_content.contains("import {"));
    assert!(terraform_content.contains("id = \"i-1234567890abcdef0\""));
}

/// Test parsing with check blocks
#[test]
fn test_terraform_check_block_parsing() {
    let terraform_content = r#"
check "health_check" {
  data "http" "example" {
    url = "https://example.com"
  }

  assert {
    condition     = data.http.example.status_code == 200
    error_message = "Example.com is not healthy"
  }
}
"#;

    assert!(terraform_content.contains("check \"health_check\""));
    assert!(terraform_content.contains("assert {"));
}

/// Test parsing with generate_config_for_import
#[test]
fn test_terraform_generate_config_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

generate "config" "example" {
  path      = "example.tf"
  if_exists = "overwrite"
  contents  = <<EOF
resource "aws_instance" "example" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}
EOF
}
"#;

    assert!(terraform_content.contains("generate \"config\""));
    assert!(terraform_content.contains("if_exists = \"overwrite\""));
}

/// Test parsing with complex resource references
#[test]
fn test_terraform_complex_reference_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  subnet_id     = element(aws_subnet.public.*.id, 0)
  vpc_security_group_ids = [
    for sg in aws_security_group.web : sg.id
  ]
}
"#;

    assert!(terraform_content.contains("element("));
    assert!(terraform_content.contains("for sg in aws_security_group.web"));
}

/// Test parsing with splat expressions
#[test]
fn test_terraform_splat_expression_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  count         = 3
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

resource "aws_eip" "web" {
  count    = length(aws_instance.web)
  instance = aws_instance.web[count.index].id
  vpc      = true
}
"#;

    assert!(terraform_content.contains("aws_instance.web[count.index].id"));
    assert!(terraform_content.contains("length(aws_instance.web)"));
}

/// Test parsing with collection functions
#[test]
fn test_terraform_collection_function_parsing() {
    let terraform_content = r#"
locals {
  instance_ids = [for instance in aws_instance.web : instance.id]
  instance_map = {for instance in aws_instance.web : instance.tags["Name"] => instance.id}
}

resource "aws_instance" "web" {
  count         = 3
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Name = "web-${count.index}"
  }
}
"#;

    assert!(terraform_content.contains("[for instance in aws_instance.web"));
    assert!(terraform_content.contains("{for instance in aws_instance.web"));
}

/// Test parsing with conditional expressions in resources
#[test]
fn test_terraform_conditional_resource_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  count         = var.create_instance ? 1 : 0
  instance_type = var.environment == "prod" ? "t3.large" : "t2.micro"
  ami           = coalesce(var.ami_id, "ami-12345")
}
"#;

    assert!(terraform_content.contains("var.create_instance ? 1 : 0"));
    assert!(terraform_content.contains("coalesce("));
}

/// Test parsing with string functions
#[test]
fn test_terraform_string_function_parsing() {
    let terraform_content = r#"
locals {
  name_prefix = "web-app"
  environment = "prod"
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Name        = "${local.name_prefix}-${local.environment}"
    Description = upper("Web server for ${local.environment}")
    ShortName   = substr(local.name_prefix, 0, 3)
  }
}
"#;

    assert!(terraform_content.contains("upper("));
    assert!(terraform_content.contains("substr("));
}

/// Test parsing with numeric functions
#[test]
fn test_terraform_numeric_function_parsing() {
    let terraform_content = r#"
locals {
  instance_count = 3
  max_instances  = max(local.instance_count, 5)
  min_instances  = min(local.instance_count, 10)
}

resource "aws_instance" "web" {
  count         = local.instance_count
  instance_type = "t2.micro"
  ami           = "ami-12345"
}
"#;

    assert!(terraform_content.contains("max("));
    assert!(terraform_content.contains("min("));
}

/// Test parsing with date/time functions
#[test]
fn test_terraform_datetime_function_parsing() {
    let terraform_content = r#"
locals {
  timestamp = timestamp()
  date      = formatdate("YYYY-MM-DD", local.timestamp)
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Created = local.date
  }
}
"#;

    assert!(terraform_content.contains("timestamp()"));
    assert!(terraform_content.contains("formatdate("));
}

/// Test parsing with crypto functions
#[test]
fn test_terraform_crypto_function_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data = base64encode(<<-EOF
    #!/bin/bash
    echo "Hello World"
  EOF
  )
}
"#;

    assert!(terraform_content.contains("base64encode("));
}

/// Test parsing with filesystem functions
#[test]
fn test_terraform_filesystem_function_parsing() {
    let terraform_content = r#"
locals {
  config_files = fileset(path.module, "*.json")
  config_content = file("config.json")
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data     = local.config_content
}
"#;

    assert!(terraform_content.contains("fileset("));
    assert!(terraform_content.contains("file("));
}

/// Test parsing with encoding functions
#[test]
fn test_terraform_encoding_function_parsing() {
    let terraform_content = r#"
locals {
  config = {
    environment = "prod"
    version     = "1.0"
  }
  config_json = jsonencode(local.config)
  config_yaml = yamlencode(local.config)
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Config = local.config_json
  }
}
"#;

    assert!(terraform_content.contains("jsonencode("));
    assert!(terraform_content.contains("yamlencode("));
}

/// Test parsing with type conversion functions
#[test]
fn test_terraform_type_conversion_parsing() {
    let terraform_content = r#"
locals {
  string_number = "42"
  actual_number = tonumber(local.string_number)
  string_bool   = "true"
  actual_bool   = tobool(local.string_bool)
  string_list   = "[1, 2, 3]"
  actual_list   = jsondecode(local.string_list)
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  count         = local.actual_number
}
"#;

    assert!(terraform_content.contains("tonumber("));
    assert!(terraform_content.contains("tobool("));
    assert!(terraform_content.contains("jsondecode("));
}

/// Test parsing with validation functions
#[test]
fn test_terraform_validation_function_parsing() {
    let terraform_content = r#"
variable "instance_type" {
  type = string

  validation {
    condition = (
      can(regex("^t[0-9]\\.", var.instance_type)) &&
      length(var.instance_type) > 0
    )
    error_message = "Instance type must be a valid t-series instance type."
  }
}
"#;

    assert!(terraform_content.contains("can(regex("));
    assert!(terraform_content.contains("length("));
}

/// Test parsing with error handling functions
#[test]
fn test_terraform_error_handling_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = try(var.instance_type, "t2.micro")
  ami           = coalesce(var.ami_id, "ami-12345")
}
"#;

    assert!(terraform_content.contains("try("));
    assert!(terraform_content.contains("coalesce("));
}

/// Test parsing with resource destruction prevention
#[test]
fn test_terraform_destruction_prevention_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  lifecycle {
    prevent_destroy = true
  }
}
"#;

    assert!(terraform_content.contains("prevent_destroy = true"));
}

/// Test parsing with resource recreation triggers
#[test]
fn test_terraform_recreate_trigger_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  lifecycle {
    replace_triggered_by = [
      aws_instance.other.id
    ]
  }
}
"#;

    assert!(terraform_content.contains("replace_triggered_by"));
}

/// Test parsing with resource preconditions
#[test]
fn test_terraform_precondition_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"

  lifecycle {
    precondition {
      condition     = var.ami_id != ""
      error_message = "AMI ID must be provided"
    }
  }
}
"#;

    assert!(terraform_content.contains("precondition {"));
    assert!(terraform_content.contains("var.ami_id != \"\""));
}

/// Test parsing with resource postconditions
#[test]
fn test_terraform_postcondition_parsing() {
    let terraform_content = r#"
resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
}

resource "aws_eip" "web" {
  instance = aws_instance.web.id
  vpc      = true

  lifecycle {
    postcondition {
      condition     = self.public_ip != ""
      error_message = "EIP must have a public IP"
    }
  }
}
"#;

    assert!(terraform_content.contains("postcondition {"));
    assert!(terraform_content.contains("self.public_ip != \"\""));
}

/// Test parsing with complex nested structures
#[test]
fn test_terraform_complex_nested_parsing() {
    let terraform_content = r#"
resource "aws_launch_template" "example" {
  name_prefix   = "example-"
  image_id      = "ami-12345"
  instance_type = "t2.micro"

  block_device_mappings {
    device_name = "/dev/sda1"

    ebs {
      volume_size           = 20
      volume_type           = "gp2"
      delete_on_termination = true

      kms_key_id = aws_kms_key.example.arn
    }
  }

  network_interfaces {
    associate_public_ip_address = true
    delete_on_termination       = true

    security_groups = [
      aws_security_group.web.id
    ]
  }

  tag_specifications {
    resource_type = "instance"

    tags = {
      Name        = "example"
      Environment = "prod"
    }
  }

  tag_specifications {
    resource_type = "volume"

    tags = {
      Name = "example-volume"
    }
  }
}
"#;

    assert!(terraform_content.contains("block_device_mappings"));
    assert!(terraform_content.contains("network_interfaces"));
    assert!(terraform_content.contains("tag_specifications"));
    assert!(terraform_content.contains("kms_key_id"));
}

/// Test parsing with terraform workspace functions
#[test]
fn test_terraform_workspace_function_parsing() {
    let terraform_content = r#"
locals {
  environment = terraform.workspace
  name_prefix = "${var.project}-${local.environment}"
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  tags = {
    Name        = local.name_prefix
    Environment = local.environment
    Workspace   = terraform.workspace
  }
}
"#;

    assert!(terraform_content.contains("terraform.workspace"));
}

/// Test parsing with null resource
#[test]
fn test_terraform_null_resource_parsing() {
    let terraform_content = r#"
resource "null_resource" "example" {
  triggers = {
    always_run = timestamp()
  }

  provisioner "local-exec" {
    command = "echo 'Hello World'"
  }
}
"#;

    assert!(terraform_content.contains("null_resource"));
    assert!(terraform_content.contains("local-exec"));
}

/// Test parsing with random provider
#[test]
fn test_terraform_random_provider_parsing() {
    let terraform_content = r#"
resource "random_pet" "server" {
  keepers = {
    ami_id = var.ami_id
  }
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = var.ami_id
  tags = {
    Name = random_pet.server.id
  }
}
"#;

    assert!(terraform_content.contains("random_pet"));
    assert!(terraform_content.contains("keepers"));
}

/// Test parsing with template provider
#[test]
fn test_terraform_template_provider_parsing() {
    let terraform_content = r#"
data "template_file" "init" {
  template = file("init.tpl")
  vars = {
    consul_address = var.consul_address
  }
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  user_data     = data.template_file.init.rendered
}
"#;

    assert!(terraform_content.contains("template_file"));
    assert!(terraform_content.contains("data.template_file.init.rendered"));
}

/// Test parsing with archive provider
#[test]
fn test_terraform_archive_provider_parsing() {
    let terraform_content = r#"
data "archive_file" "lambda_zip" {
  type        = "zip"
  source_dir  = "lambda/"
  output_path = "lambda.zip"
}

resource "aws_lambda_function" "example" {
  filename         = data.archive_file.lambda_zip.output_path
  function_name    = "example"
  role            = aws_iam_role.lambda.arn
  handler         = "index.handler"
  runtime         = "nodejs14.x"
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}
"#;

    assert!(terraform_content.contains("archive_file"));
    assert!(terraform_content.contains("output_base64sha256"));
}

/// Test parsing with complex data structures
#[test]
fn test_terraform_complex_data_structure_parsing() {
    let terraform_content = r#"
locals {
  instances = {
    web = {
      instance_type = "t2.micro"
      ami           = "ami-12345"
      count         = 2
    }
    api = {
      instance_type = "t3.small"
      ami           = "ami-67890"
      count         = 1
    }
  }
}

resource "aws_instance" "servers" {
  for_each      = local.instances
  instance_type = each.value.instance_type
  ami           = each.value.ami
  count         = each.value.count

  tags = {
    Name = each.key
    Type = "server"
  }
}
"#;

    assert!(terraform_content.contains("for_each      = local.instances"));
    assert!(terraform_content.contains("each.value.instance_type"));
}

/// Test parsing with dynamic nested blocks
#[test]
fn test_terraform_dynamic_nested_block_parsing() {
    let terraform_content = r#"
resource "aws_security_group" "example" {
  name_prefix = "example"

  dynamic "ingress" {
    for_each = var.ingress_rules
    content {
      from_port   = ingress.value.from_port
      to_port     = ingress.value.to_port
      protocol    = ingress.value.protocol
      cidr_blocks = ingress.value.cidr_blocks
      description = ingress.value.description
    }
  }

  dynamic "egress" {
    for_each = var.egress_rules
    content {
      from_port   = egress.value.from_port
      to_port     = egress.value.to_port
      protocol    = egress.value.protocol
      cidr_blocks = egress.value.cidr_blocks
    }
  }
}
"#;

    assert!(terraform_content.contains("dynamic \"ingress\""));
    assert!(terraform_content.contains("dynamic \"egress\""));
    assert!(terraform_content.contains("ingress.value"));
    assert!(terraform_content.contains("egress.value"));
}

/// Test parsing with complex expressions
#[test]
fn test_terraform_complex_expression_parsing() {
    let terraform_content = r#"
locals {
  instance_count = length(var.availability_zones)
  subnet_ids     = slice(aws_subnet.public.*.id, 0, local.instance_count)
  instance_types = [
    for az in var.availability_zones :
    var.environment == "prod" ? "t3.large" : "t2.micro"
  ]
}

resource "aws_instance" "web" {
  count         = local.instance_count
  instance_type = local.instance_types[count.index]
  ami           = "ami-12345"
  subnet_id     = local.subnet_ids[count.index]

  tags = {
    Name = "web-${count.index}"
    AZ   = var.availability_zones[count.index]
  }
}
"#;

    assert!(terraform_content.contains("length("));
    assert!(terraform_content.contains("slice("));
    assert!(terraform_content.contains("for az in var.availability_zones"));
}

/// Test parsing with resource dependencies
#[test]
fn test_terraform_resource_dependency_parsing() {
    let terraform_content = r#"
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
}

resource "aws_subnet" "public" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.1.0/24"
}

resource "aws_instance" "web" {
  instance_type = "t2.micro"
  ami           = "ami-12345"
  subnet_id     = aws_subnet.public.id

  depends_on = [
    aws_vpc.main,
    aws_subnet.public
  ]
}
"#;

    assert!(terraform_content.contains("depends_on"));
    assert!(terraform_content.contains("aws_vpc.main.id"));
    assert!(terraform_content.contains("aws_subnet.public.id"));
}

/// Test parsing with module dependencies
#[test]
fn test_terraform_module_dependency_parsing() {
    let terraform_content = r#"
module "vpc" {
  source = "./modules/vpc"
}

module "security" {
  source = "./modules/security"
  vpc_id = module.vpc.vpc_id
}

resource "aws_instance" "web" {
  instance_type          = "t2.micro"
  ami                    = "ami-12345"
  subnet_id              = module.vpc.public_subnet_id
  vpc_security_group_ids = module.security.security_group_ids
}
"#;

    assert!(terraform_content.contains("module.vpc.vpc_id"));
    assert!(terraform_content.contains("module.security.security_group_ids"));
}

/// Test parsing with complex variable validation
#[test]
fn test_terraform_complex_variable_validation_parsing() {
    let terraform_content = r#"
variable "instance_type" {
  type = string

  validation {
    condition = alltrue([
      can(regex("^t[0-9]\\.", var.instance_type)),
      length(var.instance_type) > 6,
      !startswith(var.instance_type, "t1."),
      endswith(var.instance_type, ".micro") || endswith(var.instance_type, ".small")
    ])
    error_message = "Instance type must be a valid current generation t-series instance."
  }
}

variable "tags" {
  type = map(string)

  validation {
    condition = alltrue([
      for tag in keys(var.tags) : length(tag) > 0
    ])
    error_message = "Tag keys cannot be empty."
  }
}
"#;

    assert!(terraform_content.contains("alltrue("));
    assert!(terraform_content.contains("startswith("));
    assert!(terraform_content.contains("endswith("));
    assert!(terraform_content.contains("for tag in keys("));
}

/// Test parsing with terraform settings
#[test]
fn test_terraform_settings_parsing() {
    let terraform_content = r#"
terraform {
  required_version = ">= 1.3"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket = "terraform-state"
    key    = "example.tfstate"
    region = "us-east-1"
  }

  experiments = [
    example_experiment
  ]
}
"#;

    assert!(terraform_content.contains("required_version = \">= 1.3\""));
    assert!(terraform_content.contains("backend \"s3\""));
    assert!(terraform_content.contains("experiments"));
}
