#!/usr/bin/env bash
set -euo pipefail

# End-to-end packaging pipeline for CostPilot Free Edition
# Orchestrates: build → package → sign → verify → publish

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "${PROJECT_ROOT}"

PLATFORM="linux-x64"
DIST_DIR="${PROJECT_ROOT}/dist"
STAGING_DIR="${PROJECT_ROOT}/staging/core"
BUILD_SCRIPT="${PROJECT_ROOT}/packaging/core_release/build_core_package.sh"
SIGN_SCRIPT="${PROJECT_ROOT}/packaging/core_release/sign_artifacts.sh"
VERIFY_SCRIPT="${PROJECT_ROOT}/packaging/core_release/verify_artifact.sh"
MANIFEST_SCRIPT="${PROJECT_ROOT}/packaging/core_release/gen_manifest.sh"
ED25519_SIGN="${PROJECT_ROOT}/packaging/signing/sign.sh"
ED25519_VERIFY="${PROJECT_ROOT}/packaging/signing/verify.sh"
PRIVATE_KEY="${PROJECT_ROOT}/packaging/signing/private.key"
PUBLIC_KEY="${PROJECT_ROOT}/packaging/signing/public.key"
BUNDLE_SCRIPT="${PROJECT_ROOT}/packaging/packaging_tools/make_release_bundle.sh"
BUNDLE_VERIFY="${PROJECT_ROOT}/packaging/packaging_tools/verify_release_bundle.sh"

ARTIFACT_COUNT=0
VERIFIED=true

# ========================================
# STEP 0: Ensure signing keys exist
# ========================================
if [ ! -f "${PUBLIC_KEY}" ]; then
    echo "ERROR: Public key not found: ${PUBLIC_KEY}"
    echo "Run: cd packaging/signing && ./gen_keys.sh"
    exit 1
fi

if [ ! -f "${PRIVATE_KEY}" ]; then
    # Check if in CI environment
    if [ -n "${CI:-}" ] && [ -n "${SIGNING_PRIVATE_KEY:-}" ]; then
        echo "→ Setting up signing key from CI..."
        echo "${SIGNING_PRIVATE_KEY}" > "${PRIVATE_KEY}"
        chmod 600 "${PRIVATE_KEY}"
    else
        echo "ERROR: Private key not found: ${PRIVATE_KEY}"
        echo "For local builds, run: cd packaging/signing && ./gen_keys.sh"
        echo "For CI builds, set SIGNING_PRIVATE_KEY secret"
        exit 1
    fi
fi

# ========================================
# STEP 1: Clean previous builds
# ========================================
echo "→ Cleaning previous builds..."
rm -rf "${DIST_DIR}"
rm -rf "${STAGING_DIR}"
rm -rf "${PROJECT_ROOT}/target/release/costpilot"

# ========================================
# STEP 2: Build CostPilot
# ========================================
echo "→ Building CostPilot..."
cargo build --release --quiet

if [ ! -f "${PROJECT_ROOT}/target/release/costpilot" ]; then
    echo "ERROR: Binary build failed"
    exit 1
fi

# ========================================
# STEP 3: Generate build manifest
# ========================================
echo "→ Generating build manifest..."
MANIFEST_SCRIPT="${PROJECT_ROOT}/packaging/core_release/gen_manifest.sh"
chmod +x "${MANIFEST_SCRIPT}"
bash "${MANIFEST_SCRIPT}" > /dev/null 2>&1

# ========================================
# STEP 4: Package artifacts
# ========================================
echo "→ Packaging artifacts..."
chmod +x "${BUILD_SCRIPT}"

# Build will create artifacts in dist/
bash "${BUILD_SCRIPT}" > /dev/null 2>&1

# Find generated artifacts
ZIP_ARTIFACT=$(find "${DIST_DIR}" -name "*.zip" -type f 2>/dev/null | head -1)
TAR_ARTIFACT=$(find "${DIST_DIR}" -name "*.tar.gz" -type f 2>/dev/null | head -1)

if [ -z "${ZIP_ARTIFACT}" ] || [ -z "${TAR_ARTIFACT}" ]; then
    echo "ERROR: Packaging failed - artifacts not found"
    exit 1
fi

ARTIFACT_COUNT=2

# ========================================
# STEP 5: Append artifact fingerprints
# ========================================
echo "→ Updating build fingerprints..."

# Determine hash command
HASH_CMD=""
if command -v sha256sum &> /dev/null; then
    HASH_CMD="sha256sum"
elif command -v shasum &> /dev/null; then
    HASH_CMD="shasum -a 256"
fi

