# Padavan-Next ⚡

> **Next-generation high-performance firmware and modular rewrite project for MediaTek MT7620 / SNR-CPE-MD1 routers.**

[![Target SoC](https://img.shields.io/badge/SoC-MediaTek%20MT7620A-blue.svg)](https://www.mediatek.com/)
[![Architecture](https://img.shields.io/badge/Arch-MIPS%2024KEc%20(580%2F600MHz)-orange.svg)]()
[![Flash/RAM](https://img.shields.io/badge/Memory-8MB%20SPI%20%2F%2064MB%20DDR2-green.svg)]()
[![Status](https://img.shields.io/badge/Status-Architecture%20%26%20Planning-purple.svg)]()

---

## 🎯 Project Vision

The classic Padavan (N56U / Padavan-ng) firmware is legendary for its wire-speed HWNAT routing and Rock-solid stability on MediaTek/Ralink chips. However, its architecture dates back to the early 2010s:
- **Monolithic C daemons** (`rc`, `httpd`) with high coupling and direct global state mutations.
- **Single-threaded blocking Web Server** with server-side ASP macro interpretation on every page hit.
- **Fragile text-based NVRAM** storage synchronized to flash via custom shell scripts.
- **Legacy frontend** with aggressive client-side polling and heavy page reloads.

**Padavan-Next** is an architectural initiative to modernize, streamline, and systematically rewrite key router subsystems (leveraging memory-safe languages like **Rust** and modern C), delivering:
1. **Instant, Zero-Lag Web UI & API**: Replacing synchronous ASP/EJ templates with a fast, lightweight REST/WebSocket micro-daemon and modern reactive single-page app (SPA).
2. **Native DPI Bypass Engine**: First-class integration of **Zapret** (`nfqws`/`tpws`) with fine-grained web controls for YouTube (4K streaming, QUIC drop), YouTube Music, and Discord voice.
3. **Modular Event-Driven Architecture**: Safe, typed configuration and state management replacing error-prone shell scripts.
4. **Maximized Hardware Efficiency**: Exploiting 100% of the MediaTek MT7620A hardware capabilities (HWNAT PPE flow offload, MT7610 802.11ac 5GHz Wi-Fi, 600 MHz CPU overclock).

---

## 📁 Repository Structure & Documentation

- 📐 [**System Architecture (`docs/ARCHITECTURE.md`)**](docs/ARCHITECTURE.md) — Comprehensive technical architecture covering hardware registers, memory map, MTD flash partitioning, kernel subsystems, userland daemons, and target rewrite architecture.
- 📋 [**Feature Matrix & Inventory (`docs/FEATURE_MATRIX.md`)**](docs/FEATURE_MATRIX.md) — Exhaustive catalog of all ~100+ features and services in current Padavan firmware, categorized with rewrite recommendations (Keep / Rust / Rewrite / Drop).
- ⚙️ [**Hardware Specification (`docs/HARDWARE_SPEC.md`)**](docs/HARDWARE_SPEC.md) — Hardware pinouts, Ethernet switch layout, GPIOs, memory limits, and RF specifications for SNR-CPE-MD1.1.
- 🗺️ [**Rewrite Roadmap (`docs/REWRITE_ROADMAP.md`)**](docs/REWRITE_ROADMAP.md) — Phased milestone plan for decoupling, API construction, and subsystem modernization.

---

## 📟 Target Device: NAG SNR-CPE-MD1.1

| Parameter | Specification |
| :--- | :--- |
| **SoC** | MediaTek MT7620A (RevID 0206) |
| **CPU Core** | MIPS 24KEc @ 580 MHz (Overclockable to 600 MHz) |
| **RAM** | 64 MB DDR2 SDRAM (Nanya/Winbond) |
| **Flash** | 8 MB SPI NOR Flash (Winbond W25Q64BV) |
| **Ethernet** | 5x 10/100 Mbps Fast Ethernet Switch (1 WAN + 4 LAN) |
| **Wi-Fi 2.4 GHz** | MediaTek MT7620 SoC integrated 802.11b/g/n (2T2R, up to 300 Mbps) |
| **Wi-Fi 5 GHz** | MediaTek MT7610EN via internal PCIe (1T1R 802.11a/n/ac, up to 433 Mbps) |
| **HW Acceleration** | Ralink PPE (Packet Processing Engine) HWNAT / IPv4-IPv6 FOE |
| **Buttons / LEDs** | Reset Button (GPIO 13), Power/Status LEDs |

---

## 🚀 Quick Navigation

- Jump to [System Architecture](docs/ARCHITECTURE.md) to inspect the kernel, flash, and daemon layout.
- Jump to [Feature Matrix](docs/FEATURE_MATRIX.md) to review and select components for rewriting.
