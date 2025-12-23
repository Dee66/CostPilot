#!/bin/bash
set -euo pipefail

# Create Windows MSI installer for CostPilot
# Requires: WiX Toolset (wixl or candle/light)
# Usage: ./scripts/create_windows_installer.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

echo "Creating Windows MSI installer for CostPilot ${VERSION}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
MSI_DIR="${PROJECT_ROOT}/target/msi"

# Clean and create directories
rm -rf "$MSI_DIR"
mkdir -p "$MSI_DIR"

# Create WiX source file
cat > "$MSI_DIR/costpilot.wxs" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="CostPilot" Language="1033" Version="${VERSION}.0" Manufacturer="GuardSuite" UpgradeCode="YOUR-GUID-HERE">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />

    <MajorUpgrade DowngradeErrorMessage="A newer version of [ProductName] is already installed." />
    <MediaTemplate />

    <Feature Id="ProductFeature" Title="CostPilot" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
  </Product>

  <Fragment>
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="CostPilotDir" Name="CostPilot">
          <Component Id="CostPilotExe" Guid="YOUR-GUID-HERE">
            <File Id="CostPilotExe" Source="${BUILD_DIR}/costpilot.exe" KeyPath="yes" />
          </Component>
        </Directory>
      </Directory>
    </Directory>
  </Fragment>

  <Fragment>
    <ComponentGroup Id="ProductComponents">
      <ComponentRef Id="CostPilotExe" />
    </ComponentGroup>
  </Fragment>
</Wix>
EOF

# Copy binary
cp "${BUILD_DIR}/costpilot.exe" "$MSI_DIR/"

# Try to build MSI if WiX tools are available
if command -v wixl >/dev/null 2>&1; then
    echo "Building MSI with wixl..."
    cd "$MSI_DIR"
    wixl -v costpilot.wxs -o "${PROJECT_ROOT}/target/costpilot-${VERSION}-x64.msi"
    echo "MSI installer created: ${PROJECT_ROOT}/target/costpilot-${VERSION}-x64.msi"
elif command -v candle >/dev/null 2>&1 && command -v light >/dev/null 2>&1; then
    echo "Building MSI with candle/light..."
    cd "$MSI_DIR"
    candle costpilot.wxs
    light costpilot.wixobj -o "${PROJECT_ROOT}/target/costpilot-${VERSION}-x64.msi"
    echo "MSI installer created: ${PROJECT_ROOT}/target/costpilot-${VERSION}-x64.msi"
else
    echo "WiX tools not found. MSI source files created in ${MSI_DIR}"
    echo "To build MSI manually:"
    echo "  1. Install WiX Toolset: https://wixtoolset.org/"
    echo "  2. Run: candle costpilot.wxs && light costpilot.wixobj -o costpilot-${VERSION}-x64.msi"
fi</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/create_windows_installer.sh
