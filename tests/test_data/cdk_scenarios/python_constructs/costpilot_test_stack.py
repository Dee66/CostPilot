from aws_cdk import (
    Duration,
    Stack,
    CfnOutput,
    aws_ec2 as ec2,
    aws_elasticloadbalancingv2 as elbv2,
    aws_autoscaling as autoscaling,
    aws_rds as rds,
    aws_elasticache as elasticache,
    aws_cloudwatch as cloudwatch,
)
from constructs import Construct


class CostPilotTestStack(Stack):

    def __init__(self, scope: Construct, construct_id: str, environment: str, **kwargs) -> None:
        super().__init__(scope, construct_id, **kwargs)

        self.environment = environment

        # VPC with multiple subnets
        vpc = ec2.Vpc(self, "TestVpc",
            cidr="10.0.0.0/16",
            max_azs=2,
            subnet_configuration=[
                ec2.SubnetConfiguration(
                    cidr_mask=24,
                    name="Public",
                    subnet_type=ec2.SubnetType.PUBLIC,
                ),
                ec2.SubnetConfiguration(
                    cidr_mask=24,
                    name="Private",
                    subnet_type=ec2.SubnetType.PRIVATE_WITH_EGRESS,
                ),
            ],
            tags={
                "Name": f"{environment}-VPC",
                "Environment": environment,
            }
        )

        # Security Groups
        web_security_group = ec2.SecurityGroup(self, "WebSecurityGroup",
            vpc=vpc,
            description="Security group for web servers",
            allow_all_outbound=True,
        )

        web_security_group.add_ingress_rule(
            ec2.Peer.any_ipv4(),
            ec2.Port.tcp(80),
            "Allow HTTP traffic"
        )

        web_security_group.add_ingress_rule(
            ec2.Peer.any_ipv4(),
            ec2.Port.tcp(443),
            "Allow HTTPS traffic"
        )

        db_security_group = ec2.SecurityGroup(self, "DatabaseSecurityGroup",
            vpc=vpc,
            description="Security group for database",
            allow_all_outbound=True,
        )

        db_security_group.add_ingress_rule(
            web_security_group,
            ec2.Port.tcp(3306),
            "Allow MySQL traffic from web servers"
        )

        # Application Load Balancer
        alb_security_group = ec2.SecurityGroup(self, "ALBSecurityGroup",
            vpc=vpc,
            description="Security group for ALB",
            allow_all_outbound=True,
        )

        alb = elbv2.ApplicationLoadBalancer(self, "ApplicationLoadBalancer",
            vpc=vpc,
            internet_facing=True,
            security_group=alb_security_group,
            tags={
                "Name": f"{environment}-ALB",
                "Environment": environment,
            }
        )

        # ALB Security Group rules
        alb.connections.allow_from_any_ipv4(ec2.Port.tcp(80), "Allow HTTP traffic")

        # Target Group
        target_group = elbv2.ApplicationTargetGroup(self, "TargetGroup",
            vpc=vpc,
            port=80,
            protocol=elbv2.ApplicationProtocol.HTTP,
            target_type=elbv2.TargetType.INSTANCE,
            health_check=elbv2.HealthCheck(
                path="/",
                interval=Duration.seconds(30),
                timeout=Duration.seconds(5),
                healthy_threshold_count=2,
                unhealthy_threshold_count=2,
            ),
        )

        # ALB Listener
        listener = alb.add_listener("HTTPListener",
            port=80,
            protocol=elbv2.ApplicationProtocol.HTTP,
            default_action=elbv2.ListenerAction.forward([target_group]),
        )

        # Launch Template
        launch_template = ec2.LaunchTemplate(self, "LaunchTemplate",
            instance_type=ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MICRO),
            machine_image=ec2.MachineImage.latest_amazon_linux2023(),
            security_group=web_security_group,
            user_data=ec2.UserData.custom(f"""#!/bin/bash
                yum update -y
                yum install -y httpd
                systemctl start httpd
                systemctl enable httpd
                echo "<h1>Hello from {environment} environment</h1>" > /var/www/html/index.html
            """),
            tag_specifications=[
                ec2.LaunchTemplateTagSpecification(
                    resource_type=ec2.LaunchTemplateResourceType.INSTANCE,
                    tags={
                        "Name": f"{environment}-Web-Instance",
                        "Environment": environment,
                    }
                )
            ]
        )

        # Auto Scaling Group
        auto_scaling_group = autoscaling.AutoScalingGroup(self, "AutoScalingGroup",
            vpc=vpc,
            vpc_subnets=ec2.SubnetSelection(subnet_type=ec2.SubnetType.PRIVATE_WITH_EGRESS),
            launch_template=launch_template,
            min_capacity=2,
            max_capacity=10,
            desired_capacity=3,
            health_check=autoscaling.HealthCheck.ec2(
                grace=Duration.seconds(300),
            ),
        )

        # Attach ASG to Target Group
        auto_scaling_group.attach_to_application_target_group(target_group)

        # RDS Instance
        rds_instance = rds.DatabaseInstance(self, "DatabaseInstance",
            vpc=vpc,
            vpc_subnets=ec2.SubnetSelection(subnet_type=ec2.SubnetType.PRIVATE_WITH_EGRESS),
            engine=rds.DatabaseInstanceEngine.mysql(
                version=rds.MysqlEngineVersion.VER_8_0
            ),
            instance_type=ec2.InstanceType.of(ec2.InstanceClass.DB, ec2.InstanceSize.T3_MICRO),
            credentials=rds.Credentials.from_generated_secret("admin"),
            security_groups=[db_security_group],
            allocated_storage=20,
            storage_type=rds.StorageType.GP2,
            backup_retention=Duration.days(7),
            multi_az=False,
            storage_encrypted=True,
            enable_performance_insights=True,
            tags={
                "Name": f"{environment}-RDS-Instance",
                "Environment": environment,
            }
        )

        # ElastiCache Cluster
        cache_subnet_group = elasticache.CfnSubnetGroup(self, "CacheSubnetGroup",
            description=f"Subnet group for {environment} ElastiCache",
            subnet_ids=vpc.select_subnets(subnet_type=ec2.SubnetType.PRIVATE_WITH_EGRESS).subnet_ids,
            tags=[{
                "key": "Name",
                "value": f"{environment}-Cache-Subnet-Group",
            }]
        )

        cache_security_group = ec2.SecurityGroup(self, "CacheSecurityGroup",
            vpc=vpc,
            description="Security group for ElastiCache",
            allow_all_outbound=True,
        )

        cache_security_group.add_ingress_rule(
            web_security_group,
            ec2.Port.tcp(6379),
            "Allow Redis traffic from web servers"
        )

        cache_cluster = elasticache.CfnCacheCluster(self, "CacheCluster",
            cache_node_type="cache.t3.micro",
            engine="redis",
            num_cache_nodes=1,
            cache_subnet_group_name=cache_subnet_group.ref,
            vpc_security_group_ids=[cache_security_group.security_group_id],
            tags=[{
                "key": "Name",
                "value": f"{environment}-ElastiCache-Cluster",
            }]
        )

        # CloudWatch Alarms
        cloudwatch.Alarm(self, "CPUAlarm",
            alarm_name=f"{environment}-cpu-alarm",
            alarm_description="Alarm for high CPU utilization",
            metric=cloudwatch.Metric(
                namespace="AWS/EC2",
                metric_name="CPUUtilization",
                dimensions_map={
                    "AutoScalingGroupName": auto_scaling_group.auto_scaling_group_name,
                },
                statistic="Average",
            ),
            threshold=80,
            evaluation_periods=2,
            comparison_operator=cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
        )

        cloudwatch.Alarm(self, "DatabaseCPUAlarm",
            alarm_name=f"{environment}-db-cpu-alarm",
            alarm_description="Alarm for high database CPU utilization",
            metric=cloudwatch.Metric(
                namespace="AWS/RDS",
                metric_name="CPUUtilization",
                dimensions_map={
                    "DBInstanceIdentifier": rds_instance.instance_identifier,
                },
                statistic="Average",
            ),
            threshold=80,
            evaluation_periods=2,
            comparison_operator=cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
        )

        # Scaling Policy
        auto_scaling_group.scale_on_cpu_utilization("CpuScalingPolicy",
            target_utilization_percent=70,
        )

        # Outputs
        CfnOutput(self, "VpcId",
            description="VPC ID",
            value=vpc.vpc_id,
            export_name=f"{environment}-VpcId",
        )

        CfnOutput(self, "LoadBalancerDNS",
            description="Load Balancer DNS Name",
            value=alb.load_balancer_dns_name,
            export_name=f"{environment}-LoadBalancerDNS",
        )

        CfnOutput(self, "DatabaseEndpoint",
            description="Database Endpoint",
            value=rds_instance.db_instance_endpoint_address,
            export_name=f"{environment}-DatabaseEndpoint",
        )

        CfnOutput(self, "CacheEndpoint",
            description="Cache Cluster Endpoint",
            value=cache_cluster.attr_redis_endpoint_address,
            export_name=f"{environment}-CacheEndpoint",
        )
