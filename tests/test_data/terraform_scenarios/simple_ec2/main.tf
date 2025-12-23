# Simple EC2 Instance Terraform Scenario
# This represents a basic single EC2 instance deployment for testing CostPilot analysis

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

# Simple EC2 instance with basic configuration
resource "aws_instance" "simple_web_server" {
  ami           = "ami-0c55b159cbfafe1d0"  # Amazon Linux 2
  instance_type = "t3.medium"

  tags = {
    Name        = "SimpleWebServer"
    Environment = "test"
    Project     = "CostPilot-Test"
  }

  # Basic security group
  vpc_security_group_ids = [aws_security_group.web_sg.id]

  # Root block device
  root_block_device {
    volume_size = 20
    volume_type = "gp2"
  }
}

# Security group for the instance
resource "aws_security_group" "web_sg" {
  name_prefix = "web-sg-"

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "WebServer-SG"
  }
}

# Basic EBS volume attachment
resource "aws_ebs_volume" "data_volume" {
  availability_zone = aws_instance.simple_web_server.availability_zone
  size              = 50
  type              = "gp2"

  tags = {
    Name = "DataVolume"
  }
}

resource "aws_volume_attachment" "data_attachment" {
  device_name = "/dev/sdh"
  volume_id   = aws_ebs_volume.data_volume.id
  instance_id = aws_instance.simple_web_server.id
}

# Elastic IP for the instance
resource "aws_eip" "web_eip" {
  instance = aws_instance.simple_web_server.id
  vpc      = true

  tags = {
    Name = "WebServer-EIP"
  }
}
