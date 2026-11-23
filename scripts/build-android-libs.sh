#!/usr/bin/env bash
# ==============================================================================
# NetFreedom Android Multi-ABI Native Libraries Build Script
# Compiles hev-socks5-tunnel and byedpi (ciadpi) for all Android ABIs:
#   - arm64-v8a  (64-bit ARM - modern smartphones/tablets)
#   - armeabi-v7a (32-bit ARM - older devices)
#   - x86_64     (64-bit x86 - ChromeOS, emulators, Intel hardware)
#   - x86        (32-bit x86 - legacy emulators)
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

# 1. Build hev-socks5-tunnel via ndk-build for all 4 ABIs
echo "[+] Building hev-socks5-tunnel for all 4 ABIs (arm64-v8a, armeabi-v7a, x86_64, x86)..."
cd "${ROOT_DIR}/third_party/hev-socks5-tunnel"
git submodule update --init --recursive
"${NDK_HOME}/ndk-build" NDK_PROJECT_PATH=. APP_BUILD_SCRIPT=Android.mk NDK_APPLICATION_MK=Application.mk APP_ABI="arm64-v8a armeabi-v7a x86_64 x86" -j8

# 2. Build byedpi for all 4 ABIs
echo "[+] Building ByeDPI (ciadpi) native binaries for all 4 ABIs..."
TOOLCHAIN="${NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin"
BYEDPI_DIR="${ROOT_DIR}/third_party/byedpi"

C_SOURCES=(
    "${BYEDPI_DIR}/conev.c"
    "${BYEDPI_DIR}/desync.c"
    "${BYEDPI_DIR}/extend.c"
    "${BYEDPI_DIR}/mpool.c"
    "${BYEDPI_DIR}/packets.c"
    "${BYEDPI_DIR}/proxy.c"
    "${BYEDPI_DIR}/main.c"
)

# Detect clang compilers for each ABI dynamically
CLANG_ARM64=$(find "${TOOLCHAIN}" -maxdepth 1 -name "aarch64-linux-android*-clang" | grep -E 'android[0-9]+-clang$' | sort -V | tail -n 1 || true)
CLANG_ARM32=$(find "${TOOLCHAIN}" -maxdepth 1 -name "armv7a-linux-androideabi*-clang" | grep -E 'androideabi[0-9]+-clang$' | sort -V | tail -n 1 || true)
CLANG_X86_64=$(find "${TOOLCHAIN}" -maxdepth 1 -name "x86_64-linux-android*-clang" | grep -E 'android[0-9]+-clang$' | sort -V | tail -n 1 || true)
CLANG_X86=$(find "${TOOLCHAIN}" -maxdepth 1 -name "i686-linux-android*-clang" | grep -E 'android[0-9]+-clang$' | sort -V | tail -n 1 || true)

mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a"
mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/armeabi-v7a"
mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/x86_64"
mkdir -p "${ANDROID_DIR}/app/src/main/jniLibs/x86"

# arm64-v8a
echo " -> Compiling ciadpi for arm64-v8a using ${CLANG_ARM64}..."
"${CLANG_ARM64}" -O2 -Wall -D_GNU_SOURCE "${C_SOURCES[@]}" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a/libciadpi.so"

# armeabi-v7a
echo " -> Compiling ciadpi for armeabi-v7a using ${CLANG_ARM32}..."
"${CLANG_ARM32}" -O2 -Wall -D_GNU_SOURCE "${C_SOURCES[@]}" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/armeabi-v7a/libciadpi.so"

# x86_64
echo " -> Compiling ciadpi for x86_64 using ${CLANG_X86_64}..."
"${CLANG_X86_64}" -O2 -Wall -D_GNU_SOURCE "${C_SOURCES[@]}" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/x86_64/libciadpi.so"

# x86
echo " -> Compiling ciadpi for x86 using ${CLANG_X86}..."
"${CLANG_X86}" -O2 -Wall -D_GNU_SOURCE "${C_SOURCES[@]}" \
    -o "${ANDROID_DIR}/app/src/main/jniLibs/x86/libciadpi.so"

# Copy hev-socks5-tunnel shared libraries
echo "[+] Staging libhev-socks5-tunnel.so..."
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/arm64-v8a/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/arm64-v8a/"
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/armeabi-v7a/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/armeabi-v7a/"
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/x86_64/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/x86_64/"
cp "${ROOT_DIR}/third_party/hev-socks5-tunnel/libs/x86/libhev-socks5-tunnel.so" "${ANDROID_DIR}/app/src/main/jniLibs/x86/"

echo "[+] Multi-ABI Android native libraries built and staged successfully:"
ls -lh "${ANDROID_DIR}/app/src/main/jniLibs/"*
