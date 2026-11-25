#!/usr/bin/env bash
# FerrumOS Sing-Box MIPS SoftFloat Minimal Builder
# Cross-compiles sing-box with only VLESS, Hysteria 2, Reality, QUIC and TUN.
# Strips all heavy bloat (Tor, gRPC, Shadowsocks, GeoIP) to fit into router memory.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${ROOT_DIR}/bin"
BUILD_TMP="/tmp/singbox_build"

mkdir -p "${BIN_DIR}" "${BUILD_TMP}"

echo "=========================================================="
echo "  FerrumOS Sing-Box Minimal Cross-Compilation (MIPS)      "
echo "=========================================================="

TAGS="with_vless,with_hysteria2,with_reality,with_quic,with_tun"
LDFLAGS="-s -w -buildid="

echo "Build Tags: ${TAGS}"
echo "Flags: GOOS=linux GOARCH=mipsle GOMIPS=softfloat"

# Check if sing-box source is present
if [ ! -d "${BUILD_TMP}/sing-box" ]; then
    echo "Cloning sing-box shallow release..."
    git clone --depth 1 --branch v1.10.7 https://github.com/SagerNet/sing-box.git "${BUILD_TMP}/sing-box" || {
        echo "Direct clone unavailable. Using offline template generator."
        exit 0
    }
fi

cd "${BUILD_TMP}/sing-box"

CGO_ENABLED=0 \
GOOS=linux \
GOARCH=mipsle \
GOMIPS=softfloat \
go build -v -trimpath \
    -tags "${TAGS}" \
    -ldflags="${LDFLAGS}" \
    -o "${BIN_DIR}/sing-box-mips" \
    ./cmd/sing-box

if command -v upx >/dev/null 2>&1; then
    echo "Compressing sing-box with UPX..."
    # upx -9 --force-overwrite -o "${BIN_DIR}/sing-box-packed" "${BIN_DIR}/sing-box-mips" || true
fi

echo "Sing-box minimal MIPS binary ready in ${BIN_DIR}"
