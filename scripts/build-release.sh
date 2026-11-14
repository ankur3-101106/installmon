#!/usr/bin/env bash
set -euo pipefail

# Script to build release binaries for InstallMon
TARGET="${1:-x86_64-unknown-linux-gnu}"
VERSION="v0.1.0"

echo "=== Building InstallMon Release Binary for Target: ${TARGET} ==="

mkdir -p dist/staging

# Build target binary with static linking
if [[ "${TARGET}" == *"x86_64"* ]]; then
    cargo rustc -p installmon-cli --release --bin installmon -- -C target-feature=+crt-static
    BINARY_PATH="target/release/installmon"
else
    cargo build --release --target "${TARGET}" --bin installmon
    BINARY_PATH="target/${TARGET}/release/installmon"
fi

# Strip binary
strip "${BINARY_PATH}"

# Package
ARCH_NAME=$(echo "${TARGET}" | cut -d'-' -f1)
OUTPUT_ARCHIVE="dist/installmon-${VERSION}-linux-${ARCH_NAME}.tar.gz"

cp "${BINARY_PATH}" dist/staging/installmon
cp README.md LICENSE dist/staging/

tar -czvf "${OUTPUT_ARCHIVE}" -C dist/staging installmon README.md LICENSE
rm -rf dist/staging

# Checksum
(cd dist && sha256sum "installmon-${VERSION}-linux-${ARCH_NAME}.tar.gz" > "installmon-${VERSION}-linux-${ARCH_NAME}.tar.gz.sha256")

echo "=== Release package created at: ${OUTPUT_ARCHIVE} ==="
cat "${OUTPUT_ARCHIVE}.sha256"
