#!/bin/bash
# Restore Script for CostPilot

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKUP_ROOT="${BACKUP_ROOT:-/tmp/costpilot_backups}"
RESTORE_TYPE="${1:-full}"
BACKUP_FILE="$2"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

# Verify backup file
verify_backup_file() {
    local backup_file="$1"

    if [[ ! -f "$backup_file" ]]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi

    # Verify checksum if available
    local checksum_file="$backup_file.sha256"
    if [[ -f "$checksum_file" ]]; then
        log_info "Verifying backup checksum..."
        if ! sha256sum -c "$checksum_file" >/dev/null 2>&1; then
            log_error "Backup checksum verification failed"
            return 1
        fi
    fi

    # Verify archive integrity
    log_info "Verifying backup archive integrity..."
    if ! tar tzf "$backup_file" >/dev/null 2>&1; then
        log_error "Backup archive is corrupted"
        return 1
    fi

    log_success "Backup file verified"
}

# Create restore directory
create_restore_dir() {
    local restore_dir="/tmp/costpilot_restore_$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$restore_dir"
    echo "$restore_dir"
}

# Extract backup
extract_backup() {
    local backup_file="$1"
    local restore_dir="$2"

    log_info "Extracting backup to $restore_dir..."

    # Extract archive
    tar xzf "$backup_file" -C "$restore_dir"

    # Find the backup directory inside the archive
    local backup_name=$(tar tf "$backup_file" | head -1 | cut -d'/' -f1)
    echo "$restore_dir/$backup_name"
}

# Restore configuration files
restore_config() {
    local backup_dir="$1"
    log_info "Restoring configuration files..."

    local config_dir="$backup_dir/config"

    if [[ ! -d "$config_dir" ]]; then
        log_warning "No configuration data found in backup"
        return 0
    fi

    # Restore environment configurations
    if [[ -d "$config_dir/environments" ]]; then
        cp -r "$config_dir/environments" "$PROJECT_ROOT/tests/"
        log_success "Environment configurations restored"
    fi

    # Restore GitHub workflows
    if [[ -d "$config_dir/.github" ]]; then
        cp -r "$config_dir/.github" "$PROJECT_ROOT/"
        log_success "GitHub workflows restored"
    fi

    # Restore Cargo files
    if [[ -f "$config_dir/Cargo.toml" ]]; then
        cp "$config_dir/Cargo.toml" "$PROJECT_ROOT/"
    fi
    if [[ -f "$config_dir/Cargo.lock" ]]; then
        cp "$config_dir/Cargo.lock" "$PROJECT_ROOT/"
    fi

    log_success "Configuration files restored"
}

# Restore test data
restore_test_data() {
    local backup_dir="$1"
    log_info "Restoring test data..."

    local data_dir="$backup_dir/test_data"

    if [[ ! -d "$data_dir" ]]; then
        log_warning "No test data found in backup"
        return 0
    fi

    # Restore test fixtures
    if [[ -d "$data_dir/fixtures" ]]; then
        cp -r "$data_dir/fixtures" "$PROJECT_ROOT/tests/"
        log_success "Test fixtures restored"
    fi

    # Restore baseline files
    if [[ -f "$data_dir/baseline.json" ]]; then
        cp "$data_dir/baseline.json" "$PROJECT_ROOT/"
    fi

    # Restore test JSON files
    cp "$data_dir/test_"*.json "$PROJECT_ROOT/" 2>/dev/null || true

    log_success "Test data restored"
}

# Restore source code
restore_source() {
    local backup_dir="$1"
    log_info "Restoring source code..."

    local source_dir="$backup_dir/source"

    if [[ ! -d "$source_dir" ]]; then
        log_warning "No source code found in backup"
        return 0
    fi

    # Restore source files (be careful not to overwrite important files)
    # This would typically be done in a clean directory for safety

    log_warning "Source code restoration requires manual intervention"
    log_info "Source code is available in: $source_dir"
    log_info "Please review and manually restore source files as needed"

    log_success "Source code location identified"
}

# Validate restoration
validate_restoration() {
    local restore_type="$1"
    log_info "Validating restoration..."

    case "$restore_type" in
        full|config)
            # Check if configuration files exist
            if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
                log_error "Cargo.toml not found after restoration"
                return 1
            fi
            if [[ ! -d "$PROJECT_ROOT/tests/environments" ]]; then
                log_error "Environment configurations not found after restoration"
                return 1
            fi
            ;;
        data)
            # Check if test data exists
            if [[ ! -d "$PROJECT_ROOT/tests/fixtures" ]]; then
                log_error "Test fixtures not found after restoration"
                return 1
            fi
            ;;
    esac

    log_success "Restoration validation passed"
}

# Cleanup temporary files
cleanup() {
    local restore_dir="$1"
    log_info "Cleaning up temporary files..."

    rm -rf "$restore_dir"

    log_success "Cleanup completed"
}

# Main restore function
main() {
    if [[ -z "$BACKUP_FILE" ]]; then
        log_error "Backup file not specified"
        echo "Usage: $0 <restore_type> <backup_file>"
        echo "Restore types: full, config, data, source"
        exit 1
    fi

    log_info "Starting CostPilot restore: $RESTORE_TYPE from $BACKUP_FILE"

    # Validate restore type
    case "$RESTORE_TYPE" in
        full|config|data|source)
            ;;
        *)
            log_error "Invalid restore type: $RESTORE_TYPE"
            echo "Usage: $0 [full|config|data|source] <backup_file>"
            exit 1
            ;;
    esac

    # Verify backup file
    verify_backup_file "$BACKUP_FILE"

    # Create restore directory
    local restore_dir=$(create_restore_dir)

    # Extract backup
    local backup_dir=$(extract_backup "$BACKUP_FILE" "$restore_dir")

    # Perform restore based on type
    case "$RESTORE_TYPE" in
        full)
            restore_config "$backup_dir"
            restore_test_data "$backup_dir"
            restore_source "$backup_dir"
            ;;
        config)
            restore_config "$backup_dir"
            ;;
        data)
            restore_test_data "$backup_dir"
            ;;
        source)
            restore_source "$backup_dir"
            ;;
    esac

    # Validate restoration
    validate_restoration "$RESTORE_TYPE"

    # Cleanup
    cleanup "$restore_dir"

    log_success "Restore completed successfully"
}

# Run main function
main "$@"
