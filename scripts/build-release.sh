#!/usr/bin/env bash
set -euo pipefail

# Script to build and package release binaries for InstallMon
# Usage: ./scripts/build-release.sh [TARGET] [VERSION]

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TARGET="${1:-$(rustc -vV | sed -n 's|host: ||p')}"
VERSION="${2:-}"

if [[ -z "${VERSION}" ]]; then
    if git describe --tags --exact-match >/dev/null 2>&1; then
        VERSION="$(git describe --tags --exact-match)"
    else
        VERSION="v$(cargo metadata --no-deps --format-version 1 | grep -o '"version":"[^"]*"' | head -1 | cut -d'"' -f4)"
    fi
fi

# Ensure version starts with 'v'
if [[ "${VERSION}" != v* ]]; then
    VERSION="v${VERSION}"
fi

echo "============================================================"
echo " Building InstallMon Release Artifact"
echo " Target  : ${TARGET}"
echo " Version : ${VERSION}"
echo "============================================================"

HOST_TARGET="$(rustc -vV | sed -n 's|host: ||p')"
DIST_DIR="${ROOT_DIR}/dist"
STAGING_DIR="${DIST_DIR}/staging-${TARGET}"

mkdir -p "${DIST_DIR}"
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

if [[ "${TARGET}" == "${HOST_TARGET}" ]]; then
    echo "Building for host target (${TARGET})..."
    cargo build --release --bin installmon
    BINARY_PATH="target/release/installmon"
else
    echo "Building for cross/specified target (${TARGET})..."
    cargo build --release --target "${TARGET}" --bin installmon
    BINARY_PATH="target/${TARGET}/release/installmon"
fi

if [[ ! -f "${BINARY_PATH}" ]]; then
    echo "Error: Binary not found at ${BINARY_PATH}" >&2
    exit 1
fi

# Strip binary if strip is available
if command -v strip >/dev/null 2>&1; then
    echo "Stripping binary..."
    strip "${BINARY_PATH}" 2>/dev/null || true
fi

# Package archive
ARCHIVE_NAME="installmon-${VERSION}-${TARGET}.tar.gz"
ARCHIVE_PATH="${DIST_DIR}/${ARCHIVE_NAME}"

cp "${BINARY_PATH}" "${STAGING_DIR}/installmon"
chmod 0755 "${STAGING_DIR}/installmon"
cp README.md LICENSE "${STAGING_DIR}/"

echo "Creating release archive: ${ARCHIVE_NAME}..."
tar -czf "${ARCHIVE_PATH}" -C "${STAGING_DIR}" installmon README.md LICENSE
rm -rf "${STAGING_DIR}"

# Compute SHA256 checksum
echo "Generating SHA-256 checksum..."
(cd "${DIST_DIR}" && sha256sum "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256")

echo "============================================================"
echo " Release artifact created successfully:"
echo " Archive  : ${ARCHIVE_PATH}"
echo " Checksum : $(cat "${ARCHIVE_PATH}.sha256")"
echo "============================================================"
