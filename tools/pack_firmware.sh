#!/usr/bin/env bash
# FerrumOS Firmware Image Packager
# Integrates Ferrum-Core Rust daemon, Zapret NFQWS, and init scripts
# into the final flashable .trx image for NAG SNR-CPE-MD1.1 (MT7620A).

set -euo pipefail

# Re-run inside container if not root to handle /dev block and char device nodes
if [ "$(id -u)" != "0" ]; then
    echo "[FerrumOS] Invoking firmware packager inside padavan-builder container..."
    exec docker run --rm -v /home/mihail:/home/mihail -w /home/mihail/FerrumOS padavan-builder bash tools/pack_firmware.sh
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
PADAVAN_ROOT="/home/mihail/padavan_build/padavan-ng/trunk"
TOOLS_DIR="${PADAVAN_ROOT}/tools"
OUT_DIR="${ROOT_DIR}/release"
WORK_DIR="${ROOT_DIR}/build_tmp"

mkdir -p "${OUT_DIR}" "${WORK_DIR}"

echo "=========================================================="
echo "  FerrumOS Firmware Packaging: NAG SNR-CPE-MD1.1 (MT7620) "
echo "=========================================================="

# Ensure ferrum-core binary is built
if [ ! -f "${ROOT_DIR}/bin/ferrum-core" ]; then
    echo "[1/5] Building ferrum-core MIPS binary..."
    make -C "${ROOT_DIR}" build-mips
fi

echo "[2/5] Preparing rootfs staging area..."
rm -rf "${WORK_DIR}/romfs"
cp -a "${PADAVAN_ROOT}/romfs" "${WORK_DIR}/romfs"

# Install Ferrum-Core binary (unpacked, safe for MIPS Linux 3.4)
echo "[3/5] Installing Ferrum-Core Rust daemon, symlinks, and init services..."
mkdir -p "${WORK_DIR}/romfs/usr/bin" "${WORK_DIR}/romfs/etc_ro/init.d"
cp "${ROOT_DIR}/bin/ferrum-core" "${WORK_DIR}/romfs/usr/bin/ferrum-core"
chmod +x "${WORK_DIR}/romfs/usr/bin/ferrum-core"

# Replace legacy C httpd with ferrum-core so Padavan rc starts Ferrum-Core on port 80
rm -f "${WORK_DIR}/romfs/usr/sbin/httpd"
ln -sf /usr/bin/ferrum-core "${WORK_DIR}/romfs/usr/sbin/httpd"

# Install Init Scripts into /etc_ro/init.d (read-only persistent rootfs)
cp "${ROOT_DIR}/init/S01switch" "${WORK_DIR}/romfs/etc_ro/init.d/S01switch"
cp "${ROOT_DIR}/init/S10network" "${WORK_DIR}/romfs/etc_ro/init.d/S10network"
cp "${ROOT_DIR}/init/S15ssh" "${WORK_DIR}/romfs/etc_ro/init.d/S15ssh"
cp "${ROOT_DIR}/init/S20ferrum" "${WORK_DIR}/romfs/etc_ro/init.d/S20ferrum"
cp "${ROOT_DIR}/init/S30zapret" "${WORK_DIR}/romfs/etc_ro/init.d/S30zapret"
cp "${ROOT_DIR}/init/S40singbox" "${WORK_DIR}/romfs/etc_ro/init.d/S40singbox"
chmod +x "${WORK_DIR}/romfs/etc_ro/init.d/"*

# Patch dev_init.sh:
# 1. Expand /tmp tmpfs from 24M to 48M so 24MB sing-box can be downloaded/extracted in RAM without ENOSPC
sed -i 's/size_tmp="24M"/size_tmp="48M"/g' "${WORK_DIR}/romfs/sbin/dev_init.sh"

# 2. Symlink /etc/init.d and start SSH and Ferrum-Core web daemon early
sed -i '/mount -t tmpfs tmpfs \/etc/a \ln -sf /etc_ro/init.d /etc/init.d\n/etc/init.d/S15ssh start &\n/etc/init.d/S20ferrum start &' "${WORK_DIR}/romfs/sbin/dev_init.sh"

# Install robust udhcpc callback script for automatic WAN DHCP IP, gateway and DNS resolution
mkdir -p "${WORK_DIR}/romfs/usr/share/udhcpc"
cat << 'EOF' > "${WORK_DIR}/romfs/usr/share/udhcpc/default.script"
#!/bin/sh
RESOLV_CONF="/etc/resolv.conf"

