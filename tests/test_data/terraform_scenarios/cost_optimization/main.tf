# Cost Optimization Opportunities Terraform Scenario
# This scenario includes resources with clear cost-saving opportunities for CostPilot testing

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

# 1. Over-provisioned EC2 instances (cost optimization opportunity)
resource "aws_instance" "over_provisioned_server" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "m5.8xlarge"  # Very large instance, underutilized

  tags = {
    Name        = "OverProvisioned-Server"
    Environment = "development"
    Project     = "CostPilot-Test"
  }

  # Minimal CPU credits for burstable instance
  credit_specification {
    cpu_credits = "standard"
  }
}

# 2. Idle EBS volumes (cost optimization opportunity)
resource "aws_ebs_volume" "idle_volume" {
  availability_zone = "us-east-1a"
  size              = 500  # Large volume
  type              = "gp2"

  tags = {
    Name = "Idle-Data-Volume"
  }
}

# Volume not attached to any instance - represents wasted storage cost

# 3. Underutilized RDS instance (cost optimization opportunity)
resource "aws_db_instance" "underutilized_db" {
  identifier             = "underutilized-db"
  allocated_storage      = 1000  # Very large storage
  storage_type           = "gp2"
  engine                 = "postgres"
  engine_version         = "13.7"
  instance_class         = "db.r5.8xlarge"  # Very large instance
  db_name                = "underutilized"
  username               = "admin"
  password               = "TempPass123!"
  parameter_group_name   = "default.postgres13"
  skip_final_snapshot    = true

  # Minimal backup retention
  backup_retention_period = 1

  tags = {
    Name = "Underutilized-DB"
  }
}

# 4. Reserved Instance opportunity (cost optimization opportunity)
resource "aws_instance" "ri_candidate" {
  count         = 5  # Multiple identical instances
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.medium"

  tags = {
    Name        = "RI-Candidate-${count.index + 1}"
    Environment = "production"
    Project     = "CostPilot-Test"
  }
}

# 5. Savings Plan opportunity (cost optimization opportunity)
resource "aws_instance" "savings_plan_candidate" {
  count         = 3
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "c5.large"

  tags = {
    Name        = "SavingsPlan-Candidate-${count.index + 1}"
    Environment = "production"
    Project     = "CostPilot-Test"
  }
}

# 6. Unused Elastic Load Balancer (cost optimization opportunity)
resource "aws_lb" "unused_lb" {
  name               = "unused-app-lb"
  internal           = false
  load_balancer_type = "application"
  subnets            = ["subnet-12345678", "subnet-87654321"]  # Dummy subnets

  tags = {
    Name = "Unused-Load-Balancer"
  }
}

# No target groups or listeners attached - represents unused ELB cost

# 7. Over-provisioned NAT Gateway (cost optimization opportunity)
resource "aws_nat_gateway" "over_provisioned_nat" {
  allocation_id = aws_eip.nat_eip.id
  subnet_id     = "subnet-12345678"  # Dummy subnet

  tags = {
    Name = "OverProvisioned-NAT"
  }
}

resource "aws_eip" "nat_eip" {
  vpc = true

  tags = {
    Name = "NAT-EIP"
  }
}

# 8. Unused Elastic IP (cost optimization opportunity)
resource "aws_eip" "unused_eip" {
  vpc = true

  tags = {
    Name = "Unused-Elastic-IP"
  }
}

# Not associated with any instance - represents wasted EIP cost

# 9. Large EBS volumes with infrequent access (cost optimization opportunity)
resource "aws_ebs_volume" "infrequent_access_volume" {
  availability_zone = "us-east-1a"
  size              = 1000  # Very large volume
  type              = "gp2"  # Should be moved to S3 or EFS

  tags = {
    Name = "Infrequent-Access-Data"
  }
}

# 10. Development environment running 24/7 (cost optimization opportunity)
resource "aws_instance" "dev_always_on" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.large"

  tags = {
    Name        = "Dev-Always-On"
    Environment = "development"
    Project     = "CostPilot-Test"
  }
}

# No auto-shutdown schedules - represents development waste

# 11. Multiple small EBS volumes (cost optimization opportunity)
resource "aws_ebs_volume" "small_volume_1" {
  availability_zone = "us-east-1a"
  size              = 10
  type              = "gp2"

  tags = {
    Name = "Small-Volume-1"
  }
}

resource "aws_ebs_volume" "small_volume_2" {
  availability_zone = "us-east-1a"
  size              = 15
  type              = "gp2"

  tags = {
    Name = "Small-Volume-2"
  }
}

resource "aws_ebs_volume" "small_volume_3" {
  availability_zone = "us-east-1a"
  size              = 8
  type              = "gp2"

  tags = {
    Name = "Small-Volume-3"
  }
}

# Could be consolidated into fewer, larger volumes

# 12. GPU instance for non-GPU workload (cost optimization opportunity)
resource "aws_instance" "gpu_for_cpu_workload" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "p3.2xlarge"  # Expensive GPU instance

  tags = {
    Name        = "GPU-for-CPU-Workload"
    Environment = "development"
    Project     = "CostPilot-Test"
  }
}

# 13. Cross-region data transfer (cost optimization opportunity)
resource "aws_instance" "us_east_instance" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"
  availability_zone = "us-east-1a"

  tags = {
    Name = "US-East-Instance"
  }
}

resource "aws_instance" "us_west_instance" {
  provider      = aws.us_west
  ami           = "ami-0abcdef1234567890"  # Different AMI for us-west
  instance_type = "t3.micro"
  availability_zone = "us-west-2a"

  tags = {
    Name = "US-West-Instance"
  }
}

# Separate provider for cross-region resources
provider "aws" {
  alias  = "us_west"
  region = "us-west-2"
}

# 14. Unused CloudWatch alarms (cost optimization opportunity)
resource "aws_cloudwatch_metric_alarm" "unused_alarm" {
  alarm_name          = "unused-cpu-alarm"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "300"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "Unused alarm for testing"

  # No dimensions - alarm not actually monitoring anything
}

# 15. Over-provisioned Lambda function (cost optimization opportunity)
resource "aws_lambda_function" "over_provisioned_lambda" {
  function_name = "over-provisioned-function"
  runtime       = "python3.9"
  handler       = "lambda_function.lambda_handler"
  memory_size   = 3008  # Maximum memory
  timeout       = 900   # Maximum timeout

  filename         = "dummy.zip"  # Would contain actual code
  source_code_hash = filebase64sha256("dummy.zip")

  role = aws_iam_role.lambda_role.arn

  tags = {
    Name = "OverProvisioned-Lambda"
  }
}

resource "aws_iam_role" "lambda_role" {
  name = "over-provisioned-lambda-role"

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

# Dummy file for Lambda
resource "local_file" "dummy_lambda" {
  content  = "dummy lambda code"
  filename = "dummy.zip"
}