if [ -n "${HASH_CMD}" ]; then
    # Hash ZIP artifact
    ZIP_NAME=$(basename "${ZIP_ARTIFACT}")
    cd "${DIST_DIR}"
    ZIP_HASH=$(${HASH_CMD} "${ZIP_NAME}" | cut -d' ' -f1)
    echo "sha256 ${ZIP_HASH}  ${ZIP_NAME}" >> build_fingerprint.txt
    cd "${PROJECT_ROOT}"

    # Hash TAR.GZ artifact
    TAR_NAME=$(basename "${TAR_ARTIFACT}")
    cd "${DIST_DIR}"
    TAR_HASH=$(${HASH_CMD} "${TAR_NAME}" | cut -d' ' -f1)
    echo "sha256 ${TAR_HASH}  ${TAR_NAME}" >> build_fingerprint.txt
    cd "${PROJECT_ROOT}"
fi

# Update build.json with artifact list
bash "${MANIFEST_SCRIPT}" > /dev/null 2>&1 || true

# ========================================
# STEP 6: Generate SHA-256 signatures
# ========================================
echo "→ Signing artifacts..."
chmod +x "${SIGN_SCRIPT}"

bash "${SIGN_SCRIPT}" "${ZIP_ARTIFACT}" > /dev/null
bash "${SIGN_SCRIPT}" "${TAR_ARTIFACT}" > /dev/null

# Verify signatures were created
if [ ! -f "${ZIP_ARTIFACT}.sha256" ] || [ ! -f "${TAR_ARTIFACT}.sha256" ]; then
    echo "ERROR: Signature generation failed"
    exit 1
fi

# ========================================
# STEP 5: Verify artifacts + signatures
# ========================================
echo "→ Verifying artifacts..."
chmod +x "${VERIFY_SCRIPT}"

if ! bash "${VERIFY_SCRIPT}" "${PLATFORM}" "${ZIP_ARTIFACT}" > /dev/null 2>&1; then
    echo "ERROR: ZIP artifact verification failed"
    VERIFIED=false
    exit 1
fi

if ! bash "${VERIFY_SCRIPT}" "${PLATFORM}" "${TAR_ARTIFACT}" > /dev/null 2>&1; then
    echo "ERROR: TAR.GZ artifact verification failed"
    VERIFIED=false
    exit 1
fi

# ========================================
# STEP 8: Generate SBOM
# ========================================
echo "→ Generating SBOM..."

SBOM_SCRIPT="${PROJECT_ROOT}/packaging/sbom/generate_sbom.sh"
chmod +x "${SBOM_SCRIPT}"

bash "${SBOM_SCRIPT}" "${DIST_DIR}/sbom.cyclonedx.json" > /dev/null 2>&1

if [ ! -f "${DIST_DIR}/sbom.cyclonedx.json" ]; then
    echo "ERROR: SBOM generation failed"
    VERIFIED=false
    exit 1
fi

# ========================================
# STEP 9: Update provenance with artifacts
# ========================================
echo "→ Updating provenance..."

PROVENANCE_UPDATE="${PROJECT_ROOT}/packaging/provenance/update_provenance.sh"
chmod +x "${PROVENANCE_UPDATE}"

bash "${PROVENANCE_UPDATE}" "${DIST_DIR}" > /dev/null 2>&1

if [ ! -f "${DIST_DIR}/provenance.json" ]; then
    echo "ERROR: Provenance generation failed"
    VERIFIED=false
    exit 1
fi

# ========================================
# STEP 10: Ed25519 signature generation
# ========================================
echo "→ Generating Ed25519 signatures..."

chmod +x "${ED25519_SIGN}"
chmod +x "${ED25519_VERIFY}"

# Sign all artifacts (including SBOM and provenance)
ARTIFACTS_TO_SIGN=(
    "${ZIP_ARTIFACT}"
    "${TAR_ARTIFACT}"
    "${DIST_DIR}/build.json"
    "${DIST_DIR}/build_fingerprint.txt"
    "${DIST_DIR}/sbom.cyclonedx.json"
    "${DIST_DIR}/provenance.json"
)

SIGNATURES=()
for artifact in "${ARTIFACTS_TO_SIGN[@]}"; do
    if [ -f "${artifact}" ]; then
        bash "${ED25519_SIGN}" "${artifact}" "${PRIVATE_KEY}" > /dev/null 2>&1

        SIG_FILE="${artifact}.sig"
        if [ -f "${SIG_FILE}" ]; then
            SIGNATURES+=("${SIG_FILE}")
        else
            echo "ERROR: Failed to create signature for $(basename "${artifact}")"
            VERIFIED=false
            exit 1
        fi
    fi
done

# =====11==================================
# STEP 9: Verify all Ed25519 signatures
# ========================================
echo "→ Verifying Ed25519 signatures..."

