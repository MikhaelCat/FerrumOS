# FerrumOS Subsystem Milestones and Engineering Roadmap

This document outlines the architectural milestones, completed development phases, and ongoing engineering objectives for **FerrumOS**.

---

## Release Status: Version 1.0.0 (Production Ready)

The core FerrumOS architecture has achieved full functional parity and verification on the MediaTek MT7620A / NAG SNR-CPE-MD1.1 platform. All foundational subsystems—including memory-safe daemon services, hardware packet offloading, modern proxy integration, anti-censorship routing, and anti-brick flash safeguards—are implemented and certified.

---

## Completed Architectural Milestones

```
+--------------------------------------------------------------------------------+
|  Phase 1: Memory-Safe Core Daemon & Single-Page Management Console [VERIFIED]  |
+--------------------------------------------------------------------------------+
                                       |
                                       v
+--------------------------------------------------------------------------------+
|  Phase 2: Typed State Management & Transactional A/B Persistence    [VERIFIED]  |
+--------------------------------------------------------------------------------+
                                       |
                                       v
+--------------------------------------------------------------------------------+
|  Phase 3: DPI Mitigation Engine & Modern Proxy Protocols            [VERIFIED]  |
+--------------------------------------------------------------------------------+
                                       |
                                       v
+--------------------------------------------------------------------------------+
|  Phase 4: Hardware Offloading, Seamless Roaming & Network Isolation [VERIFIED]  |
+--------------------------------------------------------------------------------+
                                       |
                                       v
+--------------------------------------------------------------------------------+
|  Phase 5: Automated Firmware Toolchain, Test Harness & Verification  [VERIFIED]  |
+--------------------------------------------------------------------------------+
```

---

### Phase 1: Memory-Safe Core Daemon & Single-Page Management Console
- [x] **Asynchronous HTTP & WebSocket Daemon (`ferrum-core`)**:
  - Implemented lightweight asynchronous HTTP/1.1 and WebSocket server in Rust compiled with MUSL libc (`mipsel-unknown-linux-musl`).
  - Total stripped and compressed binary size: < 500 KB (UPX 5.2.0).
  - Runtime resident memory consumption: < 2.5 MB RSS.
- [x] **RESTful Device Management API (`/api/v1`)**:
  - `GET /api/v1/system/status`: Real-time CPU, RAM, uptime, and load averages.
  - `GET /api/v1/network/interfaces`: Real-time interface statistics, IP addresses, MAC addresses, and carrier link states.
  - `GET /api/v1/wifi/clients`: Associated 2.4 GHz and 5 GHz wireless stations with RSSI and negotiated PHY rates.
  - `POST /api/v1/wifi/settings`: Atomic configuration of SSID, security mode, WPA passphrase, and channel parameters.
  - `POST /api/v1/network/flow-offload`: Dynamic toggling of hardware PPE and software fast-path offloading.
  - `POST /api/v1/network/speedtest`: Embedded microsecond-metered WAN latency and throughput diagnostic engine.
- [x] **WebSocket Telemetry Streamer (`/ws/telemetry`)**:
  - Real-time event and bandwidth streaming directly to connected clients without HTTP polling overhead.
- [x] **Modern Single-Page Application (SPA)**:
  - Engineered in Preact and Tailwind CSS, compiled into pre-compressed static assets directly embedded in `ferrum-core`.
  - Client-side routing with zero server-side rendering latency.

---

### Phase 2: Typed State Management & Transactional A/B Persistence
- [x] **Strongly-Typed Configuration Engine**:
  - Implemented strict data structures using `serde` with automatic serialization and deserialization.
  - Input validation for IP subnets, netmasks, Wi-Fi regulatory channels, and credential formats.
- [x] **Transactional A/B Flash Double-Buffering**:
  - Partition `mtd1` configured into active and inactive 32 KB blocks with CRC32 integrity verification.
  - Safe recovery to active slot on mid-write power interruptions.
