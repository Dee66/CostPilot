import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as elbv2 from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import * as autoscaling from 'aws-cdk-lib/aws-autoscaling';
import * as rds from 'aws-cdk-lib/aws-rds';
import * as elasticache from 'aws-cdk-lib/aws-elasticache';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import { Construct } from 'constructs';

export interface CostPilotTestStackProps extends cdk.StackProps {
  environment: string;
}

export class CostPilotTestStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props: CostPilotTestStackProps) {
    super(scope, id, props);

    const { environment } = props;

    // VPC with multiple subnets
    const vpc = new ec2.Vpc(this, 'TestVpc', {
      cidr: '10.0.0.0/16',
      maxAzs: 2,
      subnetConfiguration: [
        {
          cidrMask: 24,
          name: 'Public',
          subnetType: ec2.SubnetType.PUBLIC,
        },
        {
          cidrMask: 24,
          name: 'Private',
          subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS,
        },
      ],
      tags: {
        Name: `${environment}-VPC`,
        Environment: environment,
      },
    });

    // Security Groups
    const webSecurityGroup = new ec2.SecurityGroup(this, 'WebSecurityGroup', {
      vpc,
      description: 'Security group for web servers',
      allowAllOutbound: true,
    });

    webSecurityGroup.addIngressRule(
      ec2.Peer.anyIpv4(),
      ec2.Port.tcp(80),
      'Allow HTTP traffic'
    );

    webSecurityGroup.addIngressRule(
      ec2.Peer.anyIpv4(),
      ec2.Port.tcp(443),
      'Allow HTTPS traffic'
    );

    const dbSecurityGroup = new ec2.SecurityGroup(this, 'DatabaseSecurityGroup', {
      vpc,
      description: 'Security group for database',
      allowAllOutbound: true,
    });

    dbSecurityGroup.addIngressRule(
      webSecurityGroup,
      ec2.Port.tcp(3306),
      'Allow MySQL traffic from web servers'
    );

    // Application Load Balancer
    const alb = new elbv2.ApplicationLoadBalancer(this, 'ApplicationLoadBalancer', {
      vpc,
      internetFacing: true,
      securityGroup: new ec2.SecurityGroup(this, 'ALBSecurityGroup', {
        vpc,
        description: 'Security group for ALB',
        allowAllOutbound: true,
      }),
      tags: {
        Name: `${environment}-ALB`,
        Environment: environment,
      },
    });

    // ALB Security Group rules
    alb.connections.allowFromAnyIpv4(ec2.Port.tcp(80), 'Allow HTTP traffic');

    // Target Group
    const targetGroup = new elbv2.ApplicationTargetGroup(this, 'TargetGroup', {
      vpc,
      port: 80,
      protocol: elbv2.ApplicationProtocol.HTTP,
      targetType: elbv2.TargetType.INSTANCE,
      healthCheck: {
        path: '/',
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        healthyThresholdCount: 2,
        unhealthyThresholdCount: 2,
      },
    });

    // ALB Listener
    const listener = alb.addListener('HTTPListener', {
      port: 80,
      protocol: elbv2.ApplicationProtocol.HTTP,
      defaultAction: elbv2.ListenerAction.forward([targetGroup]),
    });

