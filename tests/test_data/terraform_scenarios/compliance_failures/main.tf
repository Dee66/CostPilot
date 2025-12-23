# Compliance Failures Terraform Scenario
# This scenario includes resources that violate compliance standards for CostPilot compliance testing

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

# 1. HIPAA Compliance Violation - Unencrypted PHI data in S3
resource "aws_s3_bucket" "phi_unencrypted" {
  bucket = "phi-unencrypted-${random_string.bucket_suffix.result}"

  # COMPLIANCE VIOLATION: PHI data not encrypted at rest
  tags = {
    Name        = "PHI-Unencrypted-Bucket"
    Compliance  = "HIPAA"
    DataClass   = "PHI"
  }
}

resource "aws_s3_bucket_versioning" "phi_versioning" {
  bucket = aws_s3_bucket.phi_unencrypted.id
  versioning_configuration {
    status = "Enabled"
  }
}

# 2. PCI-DSS Compliance Violation - Cardholder data without encryption
resource "aws_db_instance" "cardholder_data_db" {
  identifier             = "cardholder-data-db"
  allocated_storage      = 20
  storage_type           = "gp2"
  engine                 = "mysql"
  engine_version         = "8.0"
  instance_class         = "db.t3.micro"
  db_name                = "carddata"
  username               = "admin"
  password               = "SecurePass123!"
  parameter_group_name   = "default.mysql8.0"
  skip_final_snapshot    = true

  # COMPLIANCE VIOLATION: Cardholder data not encrypted
  storage_encrypted = false

  tags = {
    Name        = "Cardholder-Data-DB"
    Compliance  = "PCI-DSS"
    DataClass   = "Cardholder"
  }
}

# 3. SOC 2 Compliance Violation - No access logging
resource "aws_s3_bucket" "no_access_logging" {
  bucket = "no-access-logging-${random_string.bucket_suffix.result}"

  tags = {
    Name       = "No-Access-Logging-Bucket"
    Compliance = "SOC2"
  }
}

# No aws_s3_bucket_logging resource - COMPLIANCE VIOLATION

# 4. GDPR Compliance Violation - Data retention policy not enforced
resource "aws_dynamodb_table" "gdpr_data_table" {
  name           = "gdpr-data-table"
  billing_mode   = "PAY_PER_REQUEST"
  hash_key       = "id"

  attribute {
    name = "id"
    type = "S"
  }

  # COMPLIANCE VIOLATION: No point-in-time recovery for GDPR data
  point_in_time_recovery {
    enabled = false
  }

  tags = {
    Name       = "GDPR-Data-Table"
    Compliance = "GDPR"
    DataClass  = "PII"
  }
}

# 5. ISO 27001 Compliance Violation - No backup strategy
resource "aws_instance" "no_backup_instance" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  tags = {
    Name       = "No-Backup-Instance"
    Compliance = "ISO27001"
    Backup     = "None"
  }
}

# No AWS Backup or snapshot resources - COMPLIANCE VIOLATION

# 6. FedRAMP Compliance Violation - Using non-approved services
resource "aws_instance" "fedramp_non_compliant" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  # COMPLIANCE VIOLATION: Using t3.micro which may not be FedRAMP approved
  tags = {
    Name       = "FedRAMP-Non-Compliant-Instance"
    Compliance = "FedRAMP"
    Approved   = "false"
  }
}

# 7. CIS Benchmarks Violation - SSH password authentication enabled
resource "aws_instance" "ssh_password_auth" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  user_data = base64encode(<<-EOF
              #!/bin/bash
              # COMPLIANCE VIOLATION: CIS Benchmark - SSH password auth enabled
              sed -i 's/PasswordAuthentication no/PasswordAuthentication yes/' /etc/ssh/sshd_config
              systemctl restart sshd
              EOF
  )

  tags = {
    Name       = "SSH-Password-Auth-Enabled"
    Compliance = "CIS"
    Benchmark  = "SSH-Password-Auth"
  }
}

