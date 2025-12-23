#!/bin/bash
# CostPilot Peer Review Requirements Script
# Automated peer review requirement checking

set -e

echo "👥 Running CostPilot Peer Review Requirements"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Results directory
RESULTS_DIR="code-review-results"
mkdir -p "$RESULTS_DIR"

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
            echo -e "${BLUE}ℹ️  INFO${NC}: $message"
            ;;
    esac
}

# Function to check PR size and complexity
check_pr_size() {
    print_status "INFO" "Checking PR size and complexity..."

    # Get diff stats
    local insertions=0
    local deletions=0
    local files_changed=0

    local stat_output
    stat_output=$(git diff --cached --stat 2>/dev/null | tail -1 2>/dev/null || echo "")
    if [ -n "$stat_output" ]; then
        local ins_count
        ins_count=$(echo "$stat_output" | awk '{print $4}' 2>/dev/null | sed 's/+//' | sed 's/,//' || echo 0)
        insertions=$(printf '%d' "$ins_count" 2>/dev/null || echo 0)

        local del_count
        del_count=$(echo "$stat_output" | awk '{print $6}' 2>/dev/null | sed 's/-//' | sed 's/,//' || echo 0)
        deletions=$(printf '%d' "$del_count" 2>/dev/null || echo 0)
    fi

    local files_count
    files_count=$(git diff --cached --name-only 2>/dev/null | wc -l 2>/dev/null || echo 0)
    files_changed=$(printf '%d' "$files_count" 2>/dev/null || echo 0)

    local total_lines=$((insertions + deletions))

    echo "{\"insertions\": $insertions, \"deletions\": $deletions, \"files_changed\": $files_changed, \"total_lines\": $total_lines}" > "$RESULTS_DIR/pr-size.json"

    # Check PR size limits
    if [ $total_lines -gt 1000 ]; then
        print_status "FAIL" "PR too large: $total_lines lines changed (max 1000 recommended)"
    elif [ $total_lines -gt 500 ]; then
        print_status "WARN" "PR is large: $total_lines lines changed (consider splitting)"
    else
        print_status "PASS" "PR size acceptable: $total_lines lines changed"
    fi

    if [ $files_changed -gt 20 ]; then
        print_status "FAIL" "Too many files changed: $files_changed (max 20 recommended)"
    elif [ $files_changed -gt 10 ]; then
        print_status "WARN" "Many files changed: $files_changed (consider splitting)"
    else
        print_status "PASS" "File count acceptable: $files_changed files changed"
    fi
}

# Function to check for required reviewers
check_required_reviewers() {
    print_status "INFO" "Checking required reviewers..."

    # This would typically integrate with GitHub API
    # For now, we'll check for certain file changes that require specific reviewers

    local requires_security_reviewer=false
    local requires_performance_reviewer=false
    local requires_api_reviewer=false

    # Check for security-related changes
    local requires_security_reviewer_json="false"
    if git diff --cached --name-only 2>/dev/null | grep -q -E "\.(toml|lock)$|security|auth|crypto|encrypt"; then
        requires_security_reviewer=true
        requires_security_reviewer_json="true"
    fi

    # Check for performance-critical changes
    local requires_performance_reviewer_json="false"
    if git diff --cached --name-only 2>/dev/null | grep -q -E "engine|performance|benchmark|optimize"; then
        requires_performance_reviewer=true
        requires_performance_reviewer_json="true"
    fi

    # Check for API changes
    local requires_api_reviewer_json="false"
    if git diff --cached --name-only 2>/dev/null | grep -q -E "api|interface|protocol|contract"; then
        requires_api_reviewer=true
        requires_api_reviewer_json="true"
    fi

    echo "{\"requires_security_reviewer\": $requires_security_reviewer_json, \"requires_performance_reviewer\": $requires_performance_reviewer_json, \"requires_api_reviewer\": $requires_api_reviewer_json}" > "$RESULTS_DIR/required-reviewers.json"

    if $requires_security_reviewer; then
        print_status "INFO" "Security reviewer required for this PR"
    fi
    if $requires_performance_reviewer; then
        print_status "INFO" "Performance reviewer required for this PR"
    fi
    if $requires_api_reviewer; then
        print_status "INFO" "API reviewer required for this PR"
    fi

    print_status "PASS" "Reviewer requirements analyzed"
}

