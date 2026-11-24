#!/usr/bin/env bash
# ==============================================================================
# NetFreedom Linux Installation Script
# Installs netfreedom-cli, netfreedom-daemon, systemd unit, and Polkit policy
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== NetFreedom Linux Installer ==="

if [ "$(id -u)" -ne 0 ]; then
    echo "[-] This installation script requires root privileges. Please run with sudo."
    exit 1
fi

echo "[1/4] Building release binaries & ByeDPI engine..."
cd "${ROOT_DIR}"
cargo build --release -p netfreedom-cli -p netfreedom-daemon

if [ -d "${ROOT_DIR}/third_party/byedpi" ]; then
    echo "[*] Compiling native ByeDPI engine..."
    cd "${ROOT_DIR}/third_party/byedpi"
    gcc -O2 -I. -D_DEFAULT_SOURCE packets.c main.c conev.c proxy.c desync.c mpool.c extend.c -o ciadpi || true
fi

echo "[2/4] Installing binaries to /usr/local/bin..."
install -m 755 "${ROOT_DIR}/target/release/netfreedom" /usr/local/bin/netfreedom
install -m 755 "${ROOT_DIR}/target/release/netfreedom-daemon" /usr/local/bin/netfreedom-daemon
if [ -f "${ROOT_DIR}/third_party/byedpi/ciadpi" ]; then
    install -m 755 "${ROOT_DIR}/third_party/byedpi/ciadpi" /usr/local/bin/ciadpi
fi

echo "[3/4] Installing systemd service..."
install -m 644 "${SCRIPT_DIR}/netfreedom-daemon.service" /etc/systemd/system/netfreedom-daemon.service
systemctl daemon-reload
systemctl enable netfreedom-daemon.service

echo "[4/4] Installing Polkit policy..."
if [ -d "/usr/share/polkit-1/actions" ]; then
    install -m 644 "${SCRIPT_DIR}/org.netfreedom.policy" /usr/share/polkit-1/actions/org.netfreedom.policy
fi

echo "=== NetFreedom Installed Successfully! ==="
echo "Run 'systemctl start netfreedom-daemon' to start the background service."
echo "Run 'netfreedom auto' or 'netfreedom status' to manage network connectivity."
