#!/bin/bash
# Test Environment Management Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Environment management functions
setup_local() {
    log_info "Setting up local test environment..."

    # Check if localstack is running
    if ! docker ps | grep -q localstack; then
        log_info "Starting LocalStack..."
        docker run -d --name localstack \
            -p 4566:4566 \
            -p 4572:4572 \
            -e SERVICES=lambda,dynamodb,s3,ec2,rds,iam \
            -e DEBUG=1 \
            localstack/localstack:2.0
        sleep 10
    fi

    # Set environment variables
    export COSTPILOT_ENV=local
    export AWS_ENDPOINT=http://localhost:4566
    export AWS_REGION=us-east-1
    export DATABASE_URL=sqlite::memory:

    log_success "Local environment ready"
}

setup_staging() {
    log_info "Setting up staging test environment..."

    # Check AWS credentials
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        log_error "AWS credentials not configured"
        exit 1
    fi

    # Deploy infrastructure
    cd "$PROJECT_ROOT/tests/environments/staging"
    terraform init
    terraform apply -auto-approve

    # Set environment variables
    export COSTPILOT_ENV=staging
    export AWS_REGION=us-west-2
    export DATABASE_URL=$(terraform output -raw database_url)

    log_success "Staging environment ready"
}

setup_production_sim() {
    log_info "Setting up production simulation environment..."

    # Check AWS credentials for multiple regions
    for region in us-east-1 us-west-2 eu-west-1; do
        if ! AWS_REGION=$region aws sts get-caller-identity >/dev/null 2>&1; then
            log_error "AWS credentials not configured for region $region"
            exit 1
        fi
    done

    # Deploy multi-region infrastructure
    cd "$PROJECT_ROOT/tests/environments/production"
    terraform init
    terraform apply -auto-approve

    # Generate production-like data
    "$PROJECT_ROOT/scripts/generate_prod_data.sh"

    # Set environment variables
    export COSTPILOT_ENV=production-sim
    export AWS_REGIONS="us-east-1,us-west-2,eu-west-1"

    log_success "Production simulation environment ready"
}

teardown_local() {
    log_info "Tearing down local environment..."

    # Stop LocalStack
    docker stop localstack >/dev/null 2>&1 || true
    docker rm localstack >/dev/null 2>&1 || true

    # Clean up temp files
    rm -f /tmp/costpilot_test_*.db

    log_success "Local environment cleaned up"
}

teardown_staging() {
    log_info "Tearing down staging environment..."

    cd "$PROJECT_ROOT/tests/environments/staging"
    terraform destroy -auto-approve

    log_success "Staging environment cleaned up"
}

teardown_production_sim() {
    log_warning "Production simulation teardown requires manual confirmation"
    read -p "Are you sure you want to destroy production simulation environment? (yes/no): " confirm

    if [[ $confirm == "yes" ]]; then
        cd "$PROJECT_ROOT/tests/environments/production"
        terraform destroy -auto-approve
        log_success "Production simulation environment cleaned up"
    else
        log_info "Teardown cancelled"
    fi
}

run_tests() {
    local env=$1
    log_info "Running tests in $env environment..."

    cd "$PROJECT_ROOT"

    case $env in
        local)
            cargo test --features local -- --test-threads 2
            ;;
        staging)
            cargo test --features staging -- --test-threads 4
            ;;
        production-sim)
            cargo test --features production-sim -- --test-threads 8
            ;;
        *)
            log_error "Unknown environment: $env"
            exit 1
            ;;
    esac

    log_success "Tests completed in $env environment"
}

# Main script logic
usage() {
    echo "Usage: $0 <command> <environment>"
    echo ""
    echo "Commands:"
    echo "  setup     - Set up test environment"
    echo "  teardown  - Tear down test environment"
    echo "  test      - Run tests in environment"
    echo ""
    echo "Environments:"
    echo "  local          - Local development environment"
    echo "  staging        - Cloud-based staging environment"
    echo "  production-sim - Production simulation environment"
    echo ""
    echo "Examples:"
    echo "  $0 setup local"
    echo "  $0 test staging"
    echo "  $0 teardown production-sim"
}

if [[ $# -ne 2 ]]; then
    usage
    exit 1
fi

command=$1
environment=$2

case $command in
    setup)
        case $environment in
            local)
                setup_local
                ;;
            staging)
                setup_staging
                ;;
            production-sim)
                setup_production_sim
                ;;
            *)
                log_error "Unknown environment: $environment"
                usage
                exit 1
                ;;
        esac
        ;;
    teardown)
        case $environment in
            local)
                teardown_local
                ;;
            staging)
                teardown_staging
                ;;
            production-sim)
                teardown_production_sim
                ;;
            *)
                log_error "Unknown environment: $environment"
                usage
                exit 1
                ;;
        esac
        ;;
    test)
        case $environment in
            local|staging|production-sim)
                run_tests $environment
                ;;
            *)
                log_error "Unknown environment: $environment"
                usage
                exit 1
                ;;
        esac
        ;;
    *)
        log_error "Unknown command: $command"
        usage
        exit 1
        ;;
esac
