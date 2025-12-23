#!/bin/bash
set -euo pipefail

# Create macOS PKG installer for CostPilot
# Usage: ./scripts/create_macos_installer.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

echo "Creating macOS PKG installer for CostPilot ${VERSION}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
PKG_DIR="${PROJECT_ROOT}/target/pkg"

# Clean and create directories
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/scripts"
mkdir -p "$PKG_DIR/root/usr/local/bin"

# Copy binary
cp "$BUILD_DIR/costpilot" "$PKG_DIR/root/usr/local/bin/"

# Create preinstall script
cat > "$PKG_DIR/scripts/preinstall" << 'EOF'
#!/bin/bash
# Pre-installation script for CostPilot
echo "Installing CostPilot..."
EOF
chmod 755 "$PKG_DIR/scripts/preinstall"

# Create postinstall script
cat > "$PKG_DIR/scripts/postinstall" << 'EOF'
#!/bin/bash
# Post-installation script for CostPilot
echo "CostPilot installed successfully!"
echo "Run 'costpilot --help' to get started."
echo "Documentation: https://github.com/guardsuite/costpilot"
EOF
chmod 755 "$PKG_DIR/scripts/postinstall"

# Create distribution file
cat > "$PKG_DIR/Distribution" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1">
    <title>CostPilot</title>
    <license file="LICENSE" />
    <pkg-ref id="com.guardsuite.costpilot"/>
    <options customize="never" require-scripts="false"/>
    <domains enable_localSystem="true"/>
    <choices-outline>
        <line choice="default">
            <line choice="com.guardsuite.costpilot"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="com.guardsuite.costpilot" visible="false">
        <pkg-ref id="com.guardsuite.costpilot"/>
    </choice>
    <pkg-ref id="com.guardsuite.costpilot" version="${VERSION}" onConclusion="none">costpilot.pkg</pkg-ref>
</installer-gui-script>
EOF

# Create component plist
cat > "$PKG_DIR/root/usr/local/bin/costpilot.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.guardsuite.costpilot</string>
    <key>CFBundleName</key>
    <string>CostPilot</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
</dict>
</plist>
EOF

# Copy license
cp "$PROJECT_ROOT/LICENSE" "$PKG_DIR/"

# Build component package
cd "$PKG_DIR"
pkgbuild --root root \
         --scripts scripts \
         --identifier com.guardsuite.costpilot \
         --version "${VERSION}" \
         --install-location / \
         costpilot.pkg

# Build distribution package
productbuild --distribution Distribution \
             --package-path . \
             "${PROJECT_ROOT}/target/costpilot-${VERSION}.pkg"

echo "macOS PKG installer created: ${PROJECT_ROOT}/target/costpilot-${VERSION}.pkg"</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/create_macos_installer.sh
