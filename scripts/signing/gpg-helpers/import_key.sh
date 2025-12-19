#!/usr/bin/env bash
set -euo pipefail

# Usage: import_key.sh <gnupghome> <private_key_content>
# Imports GPG private key into ephemeral home

if [[ $# -ne 2 ]]; then
  echo "ERROR: Usage: import_key.sh <gnupghome> <private_key_content>" >&2
  exit 1
fi

GNUPGHOME="$1"
PRIVATE_KEY="$2"

export GNUPGHOME

# Create GPG home if not exists
mkdir -p "$GNUPGHOME"
chmod 700 "$GNUPGHOME"

# Import key
echo "$PRIVATE_KEY" | gpg --batch --import 2>/dev/null

echo "KEY_IMPORTED: ${GNUPGHOME}"
