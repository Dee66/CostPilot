# Security Violations Terraform Scenario
# This scenario includes resources with security violations for CostPilot security testing

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

# 1. Security Group with overly permissive rules (security violation)
resource "aws_security_group" "insecure_sg" {
  name_prefix = "insecure-sg-"

  # Allows all inbound traffic - major security violation
  ingress {
    from_port   = 0
    to_port     = 65535
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Allows all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "Insecure-Security-Group"
  }
}

# 2. EC2 instance with insecure SSH access (security violation)
resource "aws_instance" "insecure_ssh" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  # Security group allows SSH from anywhere
  vpc_security_group_ids = [aws_security_group.ssh_anywhere.id]

  tags = {
    Name = "Insecure-SSH-Access"
  }
}

resource "aws_security_group" "ssh_anywhere" {
  name_prefix = "ssh-anywhere-"

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]  # SECURITY VIOLATION: SSH from anywhere
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# 3. RDS instance with weak password (security violation)
resource "aws_db_instance" "weak_password_db" {
  identifier             = "weak-password-db"
  allocated_storage      = 20
  storage_type           = "gp2"
  engine                 = "mysql"
  engine_version         = "8.0"
  instance_class         = "db.t3.micro"
  db_name                = "weakdb"
  username               = "admin"
  password               = "password123"  # SECURITY VIOLATION: Weak password
  parameter_group_name   = "default.mysql8.0"
  skip_final_snapshot    = true

  # No encryption - security violation
  storage_encrypted = false

  tags = {
    Name = "Weak-Password-DB"
  }
}

# 4. S3 bucket without encryption (security violation)
resource "aws_s3_bucket" "unencrypted_bucket" {
  bucket = "unencrypted-bucket-${random_string.bucket_suffix.result}"

  # No server-side encryption - security violation
  tags = {
    Name = "Unencrypted-Bucket"
  }
}

resource "aws_s3_bucket_versioning" "unencrypted_versioning" {
  bucket = aws_s3_bucket.unencrypted_bucket.id
  versioning_configuration {
    status = "Enabled"
  }
}

# 5. S3 bucket with public read access (security violation)
resource "aws_s3_bucket" "public_bucket" {
  bucket = "public-bucket-${random_string.bucket_suffix.result}"

  tags = {
    Name = "Public-Bucket"
  }
}

resource "aws_s3_bucket_public_access_block" "public_access" {
  bucket = aws_s3_bucket.public_bucket.id

  # SECURITY VIOLATION: Allows public access
  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

resource "aws_s3_bucket_policy" "public_read" {
  bucket = aws_s3_bucket.public_bucket.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "PublicReadGetObject"
        Effect    = "Allow"
        Principal = "*"
        Action    = "s3:GetObject"
        Resource  = "${aws_s3_bucket.public_bucket.arn}/*"
      }
    ]
  })
}

# 6. IAM user with overly broad permissions (security violation)
resource "aws_iam_user" "over_privileged_user" {
  name = "over-privileged-user"

  tags = {
    Name = "Over-Privileged-User"
  }
}

resource "aws_iam_user_policy" "admin_policy" {
  name = "admin-policy"
  user = aws_iam_user.over_privileged_user.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = "*"  # SECURITY VIOLATION: All actions allowed
        Resource = "*"
      }
    ]
  })
}

# 7. CloudTrail not enabled (security violation)
# Note: No aws_cloudtrail resource - represents missing security logging

# 8. VPC without flow logs (security violation)
resource "aws_vpc" "no_flow_logs" {
  cidr_block = "10.1.0.0/16"

  tags = {
    Name = "VPC-No-Flow-Logs"
  }
}

# No aws_flow_log resource - security violation

# 9. Load balancer without HTTPS (security violation)
resource "aws_lb" "http_only_lb" {
  name               = "http-only-lb"
  internal           = false
  load_balancer_type = "application"
  subnets            = ["subnet-12345678", "subnet-87654321"]  # Dummy subnets

  tags = {
    Name = "HTTP-Only-LB"
  }
}

