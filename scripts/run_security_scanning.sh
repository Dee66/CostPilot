#!/bin/bash
# CostPilot Local Security Scanning Runner
# For development and testing of security scanning

set -e

echo "🔒 CostPilot Local Security Scanning"
echo "===================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SECURITY_SCRIPT="$PROJECT_ROOT/scripts/security_review.sh"

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}Checking prerequisites...${NC}"

    local missing_deps=()

    if ! command -v cargo >/dev/null 2>&1; then
        missing_deps+=("cargo")
    fi

    if ! command -v git >/dev/null 2>&1; then
        missing_deps+=("git")
    fi

    if ! command -v jq >/dev/null 2>&1; then
        missing_deps+=("jq")
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo -e "${RED}Missing dependencies: ${missing_deps[*]}${NC}"
        echo "Please install them and try again."
        exit 1
    fi

    echo -e "${GREEN}Prerequisites OK${NC}"
}

# Function to install security tools
install_security_tools() {
    echo -e "${BLUE}Installing security tools...${NC}"

    # Install cargo-audit if not present
    if ! command -v cargo-audit >/dev/null 2>&1; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit --locked
    fi

    # Install cargo-deny if not present
    if ! command -v cargo-deny >/dev/null 2>&1; then
        echo "Installing cargo-deny..."
        cargo install cargo-deny --locked
    fi

    echo -e "${GREEN}Security tools ready${NC}"
}

# Function to run security review
run_security_review() {
    echo -e "${BLUE}Running security review...${NC}"

    cd "$PROJECT_ROOT"

    if [ ! -x "$SECURITY_SCRIPT" ]; then
        chmod +x "$SECURITY_SCRIPT"
    fi

    if "$SECURITY_SCRIPT"; then
        echo -e "${GREEN}Security review completed${NC}"
    else
        echo -e "${RED}Security review failed${NC}"
        exit 1
    fi
}

# Function to run dependency audit
run_dependency_audit() {
    echo -e "${BLUE}Running dependency audit...${NC}"

    cd "$PROJECT_ROOT"

    if command -v cargo-audit >/dev/null 2>&1; then
        cargo audit --format json > security-audit.json
        echo -e "${GREEN}Dependency audit completed${NC}"

        # Parse results
        local vuln_count
        vuln_count=$(jq '.vulnerabilities.count // 0' security-audit.json 2>/dev/null || echo "0")

        if [ "$vuln_count" -gt 0 ]; then
            echo -e "${RED}Found $vuln_count vulnerabilities in dependencies${NC}"
            jq '.vulnerabilities.list[]? | "\(.package.name) \(.package.version): \(.title) (Severity: \(.severity))"' security-audit.json 2>/dev/null || true
        else
            echo -e "${GREEN}No dependency vulnerabilities found${NC}"
        fi
    else
        echo -e "${YELLOW}cargo-audit not installed, skipping dependency audit${NC}"
    fi
}

# Function to run cargo deny checks
run_cargo_deny() {
    echo -e "${BLUE}Running cargo deny checks...${NC}"

    cd "$PROJECT_ROOT"

    if command -v cargo-deny >/dev/null 2>&1; then
        if cargo deny check --format json > deny-check.json 2>/dev/null; then
            echo -e "${GREEN}Cargo deny checks passed${NC}"
        else
            echo -e "${RED}Cargo deny checks failed${NC}"
            cat deny-check.json
        fi
    else
        echo -e "${YELLOW}cargo-deny not installed, skipping${NC}"
    fi
}

# Function to check for secrets
check_secrets() {
    echo -e "${BLUE}Checking for hardcoded secrets...${NC}"

    cd "$PROJECT_ROOT"

    local secret_patterns=(
        "password.*="
        "secret.*="
        "key.*="
        "token.*="
        "api_key.*="
        "aws_access_key_id.*="
        "aws_secret_access_key.*="
    )

    local secrets_found=0
    for pattern in "${secret_patterns[@]}"; do
        local matches
        matches=$(git ls-files | xargs grep -l "$pattern" 2>/dev/null | wc -l || echo "0")
        if [ "$matches" -gt 0 ]; then
            echo -e "${RED}Found potential secrets matching pattern: $pattern${NC}"
            git ls-files | xargs grep -l "$pattern" 2>/dev/null | head -5
            secrets_found=$((secrets_found + matches))
        fi
    done

    if [ $secrets_found -eq 0 ]; then
        echo -e "${GREEN}No hardcoded secrets detected${NC}"
    else
        echo -e "${RED}Found $secrets_found files with potential secrets${NC}"
    fi
}

