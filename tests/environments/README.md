# Test Environments

This directory contains configurations and scripts for managing different test environments for CostPilot.

## Environment Types

### Local Environment
- **Purpose**: Development and unit testing
- **Infrastructure**: LocalStack for AWS services, SQLite database
- **Setup**: `./manage.sh setup local`
- **Testing**: `./manage.sh test local`
- **Cleanup**: `./manage.sh teardown local`

### Staging Environment
- **Purpose**: Integration testing and pre-production validation
- **Infrastructure**: Cloud-based VPC with PostgreSQL, CloudWatch monitoring
- **Setup**: `./manage.sh setup staging`
- **Testing**: `./manage.sh test staging`
- **Cleanup**: `./manage.sh teardown staging`

### Production Simulation Environment
- **Purpose**: Production-like testing with full scale simulation
- **Infrastructure**: Multi-region, high-availability setup with Aurora PostgreSQL
- **Setup**: `./manage.sh setup production-sim`
- **Testing**: `./manage.sh test production-sim`
- **Cleanup**: `./manage.sh teardown production-sim`

## Prerequisites

### Local Environment
- Docker (for LocalStack)
- Rust toolchain

### Staging Environment
- AWS CLI configured
- Terraform >= 1.0
- Rust toolchain

### Production Simulation Environment
- AWS CLI configured for multiple regions
- Terraform >= 1.0
- Rust toolchain
- Production data generation script

## Configuration Files

Each environment has a `config.yml` file that defines:
- Environment settings
- Database configuration
- External service endpoints
- Logging and monitoring setup
- Security policies
- Resource limits

## Usage Examples

```bash
# Set up local environment
./manage.sh setup local

# Run tests in staging
./manage.sh test staging

# Clean up production simulation
./manage.sh teardown production-sim
```

## Environment Variables

The management script sets appropriate environment variables for each environment:
- `COSTPILOT_ENV`: Environment name
- `AWS_REGION` / `AWS_REGIONS`: AWS region configuration
- `DATABASE_URL`: Database connection string
- `AWS_ENDPOINT`: LocalStack endpoint (local only)

## Monitoring and Logging

- **Local**: Console output only
- **Staging**: CloudWatch logs and metrics
- **Production-sim**: DataDog monitoring with SLO tracking

## Security Considerations

- Local: No authentication/encryption
- Staging: Basic auth and encryption
- Production-sim: Full security (SAML, KMS, Vault)

## Cost Management

- Local: Free (LocalStack)
- Staging: ~$50-100/day
- Production-sim: ~$200-500/day

Monitor costs in AWS Cost Explorer and set up billing alerts.
