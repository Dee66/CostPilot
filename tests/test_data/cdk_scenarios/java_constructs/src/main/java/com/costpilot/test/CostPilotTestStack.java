package com.costpilot.test;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import software.amazon.awscdk.*;
import software.amazon.awscdk.services.autoscaling.*;
import software.amazon.awscdk.services.cloudwatch.*;
import software.amazon.awscdk.services.ec2.*;
import software.amazon.awscdk.services.elasticache.*;
import software.amazon.awscdk.services.elasticloadbalancingv2.*;
import software.amazon.awscdk.services.rds.*;
import software.constructs.Construct;

public class CostPilotTestStack extends Stack {
  private final String environment;

  public CostPilotTestStack(final Construct scope, final String id,
                            final StackProps props, final String environment) {
    super(scope, id, props);
    this.environment = environment;

    // VPC with multiple subnets
    Vpc vpc = Vpc.Builder.create(this, "TestVpc")
                  .cidr("10.0.0.0/16")
                  .maxAzs(2)
                  .subnetConfiguration(Arrays.asList(
                      SubnetConfiguration.builder()
                          .cidrMask(24)
                          .name("Public")
                          .subnetType(SubnetType.PUBLIC)
                          .build(),
                      SubnetConfiguration.builder()
                          .cidrMask(24)
                          .name("Private")
                          .subnetType(SubnetType.PRIVATE_WITH_EGRESS)
                          .build()))
                  .tags(Map.of("Name", environment + "-VPC", "Environment",
                               environment))
                  .build();

    // Security Groups
    SecurityGroup webSecurityGroup =
        SecurityGroup.Builder.create(this, "WebSecurityGroup")
            .vpc(vpc)
            .description("Security group for web servers")
            .allowAllOutbound(true)
            .build();

    webSecurityGroup.addIngressRule(Peer.anyIpv4(), Port.tcp(80),
                                    "Allow HTTP traffic");

    webSecurityGroup.addIngressRule(Peer.anyIpv4(), Port.tcp(443),
                                    "Allow HTTPS traffic");

    SecurityGroup dbSecurityGroup =
        SecurityGroup.Builder.create(this, "DatabaseSecurityGroup")
            .vpc(vpc)
            .description("Security group for database")
            .allowAllOutbound(true)
            .build();

    dbSecurityGroup.addIngressRule(webSecurityGroup, Port.tcp(3306),
                                   "Allow MySQL traffic from web servers");

    // Application Load Balancer
    SecurityGroup albSecurityGroup =
        SecurityGroup.Builder.create(this, "ALBSecurityGroup")
            .vpc(vpc)
            .description("Security group for ALB")
            .allowAllOutbound(true)
            .build();

    ApplicationLoadBalancer alb =
        ApplicationLoadBalancer.Builder.create(this, "ApplicationLoadBalancer")
            .vpc(vpc)
            .internetFacing(true)
            .securityGroup(albSecurityGroup)
            .tags(Map.of("Name", environment + "-ALB", "Environment",
                         environment))
            .build();

    // ALB Security Group rules
    alb.getConnections().allowFromAnyIpv4(Port.tcp(80), "Allow HTTP traffic");

    // Target Group
    ApplicationTargetGroup targetGroup =
        ApplicationTargetGroup.Builder.create(this, "TargetGroup")
            .vpc(vpc)
            .port(80)
            .protocol(ApplicationProtocol.HTTP)
            .targetType(TargetType.INSTANCE)
            .healthCheck(HealthCheck.builder()
                             .path("/")
                             .interval(Duration.seconds(30))
                             .timeout(Duration.seconds(5))
                             .healthyThresholdCount(2)
                             .unhealthyThresholdCount(2)
                             .build())
            .build();

    // ALB Listener
    alb.addListener("HTTPListener", BaseApplicationListenerProps.builder()
                                        .port(80)
                                        .protocol(ApplicationProtocol.HTTP)
                                        .defaultAction(ListenerAction.forward(
                                            Arrays.asList(targetGroup)))
                                        .build());

    // Launch Template
    LaunchTemplate launchTemplate =
        LaunchTemplate.Builder.create(this, "LaunchTemplate")
            .instanceType(InstanceType.of(InstanceClass.T3, InstanceSize.MICRO))
            .machineImage(MachineImage.latestAmazonLinux2023())
            .securityGroup(webSecurityGroup)
            .userData(UserData.custom(
                "#!/bin/bash\n"
                + "yum update -y\n"
                + "yum install -y httpd\n"
                + "systemctl start httpd\n"
                + "systemctl enable httpd\n"
                + "echo \"<h1>Hello from " + environment +
                " environment</h1>\" > /var/www/html/index.html\n"))
            .tagSpecifications(Arrays.asList(
                LaunchTemplateTagSpecification.builder()
                    .resourceType(LaunchTemplateResourceType.INSTANCE)
                    .tags(Map.of("Name", environment + "-Web-Instance",
                                 "Environment", environment))
                    .build()))
            .build();

    // Auto Scaling Group
    AutoScalingGroup autoScalingGroup =
        AutoScalingGroup.Builder.create(this, "AutoScalingGroup")
            .vpc(vpc)
            .vpcSubnets(SubnetSelection.builder()
                            .subnetType(SubnetType.PRIVATE_WITH_EGRESS)
                            .build())
            .launchTemplate(launchTemplate)
            .minCapacity(2)
            .maxCapacity(10)
            .desiredCapacity(3)
            .healthCheck(HealthCheck.ec2(HealthCheckOptions.builder()
                                             .grace(Duration.seconds(300))
                                             .build()))
            .build();

    // Attach ASG to Target Group
    autoScalingGroup.attachToApplicationTargetGroup(targetGroup);

    // RDS Instance
    DatabaseInstance rdsInstance =
        DatabaseInstance.Builder.create(this, "DatabaseInstance")
            .vpc(vpc)
            .vpcSubnets(SubnetSelection.builder()
                            .subnetType(SubnetType.PRIVATE_WITH_EGRESS)
                            .build())
            .engine(DatabaseInstanceEngine.mysql(MySqlEngineVersion.VER_8_0))
            .instanceType(
                InstanceType.of(InstanceClass.DB, InstanceSize.T3_MICRO))
            .credentials(Credentials.fromGeneratedSecret("admin"))
            .securityGroups(Arrays.asList(dbSecurityGroup))
            .allocatedStorage(20)
            .storageType(StorageType.GP2)
            .backupRetention(Duration.days(7))
            .multiAz(false)
            .storageEncrypted(true)
            .enablePerformanceInsights(true)
            .tags(Map.of("Name", environment + "-RDS-Instance", "Environment",
                         environment))
            .build();

    // ElastiCache Cluster
    CfnSubnetGroup cacheSubnetGroup =
        CfnSubnetGroup.Builder.create(this, "CacheSubnetGroup")
            .description("Subnet group for " + environment + " ElastiCache")
            .subnetIds(vpc.selectSubnets(
                              SubnetSelection.builder()
                                  .subnetType(SubnetType.PRIVATE_WITH_EGRESS)
                                  .build())
                           .getSubnetIds())
            .tags(
                Arrays.asList(CfnTag.builder()
                                  .key("Name")
                                  .value(environment + "-Cache-Subnet-Group")
                                  .build()))
            .build();

    SecurityGroup cacheSecurityGroup =
        SecurityGroup.Builder.create(this, "CacheSecurityGroup")
            .vpc(vpc)
            .description("Security group for ElastiCache")
            .allowAllOutbound(true)
            .build();

    cacheSecurityGroup.addIngressRule(webSecurityGroup, Port.tcp(6379),
                                      "Allow Redis traffic from web servers");

    CfnCacheCluster cacheCluster =
        CfnCacheCluster.Builder.create(this, "CacheCluster")
            .cacheNodeType("cache.t3.micro")
            .engine("redis")
            .numCacheNodes(1)
            .cacheSubnetGroupName(cacheSubnetGroup.getRef())
            .vpcSecurityGroupIds(
                Arrays.asList(cacheSecurityGroup.getSecurityGroupId()))
            .tags(
                Arrays.asList(CfnTag.builder()
                                  .key("Name")
                                  .value(environment + "-ElastiCache-Cluster")
                                  .build()))
            .build();

    // CloudWatch Alarms
    Alarm.Builder.create(this, "CPUAlarm")
        .alarmName(environment + "-cpu-alarm")
        .alarmDescription("Alarm for high CPU utilization")
        .metric(Metric.Builder.create()
                    .namespace("AWS/EC2")
                    .metricName("CPUUtilization")
                    .dimensionsMap(
                        Map.of("AutoScalingGroupName",
                               autoScalingGroup.getAutoScalingGroupName()))
                    .statistic("Average")
                    .build())
        .threshold(80)
        .evaluationPeriods(2)
        .comparisonOperator(ComparisonOperator.GREATER_THAN_THRESHOLD)
        .build();

    Alarm.Builder.create(this, "DatabaseCPUAlarm")
        .alarmName(environment + "-db-cpu-alarm")
        .alarmDescription("Alarm for high database CPU utilization")
        .metric(Metric.Builder.create()
                    .namespace("AWS/RDS")
                    .metricName("CPUUtilization")
                    .dimensionsMap(Map.of("DBInstanceIdentifier",
                                          rdsInstance.getInstanceIdentifier()))
                    .statistic("Average")
                    .build())
        .threshold(80)
        .evaluationPeriods(2)
        .comparisonOperator(ComparisonOperator.GREATER_THAN_THRESHOLD)
        .build();

    // Scaling Policy
    autoScalingGroup.scaleOnCpuUtilization("CpuScalingPolicy",
                                           CpuUtilizationScalingProps.builder()
                                               .targetUtilizationPercent(70)
                                               .build());

    // Outputs
    CfnOutput.Builder.create(this, "VpcId")
        .description("VPC ID")
        .value(vpc.getVpcId())
        .exportName(environment + "-VpcId")
        .build();

    CfnOutput.Builder.create(this, "LoadBalancerDNS")
        .description("Load Balancer DNS Name")
        .value(alb.getLoadBalancerDnsName())
        .exportName(environment + "-LoadBalancerDNS")
        .build();

    CfnOutput.Builder.create(this, "DatabaseEndpoint")
        .description("Database Endpoint")
        .value(rdsInstance.getDbInstanceEndpointAddress())
        .exportName(environment + "-DatabaseEndpoint")
        .build();

    CfnOutput.Builder.create(this, "CacheEndpoint")
        .description("Cache Cluster Endpoint")
        .value(cacheCluster.getAttrRedisEndpointAddress())
        .exportName(environment + "-CacheEndpoint")
        .build();
  }
}