    // Launch Template
    const launchTemplate = new ec2.LaunchTemplate(this, 'LaunchTemplate', {
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.T3, ec2.InstanceSize.MICRO),
      machineImage: ec2.MachineImage.latestAmazonLinux2023(),
      securityGroup: webSecurityGroup,
      userData: ec2.UserData.custom(`#!/bin/bash
        yum update -y
        yum install -y httpd
        systemctl start httpd
        systemctl enable httpd
        echo "<h1>Hello from ${environment} environment</h1>" > /var/www/html/index.html
      `),
      tagSpecifications: [
        {
          resourceType: ec2.LaunchTemplateResourceType.INSTANCE,
          tags: {
            Name: `${environment}-Web-Instance`,
            Environment: environment,
          },
        },
      ],
    });

    // Auto Scaling Group
    const autoScalingGroup = new autoscaling.AutoScalingGroup(this, 'AutoScalingGroup', {
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
      launchTemplate,
      minCapacity: 2,
      maxCapacity: 10,
      desiredCapacity: 3,
      healthCheck: autoscaling.HealthCheck.ec2({
        grace: cdk.Duration.seconds(300),
      }),
      groupMetrics: [new autoscaling.GroupMetric(new cloudwatch.Metric({
        namespace: 'AWS/EC2',
        metricName: 'CPUUtilization',
        dimensionsMap: {
          AutoScalingGroupName: autoScalingGroup.autoScalingGroupName,
        },
      }))],
    });

    // Attach ASG to Target Group
    autoScalingGroup.attachToApplicationTargetGroup(targetGroup);

    // RDS Instance
    const rdsInstance = new rds.DatabaseInstance(this, 'DatabaseInstance', {
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS },
      engine: rds.DatabaseInstanceEngine.mysql({
        version: rds.MysqlEngineVersion.VER_8_0,
      }),
      instanceType: ec2.InstanceType.of(ec2.InstanceClass.DB, ec2.InstanceSize.T3_MICRO),
      credentials: rds.Credentials.fromGeneratedSecret('admin'),
      securityGroups: [dbSecurityGroup],
      allocatedStorage: 20,
      storageType: rds.StorageType.GP2,
      backupRetention: cdk.Duration.days(7),
      multiAz: false,
      storageEncrypted: true,
      enablePerformanceInsights: true,
      performanceInsightRetention: rds.PerformanceInsightRetentionPeriod.DEFAULT,
      tags: {
        Name: `${environment}-RDS-Instance`,
        Environment: environment,
      },
    });

    // ElastiCache Cluster
    const cacheSubnetGroup = new elasticache.CfnSubnetGroup(this, 'CacheSubnetGroup', {
      description: `Subnet group for ${environment} ElastiCache`,
      subnetIds: vpc.selectSubnets({ subnetType: ec2.SubnetType.PRIVATE_WITH_EGRESS }).subnetIds,
      tags: [{
        key: 'Name',
        value: `${environment}-Cache-Subnet-Group`,
      }],
    });

    const cacheSecurityGroup = new ec2.SecurityGroup(this, 'CacheSecurityGroup', {
      vpc,
      description: 'Security group for ElastiCache',
      allowAllOutbound: true,
    });

    cacheSecurityGroup.addIngressRule(
      webSecurityGroup,
      ec2.Port.tcp(6379),
      'Allow Redis traffic from web servers'
    );

    const cacheCluster = new elasticache.CfnCacheCluster(this, 'CacheCluster', {
      cacheNodeType: 'cache.t3.micro',
      engine: 'redis',
      numCacheNodes: 1,
      cacheSubnetGroupName: cacheSubnetGroup.ref,
      vpcSecurityGroupIds: [cacheSecurityGroup.securityGroupId],
      tags: [{
        key: 'Name',
        value: `${environment}-ElastiCache-Cluster`,
      }],
    });

    // CloudWatch Alarms
    new cloudwatch.Alarm(this, 'CPUAlarm', {
      alarmName: `${environment}-cpu-alarm`,
      alarmDescription: 'Alarm for high CPU utilization',
      metric: new cloudwatch.Metric({
        namespace: 'AWS/EC2',
        metricName: 'CPUUtilization',
        dimensionsMap: {
          AutoScalingGroupName: autoScalingGroup.autoScalingGroupName,
        },
        statistic: 'Average',
      }),
      threshold: 80,
      evaluationPeriods: 2,
      comparisonOperator: cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
    });

    new cloudwatch.Alarm(this, 'DatabaseCPUAlarm', {
      alarmName: `${environment}-db-cpu-alarm`,
      alarmDescription: 'Alarm for high database CPU utilization',
      metric: new cloudwatch.Metric({
        namespace: 'AWS/RDS',
        metricName: 'CPUUtilization',
        dimensionsMap: {
          DBInstanceIdentifier: rdsInstance.instanceIdentifier,
        },
        statistic: 'Average',
      }),
      threshold: 80,
      evaluationPeriods: 2,
      comparisonOperator: cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
    });

    // Scaling Policy
    autoScalingGroup.scaleOnCpuUtilization('CpuScalingPolicy', {
      targetUtilizationPercent: 70,
    });

    // Outputs
    new cdk.CfnOutput(this, 'VpcId', {
      description: 'VPC ID',
      value: vpc.vpcId,
      exportName: `${environment}-VpcId`,
    });

    new cdk.CfnOutput(this, 'LoadBalancerDNS', {
      description: 'Load Balancer DNS Name',
      value: alb.loadBalancerDnsName,
      exportName: `${environment}-LoadBalancerDNS`,
    });

    new cdk.CfnOutput(this, 'DatabaseEndpoint', {
      description: 'Database Endpoint',
      value: rdsInstance.dbInstanceEndpointAddress,
      exportName: `${environment}-DatabaseEndpoint`,
    });

    new cdk.CfnOutput(this, 'CacheEndpoint', {
      description: 'Cache Cluster Endpoint',
      value: cacheCluster.attrRedisEndpointAddress,
      exportName: `${environment}-CacheEndpoint`,
    });
  }
}
