# FerrumOS

> An embedded Linux operating system and memory-safe control plane engineered for network routing hardware based on the MediaTek MT7620A System-on-Chip (specifically the NAG SNR-CPE-MD1.1 platform).

[![Target SoC](https://img.shields.io/badge/SoC-MediaTek%20MT7620A-blue.svg)](https://www.mediatek.com/)
[![Architecture](https://img.shields.io/badge/Arch-MIPS%2024KEc%20(600MHz)-orange.svg)]()
[![Memory](https://img.shields.io/badge/RAM%2FFlash-64MB%20DDR2%20%2F%208MB%20SPI-green.svg)]()
[![License](https://img.shields.io/badge/License-GPLv2%20%2F%20MIT-brightgreen.svg)]()
[![Version](https://img.shields.io/badge/Version-v1.0.0--release-blue.svg)]()
[![Test Suite](https://img.shields.io/badge/Tests-50%2F50%20Pass%20(100%25)-brightgreen.svg)]()

---

## Overview

FerrumOS is an embedded router firmware distribution designed to replace monolithic, legacy C-based router operating systems. Built on a hardened Linux kernel and an asynchronous Rust systems daemon (`ferrum-core`), FerrumOS eliminates memory safety vulnerabilities, reduces idle memory consumption, prevents flash corruption, and delivers line-rate packet forwarding under strict hardware constraints.

The distribution targets the **NAG SNR-CPE-MD1.1** wireless router, powered by the MediaTek MT7620A SoC with 64 MB DDR2 SDRAM and 8 MB SPI-NOR flash memory.

---

## Architectural Highlights

- **Memory-Safe Core Daemon (`ferrum-core`)**: Userland management services, configuration transactions, REST APIs, and telemetry streaming are consolidated into a single asynchronous Rust daemon compiled with MUSL libc. Resident memory is bounded under 2.5 MB RSS.
- **Hardware & Software Flow Offloading**: Implements line-rate IPv4/IPv6 packet forwarding via the MediaTek Packet Processing Engine (PPE HWNAT) and Linux fast-path flowtable offload, sustaining 100 Mbps throughput at low single-digit CPU utilization.
- **Anti-Brick & Fail-Safe Guarantees**: Kernel-enforced write locks permanently protect bootloader (`mtd0`) and radio frequency calibration data (`mtd2`). User configuration uses an atomic A/B double-buffered partition format with CRC32 integrity verification.
- **DPI Mitigation & Dynamic Routing**: Native integration with Zapret (`nfqws`) for zero-copy TCP/UDP packet desynchronization, BGP routing via BIRD Lite for automated Antizapret table synchronization, and Sing-Box multi-protocol client supporting VLESS, Hysteria2, TUIC v5, and ShadowTLS v3.
- **Wireless Subsystems**: Dual-band concurrent operation utilizing the internal MT7620A 2.4 GHz 2T2R radio (`rt2860v2`) and the PCIe MT7610E 5 GHz 1T1R radio (`mt7610`). Includes IEEE 802.11k/v/r Fast BSS Transition for seamless client roaming and isolated guest virtual APs.
- **Zero-Overhead Management Interface**: Single-Page Application (SPA) compiled to pre-compressed static assets directly embedded in the binary image, eliminating server-side template interpretation overhead.
- **Diagnostic Tooling**: Integrated WAN throughput and latency measurement subsystem with microsecond-level timing accuracy.

---

## Target Hardware Specifications

| Subsystem | Specification |
| :--- | :--- |
| **SoC** | MediaTek MT7620A (Silicon Revision 0206) |
| **CPU Core** | MIPS 24KEc @ 600 MHz (PLL600 configured) |
| **Physical RAM** | 64 MB DDR2-400 (usable ~61.8 MB; 32 MB ZRAM compressed swap pool) |
| **Flash Memory** | 8 MB SPI NOR Flash (Winbond W25Q64BV) |
| **Ethernet Switch** | Embedded 5-Port 10/100 Mbps Fast Ethernet Switch (1 WAN + 4 LAN ports) |
| **Switch VLANs** | LAN: Ports 0–3 (`eth2.1`, VLAN ID 1); WAN: Port 4 (`eth2.2`, VLAN ID 2); CPU GMAC: Port 6 |
| **Wi-Fi 2.4 GHz** | MediaTek MT7620 integrated 802.11b/g/n (2T2R, up to 300 Mbps PHY rate) |
| **Wi-Fi 5 GHz** | MediaTek MT7610EN via internal PCIe (1T1R 802.11a/n/ac, up to 433 Mbps PHY rate) |
| **Acceleration** | MediaTek Packet Processing Engine (PPE HWNAT) + Linux Netfilter flowtable |
| **Peripherals** | Hardware Watchdog Timer (WDT), Reset Button (GPIO 13), Status LEDs |

---

## System Architecture

```
+-------------------------------------------------------------------------+
|                  Management Plane (Preact + Tailwind SPA)               |
+-------------------------------------------------------------------------+
                                    | HTTP/1.1 REST & WebSocket
                                    v
+-------------------------------------------------------------------------+
|                   FerrumOS Core Daemon (ferrum-core)                    |
|  - Asynchronous HTTP & WebSocket Event Loop                            |
|  - Strongly-Typed Transactional State Manager (A/B Flash Config)       |
|  - Process Supervisor: dnsmasq, dropbear, zapret (nfqws), sing-box      |
|  - Network Policy Controller & WAN Speedtest Subsystem                 |
+-------------------------------------------------------------------------+
       | Netlink / ioctl               | Unix Domain Sockets / Signals
       v                               v
+-------------------------------------------------------------------------+
|                       Linux Kernel (3.4.113 MIPS)                       |
|  - Ralink PPE HWNAT Driver (hw_nat.ko)                                 |
|  - Netfilter / iptables / NFQUEUE (Queue 200) / IPSET                  |
|  - Wireless Drivers: rt2860v2 (2.4 GHz) & mt7610 (5 GHz PCIe)          |
|  - MTD Partition Driver with Read-Only Lockout on mtd0 & mtd2          |
|  - ZRAM Block Device (50% Compressed In-Memory Swap)                   |
+-------------------------------------------------------------------------+
                                    |
                                    v
+-------------------------------------------------------------------------+
|                    Hardware Layer (MediaTek MT7620A)                    |
+-------------------------------------------------------------------------+
```

---

## Flash Memory Layout

The Winbond W25Q64BV SPI NOR flash (8,192 KB total) is partitioned as follows:

| Partition | Name | Start Offset | Size | Filesystem / Format | Access Policy |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `mtd0` | Bootloader | `0x00000000` | 192 KB | U-Boot Binary | **Read-Only (`MTD_CAP_ROM`)** |
| `mtd1` | Config | `0x00030000` | 64 KB | Transactional A/B (CRC32) | Read / Write |
| `mtd2` | Factory | `0x00040000` | 64 KB | EEPROM RF Calibration | **Read-Only (`MTD_CAP_ROM`)** |
| `mtd3` | Kernel | `0x00050000` | Dynamic | Linux Kernel (`zImage.lzma`) | Read-Only (Firmware Update) |
| `mtd4` | RootFS | Dynamic | Dynamic | SquashFS 4.0 (XZ Compression)| Read-Only (Firmware Update) |
| `mtd5` | Storage | `0x007E0000` | 128 KB | Gzip Archive (`/etc/storage`)| Read / Write |

Maximum firmware image budget (`mtd3` + `mtd4`) is 8,060,928 bytes (~7.68 MB). The standard FerrumOS release image is approximately 5.61 MB, leaving over 2.4 MB of available flash headroom.

---

## Memory Budget

```
+-------------------------------------------------------------------------+
|                     Physical Memory: 64 MB (65,536 KB)                  |
+---------------------+---------------------+-----------------------------+
| Kernel & Drivers    | Userland Daemons    | Available Headroom          |
| ~12 MB              | ~10 MB              | ~42 MB (> 65% Free)         |
+---------------------+---------------------+-----------------------------+
| - Linux Kernel Slab | - ferrum-core (2MB) | - Socket Buffers (sk_buff)  |
| - Netfilter Conntrack| - dnsmasq (1.5MB)   | - Page Cache                |
| - Wi-Fi Driver Ring | - dropbear (1MB)    | - ZRAM In-Memory Swap Pool  |
| - HWNAT Table       | - zapret / singbox  | - Transient Process Space   |
+---------------------+---------------------+-----------------------------+
```

---

## Building from Source

### Prerequisites

- Linux host (x86_64, Debian/Ubuntu/Arch/Fedora)
- Rust toolchain (stable, 1.75+ or newer)
- Target: `mipsel-unknown-linux-musl`
- Standard build utilities: `gcc`, `make`, `upx`, `tar`, `python3`

### Build Instructions

```bash
# 1. Clone repository
git clone https://github.com/MikhaelCat/FerrumOS.git
cd FerrumOS

# 2. Build and verify test suite (native host)
cargo test --manifest-path core/Cargo.toml

# 3. Compile optimized release binary
cargo build --manifest-path core/Cargo.toml --release

# 4. Generate production firmware image (.trx)
make firmware
```

The resulting firmware image is generated at:
```
release/FerrumOS-v1.0.0-SNR-MD1.trx
```

---

## Technical Documentation Index

Detailed architectural specifications, verification reports, and operational manuals are available in the `docs/` directory:

- [System Architecture](docs/ARCHITECTURE.md) — Comprehensive design specification of kernel, daemon, and hardware integration.
- [Feature Matrix & Subsystem Specification](docs/FEATURE_MATRIX.md) — Complete inventory of network, wireless, firewall, and administrative features.
- [Hardware Specification](docs/HARDWARE_SPEC.md) — Low-level pinouts, Ethernet switch topology, memory registers, and RF parameters.
- [Anti-Brick & Fail-Safe Architecture](docs/FAILSAFE_AND_RECOVERY.md) — Verification of partition write locks, atomic config writes, and recovery routines.
- [Footprint & Bloatware Removal Audit](docs/BLOAT_REMOVAL.md) — Quantitative analysis of purged legacy packages and reclaimed resources.
- [Architectural Verification Report](docs/ARCHITECTURAL_VERIFICATION.md) — Line-by-line verification against Linux kernel sources and hardware registers.
- [Development Roadmap](docs/REWRITE_ROADMAP.md) — Subsystem completion status and ongoing engineering roadmap.

---

## License

FerrumOS components authored for this distribution are licensed under the MIT License. The Linux kernel, driver modules, and third-party tools retain their respective upstream licenses (GPLv2, BSD, Apache 2.0).
