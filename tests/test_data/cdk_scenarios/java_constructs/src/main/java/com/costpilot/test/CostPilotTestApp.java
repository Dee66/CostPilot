package com.costpilot.test;

import java.util.Map;
import software.amazon.awscdk.App;
import software.amazon.awscdk.Environment;
import software.amazon.awscdk.StackProps;

public class CostPilotTestApp {
  public static void main(final String[] args) {
    App app = new App();

    // Test environment stack
    new CostPilotTestStack(
        app, "CostPilotTestStack",
        StackProps.builder()
            .env(Environment.builder()
                     .account(System.getenv("CDK_DEFAULT_ACCOUNT"))
                     .region(System.getenv("CDK_DEFAULT_REGION") != null
                                 ? System.getenv("CDK_DEFAULT_REGION")
                                 : "us-east-1")
                     .build())
            .build(),
        "test");

    // Production-like environment stack
    new CostPilotTestStack(
        app, "CostPilotProdStack",
        StackProps.builder()
            .env(Environment.builder()
                     .account(System.getenv("CDK_DEFAULT_ACCOUNT"))
                     .region(System.getenv("CDK_DEFAULT_REGION") != null
                                 ? System.getenv("CDK_DEFAULT_REGION")
                                 : "us-east-1")
                     .build())
            .build(),
        "prod");

    app.synth();
  }
}