# Function to run security linting
run_security_linting() {
    echo -e "${BLUE}Running security-focused linting...${NC}"

    cd "$PROJECT_ROOT"

    if cargo clippy -- -W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic -D clippy::print_stdout -D clippy::print_stderr > clippy-security.log 2>&1; then
        echo -e "${GREEN}Security linting passed${NC}"
    else
        echo -e "${RED}Security linting found issues${NC}"
        cat clippy-security.log | head -20
        echo "... (see clippy-security.log for full output)"
    fi
}

# Function to generate SBOM
generate_sbom() {
    echo -e "${BLUE}Generating Software Bill of Materials (SBOM)...${NC}"

    cd "$PROJECT_ROOT"

    if command -v jq >/dev/null 2>&1; then
        cargo metadata --format-version 1 --no-deps | jq '{
          spdxVersion: "SPDX-2.3",
          dataLicense: "CC0-1.0",
          SPDXID: "SPDXRef-DOCUMENT",
          name: "CostPilot SBOM",
          creationInfo: {
            created: now | strftime("%Y-%m-%dT%H:%M:%SZ"),
            creators: ["Tool: cargo-metadata"]
          },
          packages: .packages | map({
            SPDXID: ("SPDXRef-Package-" + .name),
            name: .name,
            versionInfo: .version,
            downloadLocation: (.source.repr // "NOASSERTION"),
            filesAnalyzed: false,
            copyrightText: "NOASSERTION",
            licenseConcluded: (.license // "NOASSERTION"),
            licenseDeclared: (.license // "NOASSERTION")
          })
        }' > sbom.json

        echo -e "${GREEN}SBOM generated: sbom.json${NC}"
        echo "Packages: $(jq '.packages | length' sbom.json)"
    else
        echo -e "${YELLOW}jq not available, skipping SBOM generation${NC}"
    fi
}

# Function to show security results
show_results() {
    echo -e "${BLUE}Security scan results:${NC}"

    cd "$PROJECT_ROOT"

    # Show security review results
    if [ -d "code-review-results" ]; then
        echo "Security review results:"
        ls -la code-review-results/

        if [ -f "code-review-results/security_vulnerabilities.count" ]; then
            echo "Security vulnerabilities: $(cat code-review-results/security_vulnerabilities.count)"
        fi
    fi

    # Show other results
    for file in security-audit.json deny-check.json clippy-security.log sbom.json; do
        if [ -f "$file" ]; then
            echo "Generated: $file ($(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "unknown") bytes)"
        fi
    done
}

# Function to run full security scan
run_full_scan() {
    echo -e "${BLUE}Running full security scan...${NC}"

    check_prerequisites
    install_security_tools
    run_security_review
    run_dependency_audit
    run_cargo_deny
    check_secrets
    run_security_linting
    generate_sbom

    echo -e "${GREEN}Full security scan completed${NC}"
    show_results
}

# Function to show help
show_help() {
    echo "CostPilot Local Security Scanning Runner"
    echo
    echo "Usage: $0 [COMMAND]"
    echo
    echo "Commands:"
    echo "  full      - Run complete security scan (default)"
    echo "  review    - Run security review script only"
    echo "  audit     - Run dependency audit only"
    echo "  deny      - Run cargo deny checks only"
    echo "  secrets   - Check for hardcoded secrets only"
    echo "  lint      - Run security linting only"
    echo "  sbom      - Generate SBOM only"
    echo "  results   - Show previous scan results"
    echo "  help      - Show this help message"
    echo
    echo "Examples:"
    echo "  $0 full                    # Complete security scan"
    echo "  $0 audit                   # Quick dependency check"
    echo "  $0 secrets                 # Check for secrets"
    echo "  $0 results                 # View previous results"
    echo
    echo "Required tools: cargo, git, jq"
    echo "Optional tools: cargo-audit, cargo-deny (auto-installed)"
}

# Main function
main() {
    local command=${1:-"full"}

    case $command in
        "full")
            run_full_scan
            ;;
        "review")
            check_prerequisites
            run_security_review
            ;;
        "audit")
            check_prerequisites
            install_security_tools
            run_dependency_audit
            ;;
        "deny")
            check_prerequisites
            install_security_tools
            run_cargo_deny
            ;;
        "secrets")
            check_prerequisites
            check_secrets
            ;;
        "lint")
            check_prerequisites
            run_security_linting
            ;;
        "sbom")
            check_prerequisites
            generate_sbom
            ;;
        "results")
            show_results
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

main "$@"