# 8. NIST Compliance Violation - Weak encryption standards
resource "aws_kms_key" "weak_encryption_key" {
  description             = "Weak encryption key for NIST compliance testing"
  deletion_window_in_days = 7

  # COMPLIANCE VIOLATION: Using RSA_2048 instead of stronger key spec
  key_spec = "RSA_2048"

  tags = {
    Name       = "Weak-Encryption-Key"
    Compliance = "NIST"
    KeySpec    = "RSA_2048"
  }
}

# 9. SOX Compliance Violation - No audit trail
resource "aws_s3_bucket" "sox_no_audit" {
  bucket = "sox-no-audit-${random_string.bucket_suffix.result}"

  tags = {
    Name       = "SOX-No-Audit-Bucket"
    Compliance = "SOX"
    AuditTrail = "None"
  }
}

# No CloudTrail or Config rules - COMPLIANCE VIOLATION

# 10. HIPAA Compliance Violation - No MFA for root account
# Note: This would be a configuration violation, not a resource violation
# Represented by missing MFA device resource

# 11. PCI-DSS Compliance Violation - No WAF protection
resource "aws_lb" "no_waf_lb" {
  name               = "no-waf-lb"
  internal           = false
  load_balancer_type = "application"
  subnets            = ["subnet-12345678", "subnet-87654321"]  # Dummy subnets

  tags = {
    Name       = "No-WAF-Load-Balancer"
    Compliance = "PCI-DSS"
    WAF        = "None"
  }
}

resource "aws_lb_listener" "no_waf_listener" {
  load_balancer_arn = aws_lb.no_waf_lb.arn
  port              = "443"
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-2016-08"
  certificate_arn   = "arn:aws:acm:us-east-1:123456789012:certificate/dummy-cert"

  default_action {
    type = "fixed-response"

    fixed_response {
      content_type = "text/plain"
      message_body = "Hello World"
      status_code  = "200"
    }
  }
}

# No aws_wafv2_web_acl attached - COMPLIANCE VIOLATION

# 12. SOC 2 Compliance Violation - No monitoring/alerting
resource "aws_instance" "no_monitoring" {
  ami           = "ami-0c55b159cbfafe1d0"
  instance_type = "t3.micro"

  tags = {
    Name       = "No-Monitoring-Instance"
    Compliance = "SOC2"
    Monitoring = "None"
  }
}

# No CloudWatch alarms or monitoring - COMPLIANCE VIOLATION

# 13. GDPR Compliance Violation - Data not pseudonymized
resource "aws_rds_cluster" "gdpr_pii_cluster" {
  cluster_identifier = "gdpr-pii-cluster"
  engine             = "aurora-mysql"
  engine_version     = "8.0.mysql_aurora.3.02.0"
  master_username    = "admin"
  master_password    = "SecurePass123!"
  skip_final_snapshot = true

  # COMPLIANCE VIOLATION: PII data not encrypted
  storage_encrypted = false

  tags = {
    Name       = "GDPR-PII-Cluster"
    Compliance = "GDPR"
    DataClass  = "PII"
    Pseudonymized = "false"
  }
}

# 14. ISO 27001 Compliance Violation - No access control
resource "aws_s3_bucket" "no_access_control" {
  bucket = "no-access-control-${random_string.bucket_suffix.result}"

  tags = {
    Name       = "No-Access-Control-Bucket"
    Compliance = "ISO27001"
    AccessControl = "None"
  }
}

# No bucket policy or IAM policies restricting access - COMPLIANCE VIOLATION

# 15. FedRAMP Compliance Violation - Using non-compliant region
# Note: This is deployed in us-east-1, but FedRAMP may require specific regions
resource "aws_instance" "fedramp_wrong_region" {
  provider      = aws.us_west_2  # Different region
  ami           = "ami-0abcdef1234567890"  # Dummy AMI
  instance_type = "t3.micro"

  tags = {
    Name       = "FedRAMP-Wrong-Region"
    Compliance = "FedRAMP"
    Region     = "us-west-2"
  }
}

