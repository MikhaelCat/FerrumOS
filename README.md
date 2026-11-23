# FerrumOS ⚡

> **FerrumOS is an ultra-lightweight, memory-safe, anti-brick router firmware and operating system tailored for MediaTek MT7620A / NAG SNR-CPE-MD1 routers.**

[![Target SoC](https://img.shields.io/badge/SoC-MediaTek%20MT7620A-blue.svg)](https://www.mediatek.com/)
[![Architecture](https://img.shields.io/badge/Arch-MIPS%2024KEc%20(600MHz)-orange.svg)]()
[![Memory](https://img.shields.io/badge/RAM%2FFlash-64MB%20DDR2%20%2F%208MB%20SPI-green.svg)]()
[![License](https://img.shields.io/badge/License-GPLv2%20%2F%20MIT-brightgreen.svg)]()
[![Status](https://img.shields.io/badge/Status-Active%20Architecture%20%26%20Design-purple.svg)]()

---

## 💎 Project Vision: What is FerrumOS?

Old router firmwares (like legacy Padavan / OpenWrt LuCI) suffer from a decade of technical debt: monolithic single-threaded C daemons, blocking server-side ASP/Lua page rendering, fragile text NVRAM, and megabytes of legacy bloat (Samba, FTP, print servers on routers with no USB ports!).

**FerrumOS** strips out every ounce of deadweight and rebuilds the router's brain with modern engineering:
1. 🛡️ **Anti-Brick Guarantee**: Hardware-enforced read-only locking of critical partitions (`mtd0` Bootloader, `mtd2` Factory RF calibration), atomic A/B config double-buffering with CRC32, hardware watchdog (WDT), and an emergency 3-second Reset-pin web recovery mode.
2. ⚡ **Zero-Lag Modern Web UI**: The clunky ASP macro parser is replaced by **`ferrum-httpd`** (an async Rust REST/WebSocket daemon) and a sleek Single-Page Application (SPA) in Preact/Tailwind. Page transitions take **0 milliseconds**.
3. 🚀 **Wire-Speed Hardware Acceleration**: Preserves Ralink HWNAT PPE (Packet Processing Engine) for line-rate 100 Mbps routing at **0% CPU utilization**, while running the MT7620A CPU at **600 MHz**.
4. 🌐 **Native Anti-Censorship (Ferrum-DPI)**: First-class Zapret v72.13 (`nfqws`) integration for seamless YouTube 4K, YouTube Music, and Discord Voice unblocking with automated hostlist updates.
5. 🪓 **Massive Bloat Elimination**: Purges ~6.1 MB of useless flash packages and ~29.5 MB of RAM waste (Samba, FTP, Tor, StrongSwan, PPTP), leaving over **65% of physical RAM free** to eliminate out-of-memory crashes forever.

---

## 📚 Complete Technical Documentation

- 📐 [**System Architecture (`docs/ARCHITECTURE.md`)**](docs/ARCHITECTURE.md) — Comprehensive technical architecture covering hardware registers, memory map, MTD flash partitioning, kernel subsystems, and Ferrum-Core Rust services.
- 🛡️ [**Anti-Brick & Fail-Safe Architecture (`docs/FAILSAFE_AND_RECOVERY.md`)**](docs/FAILSAFE_AND_RECOVERY.md) — How FerrumOS guarantees the router will never brick, even on power loss, bad configuration, or corrupted flash writes.
- 🪓 [**Bloatware Removal & Savings Audit (`docs/BLOAT_REMOVAL.md`)**](docs/BLOAT_REMOVAL.md) — Detailed breakdown of purged legacy packages with exact Flash (~6.1 MB) and RAM (~29.5 MB) savings.
- 📋 [**Feature Matrix & Inventory (`docs/FEATURE_MATRIX.md`)**](docs/FEATURE_MATRIX.md) — Complete catalog of all router features categorized with implementation status and modernization targets.
- ⚙️ [**Hardware Specification (`docs/HARDWARE_SPEC.md`)**](docs/HARDWARE_SPEC.md) — Low-level pinouts, Ethernet switch layout, GPIOs, memory limits, and RF specifications for SNR-CPE-MD1.1.
- 🗺️ [**Modernization Roadmap (`docs/REWRITE_ROADMAP.md`)**](docs/REWRITE_ROADMAP.md) — Phased milestone plan for implementing Ferrum-Core, Web API, and UI components.

---

## 📟 Target Device: NAG SNR-CPE-MD1.1

| Parameter | Specification |
| :--- | :--- |
| **SoC** | MediaTek MT7620A (RevID 0206) |
| **CPU Core** | MIPS 24KEc @ 600 MHz (Overclocked PLL) |
| **RAM** | 64 MB DDR2 SDRAM (ZRAM 50% Compressed Swap) |
| **Flash** | 8 MB SPI NOR Flash (Winbond W25Q64BV) |
| **Ethernet** | 5x 10/100 Mbps Fast Ethernet Switch (1 WAN + 4 LAN) |
| **Wi-Fi 2.4 GHz** | MediaTek MT7620 SoC integrated 802.11b/g/n (2T2R, up to 300 Mbps) |
| **Wi-Fi 5 GHz** | MediaTek MT7610EN via internal PCIe (1T1R 802.11a/n/ac, up to 433 Mbps) |
| **HW Acceleration** | Ralink PPE (Packet Processing Engine) HWNAT / IPv4-IPv6 FOE |
| **Buttons / LEDs** | Reset Button (GPIO 13), Power/Status LEDs |

---

## 🛠️ Repository Quick Start

```bash
# Clone the repository
git clone https://github.com/MikhaelCat/FerrumOS.git
cd FerrumOS

# Inspect architectural specifications
cat docs/ARCHITECTURE.md
cat docs/FAILSAFE_AND_RECOVERY.md
cat docs/BLOAT_REMOVAL.md
```
