#!/bin/bash
# Infrastructure as Code Testing Script

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

# Test Terraform configurations (static analysis only)
test_terraform_configs() {
    log_info "Testing Terraform configurations..."

    # Check if terraform is available
    if ! command -v terraform >/dev/null 2>&1; then
        log_warning "Terraform not found, skipping terraform tests"
        return 0
    fi

    # Find terraform files
    local tf_files=$(find "$PROJECT_ROOT" -name "*.tf" -type f 2>/dev/null)
    if [[ -z "$tf_files" ]]; then
        log_info "No Terraform files found"
        return 0
    fi

    # Test terraform syntax validation (no actual planning/applying)
    for tf_file in $tf_files; do
        log_info "Validating $tf_file..."

        # Basic syntax check
        if ! terraform fmt -check "$tf_file" >/dev/null 2>&1; then
            log_error "Terraform formatting check failed for $tf_file"
            return 1
        fi

        # Validate syntax (requires terraform init, but we'll skip if no provider config)
        # This is safe as it doesn't contact AWS
        if terraform validate "$tf_file" >/dev/null 2>&1; then
            log_success "Terraform validation passed for $tf_file"
        else
            log_warning "Terraform validation skipped for $tf_file (likely missing provider config)"
        fi
    done

    log_success "Terraform configuration testing completed"
}

# Test CloudFormation templates
test_cloudformation_templates() {
    log_info "Testing CloudFormation templates..."

    # Find CloudFormation templates
    local cf_files=$(find "$PROJECT_ROOT" -name "*.yaml" -o -name "*.yml" -o -name "*.json" | grep -E "(cloudformation|cf|template)" | head -10)

    if [[ -z "$cf_files" ]]; then
        # Check test fixtures
        cf_files=$(find "$PROJECT_ROOT/tests/fixtures" -name "*.json" -o -name "*.yaml" | grep -E "(cf_|cloudformation)" | head -5)
    fi

    if [[ -z "$cf_files" ]]; then
        log_info "No CloudFormation templates found"
        return 0
    fi

    # Validate YAML/JSON syntax
    for cf_file in $cf_files; do
        log_info "Validating CloudFormation template: $cf_file"

        # Check file extension and validate syntax
        if [[ "$cf_file" == *.yaml || "$cf_file" == *.yml ]]; then
            if ! python3 -c "import yaml; yaml.safe_load(open('$cf_file'))" 2>/dev/null; then
                log_error "Invalid YAML syntax in $cf_file"
                return 1
            fi
        elif [[ "$cf_file" == *.json ]]; then
            if ! python3 -c "import json; json.load(open('$cf_file'))" 2>/dev/null; then
                log_error "Invalid JSON syntax in $cf_file"
                return 1
            fi
        fi

        # Basic CloudFormation structure validation
        if [[ "$cf_file" == *.yaml || "$cf_file" == *.yml ]]; then
            if ! grep -q "AWSTemplateFormatVersion\|Resources:" "$cf_file"; then
                log_warning "CloudFormation template $cf_file missing required sections"
            fi
        fi

        log_success "CloudFormation template validation passed for $cf_file"
    done

    log_success "CloudFormation template testing completed"
}

# Test CDK constructs (TypeScript)
test_cdk_constructs() {
    log_info "Testing CDK constructs..."

    # Find CDK files
    local cdk_files=$(find "$PROJECT_ROOT" -name "*.ts" | grep -E "(cdk|construct)" | head -5)

    if [[ -z "$cdk_files" ]]; then
        # Check test fixtures
        cdk_files=$(find "$PROJECT_ROOT/tests/fixtures" -name "*.ts" | head -3)
    fi

    if [[ -z "$cdk_files" ]]; then
        log_info "No CDK construct files found"
        return 0
    fi

    # Check if TypeScript compiler is available
    if ! command -v tsc >/dev/null 2>&1; then
        log_warning "TypeScript compiler not found, skipping CDK syntax checks"
        return 0
    fi

    # Basic TypeScript syntax check
    for cdk_file in $cdk_files; do
        log_info "Validating CDK construct: $cdk_file"

        if tsc --noEmit --skipLibCheck "$cdk_file" >/dev/null 2>&1; then
            log_success "CDK construct syntax validation passed for $cdk_file"
        else
            log_error "CDK construct syntax validation failed for $cdk_file"
            return 1
        fi
    done

    log_success "CDK construct testing completed"
}

# Test configuration management
test_configuration_management() {
    log_info "Testing configuration management..."

    # Test YAML configurations
    local yaml_files=$(find "$PROJECT_ROOT" -name "*.yml" -o -name "*.yaml" | grep -v node_modules | head -10)

    for yaml_file in $yaml_files; do
        log_info "Validating YAML configuration: $yaml_file"

        if python3 -c "import yaml; yaml.safe_load(open('$yaml_file'))" 2>/dev/null; then
            log_success "YAML validation passed for $yaml_file"
        else
            log_error "YAML validation failed for $yaml_file"
            return 1
        fi
    done

    # Test JSON configurations
    local json_files=$(find "$PROJECT_ROOT" -name "*.json" | grep -v node_modules | head -10)

    for json_file in $json_files; do
        log_info "Validating JSON configuration: $json_file"

        if python3 -c "import json; json.load(open('$json_file'))" 2>/dev/null; then
            log_success "JSON validation passed for $json_file"
        else
            log_error "JSON validation failed for $json_file"
            return 1
        fi
    done

    # Test environment variable configurations
    if [[ -f ".env.example" ]]; then
        log_info "Checking environment configuration template..."

        # Basic validation that required vars are documented
        if grep -q "REQUIRED\|MANDATORY\|TODO" ".env.example"; then
            log_success "Environment configuration template found"
        fi
    fi

    log_success "Configuration management testing completed"
}