# Additional provider for cross-region resource
provider "aws" {
  alias  = "us_west_2"
  region = "us-west-2"
}

# 16. CIS Compliance Violation - Auto minor version upgrade disabled
resource "aws_db_instance" "no_auto_upgrade" {
  identifier             = "no-auto-upgrade-db"
  allocated_storage      = 20
  storage_type           = "gp2"
  engine                 = "postgres"
  engine_version         = "13.7"
  instance_class         = "db.t3.micro"
  db_name                = "testdb"
  username               = "admin"
  password               = "SecurePass123!"
  parameter_group_name   = "default.postgres13"
  skip_final_snapshot    = true

  # COMPLIANCE VIOLATION: CIS Benchmark - Auto minor version upgrade disabled
  auto_minor_version_upgrade = false

  tags = {
    Name       = "No-Auto-Upgrade-DB"
    Compliance = "CIS"
    AutoUpgrade = "Disabled"
  }
}

# 17. NIST Compliance Violation - No key rotation
resource "aws_kms_key" "no_rotation_key" {
  description             = "KMS key without rotation for NIST compliance testing"
  deletion_window_in_days = 7

  # COMPLIANCE VIOLATION: Key rotation disabled
  key_rotation = false

  tags = {
    Name       = "No-Rotation-Key"
    Compliance = "NIST"
    Rotation   = "Disabled"
  }
}

# 18. SOX Compliance Violation - No segregation of duties
resource "aws_iam_user" "sox_combined_roles" {
  name = "sox-combined-roles-user"

  tags = {
    Name       = "SOX-Combined-Roles-User"
    Compliance = "SOX"
    Roles      = "Developer,Auditor,Admin"
  }
}

resource "aws_iam_user_policy" "sox_combined_policy" {
  name = "sox-combined-policy"
  user = aws_iam_user.sox_combined_roles.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:*",
          "ec2:*",
          "rds:*",
          "iam:*",
          "cloudtrail:*"
        ]
        Resource = "*"
      }
    ]
  })
}

# 19. HIPAA Compliance Violation - No data backup retention
resource "aws_backup_vault" "hipaa_short_retention" {
  name = "hipaa-short-retention-vault"

  # COMPLIANCE VIOLATION: Retention period too short for HIPAA
  tags = {
    Name       = "HIPAA-Short-Retention-Vault"
    Compliance = "HIPAA"
    Retention  = "30-days"
  }
}

resource "aws_backup_plan" "hipaa_backup_plan" {
  name = "hipaa-backup-plan"

  rule {
    rule_name         = "hipaa_rule"
    target_vault_name = aws_backup_vault.hipaa_short_retention.name
    schedule          = "cron(0 5 ? * * *)"

    lifecycle {
      delete_after = 30  # COMPLIANCE VIOLATION: Only 30 days retention
    }
  }

  tags = {
    Name       = "HIPAA-Backup-Plan"
    Compliance = "HIPAA"
  }
}

# 20. PCI-DSS Compliance Violation - No network segmentation
resource "aws_vpc" "pci_no_segmentation" {
  cidr_block = "10.2.0.0/16"

  tags = {
    Name       = "PCI-No-Segmentation-VPC"
    Compliance = "PCI-DSS"
    Segmentation = "None"
  }
}

resource "aws_subnet" "pci_public_subnet" {
  vpc_id     = aws_vpc.pci_no_segmentation.id
  cidr_block = "10.2.1.0/24"

  # COMPLIANCE VIOLATION: Cardholder data in public subnet
  tags = {
    Name       = "PCI-Public-Subnet"
    Compliance = "PCI-DSS"
    DataClass  = "Cardholder"
    Type       = "Public"
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
