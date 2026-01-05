#!/bin/bash
# CostPilot Pre-Push Validation Script
# Branch-aware local validation with FAST/FULL/HEAVY modes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

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

# Function to run FAST mode checks
run_fast_mode() {
    print_status "INFO" "Running FAST mode checks..."

    local start_time=$(date +%s)

    # Run static analysis
    if ! "$SCRIPT_DIR/static_analysis.sh"; then
        print_status "FAIL" "Static analysis failed"
        return 1
    fi

    # Run mental model contradiction detection
    if ! python3 "$SCRIPT_DIR/detect_mental_model_contradictions.py"; then
        print_status "FAIL" "Mental model contradiction detection failed"
        return 1
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    if [ $duration -gt 60 ]; then
        print_status "WARN" "FAST mode exceeded 60 seconds ($duration seconds)"
    else
        print_status "PASS" "FAST mode completed in $duration seconds"
    fi

    return 0
}

# Function to run FULL mode checks
run_full_mode() {
    print_status "INFO" "Running FULL mode checks..."

    # Run FAST checks first
    if ! run_fast_mode; then
        return 1
    fi

    # Additional FULL checks
    if ! "$SCRIPT_DIR/peer_review_requirements.sh"; then
        print_status "FAIL" "Peer review requirements check failed"
        return 1
    fi

    if ! "$SCRIPT_DIR/security_review.sh"; then
        print_status "FAIL" "Security review check failed"
        return 1
    fi

    print_status "PASS" "FULL mode checks completed"
    return 0
}

# Function to run HEAVY mode checks (manual only)
run_heavy_mode() {
    print_status "INFO" "Running HEAVY mode checks..."

    if ! "$SCRIPT_DIR/performance_review.sh"; then
        print_status "FAIL" "Performance review check failed"
        return 1
    fi

    print_status "PASS" "HEAVY mode checks completed"
    return 0
}

# Main function
main() {
    local mode=${1:-FAST}

    cd "$PROJECT_ROOT"

    case $mode in
        FAST)
            if ! run_fast_mode; then
                print_status "FAIL" "Pre-push validation (FAST mode) failed"
                exit 1
            fi
            ;;
        FULL)
            if ! run_full_mode; then
                print_status "FAIL" "Pre-push validation (FULL mode) failed"
                exit 1
            fi
            ;;
        HEAVY)
            if ! run_heavy_mode; then
                print_status "FAIL" "Pre-push validation (HEAVY mode) failed"
                exit 1
            fi
            ;;
        *)
            print_status "FAIL" "Invalid mode: $mode. Use FAST, FULL, or HEAVY"
            exit 1
            ;;
    esac

    print_status "PASS" "All pre-push checks passed"
}

# Run main if not sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
