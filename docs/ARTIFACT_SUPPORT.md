# Artifact Support (Terraform & CDK)

## Overview

The Artifact Support system provides unified parsing, normalization, and cost analysis for Infrastructure as Code formats, starting with Terraform and AWS CDK support.

## Architecture

### Components

1. **artifact_types.rs** (632 lines)
   - Core artifact data structures
   - Format definitions (Terraform, CDK, Pulumi)
   - Resource and metadata types
   - Intrinsic function handling
   - 15 comprehensive unit tests

2. **cdk_parser.rs** (356 lines)
   - CDK synthesized output parsing
   - CDK manifest handling (cdk.out/)
   - Stack extraction from CDK assembly
   - CDK-specific metadata enhancement
   - 6 unit tests

3. **artifact_normalizer.rs** (441 lines)
   - Multi-format normalization to common representation
   - Property name mapping (PascalCase → snake_case)
   - Intrinsic function resolution
   - 10 unit tests

**Total**: 1,869 lines code + 41 tests

## Key Features

✅ **CDK Support** - Parse CDK synthesized output
✅ **Format Detection** - Auto-detect IaC format
✅ **Normalization** - Convert to unified representation
✅ **Resource Mapping** - AWS::EC2::Instance → aws_ec2_instance
✅ **Property Mapping** - InstanceType → instance_type, ImageId → ami
✅ **Intrinsic Functions** - Resolve Ref, GetAtt, Sub, Join
✅ **Dependencies** - Extract DependsOn relationships
✅ **Parameters** - Support template parameters
✅ **Outputs** - Extract stack outputs
✅ **Metadata** - Preserve source metadata
✅ **Validation** - Check for duplicate IDs, missing dependencies

## Supported Formats

### AWS CDK
- **Synthesized Templates** - Templates from `cdk synth`
- **CDK Manifest** - manifest.json parsing
- **Multiple Stacks** - All stacks in cdk.out/
- **CDK Metadata** - aws:cdk:path, asset references
- **Stack Tags** - CDK stack-level tags
- **Environment** - Account/region extraction
- **Nested Stacks** - Nested stack resources

### Coming Soon
- **Pulumi** - Planned for future release

## Usage

### Parsing CDK Output

```rust
use costpilot::artifact::*;

// Parse CDK synthesized template
let parser = CdkParser::new();
let artifact = parser.parse_file("cdk.out/MyStack.template.json")?;

// Or parse entire CDK output directory
let artifacts = parser.parse_cdk_output("cdk.out/")?;

for artifact in artifacts {
    println!("Stack: {:?}", artifact.metadata.stack_name);
    println!("Resources: {}", artifact.resource_count());
}
```

### Auto-Detection

```rust
use costpilot::artifact::*;

// Auto-detect format from filename/content
let artifact = parse_artifact_file("MyStack.template.json")?;  // CDK
let artifact = parse_artifact_file("plan.json")?;  // Terraform (future)

// Check format
match artifact.format {
    ArtifactFormat::Cdk => println!("CDK stack"),
    ArtifactFormat::Terraform => println!("Terraform plan"),
    _ => println!("Other format"),
}
```

### Normalizing to Terraform Format

```rust
use costpilot::artifact::*;

// Parse CDK template
let artifact = parse_artifact_file("cdk.out/MyStack.template.json")?;

// Normalize to Terraform-like format
let normalized = ArtifactNormalizer::normalize(&artifact);

// Now works with existing CostPilot engines
println!("Created resources: {}", normalized.created_resources().len());

// Convert to Terraform plan JSON
let tf_plan = normalized.to_terraform_plan();
```

### Working with Resources

```rust
// Get specific resource
if let Some(instance) = artifact.get_resource("MyInstance") {
    println!("Type: {}", instance.resource_type);
    println!("Normalized: {}", instance.normalized_type());

    // Get properties
    if let Some(instance_type) = instance.get_property_string("InstanceType") {
        println!("Instance type: {}", instance_type);
    }
}

// Get all resources of a type
let instances = artifact.get_resources_by_type("aws_ec2_instance");
println!("Found {} EC2 instances", instances.len());

// Count by type
let counts = artifact.count_by_type();
for (type_name, count) in counts {
    println!("{}: {}", type_name, count);
}
```

### Handling Intrinsic Functions

