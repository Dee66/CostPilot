#!/bin/bash
# CostPilot Static Analysis Script
# Comprehensive code quality and security analysis

set -e

echo "🧪 Running CostPilot Static Analysis"
echo "==================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✅ PASS${NC}: $message"
            ;;
        "FAIL")
            echo -e "${RED}❌ FAIL${NC}: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  WARN${NC}: $message"
            ;;
        "INFO")
            echo -e "ℹ️  INFO: $message"
            ;;
    esac
}

# Check if required tools are installed
check_tools() {
    print_status "INFO" "Checking required tools..."

    local missing_tools=()

    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi

    if ! command -v rustc &> /dev/null; then
        missing_tools+=("rustc")
    fi

    if ! command -v cargo-tarpaulin &> /dev/null; then
        print_status "WARN" "cargo-tarpaulin not found - code coverage checks will be skipped"
        print_status "INFO" "Install with: cargo install cargo-tarpaulin"
    fi

    if ! command -v cargo-audit &> /dev/null; then
        print_status "WARN" "cargo-audit not found - security audit will be skipped"
        print_status "INFO" "Install with: cargo install cargo-audit"
    fi

    if ! command -v cargo-outdated &> /dev/null; then
        print_status "WARN" "cargo-outdated not found - dependency checks will be skipped"
        print_status "INFO" "Install with: cargo install cargo-outdated"
    fi

    if ! command -v cargo-udeps &> /dev/null; then
        print_status "WARN" "cargo-udeps not found - unused dependency checks will be skipped"
        print_status "INFO" "Install with: cargo install cargo-udeps"
    fi

    if [ ${#missing_tools[@]} -gt 0 ]; then
        print_status "FAIL" "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi

    print_status "PASS" "All required tools are available"
}

# Run formatting checks
check_formatting() {
    print_status "INFO" "Checking code formatting..."

    if cargo fmt -- --check; then
        print_status "PASS" "Code formatting is correct"
    else
        print_status "FAIL" "Code formatting issues found"
        print_status "INFO" "Fix with: cargo fmt"
        return 1
    fi
}

# Run Clippy linting
check_clippy() {
    print_status "INFO" "Running Clippy linting..."

    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_status "PASS" "Clippy linting passed"
    else
        print_status "FAIL" "Clippy found issues"
        return 1
    fi
}

# Run tests
check_tests() {
    print_status "INFO" "Running tests..."

    if cargo test --verbose; then
        print_status "PASS" "All tests passed"
    else
        print_status "FAIL" "Some tests failed"
        return 1
    fi
}

# Check code coverage
check_coverage() {
    print_status "INFO" "Checking code coverage..."

    if command -v cargo-tarpaulin &> /dev/null; then
        if cargo tarpaulin --fail-under 90; then
            print_status "PASS" "Code coverage meets minimum requirements (90%)"
        else
            print_status "FAIL" "Code coverage below minimum threshold"
            return 1
        fi
    else
        print_status "WARN" "Skipping coverage check - cargo-tarpaulin not installed"
    fi
}

# Security audit
check_security() {
    print_status "INFO" "Running security audit..."

    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            print_status "PASS" "Security audit passed"
        else
            print_status "FAIL" "Security vulnerabilities found"
            return 1
        fi
    else
        print_status "WARN" "Skipping security audit - cargo-audit not installed"
    fi
}

# Check for outdated dependencies
check_dependencies() {
    print_status "INFO" "Checking for outdated dependencies..."

    if command -v cargo-outdated &> /dev/null; then
        if cargo outdated --exit-code 1; then
            print_status "PASS" "All dependencies are up to date"
        else
            print_status "WARN" "Outdated dependencies found"
            # Don't fail on outdated deps, just warn
        fi
    else
        print_status "WARN" "Skipping dependency check - cargo-outdated not installed"
    fi
}

# Check for unused dependencies
check_unused_deps() {
    print_status "INFO" "Checking for unused dependencies..."

    if command -v cargo-udeps &> /dev/null; then
        if cargo +nightly udeps; then
            print_status "PASS" "No unused dependencies found"
        else
            print_status "WARN" "Unused dependencies detected"
            # Don't fail on unused deps, just warn
        fi
    else
        print_status "WARN" "Skipping unused dependency check - cargo-udeps not installed"
    fi
}

# Check binary size
check_binary_size() {
    print_status "INFO" "Checking binary size..."

    # Build release binary
    if cargo build --release --quiet; then
        local binary_size
        binary_size=$(stat -f%z target/release/costpilot 2>/dev/null || stat -c%s target/release/costpilot 2>/dev/null || echo "0")

        if [ "$binary_size" -gt 0 ]; then
            local size_mb
            size_mb=$(( binary_size / 1024 / 1024 ))

            if [ $size_mb -lt 50 ]; then
                print_status "PASS" "Binary size is acceptable (${size_mb}MB)"
            else
                print_status "WARN" "Binary size is large (${size_mb}MB)"
            fi
        else
            print_status "WARN" "Could not determine binary size"
        fi
    else
        print_status "FAIL" "Failed to build release binary"
        return 1
    fi
}

# Check compilation warnings
check_compilation() {
    print_status "INFO" "Checking for compilation warnings..."

    # Capture warnings from cargo check
    local warnings
    warnings=$(cargo check 2>&1 | grep -c "warning:" || true)

    if [ "$warnings" -eq 0 ]; then
        print_status "PASS" "No compilation warnings"
    else
        print_status "WARN" "$warnings compilation warnings found"
        # Show the warnings
        cargo check 2>&1 | grep "warning:"
    fi
}

# Main execution
main() {
    local failed_checks=0

    # Change to the correct directory if running in CI
    if [ -d "products/costpilot" ]; then
        cd products/costpilot
        print_status "INFO" "Running in CI environment, changed to products/costpilot"
    fi

    check_tools

    # Critical checks (must pass)
    check_formatting || ((failed_checks++))
    check_clippy || ((failed_checks++))
    check_tests || ((failed_checks++))

    # Quality checks (warnings allowed)
    check_coverage
    check_security
    check_dependencies
    check_unused_deps
    check_binary_size
    check_compilation

    echo
    echo "==================================="
    if [ $failed_checks -eq 0 ]; then
        print_status "PASS" "All critical static analysis checks passed"
        exit 0
    else
        print_status "FAIL" "$failed_checks critical checks failed"
        exit 1
    fi
}

# Run main function
main "$@"
