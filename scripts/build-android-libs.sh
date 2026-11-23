#!/usr/bin/env bash
# ==============================================================================
# NetFreedom Android Native Libraries Build Script
# Compiles hev-socks5-tunnel and byedpi (ciadpi) for Android ABIs (arm64-v8a, x86_64)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ANDROID_DIR="${ROOT_DIR}/apps/android"
NDK_HOME="${ANDROID_NDK_HOME:-${NDK_HOME:-/home/adityapatra/Android/Sdk/ndk/28.2.13676358}}"

if [ ! -d "${NDK_HOME}" ]; then
    echo "[-] Android NDK not found at ${NDK_HOME}. Please set ANDROID_NDK_HOME."
    exit 1
fi

echo "[+] Using Android NDK: ${NDK_HOME}"

# 1. Build hev-socks5-tunnel via ndk-build
echo "[+] Building hev-socks5-tunnel for arm64-v8a and x86_64..."
cd "${ROOT_DIR}/third_party/hev-socks5-tunnel"
git submodule update --init --recursive
"${NDK_HOME}/ndk-build" NDK_PROJECT_PATH=. APP_BUILD_SCRIPT=Android.mk NDK_APPLICATION_MK=Application.mk APP_ABI="arm64-v8a x86_64" -j8

# 2. Build byedpi for arm64-v8a and x86_64
echo "[+] Building ByeDPI (ciadpi) native binaries..."
TOOLCHAIN="${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin"
BYEDPI_DIR="${ROOT_DIR}/third_party/byedpi"

mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a"
mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/x86_64"

# arm64-v8a
"${TOOLCHAIN}/aarch64-linux-android35-clang" -O2 -Wall -D_GNU_SOURCE \
    "${BYEDPI_DIR}/conev.c" "${BYEDPI_DIR}/desync.c" "${BYEDPI_DIR}/extend.c" \
    "${BYEDPI_DIR}/mpool.c" "${BYEDPI_DIR}/packets.c" "${BYEDPI_DIR}/proxy.c" "${BYEDPI_DIR}/main.c" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a/libciadpi.so"

# x86_64
"${TOOLCHAIN}/x86_64-linux-android35-clang" -O2 -Wall -D_GNU_SOURCE \
    "${BYEDPI_DIR}/conev.c" "${BYEDPI_DIR}/desync.c" "${BYEDPI_DIR}/extend.c" \
    "${BYEDPI_DIR}/mpool.c" "${BYEDPI_DIR}/packets.c" "${BYEDPI_DIR}/proxy.c" "${BYEDPI_DIR}/main.c" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/x86_64/libciadpi.so"

# Copy hev-socks5-tunnel shared libraries
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/arm64-v8a/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a/"
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/x86_64/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/x86_64/"

echo "[+] Android native libraries built and staged successfully:"
ls -lh "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a/"
ls -lh "${ANDROID_DIR}/app/src/main/jniLibs/x86_64/"
