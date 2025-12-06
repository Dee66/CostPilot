# CostPilot

Cost analysis and prediction for infrastructure as code.

## Features

- **Multi-IaC Support**: Terraform, CDK, CloudFormation
- **Cost Prediction**: Forecast infrastructure costs
- **Autofix**: Generate cost optimization patches
- **Policy Enforcement**: Custom cost policies
- **Dependency Mapping**: Visualize resource dependencies
- **SLO Tracking**: Cost service level objectives
- **Trend Analysis**: Track cost changes over time

## Quick Start

```bash
# Install
cargo install costpilot

# Scan your infrastructure
costpilot scan --plan terraform.plan.json

# Generate cost map
costpilot map --output diagram.mmd

# Check SLOs
costpilot slo-check --config slo.yml
```

## Documentation

- [Quickstart Guide](docs/quickstart.md)
- [CLI Reference](docs/cli_reference.md)
- [Architecture](docs/architecture.md)
- [Policies Guide](docs/policies_guide.md)

## Contributing

Contributions welcome! Please read our contributing guidelines.

## License

MIT License - see LICENSE file for details.
