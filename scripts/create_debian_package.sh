#!/bin/bash
set -euo pipefail

# Create Debian package installer for CostPilot
# Usage: ./scripts/create_debian_package.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

echo "Creating Debian package for CostPilot ${VERSION}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
DEB_DIR="${PROJECT_ROOT}/target/debian"
PKG_NAME="costpilot"
ARCH="amd64"

# Clean and create directories
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/usr/share/doc/${PKG_NAME}"
mkdir -p "$DEB_DIR/usr/share/man/man1"

# Copy binary
cp "$BUILD_DIR/costpilot" "$DEB_DIR/usr/bin/"

# Create control file
cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: GuardSuite <support@guardsuite.com>
Description: Zero-IAM FinOps engine for Terraform
 CostPilot analyzes infrastructure-as-code changes before they merge
 and blocks only irreversible cloud cost regressions.
 .
 Features:
  * Deterministic cost analysis
  * No cloud credentials required
  * Supports Terraform, CDK, CloudFormation
  * Enterprise-grade SLO monitoring
Homepage: https://github.com/guardsuite/costpilot
Section: utils
Priority: optional
Depends: libc6 (>= 2.27)
EOF

# Create postinst script
cat > "$DEB_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Post-installation script for CostPilot
echo "CostPilot installed successfully!"
echo "Run 'costpilot --help' to get started."
echo "Documentation: https://github.com/guardsuite/costpilot"
EOF
chmod 755 "$DEB_DIR/DEBIAN/postinst"

# Create prerm script
cat > "$DEB_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e

# Pre-removal script for CostPilot
echo "Removing CostPilot..."
EOF
chmod 755 "$DEB_DIR/DEBIAN/prerm"

# Create copyright file
cat > "$DEB_DIR/usr/share/doc/${PKG_NAME}/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: CostPilot
Upstream-Contact: GuardSuite <support@guardsuite.com>
Source: https://github.com/guardsuite/costpilot

Files: *
Copyright: 2024 GuardSuite
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF

# Create changelog
cat > "$DEB_DIR/usr/share/doc/${PKG_NAME}/changelog.Debian.gz" << EOF
costpilot (${VERSION}) unstable; urgency=medium

  * Release ${VERSION}

 -- GuardSuite <support@guardsuite.com>  $(date -R)
EOF
gzip "$DEB_DIR/usr/share/doc/${PKG_NAME}/changelog.Debian.gz"

# Build the package
cd "$PROJECT_ROOT/target"
dpkg-deb --build debian "${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "Debian package created: ${PROJECT_ROOT}/target/${PKG_NAME}_${VERSION}_${ARCH}.deb"</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/create_debian_package.sh