resource "aws_lb_listener" "http_only" {
  load_balancer_arn = aws_lb.http_only_lb.arn
  port              = "80"
  protocol          = "HTTP"  # SECURITY VIOLATION: No HTTPS

  default_action {
    type = "fixed-response"

    fixed_response {
      content_type = "text/plain"
      message_body = "Hello World"
      status_code  = "200"
    }
  }
}

# 10. EBS volumes not encrypted (security violation)
resource "aws_ebs_volume" "unencrypted_ebs" {
  availability_zone = "us-east-1a"
  size              = 50
  type              = "gp2"
  encrypted         = false  # SECURITY VIOLATION: Not encrypted

  tags = {
    Name = "Unencrypted-EBS"
  }
}

# 11. Lambda function with excessive permissions (security violation)
resource "aws_lambda_function" "excessive_permissions_lambda" {
  function_name = "excessive-permissions-function"
  runtime       = "python3.9"
  handler       = "lambda_function.lambda_handler"
  memory_size   = 128
  timeout       = 30

  filename         = "dummy.zip"
  source_code_hash = filebase64sha256("dummy.zip")

  role = aws_iam_role.excessive_lambda_role.arn

  tags = {
    Name = "Excessive-Permissions-Lambda"
  }
}

resource "aws_iam_role" "excessive_lambda_role" {
  name = "excessive-lambda-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "excessive_lambda_policy" {
  name = "excessive-lambda-policy"
  role = aws_iam_role.excessive_lambda_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "*"  # SECURITY VIOLATION: All actions allowed
        ]
        Resource = "*"
      }
    ]
  })
}

# 12. API Gateway without authentication (security violation)
resource "aws_api_gateway_rest_api" "unauthenticated_api" {
  name        = "unauthenticated-api"
  description = "API without authentication"

  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

resource "aws_api_gateway_resource" "api_resource" {
  rest_api_id = aws_api_gateway_rest_api.unauthenticated_api.id
  parent_id   = aws_api_gateway_rest_api.unauthenticated_api.root_resource_id
  path_part   = "test"
}

resource "aws_api_gateway_method" "api_method" {
  rest_api_id   = aws_api_gateway_rest_api.unauthenticated_api.id
  resource_id   = aws_api_gateway_resource.api_resource.id
  http_method   = "GET"
  authorization = "NONE"  # SECURITY VIOLATION: No authentication
}

resource "aws_api_gateway_integration" "api_integration" {
  rest_api_id = aws_api_gateway_rest_api.unauthenticated_api.id
  resource_id = aws_api_gateway_resource.api_resource.id
  http_method = aws_api_gateway_method.api_method.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.excessive_permissions_lambda.invoke_arn
}

resource "aws_api_gateway_deployment" "api_deployment" {
  depends_on = [aws_api_gateway_integration.api_integration]

  rest_api_id = aws_api_gateway_rest_api.unauthenticated_api.id
  stage_name  = "prod"
}

# 13. CloudWatch logs not encrypted (security violation)
resource "aws_cloudwatch_log_group" "unencrypted_logs" {
  name              = "/aws/lambda/unencrypted-function"
  retention_in_days = 30
  # kms_key_id not specified - security violation
}

# 14. Secrets in plain text (security violation)
resource "aws_instance" "secrets_in_userdata" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  user_data = base64encode(<<-EOF
              #!/bin/bash
              # SECURITY VIOLATION: Secrets in user data
              export DATABASE_PASSWORD="SuperSecretPass123!"
              export API_KEY="sk-1234567890abcdef"
              echo "Database password: $DATABASE_PASSWORD" > /tmp/secrets.txt
              EOF
  )

  tags = {
    Name = "Secrets-in-UserData"
  }
}

# 15. Default security group used (security violation)
resource "aws_instance" "default_sg_instance" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  # Using default security group - security violation
  # (In real scenarios, default SG allows all traffic within VPC)

  tags = {
    Name = "Default-SG-Instance"
  }
}

# Random suffix for unique bucket names
resource "random_string" "bucket_suffix" {
  length  = 8
  lower   = true
  upper   = false
  numeric = true
  special = false
}

# Dummy file for Lambda
resource "local_file" "dummy_lambda" {
  content  = "dummy lambda code"
  filename = "dummy.zip"
}
