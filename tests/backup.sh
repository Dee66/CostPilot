#!/bin/bash
# Backup Script for CostPilot

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
BACKUP_TYPE="${1:-full}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="costpilot_${BACKUP_TYPE}_${TIMESTAMP}"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1" >&2
}

# Create backup directory
create_backup_dir() {
    local backup_dir="$BACKUP_ROOT/$BACKUP_NAME"
    mkdir -p "$backup_dir"
    echo "$backup_dir"
}

# Backup configuration files
backup_config() {
    local backup_dir="$1"
    log_info "Backing up configuration files..."

    local config_dir="$backup_dir/config"
    mkdir -p "$config_dir"

    # Copy configuration files
    cp -r "$PROJECT_ROOT/tests/environments" "$config_dir/" 2>/dev/null || true
    cp -r "$PROJECT_ROOT/.github" "$config_dir/" 2>/dev/null || true
    cp "$PROJECT_ROOT/Cargo.toml" "$config_dir/" 2>/dev/null || true
    cp "$PROJECT_ROOT/Cargo.lock" "$config_dir/" 2>/dev/null || true

    log_success "Configuration backup completed"
}

# Backup test data
backup_test_data() {
    local backup_dir="$1"
    log_info "Backing up test data..."

    local data_dir="$backup_dir/test_data"
    mkdir -p "$data_dir"

    # Copy test fixtures and data
    cp -r "$PROJECT_ROOT/tests/fixtures" "$data_dir/" 2>/dev/null || true
    cp -r "$PROJECT_ROOT/baseline.json" "$data_dir/" 2>/dev/null || true
    cp -r "$PROJECT_ROOT/test_*.json" "$data_dir/" 2>/dev/null || true

    log_success "Test data backup completed"
}

# Backup source code
backup_source() {
    local backup_dir="$1"
    log_info "Backing up source code..."

    local source_dir="$backup_dir/source"
    mkdir -p "$source_dir"

    # Copy source code (excluding target and temp directories)
    rsync -a --exclude='target/' --exclude='.git/' --exclude='*.log' \
          --exclude='temp/' --exclude='cleanup/' \
          "$PROJECT_ROOT/" "$source_dir/"

    log_success "Source code backup completed"
}

# Create backup archive
create_archive() {
    local backup_dir="$1"
    log_info "Creating backup archive..."

    local archive_file="$BACKUP_ROOT/${BACKUP_NAME}.tar.gz"

    # Create compressed archive
    cd "$BACKUP_ROOT"
    tar czf "$archive_file" "$BACKUP_NAME"

    # Calculate checksum
    local checksum_file="$archive_file.sha256"
    sha256sum "$archive_file" > "$checksum_file"

    # Clean up uncompressed backup
    rm -rf "$backup_dir"

    log_success "Backup archive created: $archive_file"
    # Return the archive file path (this will be captured by command substitution)
    echo "$archive_file"
}

# Verify backup integrity
verify_backup() {
    local archive_file="$1"
    log_info "Verifying backup integrity..."

    # Verify archive integrity
    if ! tar tzf "$archive_file" >/dev/null 2>&1; then
        log_error "Backup archive is corrupted"
        return 1
    fi

    # Verify checksum
    local checksum_file="$archive_file.sha256"
    if [[ -f "$checksum_file" ]]; then
        if ! sha256sum -c "$checksum_file" >/dev/null 2>&1; then
            log_error "Backup checksum verification failed"
            return 1
        fi
    fi

    log_success "Backup integrity verified"
}

# Main backup function
main() {
    log_info "Starting CostPilot backup: $BACKUP_TYPE"

    # Validate backup type
    case "$BACKUP_TYPE" in
        full|config|data|source)
            ;;
        *)
            log_error "Invalid backup type: $BACKUP_TYPE"
            echo "Usage: $0 [full|config|data|source]"
            exit 1
            ;;
    esac

    # Create backup directory
    local backup_dir=$(create_backup_dir)

    # Perform backup based on type
    case "$BACKUP_TYPE" in
        full)
            backup_config "$backup_dir"
            backup_test_data "$backup_dir"
            backup_source "$backup_dir"
            ;;
        config)
            backup_config "$backup_dir"
            ;;
        data)
            backup_test_data "$backup_dir"
            ;;
        source)
            backup_source "$backup_dir"
            ;;
    esac

    # Create and verify archive
    local archive_file=$(create_archive "$backup_dir")
    verify_backup "$archive_file"

    log_success "Backup completed successfully: $archive_file"

    # Print backup information
    echo "Backup Details:"
    echo "  Type: $BACKUP_TYPE"
    echo "  Archive: $archive_file"
    echo "  Size: $(du -sh "$archive_file" | cut -f1)"
    echo "  Checksum: $(cat "${archive_file}.sha256")"
}

# Run main function
main "$@"