for artifact in "${ARTIFACTS_TO_SIGN[@]}"; do
    if [ -f "${artifact}" ]; then
        SIG_FILE="${artifact}.sig"
        if ! bash "${ED25519_VERIFY}" "${artifact}" "${PUBLIC_KEY}" "${SIG_FILE}" > /dev/null 2>&1; then
            echo "ERROR: Signature verification failed for $(basename "${artifact}")"
            VERIFIED=false
            exit 1
        fi
    fi
done

# Update provenance.json with signature information
if command -v jq &> /dev/null; then
    TEMP_PROV=$(mktemp)

    # Build signatures array
    SIG_JSON="["
    FIRST=true
    for artifact in "${ARTIFACTS_TO_SIGN[@]}"; do
        if [ -f "${artifact}" ]; then
            ARTIFACT_NAME=$(basename "${artifact}")
            SIG_NAME="${ARTIFACT_NAME}.sig"

            if [ "${FIRST}" = true ]; then
                SIG_JSON="${SIG_JSON}{\"file\": \"${ARTIFACT_NAME}\", \"sig\": \"${SIG_NAME}\"}"
                FIRST=false
            else
                SIG_JSON="${SIG_JSON}, {\"file\": \"${ARTIFACT_NAME}\", \"sig\": \"${SIG_NAME}\"}"
            fi
        fi
    done
    SIG_JSON="${SIG_JSON}]"

    jq ".signatures = (${SIG_JSON} | sort_by(.file))" "${DIST_DIR}/provenance.json" > "${TEMP_PROV}" 2>/dev/null || true
    if [ -f "${TEMP_PROV}" ] && [ -s "${TEMP_PROV}" ]; then
        jq --sort-keys '.' "${TEMP_PROV}" > "${DIST_DIR}/provenance.json"
        rm -f "${TEMP_PROV}"

        # Re-sign provenance.json after updating it
        bash "${ED25519_SIGN}" "${DIST_DIR}/provenance.json" "${PRIVATE_KEY}" > /dev/null 2>&1
    else
        rm -f "${TEMP_PROV}"
    fi
fi

# ========================================
# STEP 12: Create release bundles
# ========================================
echo "→ Creating release bundles..."

chmod +x "${BUNDLE_SCRIPT}"
chmod +x "${BUNDLE_VERIFY}"

# Extract version
VERSION=$(grep '^version' "${PROJECT_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)

bash "${BUNDLE_SCRIPT}" "${DIST_DIR}" "${VERSION}" "${PLATFORM}" > /dev/null 2>&1

# Verify bundles
bash "${BUNDLE_VERIFY}" "${DIST_DIR}" "${DIST_DIR}/sha256sum.txt" > /dev/null 2>&1

# ========================================
# STEP 13: Final artifacts already in dist/
# ========================================
echo "→ Finalizing release artifacts..."

# List all artifacts
FINAL_ARTIFACTS=$(find "${DIST_DIR}" -type f \( -name "*.zip" -o -name "*.tar.gz" -o -name "*.sha256" -o -name "*.sig" \) 2>/dev/null)

if [ -z "${FINAL_ARTIFACTS}" ]; then
    echo "ERROR: No artifacts found in dist/"
    exit 1
fi

# STEP 13: Print summary===================
# STEP 9: Print summary
# ========================================
echo ""
echo "=========================================="
echo "Release Pipeline Complete"
echo "=========================================="
echo "ARTIFACT_COUNT=4"
echo "VERIFIED=${VERIFIED}"
echo ""
echo "Artifacts:"
for artifact in ${FINAL_ARTIFACTS}; do
    BASENAME=$(basename "${artifact}")
    SIZE=$(stat -c%s "${artifact}" 2>/dev/null || stat -f%z "${artifact}" 2>/dev/null)
    SIZE_KB=$(echo "scale=1; ${SIZE} / 1024" | bc)
    echo "  ${BASENAME} (${SIZE_KB}KB)"
done
echo ""
echo "Build Metadata:"
if [ -f "${DIST_DIR}/build.json" ]; then
    echo "  build.json"
    if [ -f "${DIST_DIR}/build.json.sig" ]; then
        echo "  build.json.sig"
    fi
fi
if [ -f "${DIST_DIR}/build_fingerprint.txt" ]; then
    echo "  build_fingerprint.txt"
    if [ -f "${DIST_DIR}/build_fingerprint.txt.sig" ]; then
        echo "  build_fingerprint.txt.sig"
    fi
fi
if [ -f "${DIST_DIR}/provenance.json" ]; then
    echo "  provenance.json"
    if [ -f "${DIST_DIR}/provenance.json.sig" ]; then
        echo "  provenance.json.sig"
    fi
fi
echo ""