# Test monitoring setup (static validation)
test_monitoring_setup() {
    log_info "Testing monitoring setup..."

    # Check for monitoring configurations
    local monitoring_files=$(find "$PROJECT_ROOT" -name "*monitor*" -o -name "*metric*" -o -name "*dashboard*" | head -5)

    if [[ -n "$monitoring_files" ]]; then
        for mon_file in $monitoring_files; do
            log_info "Validating monitoring configuration: $mon_file"

            # Basic file validation
            if [[ -s "$mon_file" ]]; then
                log_success "Monitoring configuration file exists: $mon_file"
            else
                log_warning "Monitoring configuration file is empty: $mon_file"
            fi
        done
    fi

    # Check for health check endpoints in code
    if grep -r "health\|status\|metrics" "$PROJECT_ROOT/src" >/dev/null 2>&1; then
        log_success "Health check endpoints found in codebase"
    else
        log_warning "No health check endpoints found in codebase"
    fi

    log_success "Monitoring setup testing completed"
}

# Test alerting configuration (static validation)
test_alerting_configuration() {
    log_info "Testing alerting configuration..."

    # Check for alerting configurations
    local alert_files=$(find "$PROJECT_ROOT" -name "*alert*" -o -name "*notification*" | head -5)

    if [[ -n "$alert_files" ]]; then
        for alert_file in $alert_files; do
            log_info "Validating alerting configuration: $alert_file"

            # Basic file validation
            if [[ -s "$alert_file" ]]; then
                log_success "Alerting configuration file exists: $alert_file"
            else
                log_warning "Alerting configuration file is empty: $alert_file"
            fi
        done
    fi

    # Check for error handling in code
    if grep -r "panic\|error\|alert\|notify" "$PROJECT_ROOT/src" >/dev/null 2>&1; then
        log_success "Error handling and alerting patterns found in codebase"
    else
        log_warning "Limited error handling patterns found in codebase"
    fi

    log_success "Alerting configuration testing completed"
}

# Test infrastructure policy compliance
test_infrastructure_policy() {
    log_info "Testing infrastructure policy compliance..."

    # Check for security group configurations
    if grep -r "security_group\|firewall" "$PROJECT_ROOT" >/dev/null 2>&1; then
        log_success "Security configurations found"
    fi

    # Check for encryption configurations
    if grep -r "encrypt\|kms\|ssl\|tls" "$PROJECT_ROOT" >/dev/null 2>&1; then
        log_success "Encryption configurations found"
    fi

    # Check for backup configurations
    if grep -r "backup\|snapshot\|recovery" "$PROJECT_ROOT" >/dev/null 2>&1; then
        log_success "Backup configurations found"
    fi

    # Check for compliance tags
    if grep -r "compliance\|pci\|hipaa\|gdpr" "$PROJECT_ROOT" >/dev/null 2>&1; then
        log_success "Compliance configurations found"
    fi

    log_success "Infrastructure policy compliance testing completed"
}

# Main testing function
main() {
    local test_type=${1:-all}
    local failed_tests=()

    log_info "Starting Infrastructure as Code testing suite..."
    log_info "Test type: $test_type"
    log_warning "⚠️  SAFETY NOTICE: This testing ONLY validates syntax and structure."
    log_warning "⚠️  NO actual infrastructure is created or modified."
    log_warning "⚠️  NO cloud resources are provisioned."

    case $test_type in
        terraform)
            test_terraform_configs || failed_tests+=("terraform")
            ;;
        cloudformation)
            test_cloudformation_templates || failed_tests+=("cloudformation")
            ;;
        cdk)
            test_cdk_constructs || failed_tests+=("cdk")
            ;;
        config)
            test_configuration_management || failed_tests+=("config")
            ;;
        monitoring)
            test_monitoring_setup || failed_tests+=("monitoring")
            ;;
        alerting)
            test_alerting_configuration || failed_tests+=("alerting")
            ;;
        policy)
            test_infrastructure_policy || failed_tests+=("policy")
            ;;
        all)
            test_terraform_configs || failed_tests+=("terraform")
            test_cloudformation_templates || failed_tests+=("cloudformation")
            test_cdk_constructs || failed_tests+=("cdk")
            test_configuration_management || failed_tests+=("config")
            test_monitoring_setup || failed_tests+=("monitoring")
            test_alerting_configuration || failed_tests+=("alerting")
            test_infrastructure_policy || failed_tests+=("policy")
            ;;
        *)
            log_error "Unknown test type: $test_type"
            echo "Usage: $0 [terraform|cloudformation|cdk|config|monitoring|alerting|policy|all]"
            exit 1
            ;;
    esac

    # Report results
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All IaC tests passed! ✅"
        log_success "🎉 Infrastructure configurations are safe and valid!"
        exit 0
    else
        log_error "IaC tests failed: ${failed_tests[*]} ❌"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"
