#!/bin/bash
set -euo pipefail

# CostPilot auto-update script
# Checks for updates and installs new versions

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
REPO="guardsuite/costpilot"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
DOWNLOAD_DIR="${PROJECT_ROOT}/tmp"
BACKUP_DIR="${PROJECT_ROOT}/backup"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root (for system-wide installation)
is_system_install() {
    [[ "$EUID" -eq 0 ]] && [[ -w "/usr/local/bin" ]]
}

# Get current version
get_current_version() {
    if command -v costpilot >/dev/null 2>&1; then
        costpilot --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "0.0.0"
    else
        echo "0.0.0"
    fi
}

# Get latest release info from GitHub
get_latest_release() {
    if command -v curl >/dev/null 2>&1; then
        curl -s "$API_URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O - "$API_URL"
    else
        log_error "Neither curl nor wget found. Cannot check for updates."
        exit 1
    fi
}

# Compare versions (simple semantic version comparison)
version_greater() {
    local v1="$1"
    local v2="$2"

    # Remove 'v' prefix if present
    v1="${v1#v}"
    v2="${v2#v}"

    # Split versions into components
    IFS='.' read -ra V1 <<< "$v1"
    IFS='.' read -ra V2 <<< "$v2"

    # Compare major version
    if (( ${V1[0]} > ${V2[0]} )); then
        return 0
    elif (( ${V1[0]} < ${V2[0]} )); then
        return 1
    fi

    # Compare minor version
    if (( ${V1[1]} > ${V2[1]} )); then
        return 0
    elif (( ${V1[1]} < ${V2[1]} )); then
        return 1
    fi

    # Compare patch version
    if (( ${V1[2]} > ${V2[2]} )); then
        return 0
    else
        return 1
    fi
}

# Detect platform
detect_platform() {
    local uname_s="$(uname -s)"
    local uname_m="$(uname -m)"

    case "$uname_s" in
        Linux)
            case "$uname_m" in
                x86_64) echo "linux-amd64" ;;
                aarch64) echo "linux-arm64" ;;
                *) echo "linux-$uname_m" ;;
            esac
            ;;
        Darwin)
            case "$uname_m" in
                x86_64) echo "macos-amd64" ;;
                arm64) echo "macos-arm64" ;;
                *) echo "macos-$uname_m" ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows-amd64"
            ;;
        *)
            echo "unknown-$uname_s-$uname_m"
            ;;
    esac
}

# Download and install update
perform_update() {
    local latest_version="$1"
    local download_url="$2"
    local platform="$(detect_platform)"

    log_info "Updating CostPilot to version ${latest_version}..."

    # Create temporary directory
    mkdir -p "$DOWNLOAD_DIR"
    mkdir -p "$BACKUP_DIR"

    # Backup current binary
    local current_binary
    if is_system_install; then
        current_binary="/usr/local/bin/costpilot"
    else
        current_binary="$(which costpilot 2>/dev/null || echo "")"
        if [[ -z "$current_binary" ]]; then
            log_error "CostPilot binary not found in PATH"
            exit 1
        fi
    fi

    cp "$current_binary" "$BACKUP_DIR/costpilot.backup.$(date +%Y%m%d_%H%M%S)"

    # Download new version
    local archive_name="costpilot-${latest_version}-${platform}.tar.gz"
    local download_path="$DOWNLOAD_DIR/$archive_name"

    log_info "Downloading ${download_url}..."
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$download_path" "$download_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$download_path" "$download_url"
    else
        log_error "Neither curl nor wget found. Cannot download update."
        exit 1
    fi

    # Extract archive
    log_info "Extracting archive..."
    tar -xzf "$download_path" -C "$DOWNLOAD_DIR"

    # Find the binary in the extracted archive
    local extracted_binary
    extracted_binary="$(find "$DOWNLOAD_DIR" -name "costpilot*" -type f -executable | head -1)"

    if [[ -z "$extracted_binary" ]]; then
        log_error "Could not find CostPilot binary in downloaded archive"
        exit 1
    fi

    # Install new binary
    log_info "Installing new version..."
    if is_system_install; then
        cp "$extracted_binary" "$current_binary"
        chmod 755 "$current_binary"
    else
        sudo cp "$extracted_binary" "$current_binary"
        sudo chmod 755 "$current_binary"
    fi

    # Verify installation
    local installed_version
    installed_version="$(costpilot --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")"

    if [[ "$installed_version" == "$latest_version" ]]; then
        log_info "✅ CostPilot successfully updated to version ${latest_version}"

        # Clean up
        rm -rf "$DOWNLOAD_DIR"

        log_info "Backup of previous version saved in: $BACKUP_DIR"
    else
        log_error "❌ Update verification failed. Rolling back..."

        # Rollback
        local backup_file
        backup_file="$(ls -t "$BACKUP_DIR"/costpilot.backup.* | head -1)"
        if [[ -n "$backup_file" ]]; then
            cp "$backup_file" "$current_binary"
            log_info "Rolled back to previous version"
        fi

        exit 1
    fi
}

# Main update check logic
main() {
    local current_version
    current_version="$(get_current_version)"

    log_info "Current CostPilot version: ${current_version}"

    # Get latest release info
    local release_info
    release_info="$(get_latest_release)"

    if [[ -z "$release_info" ]]; then
        log_error "Could not fetch release information"
        exit 1
    fi

    # Extract latest version
    local latest_version
    latest_version="$(echo "$release_info" | grep -o '"tag_name": *"[^"]*"' | grep -o '[^"]*$' | sed 's/^v//')"

    if [[ -z "$latest_version" ]]; then
        log_error "Could not determine latest version"
        exit 1
    fi

    log_info "Latest available version: ${latest_version}"

    # Compare versions
    if version_greater "$latest_version" "$current_version"; then
        log_info "🎉 Update available!"

        # Find download URL for current platform
        local platform
        platform="$(detect_platform)"
        local download_url
        download_url="$(echo "$release_info" | grep -o "\"browser_download_url\": *\"[^\"]*${platform}[^\"]*\.tar\.gz\"" | grep -o '"[^"]*"' | tr -d '"')"

        if [[ -z "$download_url" ]]; then
            log_error "No download URL found for platform: $platform"
            exit 1
        fi

        # Prompt for update (unless --yes flag is used)
        if [[ "${1:-}" != "--yes" ]]; then
            echo
            read -p "Do you want to update CostPilot to version ${latest_version}? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Update cancelled"
                exit 0
            fi
        fi

        perform_update "$latest_version" "$download_url"
    else
        log_info "✅ CostPilot is up to date"
    fi
}

# Run main function
main "$@"</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/update_costpilot.sh
