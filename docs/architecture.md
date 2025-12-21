# Architecture

## System Overview

CostPilot is organized into several key engines:

- **Detection Engine**: Identifies cost issues
- **Prediction Engine**: Forecasts cost impact
- **Explain Engine**: Provides root cause analysis
- **Autofix Engine**: Generates remediation patches
- **Policy Engine**: Enforces cost policies
- **Mapping Engine**: Visualizes dependencies

## Design Principles

1. Deterministic output
2. WASM-based sandboxing
3. Multi-IaC support (Terraform, CDK)
