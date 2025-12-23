#!/usr/bin/env bash
set -euo pipefail

# Update provenance.json with artifact details, signatures, and SBOM reference
# DETERMINISM: normalized timestamps, sorted artifacts, stable key ordering

DIST_DIR="${1:-dist}"
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TEMPLATE="${PROJECT_ROOT}/packaging/provenance/provenance_template.json"
PROVENANCE="${DIST_DIR}/provenance.json"
PUBLIC_KEY="${PROJECT_ROOT}/packaging/signing/public.key"

if [ ! -d "${DIST_DIR}" ]; then
    echo "ERROR: dist directory not found: ${DIST_DIR}"
    exit 1
fi

if [ ! -f "${TEMPLATE}" ]; then
    echo "ERROR: provenance template not found: ${TEMPLATE}"
    exit 1
fi

# Normalized timestamp (no fractional seconds)
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Extract version from Cargo.toml
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

# Git commit (full SHA)
GIT_COMMIT="unknown"
if command -v git &> /dev/null && [ -d "${PROJECT_ROOT}/.git" ]; then
    GIT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
fi

# Determine builder ID
BUILDER_ID="local"
if [ -n "${CI:-}" ]; then
    BUILDER_ID="github-actions"
fi

# Hash command
HASH_CMD=""
if command -v sha256sum &> /dev/null; then
    HASH_CMD="sha256sum"
elif command -v shasum &> /dev/null; then
    HASH_CMD="shasum -a 256"
else
    echo "ERROR: No hash command available"
    exit 1
fi

# Start with template
cp "${TEMPLATE}" "${PROVENANCE}"

# Collect artifacts (exclude .sig files and provenance.json itself)
ARTIFACTS_JSON="["
SIGNATURES_JSON="["
FIRST_ARTIFACT=true
FIRST_SIG=true

cd "${DIST_DIR}"

for artifact in *; do
    # Skip .sig files, provenance.json, and directories
    if [[ "${artifact}" == *.sig ]] || [[ "${artifact}" == "provenance.json" ]] || [ -d "${artifact}" ]; then
        continue
    fi

    if [ -f "${artifact}" ]; then
        # Compute SHA256
        HASH=$(${HASH_CMD} "${artifact}" | cut -d' ' -f1)
        SIZE=$(stat -c%s "${artifact}" 2>/dev/null || stat -f%z "${artifact}" 2>/dev/null)

        # Add to artifacts array
        if [ "${FIRST_ARTIFACT}" = true ]; then
            ARTIFACTS_JSON="${ARTIFACTS_JSON}{\"file\":\"${artifact}\",\"sha256\":\"${HASH}\",\"size\":${SIZE}}"
            FIRST_ARTIFACT=false
        else
            ARTIFACTS_JSON="${ARTIFACTS_JSON},{\"file\":\"${artifact}\",\"sha256\":\"${HASH}\",\"size\":${SIZE}}"
        fi

        # Add signature if exists
        if [ -f "${artifact}.sig" ]; then
            if [ "${FIRST_SIG}" = true ]; then
                SIGNATURES_JSON="${SIGNATURES_JSON}{\"file\":\"${artifact}\",\"sig\":\"${artifact}.sig\"}"
                FIRST_SIG=false
            else
                SIGNATURES_JSON="${SIGNATURES_JSON},{\"file\":\"${artifact}\",\"sig\":\"${artifact}.sig\"}"
            fi
        fi
    fi
done

ARTIFACTS_JSON="${ARTIFACTS_JSON}]"
SIGNATURES_JSON="${SIGNATURES_JSON}]"

cd "${PROJECT_ROOT}"

# SBOM reference
SBOM_FILE="sbom.cyclonedx.json"
SBOM_HASH=""
if [ -f "${DIST_DIR}/${SBOM_FILE}" ]; then
    cd "${DIST_DIR}"
    SBOM_HASH=$(${HASH_CMD} "${SBOM_FILE}" | cut -d' ' -f1)
    cd "${PROJECT_ROOT}"
fi

# Update provenance with jq
if command -v jq &> /dev/null; then
    TEMP_PROV=$(mktemp)

    jq \
        --arg version "${VERSION}" \
        --arg commit "${GIT_COMMIT}" \
        --arg timestamp "${BUILD_TIMESTAMP}" \
        --arg builder "${BUILDER_ID}" \
        --arg sbom "${SBOM_FILE}" \
        --argjson artifacts "${ARTIFACTS_JSON}" \
        --argjson signatures "${SIGNATURES_JSON}" \
        '.version = $version |
         .commit = $commit |
         .build_timestamp = $timestamp |
         .builder = $builder |
         .sbom = $sbom |
         .artifacts = ($artifacts | sort_by(.file)) |
         .signatures = ($signatures | sort_by(.file))' \
        "${PROVENANCE}" > "${TEMP_PROV}"

    # Sort keys for determinism
    jq --sort-keys '.' "${TEMP_PROV}" > "${PROVENANCE}"
    rm -f "${TEMP_PROV}"
else
    echo "ERROR: jq not found, cannot update provenance"
    exit 1
fi

# Verify artifact signatures (sample check)
if [ -f "${PUBLIC_KEY}" ]; then
    VERIFY_SCRIPT="${PROJECT_ROOT}/packaging/signing/verify.sh"
    if [ -x "${VERIFY_SCRIPT}" ]; then
        SAMPLE_COUNT=0
        for artifact in "${DIST_DIR}"/*.zip "${DIST_DIR}"/*.tar.gz; do
            if [ -f "${artifact}" ] && [ -f "${artifact}.sig" ]; then
                if ! bash "${VERIFY_SCRIPT}" "${artifact}" "${PUBLIC_KEY}" "${artifact}.sig" > /dev/null 2>&1; then
                    echo "ERROR: Signature verification failed for $(basename "${artifact}")"
                    exit 1
                fi
                SAMPLE_COUNT=$((SAMPLE_COUNT + 1))
                if [ ${SAMPLE_COUNT} -ge 2 ]; then
                    break
                fi
            fi
        done
    fi
fi

echo "provenance.json written"