```rust
use costpilot::artifact::IntrinsicFunction;

// Intrinsic functions are automatically parsed
let value = json!({"Ref": "MyParameter"});
// Becomes: "${MyParameter}"

let value = json!({"Fn::GetAtt": ["MyInstance", "PublicIp"]});
// Becomes: "${MyInstance.PublicIp}"

let value = json!({"Fn::Join": ["-", ["prefix", "suffix"]]});
// Becomes: "prefix-suffix"
```

## Integration with CostPilot Engines

### Detection Engine

```rust
use costpilot::engines::detection::*;
use costpilot::artifact::*;

// Parse CDK template
let artifact = parse_artifact_file("template.json")?;

// Normalize to common format
let normalized = ArtifactNormalizer::normalize(&artifact);

// Use with detection engine
let detector = DetectionEngine::new();
let detections = detector.detect_from_normalized(&normalized)?;

for detection in detections {
    println!("Found: {} ({})", detection.resource_id, detection.change_type);
}
```

### Cost Estimation

```rust
use costpilot::engines::prediction::*;
use costpilot::artifact::*;

// Parse and normalize
let artifact = parse_artifact_file("template.json")?;
let normalized = ArtifactNormalizer::normalize(&artifact);

// Estimate costs
let predictor = PredictionEngine::new();
let estimate = predictor.estimate_from_normalized(&normalized)?;

println!("Monthly: ${:.2}", estimate.monthly);
println!("Yearly: ${:.2}", estimate.yearly);
```

### Policy Validation

```rust
use costpilot::engines::policy::*;
use costpilot::artifact::*;

// Parse and normalize
let artifact = parse_artifact_file("template.json")?;
let normalized = ArtifactNormalizer::normalize(&artifact);

// Apply policies
let policy_engine = PolicyEngine::new(config);
let changes = convert_to_resource_changes(&normalized);
let result = policy_engine.evaluate(&changes, &estimate);

if result.has_violations() {
    println!("Policy violations found!");
}
```

## CDK Specifics

### CDK Output Structure

```
cdk.out/
├── manifest.json          # CDK assembly manifest
├── MyStack.template.json  # Synthesized template
├── AnotherStack.template.json
└── asset.*/               # Asset directories
```

### CDK Manifest

```json
{
  "version": "21.0.0",
  "artifacts": {
    "MyStack": {
      "type": "aws:cloudformation:stack",
      "environment": "aws://123456789012/us-east-1",
      "properties": {
        "templateFile": "MyStack.template.json",
        "tags": {
          "Environment": "production"
        }
      }
    }
  }
}
```

### CDK Resource Metadata

CDK adds metadata to resources:

```json
{
  "MyResource": {
    "Type": "AWS::Lambda::Function",
    "Properties": { ... },
    "Metadata": {
      "aws:cdk:path": "MyStack/MyFunction/Resource",
      "aws:asset:path": "asset.123abc",
      "aws:asset:is-bundled": true
    }
  }
}
```

### Parsing CDK Directory

```rust
let parser = CdkParser::new();

// Parse all stacks in cdk.out/
let artifacts = parser.parse_cdk_output("cdk.out/")?;

for artifact in artifacts {
    let stack_name = artifact.metadata.stack_name.unwrap();
    println!("Stack: {}", stack_name);
    println!("Region: {:?}", artifact.metadata.region);

    // CDK-specific tags
    for (key, value) in &artifact.metadata.tags {
        if key.starts_with("cdk_") {
            println!("  {}: {}", key, value);
        }
    }
}

// Get CDK assembly metadata
let assembly_meta = parser.parse_assembly_metadata("cdk.out/")?;
println!("CDK version: {}", assembly_meta.version);
println!("Runtime: {}", assembly_meta.runtime);
```

## Validation

### Artifact Validation

```rust
// Validate artifact structure
match artifact.validate() {
    Ok(_) => println!("Artifact is valid"),
    Err(e) => eprintln!("Validation error: {}", e),
}

// Checks performed:
// - No duplicate resource IDs
// - All dependencies exist
// - Required fields present
```

### Common Errors

```rust
use costpilot::artifact::ArtifactError;

match parse_artifact_file("template.json") {
    Err(ArtifactError::ParseError(msg)) => {
        // JSON/YAML parsing failed
    }
    Err(ArtifactError::InvalidResource(msg)) => {
        // Resource definition invalid
    }
    Err(ArtifactError::MissingField(field)) => {
        // Required field missing
    }
    Err(ArtifactError::InvalidVersion(version)) => {
        // Unsupported template version
    }
    Ok(artifact) => {
        // Success
    }
}
```

## Examples

### Example 1: CDK Lambda API

