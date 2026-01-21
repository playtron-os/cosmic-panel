#!/bin/bash
# Build RPM for cosmic-panel
# Usage: ./packaging/rpm/build-rpm.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$PROJECT_DIR"

BINARY="cosmic-panel"
VERSION="${COSMIC_PANEL_VERSION:-1.0.4}"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
RPM_ROOT="${CARGO_TARGET_DIR}/rpm-root"
RPM_BUILD="${CARGO_TARGET_DIR}/rpm-build"

echo "Building ${BINARY} version ${VERSION}..."

# Build release binary
cargo build --release

echo "Building RPM..."
rm -rf "${RPM_ROOT}" "${RPM_BUILD}"
mkdir -p "${RPM_ROOT}" "${RPM_BUILD}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Build RPM with source directory and version passed via environment
COSMIC_PANEL_SOURCE="${PROJECT_DIR}" COSMIC_PANEL_VERSION="${VERSION}" rpmbuild -bb --nodeps \
    --define "_topdir ${PROJECT_DIR}/${RPM_BUILD}" \
    --define "_binary_payload w2.xzdio" \
    --buildroot "${PROJECT_DIR}/${RPM_ROOT}" \
    packaging/rpm/cosmic-panel.spec

# Copy to dist folder
mkdir -p dist
cp -v "${RPM_BUILD}"/RPMS/*/*.rpm dist/

echo "RPM created in dist/"
ls -la dist/*.rpm