# Function to check documentation updates
check_documentation_updates() {
    print_status "INFO" "Checking documentation updates..."

    local docs_updated=false
    local code_changes=false
    local requires_docs=false

    # Check if documentation was updated
    local docs_updated_json="false"
    if git diff --cached --name-only 2>/dev/null | grep -q -E "\.(md|txt|rst)$|docs/|README|CHANGELOG"; then
        docs_updated=true
        docs_updated_json="true"
    fi

    # Check if code was changed
    local code_changes_json="false"
    if git diff --cached --name-only 2>/dev/null | grep -q -E "\.(rs|js|ts|py|java|cpp|cxx|cc|c\+\+|h|hpp)$"; then
        code_changes=true
        code_changes_json="true"
    fi

    # Check if public APIs were changed
    local requires_docs_json="false"
    if git diff --cached 2>/dev/null | grep -q -E "^\s*pub "; then
        requires_docs=true
        requires_docs_json="true"
    fi

    # Check for new features (rough heuristic)
    if git diff --cached 2>/dev/null | grep -q -E "fn.*\{|impl.*\{|struct.*\{|enum.*\{"; then
        requires_docs=true
        requires_docs_json="true"
    fi

    echo "{\"docs_updated\": $docs_updated_json, \"code_changes\": $code_changes_json, \"requires_docs\": $requires_docs_json}" > "$RESULTS_DIR/documentation-check.json"

    if $code_changes && ! $docs_updated && $requires_docs; then
        print_status "FAIL" "Code changes detected but documentation not updated"
    elif $code_changes && ! $docs_updated; then
        print_status "WARN" "Code changes detected - consider updating documentation"
    else
        print_status "PASS" "Documentation status acceptable"
    fi
}

# Function to check test coverage
check_test_coverage() {
    print_status "INFO" "Checking test coverage requirements..."

    local has_tests=false
    local test_files=0
    local code_files=0

    # Count test files
    local test_files=0
    local code_files=0

    local test_count
    test_count=$(find . -name "*test*.rs" -o -name "*spec*.rs" 2>/dev/null | wc -l 2>/dev/null || echo 0)
    test_files=$(printf '%d' "$test_count" 2>/dev/null || echo 0)

    local code_count
    code_count=$(find src -name "*.rs" 2>/dev/null | wc -l 2>/dev/null || echo 0)
    code_files=$(printf '%d' "$code_count" 2>/dev/null || echo 0)

    local has_tests_json="false"
    if [ "$test_files" -gt 0 ]; then
        has_tests=true
        has_tests_json="true"
    fi

    # Check if new code has corresponding tests
    local new_functions=0
    local new_tests=0

    local fn_count
    fn_count=$(git diff --cached 2>/dev/null | grep -c "^+.*fn " 2>/dev/null || echo 0)
    new_functions=$(printf '%d' "$fn_count" 2>/dev/null || echo 0)

    local test_count
    test_count=$(git diff --cached 2>/dev/null | grep -c "^+.*#\[test\]" 2>/dev/null || echo 0)
    new_tests=$(printf '%d' "$test_count" 2>/dev/null || echo 0)

    echo "{\"has_tests\": $has_tests_json, \"test_files\": $test_files, \"code_files\": $code_files, \"new_functions\": $new_functions, \"new_tests\": $new_tests}" > "$RESULTS_DIR/test-coverage.json"

    if [ "$new_functions" -gt 0 ] && [ "$new_tests" -eq 0 ]; then
        print_status "FAIL" "New functions added but no corresponding tests"
    elif [ "$code_files" -gt 0 ] && [ "$test_files" -eq 0 ]; then
        print_status "FAIL" "No test files found in the codebase"
    elif [ "$new_functions" -gt 0 ] && [ "$new_tests" -gt 0 ]; then
        print_status "PASS" "New functions have corresponding tests"
    else
        print_status "PASS" "Test coverage requirements met"
    fi
}