See `examples/cdk_lambda_api.template.json` for CDK example:
- Lambda function (Node.js 18, 512MB)
- DynamoDB table (on-demand billing)
- API Gateway HTTP API
- IAM roles and policies
- CDK metadata and asset references

**Estimated Monthly Cost**: ~$5-20 (depending on usage)

## Command-Line Usage

```bash
# Analyze CDK template
costpilot analyze cdk.out/MyStack.template.json

# Analyze entire CDK app
costpilot analyze cdk.out/

# With policy checks
costpilot analyze template.yaml --policies .costpilot/policy.yml

# Generate report
costpilot analyze template.json --output report.json
```

## Best Practices

### CDK

1. **Synthesize First** - Always run `cdk synth` before analyze
2. **Review Templates** - Check generated templates
3. **Use Constructs** - Leverage CDK construct patterns
4. **Tag Stacks** - Add stack-level tags
5. **Multiple Stacks** - Separate concerns into stacks
6. **Asset Management** - Understand CDK asset bundling

### Cost Optimization

1. **Right-Size Instances** - Start small, scale up
2. **Use Spot/Reserved** - For appropriate workloads
3. **Enable Auto-Scaling** - Scale based on demand
4. **Lifecycle Policies** - Move S3 data to cheaper tiers
5. **Multi-AZ Carefully** - Only for production/critical
6. **Monitor Costs** - Regular CostPilot analysis

## Limitations

### Current

- **YAML Support** - Requires `yaml` feature flag
- **Complex Intrinsics** - Some nested functions not fully resolved
- **Nested Stacks** - Single-level only (no recursive)
- **Transform** - SAM transforms not processed
- **Macros** - Template macros not expanded

### Planned Improvements (V2)

- **Terraform Support** - Unified with CDK
- **Pulumi Support** - Parse Pulumi program output
- **SAM Templates** - AWS SAM transform support
- **Nested Stack Recursion** - Deep nested stack analysis
- **Macro Expansion** - Process template macros
- **Change Sets** - Change set analysis
- **Cost History** - Track costs over template versions

## Test Coverage

**artifact_types.rs**: 15 tests
- Format name and support checking
- Resource type normalization (AWS:: → aws_)
- Artifact creation and resource management
- Resource filtering by type
- Resource counting by type
- Duplicate ID validation
- Missing dependency validation
- Intrinsic function resolution

**cdk_parser.rs**: 6 tests
- CDK synthesized template parsing
- Nested stack handling
- CDK output directory detection
- Metadata enhancement
- Lambda with assets
- Constructs with dependencies

**artifact_normalizer.rs**: 10 tests
- Name sanitization
- Property key normalization (PascalCase → snake_case)
- Ref function normalization
- GetAtt function normalization
- Join function normalization
- Full artifact normalization
- Multiple resource normalization
- Resource address building
- Terraform plan conversion

**Total**: 41 comprehensive tests

## Related Documentation

- [POLICY_ENGINE.md](POLICY_ENGINE.md) - Policy validation
- [DETECTION_ENGINE.md](DETECTION_ENGINE.md) - Change detection
- [PREDICTION_ENGINE.md](PREDICTION_ENGINE.md) - Cost estimation
- [examples/cdk_lambda_api.template.json](../examples/cdk_lambda_api.template.json)

## Troubleshooting

### "Unsupported artifact format"

Make sure the file has correct extension (.json, .yaml, .yml) or contains recognized structure.

### "Failed to parse as JSON or YAML"

Check template syntax.

### "Invalid template version"

Only template version "2010-09-09" is supported.

### "Missing required field"

Resources must have `Type` field. Check resource definitions.

### CDK manifest not found

Run `cdk synth` first to generate cdk.out/ directory.

## API Reference

### Core Types

- `Artifact` - Parsed IaC artifact
- `ArtifactFormat` - Format enum (Terraform/CDK/Pulumi)
- `ArtifactResource` - Single resource definition
- `ArtifactMetadata` - Source metadata
- `IntrinsicFunction` - Intrinsic functions

### Parsers

- `CdkParser` - Parse CDK output
- `ArtifactParser` trait - Parser interface

### Normalizer

- `ArtifactNormalizer` - Convert to common format
- `NormalizedPlan` - Unified representation
- `NormalizedResourceChange` - Normalized resource

### Utilities

- `parse_artifact_file()` - Auto-detect and parse
- `parse_artifact()` - Parse with hint
- `is_cdk_output_dir()` - Check if CDK output
- `find_cdk_templates()` - Find templates in directory
