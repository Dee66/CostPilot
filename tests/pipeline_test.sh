#!/bin/bash
# Pipeline Testing Script

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

# Build reproducibility test
test_build_reproducibility() {
    log_info "Testing build reproducibility..."

    local temp_dir1=$(mktemp -d)
    local temp_dir2=$(mktemp -d)

    # Build in first directory
    cd "$temp_dir1"
    cp -r "$PROJECT_ROOT"/* .
    cargo build --release
    local hash1=$(sha256sum target/release/costpilot | cut -d' ' -f1)

    # Build in second directory
    cd "$temp_dir2"
    cp -r "$PROJECT_ROOT"/* .
    cargo build --release
    local hash2=$(sha256sum target/release/costpilot | cut -d' ' -f1)

    # Compare hashes
    if [[ "$hash1" == "$hash2" ]]; then
        log_success "Build reproducibility test passed"
        return 0
    else
        log_error "Build reproducibility test failed: hashes don't match"
        echo "Hash 1: $hash1"
        echo "Hash 2: $hash2"
        return 1
    fi
}

# Artifact integrity test
test_artifact_integrity() {
    log_info "Testing artifact integrity..."

    # Build release artifacts
    cd "$PROJECT_ROOT"
    cargo build --release

    # Check binary integrity
    if [[ ! -f "target/release/costpilot" ]]; then
        log_error "Release binary not found"
        return 1
    fi

    # Check if binary is executable
    if [[ ! -x "target/release/costpilot" ]]; then
        log_error "Release binary is not executable"
        return 1
    fi

    # Check if binary runs
    if ! timeout 10s ./target/release/costpilot --version >/dev/null 2>&1; then
        log_error "Release binary does not run correctly"
        return 1
    fi

    # Check for required symbols
    if ! nm target/release/costpilot | grep -q "main"; then
        log_error "Binary missing main symbol"
        return 1
    fi

    log_success "Artifact integrity test passed"
}

# Deployment automation test
test_deployment_automation() {
    log_info "Testing deployment automation..."

    # Test Docker build
    if ! docker build -t costpilot-test .; then
        log_error "Docker build failed"
        return 1
    fi

    # Test container runs
    if ! docker run --rm costpilot-test --version >/dev/null 2>&1; then
        log_error "Container does not run correctly"
        return 1
    fi

    # Cleanup
    docker rmi costpilot-test >/dev/null 2>&1 || true

    log_success "Deployment automation test passed"
}

# Rollback capability test
test_rollback_capability() {
    log_info "Testing rollback capability..."

    # This would test the rollback functionality
    # For now, just check that rollback scripts exist and are executable

    if [[ ! -f ".github/workflows/deployment-orchestration.yml" ]]; then
        log_error "Deployment orchestration workflow not found"
        return 1
    fi

    # Check for rollback job in workflow
    if ! grep -q "rollback" .github/workflows/deployment-orchestration.yml; then
        log_error "Rollback job not found in deployment workflow"
        return 1
    fi

    log_success "Rollback capability test passed"
}

# Pipeline performance test
test_pipeline_performance() {
    log_info "Testing pipeline performance..."

    local start_time=$(date +%s)

    # Run a quick build test
    cargo check

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    if (( duration > 300 )); then
        log_warning "Pipeline performance test: build took ${duration}s (should be < 300s)"
    else
        log_success "Pipeline performance test passed (${duration}s)"
    fi
}

# CI/CD integration test
test_cicd_integration() {
    log_info "Testing CI/CD integration..."

    # Check for required workflow files
    local required_workflows=(
        "test-orchestration.yml"
        "deployment-orchestration.yml"
        "tests.yml"
        "ci.yml"
    )

    for workflow in "${required_workflows[@]}"; do
        if [[ ! -f ".github/workflows/$workflow" ]]; then
            log_error "Required workflow file not found: $workflow"
            return 1
        fi
    done

    # Check workflow syntax (basic check)
    for workflow in "${required_workflows[@]}"; do
        if ! python3 -c "import yaml; yaml.safe_load(open('.github/workflows/$workflow'))" 2>/dev/null; then
            log_error "Invalid YAML syntax in workflow: $workflow"
            return 1
        fi
    done

    log_success "CI/CD integration test passed"
}

# Main test execution
main() {
    local test_type=${1:-all}
    local failed_tests=()

    log_info "Starting pipeline testing suite..."
    log_info "Test type: $test_type"

    case $test_type in
        reproducibility)
            test_build_reproducibility || failed_tests+=("reproducibility")
            ;;
        integrity)
            test_artifact_integrity || failed_tests+=("integrity")
            ;;
        deployment)
            test_deployment_automation || failed_tests+=("deployment")
            ;;
        rollback)
            test_rollback_capability || failed_tests+=("rollback")
            ;;
        performance)
            test_pipeline_performance || failed_tests+=("performance")
            ;;
        cicd)
            test_cicd_integration || failed_tests+=("cicd")
            ;;
        all)
            test_build_reproducibility || failed_tests+=("reproducibility")
            test_artifact_integrity || failed_tests+=("integrity")
            test_deployment_automation || failed_tests+=("deployment")
            test_rollback_capability || failed_tests+=("rollback")
            test_pipeline_performance || failed_tests+=("performance")
            test_cicd_integration || failed_tests+=("cicd")
            ;;
        *)
            log_error "Unknown test type: $test_type"
            echo "Usage: $0 [reproducibility|integrity|deployment|rollback|performance|cicd|all]"
            exit 1
            ;;
    esac

    # Report results
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All pipeline tests passed! ✅"
        exit 0
    else
        log_error "Pipeline tests failed: ${failed_tests[*]} ❌"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
