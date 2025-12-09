# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Azure provider support
- GCP provider support
- Cost anomaly detection with ML
- Slack/Teams notifications
- Trend forecasting (3, 6, 12 months)
- Team cost allocation

## [1.0.0] - 2025-11-15

### Added
- Initial stable release of CostPilot
- **Cost Analysis Engine**
  - Real-time Terraform cost estimation
  - Resource-by-resource cost breakdown
  - Monthly and hourly cost projections
  - Support for AWS resources
- **Policy Engine**
  - Custom policy DSL for budget limits
  - Resource quotas and restrictions
  - Approval workflows with mandatory references
  - Time-bound exemptions with expiration tracking
  - CI blocking for expired exemptions
- **Drift Detection**
  - SHA256 checksum-based drift detection
  - Critical drift blocking (security, encryption, IAM)
  - Attribute-level granularity
  - Protected environment patterns
- **AI Predictions**
  - ML-based cost trend forecasting
  - Anomaly detection (beta)
  - Historical data learning
  - Confidence scoring
- **GitHub Actions Integration**
  - Composite action for CI/CD
  - Automated PR comments with cost analysis
  - Multi-platform binary downloads (Linux, macOS, x86_64, ARM64)
  - Configurable failure modes
- **CLI Tools**
  - `costpilot analyze` - Cost analysis command
  - `costpilot init` - Project initialization
  - `costpilot validate` - Policy validation
  - Markdown, JSON, and text output formats
- **SLO Monitoring**
  - Cost SLOs with error budgets
  - Burn rate alerts
  - Breach notifications
- **Documentation**
  - Comprehensive user guides
  - API reference documentation
  - Architecture documentation
  - Policy DSL reference
  - Examples and tutorials

### Security
- Zero-IAM design (no cloud credentials required)
- WASM sandbox for safe execution
- No telemetry or data collection
- Local-only processing
- Audit trail for all policy decisions

### Performance
- Rust implementation for speed
- <5s analysis for typical plans (10k+ resources)
- Efficient caching mechanisms
- Minimal memory footprint

## [0.9.0] - 2025-10-01

### Added
- Beta release for internal testing
- Core cost estimation engine
- Basic policy support
- GitHub Actions proof of concept

### Changed
- Migrated from prototype to production-ready architecture
- Refactored policy engine for extensibility

### Fixed
- Resolved parsing issues with complex Terraform modules
- Fixed cost calculation errors for RDS multi-AZ

## [0.5.0] - 2025-08-15

### Added
- Alpha release
- Proof of concept for Terraform parsing
- Initial AWS pricing integration
- Basic CLI interface

### Known Issues
- Limited AWS resource coverage
- No policy enforcement
- Manual baseline management

---

## Version Numbering

CostPilot follows [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Release Schedule

- **Patch releases**: As needed for critical bugs
- **Minor releases**: Every 4-6 weeks
- **Major releases**: Every 6-12 months

## Links

- [GitHub Releases](https://github.com/Dee66/CostPilot/releases)
- [Documentation](https://costpilot.dev/docs)
- [Issue Tracker](https://github.com/Dee66/CostPilot/issues)
