#!/usr/bin/env bash
set -euo pipefail

# Usage: generate_sbom.sh <source_dir> <out_spdx_json>
# Generates SPDX JSON SBOM for release artifacts

if [[ $# -ne 2 ]]; then
  echo "ERROR: Usage: generate_sbom.sh <source_dir> <out_spdx_json>" >&2
  exit 1
fi

SOURCE_DIR="$1"
OUT_SPDX="$2"

if [[ ! -d "$SOURCE_DIR" ]]; then
  echo "ERROR: Source directory not found: $SOURCE_DIR" >&2
  exit 1
fi

# Try syft first (preferred)
if command -v syft >/dev/null 2>&1; then
  syft "$SOURCE_DIR" -o spdx-json > "$OUT_SPDX" 2>/dev/null
  echo "SBOM: ${OUT_SPDX}"
  exit 0
fi

# Fallback: minimal SPDX JSON
BUILD_TIME="${SOURCE_DATE_EPOCH:-$(date +%s)}"
CREATED=$(date -u -d "@${BUILD_TIME}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -r "${BUILD_TIME}" +%Y-%m-%dT%H:%M:%SZ)

# Collect files and checksums
FILES_JSON="[]"
if [[ -d "$SOURCE_DIR" ]]; then
  while IFS= read -r -d '' file; do
    rel_path="${file#$SOURCE_DIR/}"
    checksum=$(sha256sum "$file" | cut -d' ' -f1)
    FILES_JSON=$(echo "$FILES_JSON" | jq --arg path "$rel_path" --arg sum "$checksum" \
      '. + [{fileName: $path, checksums: [{algorithm: "SHA256", checksumValue: $sum}]}]')
  done < <(find "$SOURCE_DIR" -type f -print0 | sort -z)
fi

# Generate minimal SPDX
TMP_FILE="${OUT_SPDX}.tmp.$$"
jq -nc \
  --arg created "$CREATED" \
  --argjson files "$FILES_JSON" \
  '{
    spdxVersion: "SPDX-2.3",
    dataLicense: "CC0-1.0",
    SPDXID: "SPDXRef-DOCUMENT",
    name: "CostPilot Release SBOM",
    documentNamespace: "https://github.com/costpilot/costpilot/sbom",
    creationInfo: {
      created: $created,
      creators: ["Tool: generate_sbom.sh"],
      licenseListVersion: "3.21"
    },
    packages: [{
      SPDXID: "SPDXRef-Package",
      name: "costpilot",
      versionInfo: "1.0.0",
      downloadLocation: "NOASSERTION",
      filesAnalyzed: true,
      licenseConcluded: "NOASSERTION",
      licenseDeclared: "NOASSERTION",
      copyrightText: "NOASSERTION"
    }],
    files: $files
  }' > "$TMP_FILE"

mv "$TMP_FILE" "$OUT_SPDX"
echo "SBOM: ${OUT_SPDX}"
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.4",
  "serialNumber": "urn:uuid:$(uuidgen 2>/dev/null || echo 'fixed-uuid-for-determinism')",
  "version": 1,
  "metadata": {
    "timestamp": "${BUILD_TIMESTAMP}",
    "tools": [
      {
        "vendor": "CostPilot",
        "name": "SBOM Generator",
        "version": "1.0.0"
      }
    ],
    "component": {
      "type": "application",
      "bom-ref": "costpilot@${VERSION}",
      "name": "costpilot",
      "version": "${VERSION}",
      "description": "Cloud cost analysis and prediction tool",
      "licenses": [
        {
          "license": {
            "id": "Apache-2.0"
          }
        }
      ],
      "purl": "pkg:cargo/costpilot@${VERSION}",
      "externalReferences": [
        {
          "type": "vcs",
          "url": "https://github.com/guardsuite/costpilot",
          "comment": "Git commit: ${GIT_COMMIT}"
        }
      ]
    }
  },
  "components": []
}
EOF

# Parse dependencies from Cargo.lock if available
if [ -f "${PROJECT_ROOT}/Cargo.lock" ]; then
    # Extract dependencies and add to components array
    # For determinism: sort components by name
    TEMP_COMPONENTS=$(mktemp)

    # Simple extraction: get package names and versions
    awk '/\[\[package\]\]/,/^$/ {
        if ($1 == "name") name=$3;
        if ($1 == "version") {
            version=$3;
            gsub(/"/, "", name);
            gsub(/"/, "", version);
            if (name != "" && version != "") {
                printf "{\"type\":\"library\",\"bom-ref\":\"%s@%s\",\"name\":\"%s\",\"version\":\"%s\",\"purl\":\"pkg:cargo/%s@%s\"},\n", name, version, name, version, name, version;
                name=""; version="";
            }
        }
    }' "${PROJECT_ROOT}/Cargo.lock" | sort -u > "${TEMP_COMPONENTS}"

    # Remove trailing comma from last line
    sed -i '$ s/,$//' "${TEMP_COMPONENTS}" 2>/dev/null || sed -i '' '$ s/,$//' "${TEMP_COMPONENTS}" 2>/dev/null || true

    # Insert components into SBOM using jq
    if command -v jq &> /dev/null && [ -s "${TEMP_COMPONENTS}" ]; then
        COMPONENTS_JSON="[$(cat "${TEMP_COMPONENTS}")]"
        TEMP_SBOM=$(mktemp)
        jq ".components = ${COMPONENTS_JSON} | .components |= sort_by(.name)" "${OUTPUT_PATH}" > "${TEMP_SBOM}"
        mv "${TEMP_SBOM}" "${OUTPUT_PATH}"
    fi

    rm -f "${TEMP_COMPONENTS}"
fi

# Normalize with jq (sort keys, stable format)
if command -v jq &> /dev/null; then
    TEMP_SBOM=$(mktemp)
    jq --sort-keys '.' "${OUTPUT_PATH}" > "${TEMP_SBOM}"
    mv "${TEMP_SBOM}" "${OUTPUT_PATH}"
fi

echo "SBOM written: ${OUTPUT_PATH}"
