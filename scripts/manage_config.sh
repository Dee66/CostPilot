#!/bin/bash
set -euo pipefail

# CostPilot configuration management
# Handles initialization, validation, and migration of configuration files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration directories
CONFIG_DIR="${HOME}/.costpilot"
CONFIG_FILE="${CONFIG_DIR}/config.toml"
LICENSE_FILE="${CONFIG_DIR}/license.key"
SLO_CONFIG="${CONFIG_DIR}/slo.json"
BASELINES_DIR="${CONFIG_DIR}/baselines"
SNAPSHOTS_DIR="${CONFIG_DIR}/snapshots"
AUDIT_LOG="${CONFIG_DIR}/audit_log.json"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create default configuration
create_default_config() {
    cat > "$CONFIG_FILE" << 'EOF'
# CostPilot Configuration
# This file contains CostPilot settings and preferences

[general]
# Verbose logging
verbose = false

# Debug mode
debug = false

# Output format (text, json, markdown, pr-comment)
format = "text"

[engine]
# Cost calculation precision
precision = 2

# Conservative estimation mode
conservative = true

# Enable heuristics
enable_heuristics = true

[security]
# License validation
require_license = true

# Audit logging
enable_audit = true

# WASM integrity checks
verify_wasm = true

[network]
# Zero-network mode (no external API calls)
zero_network = true

# Offline mode
offline = true

[performance]
# Benchmark collection
collect_benchmarks = false

# Performance monitoring
enable_monitoring = false

[enterprise]
# SLO monitoring
enable_slo = false

# Policy enforcement
enable_policies = false

# Audit compliance
enable_compliance = false
EOF
}

