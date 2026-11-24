#!/usr/bin/env bash
# FerrumOS Automated MIPS Cross-Compilation Script
# Builds the Rust Ferrum-Core binary for MT7620 (MIPS 24Kc softfloat uClibc)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
CORE_DIR="${ROOT_DIR}/core"
OUT_DIR="${ROOT_DIR}/bin"

mkdir -p "${OUT_DIR}"

echo "=========================================================="
echo "  FerrumOS Cross-Compilation for MIPS MT7620 (Rust)       "
echo "=========================================================="

cd "${CORE_DIR}"

echo "[1/3] Building ferrum-core with cargo nightly build-std..."
cargo +nightly build \
    -Z build-std=std,panic_abort \
    --target mipsel-unknown-linux-uclibc \
    --release

RAW_BIN="${CORE_DIR}/target/mipsel-unknown-linux-uclibc/release/ferrum-core"
DEST_BIN="${OUT_DIR}/ferrum-core"

cp "${RAW_BIN}" "${DEST_BIN}"

echo "[2/3] Checking binary properties..."
file "${DEST_BIN}"
ls -lh "${DEST_BIN}"

# Note: Do NOT use UPX on MIPS (MT7620/Linux 3.4).
# UPX decompressor stubs cause cache incoherency and SIGTRAP (Trace/breakpoint trap) on MIPS 24KEc.
# SquashFS-XZ handles rootfs-level transparent compression natively and safely.

echo "=========================================================="
echo "  Build completed successfully! Output: ${OUT_DIR}         "
echo "=========================================================="