- [x] **Kernel Partition Protection**:
  - `mtd0` (Bootloader) and `mtd2` (Factory EEPROM) permanently locked as read-only (`MTD_CAP_ROM`).
- [x] **Hardware Watchdog Integration**:
  - Automated supervision of `/dev/watchdog` with 30-second hardware reset deadline.

---

### Phase 3: DPI Mitigation Engine & Modern Proxy Protocols
- [x] **Zapret `nfqws` Integration (v72.13)**:
  - Zero-copy packet interception via Netfilter queue 200 (`--qnum=200 --queue-bypass`).
  - TLS ClientHello desynchronization (`split-pos=1,midsld` with `badseq,md5sig` fooling) for YouTube and Discord Web.
  - UDP Layer-7 inspection for Discord Voice bypass (ports 50000–65535).
  - Enforced QUIC rejection (UDP 443) to guarantee TCP TLS fallback.
- [x] **Sing-Box Multi-Protocol Proxy Client**:
  - Configured high-performance proxy routing via TPROXY / TUN.
  - Complete URI parser and config generator supporting VLESS, Hysteria2, TUIC v5, and ShadowTLS v3.
- [x] **BGP Dynamic Routing for Antizapret**:
  - Integrated BIRD Lite routing daemon to peer with Antizapret route servers.
  - Dynamic route synchronization into Linux FIB table 100 (`vpn_table`).

---

### Phase 4: Hardware Offloading, Seamless Roaming & Network Isolation
- [x] **Hardware & Software Flow Offloading**:
  - MediaTek Ralink Packet Processing Engine (PPE HWNAT) initialization via `/proc/sys/net/nat/fast_nat` and `hw_nat.ko`.
  - Linux Netfilter flowtable offloading (`FLOWOFFLOAD` target) with TCP BBR congestion control.
  - Complete state persistence in `/etc/storage/flow_offload.json`.
- [x] **IEEE 802.11r Fast BSS Transition**:
  - Hostapd and driver configuration for over-the-air FT key distribution, enabling roaming transitions in < 10 ms.
- [x] **Isolated Guest Wireless Network**:
  - Virtual BSSID interfaces (`ra1` and `rai1`) isolated from the primary bridge (`br0`).
  - Idempotent iptables rules restricting guest traffic to WAN forwarding while dropping LAN and router management access.

---

### Phase 5: Automated Firmware Toolchain, Test Harness & Verification
- [x] **Comprehensive Test Suite**:
  - 50/50 unit and integration tests passing in `core/` (parsers, security validation, URI decoding).
  - 10-round live integration test suite verifying API, telemetry, speedtest, and proxy lifecycle.
- [x] **Automated Image Builder (`make firmware`)**:
  - Shell-based toolchain script (`tools/build_firmware.sh`) packaging RootFS, UPX-compressed daemon, and kernel into standard TRX image.
  - Total firmware size: ~5.61 MB (well within the 8 MB flash limit, providing 2.45 MB headroom).

---

## Ongoing Engineering Roadmap (v1.1+)

### 1. Advanced Networking & IPv6
- **Automated DHCPv6-PD Lifecycle**: Implement stateful tracking of IPv6 Prefix Delegation leases from upstream ISPs, automatically renumbering internal LAN and guest subnets on prefix change.
- **WireGuard In-Kernel Cryptographic Acceleration**: Evaluate assembly-optimized MIPS ChaCha20-Poly1305 routines to reduce handshake latency under high CPU load.

### 2. Upstream Kernel Research
- **Modern Long-Term Support Kernel Backport**: Investigate modern Linux 5.10 / 6.1 LTS kernel device-tree support for MT7620A SoC while maintaining binary driver compatibility with proprietary Wi-Fi modules (`rt2860v2`, `mt7610`).

### 3. Multi-SoC Target Expansion
- **MediaTek MT7621A Support**: Adapt `ferrum-core` and platform abstraction layer for dual-core 880 MHz MIPS 1004Kc hardware with Gigabit Ethernet switch and dual PCIe radios.