# Create default SLO configuration
create_default_slo() {
    cat > "$SLO_CONFIG" << 'EOF'
{
  "version": "1.0",
  "slos": [
    {
      "id": "monthly_budget",
      "name": "Monthly Cost Budget",
      "description": "Monthly infrastructure cost limit",
      "slo_type": "monthly_budget",
      "target": "global",
      "threshold": {
        "max_value": 1000.0
      },
      "enforcement": "warn",
      "owner": "platform-team",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
EOF
}

# Validate configuration file
validate_config() {
    local config_file="$1"

    if [[ ! -f "$config_file" ]]; then
        log_error "Configuration file not found: $config_file"
        return 1
    fi

    # Basic TOML validation (check if it parses)
    if command -v python3 >/dev/null 2>&1; then
        if ! python3 -c "import tomllib; tomllib.load(open('$config_file', 'rb'))" 2>/dev/null; then
            log_error "Invalid TOML syntax in configuration file"
            return 1
        fi
    fi

    log_success "Configuration file is valid"
    return 0
}

# Validate license file
validate_license() {
    if [[ ! -f "$LICENSE_FILE" ]]; then
        log_warn "No license file found. CostPilot will run in free mode."
        return 0
    fi

    # Check file permissions (should not be world-readable)
    local perms
    perms="$(stat -c '%a' "$LICENSE_FILE" 2>/dev/null || stat -f '%A' "$LICENSE_FILE" 2>/dev/null || echo "unknown")"

    if [[ "$perms" == "unknown" ]]; then
        log_warn "Could not check license file permissions"
    elif [[ "${perms: -1}" != "0" ]]; then
        log_warn "License file is world-readable. Consider: chmod 600 $LICENSE_FILE"
    fi

    # Basic license format validation
    local license_content
    license_content="$(head -1 "$LICENSE_FILE" 2>/dev/null || echo "")"

    if [[ -z "$license_content" ]]; then
        log_error "License file is empty"
        return 1
    fi

    # Check if it looks like a license (basic heuristic)
    if [[ ! "$license_content" =~ ^[A-Za-z0-9+/=]{20,}$ ]]; then
        log_error "License file does not contain valid license data"
        return 1
    fi

    log_success "License file is present and appears valid"
    return 0
}

# Initialize configuration
init_config() {
    log_info "Initializing CostPilot configuration..."

    # Create configuration directory
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$BASELINES_DIR"
    mkdir -p "$SNAPSHOTS_DIR"

    # Set proper permissions
    chmod 700 "$CONFIG_DIR"
    chmod 700 "$BASELINES_DIR"
    chmod 700 "$SNAPSHOTS_DIR"

    # Create default configuration if it doesn't exist
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_info "Creating default configuration..."
        create_default_config
        log_success "Default configuration created at: $CONFIG_FILE"
    else
        log_info "Configuration already exists"
    fi

    # Create default SLO config if it doesn't exist
    if [[ ! -f "$SLO_CONFIG" ]]; then
        log_info "Creating default SLO configuration..."
        create_default_slo
        log_success "Default SLO configuration created at: $SLO_CONFIG"
    fi

    # Initialize audit log if it doesn't exist
    if [[ ! -f "$AUDIT_LOG" ]]; then
        echo '{"version": "1.0", "events": []}' > "$AUDIT_LOG"
        chmod 600 "$AUDIT_LOG"
        log_success "Audit log initialized"
    fi

    log_success "CostPilot configuration initialized successfully"
}

# Validate all configuration
validate_all() {
    log_info "Validating CostPilot configuration..."

    local errors=0

    # Validate main config
    if ! validate_config "$CONFIG_FILE"; then
        ((errors++))
    fi

    # Validate license
    if ! validate_license; then
        ((errors++))
    fi

    # Check directory structure
    local dirs=("$CONFIG_DIR" "$BASELINES_DIR" "$SNAPSHOTS_DIR")
    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            log_error "Required directory missing: $dir"
            ((errors++))
        fi
    done

    # Check file permissions
    if [[ -f "$LICENSE_FILE" ]]; then
        local perms
        perms="$(stat -c '%a' "$LICENSE_FILE" 2>/dev/null || stat -f '%A' "$LICENSE_FILE" 2>/dev/null || echo "644")"
        if [[ "${perms: -1}" != "0" ]]; then
            log_warn "License file permissions too permissive: $perms (recommended: 600)"
        fi
    fi

    if [[ $errors -eq 0 ]]; then
        log_success "All configuration validation checks passed"
        return 0
    else
        log_error "$errors configuration validation errors found"
        return 1
    fi
}

# Show configuration status
show_status() {
    echo "CostPilot Configuration Status"
    echo "=============================="
    echo

    echo "Configuration Directory: $CONFIG_DIR"
    echo "Main Config: $([[ -f "$CONFIG_FILE" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo "License File: $([[ -f "$LICENSE_FILE" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo "SLO Config: $([[ -f "$SLO_CONFIG" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo "Baselines Dir: $([[ -d "$BASELINES_DIR" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo "Snapshots Dir: $([[ -d "$SNAPSHOTS_DIR" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo "Audit Log: $([[ -f "$AUDIT_LOG" ]] && echo "✅ Present" || echo "❌ Missing")"
    echo

    # Show license status
    if [[ -f "$LICENSE_FILE" ]]; then
        echo "License Status:"
        if costpilot --version >/dev/null 2>&1; then
            if costpilot license status >/dev/null 2>&1; then
                echo "  ✅ License is valid"
            else
                echo "  ❌ License is invalid or expired"
            fi
        else
            echo "  ⚠️  Cannot check license status (costpilot not in PATH)"
        fi
        echo
    fi

    # Show configuration summary
    if [[ -f "$CONFIG_FILE" ]]; then
        echo "Configuration Summary:"
        grep -E '^(enable_|require_)' "$CONFIG_FILE" 2>/dev/null | sed 's/^/  /' || echo "  No feature flags found"
        echo
    fi
}

# Migrate configuration (for future versions)
migrate_config() {
    log_info "Checking for configuration migrations..."

    # For now, no migrations needed
    # Future versions can add migration logic here

    log_success "No configuration migrations needed"
}

# Main command handling
case "${1:-}" in
    init)
        init_config
        ;;
    validate)
        validate_all
        ;;
    status)
        show_status
        ;;
    migrate)
        migrate_config
        ;;
    *)
        echo "CostPilot Configuration Management"
        echo
        echo "Usage: $0 <command>"
        echo
        echo "Commands:"
        echo "  init     Initialize CostPilot configuration"
        echo "  validate Validate existing configuration"
        echo "  status   Show configuration status"
        echo "  migrate  Run configuration migrations"
        echo
        echo "Configuration Directory: $CONFIG_DIR"
        ;;
esac</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/manage_config.sh
