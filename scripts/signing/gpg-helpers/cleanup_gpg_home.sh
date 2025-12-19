#!/usr/bin/env bash
set -euo pipefail

# Usage: cleanup_gpg_home.sh <gnupghome>
# Securely removes ephemeral GPG home

if [[ $# -ne 1 ]]; then
  echo "ERROR: Usage: cleanup_gpg_home.sh <gnupghome>" >&2
  exit 1
fi

GNUPGHOME="$1"

if [[ -d "$GNUPGHOME" ]]; then
  # Kill any gpg-agent
  gpgconf --homedir "$GNUPGHOME" --kill gpg-agent 2>/dev/null || true
  
  # Secure removal
  rm -rf "$GNUPGHOME"
  
  echo "CLEANED: ${GNUPGHOME}"
else
  echo "SKIPPED: ${GNUPGHOME} not found"
fi
