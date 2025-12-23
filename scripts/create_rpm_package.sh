#!/bin/bash
set -euo pipefail

# Create RPM package installer for CostPilot
# Usage: ./scripts/create_rpm_package.sh <version>

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

echo "Creating RPM package for CostPilot ${VERSION}"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
RPM_DIR="${PROJECT_ROOT}/target/rpm"
PKG_NAME="costpilot"

# Clean and create directories
rm -rf "$RPM_DIR"
mkdir -p "$RPM_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
cd "$PROJECT_ROOT"
tar czf "$RPM_DIR/SOURCES/${PKG_NAME}-${VERSION}.tar.gz" \
    --transform "s|^|${PKG_NAME}-${VERSION}/|" \
    target/release/costpilot

# Create spec file
cat > "$RPM_DIR/SPECS/${PKG_NAME}.spec" << EOF
Name:           ${PKG_NAME}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Zero-IAM FinOps engine for Terraform

License:        MIT
URL:            https://github.com/guardsuite/costpilot
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64
Requires:       glibc >= 2.27

%description
CostPilot analyzes infrastructure-as-code changes before they merge
and blocks only irreversible cloud cost regressions.

Features:
* Deterministic cost analysis
* No cloud credentials required
* Supports Terraform, CDK, CloudFormation
* Enterprise-grade SLO monitoring

%prep
%setup -q

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/costpilot %{buildroot}%{_bindir}/costpilot

%files
%license LICENSE
%doc README.md
%{_bindir}/costpilot

%changelog
* $(date '+%a %b %d %Y') GuardSuite <support@guardsuite.com> - ${VERSION}-1
- Release ${VERSION}
EOF

# Build the RPM
cd "$RPM_DIR"
rpmbuild --define "_topdir $RPM_DIR" -ba SPECS/${PKG_NAME}.spec

# Move the built RPM to target directory
find "$RPM_DIR/RPMS" -name "*.rpm" -exec mv {} "$PROJECT_ROOT/target/" \;

echo "RPM package created: ${PROJECT_ROOT}/target/${PKG_NAME}-${VERSION}-1.x86_64.rpm"</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/scripts/create_rpm_package.sh
