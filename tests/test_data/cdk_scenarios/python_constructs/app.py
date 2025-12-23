#!/usr/bin/env python3
import os
import aws_cdk as cdk
from costpilot_test_stack import CostPilotTestStack

app = cdk.App()

# Test environment stack
CostPilotTestStack(app, "CostPilotTestStack",
    environment="test",
    env=cdk.Environment(
        account=os.environ.get("CDK_DEFAULT_ACCOUNT"),
        region=os.environ.get("CDK_DEFAULT_REGION", "us-east-1")
    ),
)

# Production-like environment stack
CostPilotTestStack(app, "CostPilotProdStack",
    environment="prod",
    env=cdk.Environment(
        account=os.environ.get("CDK_DEFAULT_ACCOUNT"),
        region=os.environ.get("CDK_DEFAULT_REGION", "us-east-1")
    ),
)

app.synth()
