#!/usr/bin/env bash
# ==============================================================================
# NetFreedom Apple (iOS & macOS) Frameworks Build Script
# Compiles HevSocks5Tunnel.xcframework and ByeDPI for Apple targets:
#   - iOS Device (iphoneos arm64)
#   - iOS Simulator (iphonesimulator x86_64 + arm64)
#   - macOS (macosx universal x86_64 + arm64)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== NetFreedom Apple Frameworks & Native Engine Builder ==="

if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "[-] This script must be executed on macOS with Xcode command-line tools."
    echo "[-] Current OS: ${OSTYPE}"
    exit 1
fi

echo "[1/3] Building HevSocks5Tunnel.xcframework..."
cd "${ROOT_DIR}/third_party/hev-socks5-tunnel"
git submodule update --init --recursive
chmod +x build-apple.sh
./build-apple.sh

echo "[2/3] Building ByeDPI (ciadpi) for macOS universal (arm64 + x86_64)..."
BYEDPI_DIR="${ROOT_DIR}/third_party/byedpi"
OUTPUT_DIR="${ROOT_DIR}/dist/apple"
mkdir -p "${OUTPUT_DIR}"

C_SOURCES=(
    "${BYEDPI_DIR}/conev.c"
    "${BYEDPI_DIR}/desync.c"
    "${BYEDPI_DIR}/extend.c"
    "${BYEDPI_DIR}/mpool.c"
    "${BYEDPI_DIR}/packets.c"
    "${BYEDPI_DIR}/proxy.c"
    "${BYEDPI_DIR}/main.c"
)

# Compile for macOS arm64
clang -O2 -Wall -D_GNU_SOURCE -target arm64-apple-macos11.0 "${C_SOURCES[@]}" -o /tmp/ciadpi-macos-arm64
# Compile for macOS x86_64
clang -O2 -Wall -D_GNU_SOURCE -target x86_64-apple-macos10.15 "${C_SOURCES[@]}" -o /tmp/ciadpi-macos-x86_64
# Create Universal Binary
lipo -create /tmp/ciadpi-macos-arm64 /tmp/ciadpi-macos-x86_64 -output "${OUTPUT_DIR}/ciadpi-macos-universal"

echo "[3/3] Packaging Apple distribution artifacts..."
cp -r "${ROOT_DIR}/third_party/hev-socks5-tunnel/HevSocks5Tunnel.xcframework" "${OUTPUT_DIR}/"
cd "${OUTPUT_DIR}"
zip -r "NetFreedom-Apple-HevSocks5Tunnel.xcframework.zip" "HevSocks5Tunnel.xcframework"

echo "=== Apple Frameworks Built Successfully! ==="
ls -lh "${OUTPUT_DIR}"