# Function to check code style and formatting
check_code_style() {
    print_status "INFO" "Checking code style and formatting..."

    local formatting_issues=0
    local style_issues=0

    # Check if rustfmt is available and code is formatted
    if command -v rustfmt &> /dev/null; then
        if ! cargo fmt --check > /dev/null 2>&1; then
            formatting_issues=$((formatting_issues + 1))
            print_status "FAIL" "Code formatting issues detected - run 'cargo fmt'"
        else
            print_status "PASS" "Code formatting is correct"
        fi
    else
        print_status "WARN" "rustfmt not available - cannot check formatting"
    fi

    # Check for common style issues
    local long_lines=0
    local lines_count
    lines_count=$(find src -name "*.rs" 2>/dev/null | xargs -I {} sh -c 'if [ -f "{}" ]; then awk "length(\$0) > 100 {print NR \":\" \$0}" "{}"; fi' 2>/dev/null | wc -l 2>/dev/null || echo 0)
    long_lines=$(printf '%d' "$lines_count" 2>/dev/null || echo 0)

    if [ "$long_lines" -gt 0 ]; then
        style_issues=$((style_issues + long_lines))
        print_status "WARN" "Found $long_lines lines longer than 100 characters"
    fi

    # Check for trailing whitespace
    local trailing_ws=0
    local ws_count
    ws_count=$(git diff --cached 2>/dev/null | grep -c " $" 2>/dev/null || echo 0)
    trailing_ws=$(printf '%d' "$ws_count" 2>/dev/null || echo 0)

    if [ "$trailing_ws" -gt 0 ]; then
        style_issues=$((style_issues + trailing_ws))
        print_status "WARN" "Found $trailing_ws lines with trailing whitespace"
    fi

    echo "{\"formatting_issues\": $formatting_issues, \"style_issues\": $style_issues}" > "$RESULTS_DIR/code-style.json"

    if [ $((formatting_issues + style_issues)) -eq 0 ]; then
        print_status "PASS" "Code style and formatting acceptable"
    fi
}

# Function to check for breaking changes
check_breaking_changes() {
    print_status "INFO" "Checking for breaking changes..."

    local breaking_changes=false
    local breaking_reasons=""

    local breaking_changes_json="false"
    if git diff --cached 2>/dev/null | grep -q "^-.*pub "; then
        breaking_changes=true
        breaking_changes_json="true"
        breaking_reasons="$breaking_reasons, removed public API"
    fi

    # Check for changed function signatures
    if git diff --cached 2>/dev/null | grep -q "^-.*fn .*{" && git diff --cached 2>/dev/null | grep -q "^+.*fn .*{"; then
        breaking_changes=true
        breaking_changes_json="true"
        breaking_reasons="$breaking_reasons, changed function signature"
    fi

    # Check for removed files
    if git diff --cached --name-status 2>/dev/null | grep -q "^D"; then
        local deleted_files
        deleted_files=$(git diff --cached --name-status 2>/dev/null | grep "^D" | wc -l)
        breaking_changes=true
        breaking_changes_json="true"
        breaking_reasons="$breaking_reasons, $deleted_files files deleted"
    fi

    # Check for configuration changes
    if git diff --cached --name-only 2>/dev/null | grep -q -E "config|settings|\.toml$|\.yaml$|\.yml$"; then
        breaking_reasons="$breaking_reasons, configuration changes"
    fi

    # Remove leading comma and space
    breaking_reasons=$(echo "$breaking_reasons" | sed 's/^, //')

    echo "{\"breaking_changes\": $breaking_changes_json, \"reasons\": \"$breaking_reasons\"}" > "$RESULTS_DIR/breaking-changes.json"

    if $breaking_changes; then
        print_status "WARN" "Breaking changes detected: $breaking_reasons"
    else
        print_status "PASS" "No breaking changes detected"
    fi
}

