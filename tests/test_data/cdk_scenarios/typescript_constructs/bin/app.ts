#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { CostPilotTestStack } from '../lib/costpilot-test-stack';

const app = new cdk.App();

// Test environment stack
new CostPilotTestStack(app, 'CostPilotTestStack', {
  environment: 'test',
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION || 'us-east-1',
  },
});

// Production-like environment stack
new CostPilotTestStack(app, 'CostPilotProdStack', {
  environment: 'prod',
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION || 'us-east-1',
  },
});
