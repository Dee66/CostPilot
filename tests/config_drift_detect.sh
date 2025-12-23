#!/bin/bash
# Configuration Drift Detection Script

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

# Detect configuration drift in environment files
detect_config_drift() {
    log_info "Detecting configuration drift..."

    # Compare current configs with baseline
    local baseline_dir="$PROJECT_ROOT/tests/fixtures/baselines"
    local current_configs="$PROJECT_ROOT/tests/environments"

    if [[ ! -d "$baseline_dir" ]]; then
        log_warning "No baseline configurations found for drift detection"
        return 0
    fi

    # Compare environment configurations
    for env in local staging production; do
        local baseline_file="$baseline_dir/${env}_config.yml"
        local current_file="$current_configs/$env/config.yml"

        if [[ -f "$baseline_file" && -f "$current_file" ]]; then
            if ! diff "$baseline_file" "$current_file" >/dev/null 2>&1; then
                log_warning "Configuration drift detected in $env environment"
                log_info "Differences:"
                diff "$baseline_file" "$current_file" || true
            else
                log_success "No configuration drift detected in $env environment"
            fi
        else
            log_info "Baseline or current config missing for $env environment"
        fi
    done
}

# Validate configuration consistency
validate_config_consistency() {
    log_info "Validating configuration consistency..."

    # Check that all environments have required sections
    local required_sections=("environment" "logging" "security")

    for env in local staging production; do
        local config_file="$PROJECT_ROOT/tests/environments/$env/config.yml"

        if [[ ! -f "$config_file" ]]; then
            log_error "Configuration file missing for $env environment"
            continue
        fi

        for section in "${required_sections[@]}"; do
            if ! grep -q "^$section:" "$config_file"; then
                log_error "Required section '$section' missing in $env environment config"
            fi
        done

        log_success "Configuration consistency validated for $env environment"
    done
}

# Check for configuration policy violations
check_policy_violations() {
    log_info "Checking for configuration policy violations..."

    # Check for hardcoded secrets
    local config_files=$(find "$PROJECT_ROOT/tests/environments" -name "*.yml" -o -name "*.yaml")

    for config_file in $config_files; do
        # Look for potential secrets
        if grep -q "password\|secret\|key\|token" "$config_file" | grep -v "\${.*}"; then
            log_warning "Potential hardcoded secrets found in $config_file"
            log_info "Please ensure secrets are properly externalized"
        fi
    done

    # Check resource limits
    for env in local staging production; do
        local config_file="$PROJECT_ROOT/tests/environments/$env/config.yml"

        if [[ -f "$config_file" ]]; then
            # Check for reasonable resource limits
            if grep -q "memory.*[0-9]\+[GT]B" "$config_file" >/dev/null 2>&1; then
                log_warning "Large memory allocation detected in $env environment"
            fi
        fi
    done

    log_success "Policy violation check completed"
}

# Generate configuration baseline
generate_baseline() {
    log_info "Generating configuration baseline..."

    local baseline_dir="$PROJECT_ROOT/tests/fixtures/baselines"

    mkdir -p "$baseline_dir"

    # Create baseline copies of current configurations
    for env in local staging production; do
        local source_file="$PROJECT_ROOT/tests/environments/$env/config.yml"
        local baseline_file="$baseline_dir/${env}_config.yml"

        if [[ -f "$source_file" ]]; then
            cp "$source_file" "$baseline_file"
            log_success "Baseline created for $env environment"
        else
            log_warning "Source configuration not found for $env environment"
        fi
    done

    # Create timestamp file
    echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$baseline_dir/baseline_timestamp.txt"
    log_success "Configuration baseline generated"
}

# Main function
main() {
    local command=${1:-check}

    log_info "Starting Configuration Drift Detection..."
    log_warning "⚠️  SAFETY NOTICE: This tool only analyzes configurations."
    log_warning "⚠️  NO infrastructure changes are made."

    case $command in
        check)
            detect_config_drift
            validate_config_consistency
            check_policy_violations
            ;;
        baseline)
            generate_baseline
            ;;
        validate)
            validate_config_consistency
            check_policy_violations
            ;;
        *)
            log_error "Unknown command: $command"
            echo "Usage: $0 [check|baseline|validate]"
            echo "  check    - Check for configuration drift"
            echo "  baseline - Generate new configuration baseline"
            echo "  validate - Validate configuration consistency and policies"
            exit 1
            ;;
    esac

    log_success "Configuration drift detection completed"
}

# Run main function with all arguments
main "$@"