# Function to check commit message quality
check_commit_quality() {
    print_status "INFO" "Checking commit message quality..."

    local commit_msg
    commit_msg=$(git log --oneline -1 2>/dev/null | head -1 || echo "")

    local msg_length=${#commit_msg}
    local has_type=false
    local has_description=false

    local has_type_json="false"
    if echo "$commit_msg" | grep -q -E "^(feat|fix|docs|style|refactor|test|chore|perf|ci|build|revert)(\(.+\))?: "; then
        has_type=true
        has_type_json="true"
    fi

    local has_description_json="false"
    if [ $msg_length -gt 10 ]; then
        has_description=true
        has_description_json="true"
    fi

    echo "{\"commit_message\": \"$commit_msg\", \"length\": $msg_length, \"has_type\": $has_type_json, \"has_description\": $has_description_json}" > "$RESULTS_DIR/commit-quality.json"

    if ! $has_type; then
        print_status "WARN" "Commit message should follow conventional format (feat/fix/docs/etc)"
    fi

    if ! $has_description; then
        print_status "WARN" "Commit message is too short"
    fi

    if $has_type && $has_description; then
        print_status "PASS" "Commit message quality acceptable"
    fi
}

# Function to generate peer review report
generate_peer_review_report() {
    print_status "INFO" "Generating peer review requirements report..."

    local total_issues=0
    local blockers=0

    # Count blockers (FAIL status items)
    if [ -f "$RESULTS_DIR/pr-size.json" ]; then
        local total_lines
        total_lines=$(jq '.total_lines' "$RESULTS_DIR/pr-size.json" 2>/dev/null || echo "0")
        if [ $total_lines -gt 1000 ]; then
            blockers=$((blockers + 1))
        fi
    fi

    if [ -f "$RESULTS_DIR/test-coverage.json" ]; then
        local new_functions
        local new_tests
        new_functions=$(jq '.new_functions' "$RESULTS_DIR/test-coverage.json" 2>/dev/null || echo "0")
        new_tests=$(jq '.new_tests' "$RESULTS_DIR/test-coverage.json" 2>/dev/null || echo "0")
        if [ $new_functions -gt 0 ] && [ $new_tests -eq 0 ]; then
            blockers=$((blockers + 1))
        fi
    fi

    if [ -f "$RESULTS_DIR/code-style.json" ]; then
        local formatting_issues
        formatting_issues=$(jq '.formatting_issues' "$RESULTS_DIR/code-style.json" 2>/dev/null || echo "0")
        if [ $formatting_issues -gt 0 ]; then
            blockers=$((blockers + 1))
        fi
    fi

    total_issues=$blockers

    # Create JSON report
    cat > "$RESULTS_DIR/peer-review-requirements-report.json" << EOF
{
  "peer_review_requirements": {
    "timestamp": "$(date -Iseconds)",
    "commit": "$(git rev-parse HEAD)",
    "total_issues": $total_issues,
    "blockers": $blockers,
    "checks_performed": [
      "pr_size_analysis",
      "required_reviewers",
      "documentation_updates",
      "test_coverage",
      "code_style",
      "breaking_changes",
      "commit_quality"
    ],
    "recommendations": [
      "Ensure appropriate reviewers are assigned",
      "Keep PRs small and focused",
      "Add tests for new functionality",
      "Follow coding standards and formatting",
      "Update documentation for API changes",
      "Use conventional commit messages"
    ]
  }
}
EOF

    if [ $blockers -eq 0 ]; then
        print_status "PASS" "Peer review requirements met - PR ready for review"
    else
        print_status "FAIL" "Peer review requirements not met - $blockers blockers found"
    fi
}

# Main execution
main() {
    # Ensure results directory exists
    mkdir -p "$RESULTS_DIR"

    check_pr_size
    check_required_reviewers
    check_documentation_updates
    check_test_coverage
    check_code_style
    check_breaking_changes
    check_commit_quality
    generate_peer_review_report

    print_status "INFO" "Peer review requirements check completed. Results saved to $RESULTS_DIR/"
}

main "$@"