case "$1" in
    deconfig)
        /sbin/ifconfig $interface 0.0.0.0
        ;;
    renew|bound)
        /sbin/ifconfig $interface $ip netmask ${subnet:-255.255.255.0} ${broadcast:+broadcast $broadcast}

        if [ -n "$router" ]; then
            while /sbin/route del default gw 0.0.0.0 dev $interface 2>/dev/null; do :; done
            for i in $router; do
                /sbin/route add default gw $i dev $interface
            done
        fi

        echo -n > /tmp/resolv.conf.wan
        for i in $dns; do
            echo "nameserver $i" >> /tmp/resolv.conf.wan
        done

        if [ -f /tmp/dnsmasq.servers ]; then
            for i in $dns; do
                if ! grep -q "server=$i" /tmp/dnsmasq.servers 2>/dev/null; then
                    echo "server=$i" >> /tmp/dnsmasq.servers
                fi
            done
            killall -SIGHUP dnsmasq 2>/dev/null || true
        else
            cp -f /tmp/resolv.conf.wan $RESOLV_CONF 2>/dev/null || true
        fi
        ;;
esac

exit 0
EOF
chmod +x "${WORK_DIR}/romfs/usr/share/udhcpc/default.script"

# Ensure mtd_storage.sh starts SSH, Zapret, and restores Sing-Box on firewall reload
sed -i 's|/usr/bin/run_zapret.sh start &|/etc/init.d/S15ssh start \&\n/etc/init.d/S30zapret start \&\n[ -f /var/run/sing-box.pid ] \&\& /etc/init.d/S40singbox restart \&|g' "${WORK_DIR}/romfs/sbin/mtd_storage.sh"

# Neutralize legacy run_zapret.sh so rc / started_script never injects UDP 443 REJECT
cat << 'EOF' > "${WORK_DIR}/romfs/usr/bin/run_zapret.sh"
#!/bin/sh
exec /etc/init.d/S30zapret "$@"
EOF
chmod +x "${WORK_DIR}/romfs/usr/bin/run_zapret.sh"

echo "[4/5] Packing SquashFS XZ rootfs..."
RAMDISK="${WORK_DIR}/ramdisk"
rm -f "${RAMDISK}"
"${TOOLS_DIR}/mksquashfs_xz/mksquashfs" "${WORK_DIR}/romfs" "${RAMDISK}" \
    -all-root -no-exports -noappend -nopad -noI -no-xattrs

ROOTFS_SIZE=$(stat -c %s "${RAMDISK}")
echo "Packed RootFS size: ${ROOTFS_SIZE} bytes"

echo "[5/5] Assembling final .trx firmware image..."
KERNELZ="${PADAVAN_ROOT}/images/zImage"
VMLINUX="${PADAVAN_ROOT}/linux-3.4.x/vmlinux"
PART_ALIGN=64

# Freshly compress raw kernel image with lzma (without old ramdisk)
rm -f "${WORK_DIR}/zImage"*
"${TOOLS_DIR}/lzma/lzma" e -a1 -d25 "${KERNELZ}" "${WORK_DIR}/zImage.lzma"

# Pad kernel to 64-byte alignment
KSIZE=$(stat -c %s "${WORK_DIR}/zImage.lzma")
KSIZE_PAD=$((KSIZE + 64))
DIV=$((KSIZE_PAD % PART_ALIGN))
if [ "${DIV}" -gt 0 ]; then
    PAD=$((PART_ALIGN - DIV))
    dd if=/dev/zero count=1 bs="${PAD}" conv=sync 2>/dev/null | tr \\000 \\377 >> "${WORK_DIR}/zImage.lzma"
fi

# Append new SquashFS-XZ rootfs to padded kernel
cat "${RAMDISK}" >> "${WORK_DIR}/zImage.lzma"

# Prepare mkimage headers
TOTAL_SIZE=$(stat -c %s "${WORK_DIR}/zImage.lzma")
ENTRY=$(LANG=C LC_ALL=C readelf -h "${VMLINUX}" | grep "Entry" | awk '{print $4}')
KRN_SIZE=$((TOTAL_SIZE - ROOTFS_SIZE + 64))

TRX_OUTPUT="${OUT_DIR}/FerrumOS-v1.0.0-SNR-MD1.trx"
"${TOOLS_DIR}/mkimage/mkimage" -A mips -O linux -T kernel -C lzma \
    -a 80000000 -e "${ENTRY}" -k "${KRN_SIZE}" \
    -n "SNR-MD1" -V 3.4 3.9 -d "${WORK_DIR}/zImage.lzma" "${TRX_OUTPUT}"

FW_SIZE=$(stat -c %s "${TRX_OUTPUT}")
SHA256=$(sha256sum "${TRX_OUTPUT}" | awk '{print $1}')

echo "=========================================================="
echo "  FIRMWARE BUILD COMPLETE: ${TRX_OUTPUT}"
echo "  Firmware Size: ${FW_SIZE} bytes (Max: 8,060,928 bytes)"
echo "  SHA-256:       ${SHA256}"
echo "=========================================================="
