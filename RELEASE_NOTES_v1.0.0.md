# CostPilot v1.0.0 - Production Release

**Release Date:** January 8, 2026

## 🎉 First Production Release

CostPilot v1.0.0 is the first production-ready release of the infrastructure cost analysis and prediction tool. This release includes a complete license system, premium features, and comprehensive testing.

## ✨ Key Features

### Core Functionality
- **Cost Scanning**: Analyze Terraform plan files for cost implications
- **Drift Detection**: Identify unexpected cost changes between deployments
- **Policy Engine**: Enforce cost governance with customizable policies
- **SLO Monitoring**: Track Service Level Objectives for cost budgets
- **Grouping & Mapping**: Organize resources by service, environment, or custom tags

### Premium Features (License Required)
- **Diff Analysis**: Compare infrastructure cost changes between plan files
- **Autofix Engine**: Automated remediation suggestions with drift-safe mode
- **Advanced Predictions**: ML-based cost forecasting
- **Map Command**: Visual cost attribution and breakdown
- **Trend Analysis**: Historical cost tracking and visualization

### License System
- Ed25519-based cryptographic license validation
- Dual issuer support (production and testing)
- Rate-limited validation to prevent abuse
- Secure license issuance tooling included

## 🛠️ Technical Details

### Build Information
- Rust 1.75+
- Optimized for size (profile: `opt-level = "z"`)
- Stripped binaries for minimal footprint
- LTO enabled for performance

### Platform Support
- **Linux x86_64**: Fully supported and tested (this release)
- **Windows x86_64**: Coming soon (requires Windows build environment)
- **macOS ARM64**: Coming soon (requires macOS build environment)

### Testing
- **1,826 passing tests** covering:
  - Unit tests for all core engines
  - Integration tests for CLI commands
  - Security tests for authentication and validation
  - Property-based testing for parsers
  - Golden snapshot tests for output stability

## 📦 Installation

### Linux (x86_64)

```bash
# Download and extract
wget https://github.com/Dee66/CostPilot/releases/download/v1.0.0/costpilot-1.0.0-linux-amd64.tar.gz
tar -xzf costpilot-1.0.0-linux-amd64.tar.gz
cd costpilot-1.0.0-linux-amd64

# Verify checksum (optional)
sha256sum -c ../sha256sum.txt

# Install
sudo mv bin/costpilot /usr/local/bin/
sudo chmod +x /usr/local/bin/costpilot

# Verify installation
costpilot --version
```

### ZIP Archive (Windows-compatible)

```bash
unzip costpilot-1.0.0-linux-amd64.zip
cd costpilot-1.0.0-linux-amd64/bin
./costpilot --version
```

## 🚀 Quick Start

```bash
# Scan a Terraform plan
costpilot scan plan.json

# Apply cost policies
costpilot policy enforce configs/policies/cost-limits.yml plan.json

# Check SLO compliance
costpilot slo-check --config configs/slo/budget.yml

# Premium: Compare two plans (requires license)
costpilot diff old-plan.json new-plan.json
```

## 📋 Included Files

Each release bundle contains:
- `bin/costpilot` - The main executable
- `README.md` - Quick start guide
- `LICENSE` - MIT License
- `sbom.spdx.json` - Software Bill of Materials

## 🔐 License Activation

To activate Premium features:

1. Obtain a license from the CostPilot team
2. Place `license.json` in `~/.costpilot/`
3. Verify activation: `costpilot scan --help` (Premium commands will be listed)

For license issuance documentation, see `docs/LICENSE_OPERATIONS.md` in the repository.

## 📊 Checksums

**SHA256:**
```
bc1459220a856abcd33d179af780bc5712d770f6cd538c90526c644f620135c0  costpilot-1.0.0-linux-amd64.tar.gz
e4aa6cc969a15af5be8aba4b0928b4a18361a94fd8e4183579ad3b3d69fb8b14  costpilot-1.0.0-linux-amd64.zip
```

## 🐛 Known Issues

- ARM64 Linux builds require cross-compilation toolchain setup (not included in this release)
- Windows builds require MSVC toolchain (will be provided in future releases)
- macOS builds require Xcode on macOS host (will be provided in future releases)

## 📚 Documentation

- [Quick Start Guide](docs/quickstart.md)
- [CLI Reference](docs/cli_reference.md)
- [Policy Guide](docs/policies_guide.md)
- [SLO Guide](docs/slo_guide.md)
- [License Operations](docs/LICENSE_OPERATIONS.md)

## 🙏 Acknowledgments

Built with Rust and love by the GuardSuite team.

## 📄 License

CostPilot is licensed under the MIT License. See LICENSE file for details.

---

**Download:** [GitHub Releases](https://github.com/Dee66/CostPilot/releases/tag/v1.0.0)

**Repository:** https://github.com/Dee66/CostPilot

**Issues:** https://github.com/Dee66/CostPilot/issues
