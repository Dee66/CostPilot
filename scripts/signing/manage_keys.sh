#!/usr/bin/env bash
set -euo pipefail

# Key management for CostPilot license signing
# Usage: manage_keys.sh <command> [options]

COMMAND="${1:-help}"

case "$COMMAND" in
    generate)
        # Generate new Ed25519 keypair
        KEY_NAME="${2:-license_key}"
        PRIVATE_KEY="${KEY_NAME}.pem"
        PUBLIC_KEY="${KEY_NAME}.pub.pem"
        RAW_PRIVATE="${KEY_NAME}_raw.bin"

        if [[ -f "$RAW_PRIVATE" ]]; then
            echo "ERROR: Raw private key already exists: $RAW_PRIVATE" >&2
            exit 1
        fi

        # Generate 32 random bytes for Ed25519 private key
        dd if=/dev/urandom of="$RAW_PRIVATE" bs=32 count=1 2>/dev/null

        # Create PEM format for compatibility
        if command -v openssl >/dev/null 2>&1; then
            # Convert raw bytes to PEM private key
            openssl pkey -in "$RAW_PRIVATE" -inform der -out "$PRIVATE_KEY" 2>/dev/null || {
                # Fallback: just copy raw as PEM (not standard but works for our tool)
                cp "$RAW_PRIVATE" "$PRIVATE_KEY"
            }
            # Extract public key
            openssl pkey -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY" 2>/dev/null || {
                echo "Failed to extract public key" >&2
                rm -f "$RAW_PRIVATE" "$PRIVATE_KEY"
                exit 1
            }
        else
            echo "Warning: OpenSSL not found, using raw keys only" >&2
            cp "$RAW_PRIVATE" "$PRIVATE_KEY"
            # Cannot generate public key without openssl
            echo "ERROR: Cannot generate public key without OpenSSL" >&2
            rm -f "$RAW_PRIVATE" "$PRIVATE_KEY"
            exit 1
        fi

        echo "Keypair generated:"
        echo "  Raw private: $RAW_PRIVATE"
        echo "  PEM private: $PRIVATE_KEY"
        echo "  Public:      $PUBLIC_KEY"
        echo "  Fingerprint: $(openssl dgst -sha256 "$PUBLIC_KEY" 2>/dev/null | cut -d' ' -f2 || echo 'N/A')"
        ;;

    fingerprint)
        # Show key fingerprint
        KEY_FILE="${2:-license_key.pub.pem}"
        if [[ ! -f "$KEY_FILE" ]]; then
            echo "ERROR: Key file not found: $KEY_FILE" >&2
            exit 1
        fi
        FINGERPRINT=$(openssl dgst -sha256 "$KEY_FILE" 2>/dev/null | cut -d' ' -f2 || echo 'N/A')
        echo "Fingerprint for $KEY_FILE: $FINGERPRINT"
        ;;

    rotate)
        # Rotate keys with overlap period
        OLD_KEY="${2:-license_key}"
        NEW_KEY="${3:-license_key_new}"

        if [[ ! -f "${OLD_KEY}.pem" ]]; then
            echo "ERROR: Old private key not found: ${OLD_KEY}.pem" >&2
            exit 1
        fi

        echo "Rotating keys..."
        "$0" generate "$NEW_KEY"

        echo "New key generated. Update build.rs with new public key."
        echo "Keep old key for backward compatibility during transition period."
        echo "After transition, remove old key files."
        ;;

    help|*)
        echo "CostPilot Key Management Tool"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  generate [key_name]    Generate new Ed25519 keypair"
        echo "  fingerprint [key_file] Show SHA256 fingerprint of public key"
        echo "  rotate <old_key> <new_key> Rotate keys with overlap"
        echo "  help                   Show this help"
        ;;
esac
